use super::*;
use sha2::{Digest, Sha256};
use std::io::Read;

const HISTORY_LIMIT: usize = 64;
const HISTORY_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PrintIntentHistoryEntry {
    id: String,
    before: Value,
    after: Value,
    mechanical_model: Value,
    undone: bool,
    bytes: usize,
}

const ARCHIVE_BYTES: usize = 64 * 1024 * 1024;
const WINDOW_ARCHIVE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct ArchivedPrintIntentHistory {
    pub(super) path: PathBuf,
    sha256: [u8; 32],
    bytes: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PrintIntentHistoryArchive {
    version: u32,
    document: String,
    session: String,
    process_instance_id: String,
    model: Value,
    receipts: Vec<PrintIntentHistoryEntry>,
}

pub(super) fn forget_archive(publisher: &mut WindowPublisher, document: &str) {
    if let Some(archive) = publisher.archived_print_history.remove(document) {
        let _ = fs::remove_file(archive.path);
    }
}

impl SessionBridgeState {
    pub fn drop_project_context(
        &self,
        window_label: &str,
        engine: &AppState,
        document: &str,
        retain_history: bool,
        preserve_archive: bool,
    ) -> String {
        let result = self.drop_project_context_checked(
            window_label,
            engine,
            document,
            retain_history,
            preserve_archive,
        );
        result.unwrap_or_else(|error| json!({"ok":false,"error":error}).to_string())
    }

    fn drop_project_context_checked(
        &self,
        window_label: &str,
        engine: &AppState,
        document: &str,
        retain_history: bool,
        preserve_archive: bool,
    ) -> Result<String, String> {
        let mut publishers = self
            .publishers
            .lock()
            .map_err(|_| "session publisher lock poisoned")?;
        let publisher = publishers
            .get_mut(window_label)
            .ok_or("No window session")?;
        if engine.active_project_session_id() == document {
            return Err("Cannot evict the active project".into());
        }
        if preserve_archive
            && (retain_history
                || !publisher.archived_print_history.contains_key(document)
                || publisher
                    .by_project
                    .get(document)
                    .is_none_or(|project| project.last_model_generation.is_some()))
        {
            return Err("Cold rollback requires an unpublished owning context and its authenticated archive".into());
        }
        let archive = if retain_history {
            let project = publisher
                .by_project
                .get(document)
                .ok_or("Inactive project is not owned by this window")?;
            let model: Value = serde_json::from_str(&engine.project_model_for_session(document)?)
                .map_err(|error| error.to_string())?;
            let archive = PrintIntentHistoryArchive {
                version: 1,
                document: document.into(),
                session: project.session_id.clone(),
                process_instance_id: self.process_instance_id.clone(),
                model,
                receipts: project.print_intent_history.clone(),
            };
            let bytes = serde_json::to_vec(&archive).map_err(|error| error.to_string())?;
            let retained_bytes: usize = publisher
                .archived_print_history
                .iter()
                .filter(|(id, _)| id.as_str() != document)
                .map(|(_, archive)| archive.bytes)
                .sum();
            if bytes.len() > ARCHIVE_BYTES || retained_bytes + bytes.len() > WINDOW_ARCHIVE_BYTES {
                return Err(
                    "Print-history archive exceeds its bounded cache; the tab remains resident"
                        .into(),
                );
            }
            let path = session_root()
                .join(&project.session_id)
                .join("print-intent-history.json");
            atomic_write(
                &path,
                std::str::from_utf8(&bytes).map_err(|error| error.to_string())?,
            )?;
            Some(ArchivedPrintIntentHistory {
                path,
                sha256: Sha256::digest(&bytes).into(),
                bytes: bytes.len(),
            })
        } else {
            None
        };
        let result = engine.drop_project_session(document);
        if engine_envelope_ok(&result) {
            if let Some(project) = publisher.by_project.get(document) {
                if let Err(error) = write_closed_tombstone(&project.session_id) {
                    eprintln!("Could not tombstone evicted print-history session: {error}");
                }
            }
            if !preserve_archive {
                forget_archive(publisher, document);
            }
            if let Some(archive) = archive {
                publisher
                    .archived_print_history
                    .insert(document.into(), archive);
            }
            publisher.drop_project(document);
        } else if let Some(archive) = archive {
            let _ = fs::remove_file(archive.path);
        }
        drop(publishers);
        let _ = self.write_process_instance_file();
        Ok(result)
    }

    pub(super) fn restore_cold_project(
        &self,
        window_label: &str,
        engine: &AppState,
        document: &str,
        model_json: &str,
    ) -> Result<Value, String> {
        if model_json.len() > HISTORY_BYTES {
            return Err("Cold project snapshot exceeds 32 MiB".into());
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
            return Err("Cold history belongs to another document".into());
        }
        let attestation = publisher
            .archived_print_history
            .get(document)
            .ok_or("No authenticated cold history for this project")?;
        let mut bytes = Vec::new();
        fs::File::open(&attestation.path)
            .map_err(|error| error.to_string())?
            .take(ARCHIVE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        if bytes.len() > ARCHIVE_BYTES
            || bytes.len() != attestation.bytes
            || digest != attestation.sha256
        {
            return Err("Cold print-history archive failed native integrity verification".into());
        }
        let archive: PrintIntentHistoryArchive =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        let requested: Value =
            serde_json::from_str(model_json).map_err(|error| error.to_string())?;
        if archive.version != 1
            || archive.document != document
            || archive.process_instance_id != self.process_instance_id
            || archive.model != requested
        {
            return Err(
                "Cold project snapshot does not match its authenticated native archive".into(),
            );
        }
        let blank: Value = serde_json::from_str(
            &nbcad_sketch::SketchManager::new()
                .export_project_model()
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        if exported_model(engine)? != blank
            || publisher.active_mut().last_model_generation.is_some()
        {
            return Err("Cold restoration requires a fresh owning native tab".into());
        }
        let raw = engine
            .project_load(&serde_json::to_string(model_json).map_err(|error| error.to_string())?);
        let result = parse_engine_envelope(raw.clone());
        if let Ok(update) = result {
            let project = publisher.active_mut();
            project.print_intent_history = archive.receipts;
            let receipt_ids: Vec<_> = project
                .print_intent_history
                .iter()
                .map(|entry| entry.id.clone())
                .collect();
            bump_engine_revision(
                project,
                window_label,
                Some(document),
                &self.process_instance_id,
            )?;
            let response = json!({"update":update,"session_id":project.session_id,"document_id":document,
                "receipt_ids":receipt_ids});
            Ok(response)
        } else {
            if !project_replacement_is_unchanged(&raw) {
                retire_project_publisher(
                    publisher,
                    window_label,
                    document,
                    &self.process_instance_id,
                    ProjectPublisher::new(),
                );
            }
            Err(result.unwrap_err())
        }
    }
}

#[tauri::command]
pub fn mcp_session_bridge_restore_cold_project(
    window: tauri::WebviewWindow,
    state: tauri::State<'_, SessionBridgeState>,
    engine: tauri::State<'_, AppState>,
    document: String,
    model_json: String,
) -> Result<Value, String> {
    state.restore_cold_project(window.label(), &engine, &document, &model_json)
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
