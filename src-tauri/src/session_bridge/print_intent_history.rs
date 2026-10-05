use super::*;

const HISTORY_LIMIT: usize = 64;
const HISTORY_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct PrintIntentHistoryEntry {
    id: String,
    before: Value,
    after: Value,
    mechanical_model: Value,
    undone: bool,
    bytes: usize,
}

pub(super) fn mechanical_model(mut model: Value) -> Value {
    if let Some(object) = model.as_object_mut() {
        // Presentation, allocation counters and other non-Undo metadata must
        // survive a print-setting restore rather than prevent its Undo.
        for key in [
            "print_intent",
            "counters",
            "visibility",
            "views",
            "body_appearances",
            "preferences",
            "cam",
        ] {
            object.remove(key);
        }
    }
    model
}

pub(super) fn exported_model(engine: &AppState) -> Result<Value, String> {
    let json = parse_engine_envelope(engine.engine_call("project_export_model", ""))?;
    serde_json::from_str(
        json.as_str()
            .ok_or("Project export did not return model JSON")?,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn record(
    project: &mut ProjectPublisher,
    before_model: Value,
    after: Value,
) -> Option<Value> {
    let mut before = before_model["print_intent"].clone();
    // Identity is assigned once and survives Undo, including the first edit.
    before["source_document_id"] = after["source_document_id"].clone();
    if before == after {
        return None;
    }
    let mechanical_model = mechanical_model(before_model);
    let id = Uuid::new_v4().to_string();
    let receipt =
        json!({"id":id,"before":before,"after":after,"mechanical_model":mechanical_model});
    let bytes = receipt.to_string().len();
    project.print_intent_history.retain(|entry| !entry.undone);
    project.print_intent_history.push(PrintIntentHistoryEntry {
        id,
        before,
        after,
        mechanical_model,
        undone: false,
        bytes,
    });
    while project.print_intent_history.len() > HISTORY_LIMIT
        || project
            .print_intent_history
            .iter()
            .map(|entry| entry.bytes)
            .sum::<usize>()
            > HISTORY_BYTES
    {
        project.print_intent_history.remove(0);
    }
    Some(receipt)
}

impl SessionBridgeState {
    pub(super) fn restore_print_intent(
        &self,
        window_label: &str,
        engine: &AppState,
        document: &str,
        session: &str,
        receipt_id: &str,
        redo: bool,
        expected_model_json: &str,
    ) -> Result<Value, String> {
        if expected_model_json.len() > HISTORY_BYTES {
            return Err("Print-intent history snapshot exceeds 32 MiB".into());
        }
        let mut publishers = self
            .publishers
            .lock()
            .map_err(|_| "session publisher lock poisoned")?;
        let publisher = publishers
            .get_mut(window_label)
            .ok_or("No window session")?;
        if publisher.active_project_session_id.as_deref() != Some(document)
            || engine.active_project_session_id() != document
        {
            return Err("Print-intent history belongs to another document".into());
        }
        let project = publisher.active_mut();
        if project.session_id != session {
            return Err("Print-intent history belongs to a replaced document".into());
        }
        let expected: Value =
            serde_json::from_str(expected_model_json).map_err(|error| error.to_string())?;
        let current = exported_model(engine)?;
        if current != expected {
            return Err("The document changed before print-intent history restore".into());
        }
        let entry = project
            .print_intent_history
            .iter_mut()
            .find(|entry| entry.id == receipt_id)
            .ok_or("Print-intent history receipt is expired or unknown")?;
        let (source, destination) = if redo {
            (&entry.before, &entry.after)
        } else {
            (&entry.after, &entry.before)
        };
        if entry.undone != redo
            || current["print_intent"] != *source
            || mechanical_model(current) != entry.mechanical_model
        {
            return Err(
                "A newer model operation must be undone before this print-intent edit".into(),
            );
        }
        let result = parse_engine_envelope(
            engine.engine_call(
                "print_intent_set_document",
                &json!({
                    "document":destination,"expected_model_json":expected_model_json,
                })
                .to_string(),
            ),
        )?;
        entry.undone = !redo;
        bump_engine_revision(
            project,
            window_label,
            Some(document),
            &self.process_instance_id,
        )?;
        Ok(result)
    }
}

#[tauri::command]
pub fn mcp_session_bridge_restore_print_intent(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SessionBridgeState>,
    engine: tauri::State<'_, AppState>,
    document: String,
    session: String,
    receipt_id: String,
    redo: bool,
    expected_model_json: String,
) -> Result<Value, String> {
    state.restore_print_intent(
        window.label(),
        &engine,
        &document,
        &session,
        &receipt_id,
        redo,
        &expected_model_json,
    )
}

impl SessionBridgeState {
    pub(super) fn replay_history_model(
        &self,
        window_label: &str,
        engine: &AppState,
        document: &str,
        session: &str,
        model_json: &str,
        expected_model_json: &str,
    ) -> Result<String, String> {
        if model_json.len() > HISTORY_BYTES || expected_model_json.len() > HISTORY_BYTES {
            return Err("History replay snapshot exceeds 32 MiB".into());
        }
        let mut publishers = self
            .publishers
            .lock()
            .map_err(|_| "session publisher lock poisoned")?;
        let publisher = publishers
            .get_mut(window_label)
            .ok_or("No window session")?;
        if publisher.active_project_session_id.as_deref() != Some(document)
            || engine.active_project_session_id() != document
        {
            return Err("History replay belongs to another document".into());
        }
        let project = publisher.active_mut();
        if project.session_id != session {
            return Err("History replay belongs to a replaced document".into());
        }
        let expected: Value =
            serde_json::from_str(expected_model_json).map_err(|error| error.to_string())?;
        let current = exported_model(engine)?;
        if current != expected {
            return Err("The document changed before history replay".into());
        }
        let replay: Value = serde_json::from_str(model_json).map_err(|error| error.to_string())?;
        if replay["print_intent"] != current["print_intent"] {
            return Err("Geometry history replay must preserve current print intent".into());
        }
        let result = engine
            .project_load(&serde_json::to_string(model_json).map_err(|error| error.to_string())?);
        if !project_replacement_is_unchanged(&result) {
            bump_engine_revision(
                project,
                window_label,
                Some(document),
                &self.process_instance_id,
            )?;
        }
        Ok(result)
    }
}

#[tauri::command]
pub fn mcp_session_bridge_replay_history(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SessionBridgeState>,
    engine: tauri::State<'_, AppState>,
    document: String,
    session: String,
    model_json: String,
    expected_model_json: String,
) -> Result<String, String> {
    state.replay_history_model(
        window.label(),
        &engine,
        &document,
        &session,
        &model_json,
        &expected_model_json,
    )
}
