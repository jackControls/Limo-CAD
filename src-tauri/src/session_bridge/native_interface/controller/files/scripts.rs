//! File loading inspects source only. An explicit run creates a retained blank
//! tab and hands frozen, expanded source to the existing live script runner.
use super::*;

pub(super) struct Loaded {
    pub path: PathBuf,
    pub name: String,
    pub steps: u64,
    pub checks: u64,
    // Preserve authored text and include provenance alongside the execution
    // snapshot; opening a script never rewrites or executes its source file.
    inspection: Value,
}
impl Loaded {
    fn source(&self) -> &str {
        self.inspection["source"].as_str().unwrap()
    }
}

#[derive(Default)]
pub(super) struct State {
    pub path: String,
    pub loaded: Option<Arc<Loaded>>,
    pub generation: u64,
    pub status: Option<String>,
    loading: Option<Mutex<mpsc::Receiver<Result<Loaded, String>>>>,
}
impl State {
    pub fn loading(&self) -> bool {
        self.loading.is_some()
    }
    fn accept(&mut self, loaded: Loaded) -> Result<(), String> {
        let generation = self
            .generation
            .checked_add(1)
            .ok_or("Script generation exhausted")?;
        self.path = loaded.path.to_string_lossy().into_owned();
        self.loaded = Some(Arc::new(loaded));
        self.generation = generation;
        self.status = None;
        Ok(())
    }
    fn selected(&self, generation: u64) -> Result<Arc<Loaded>, String> {
        if self.loading() || self.generation != generation {
            return Err("The loaded script changed; choose Run again".into());
        }
        self.loaded
            .clone()
            .ok_or("Open a script before running it".into())
    }
}

fn inspect(path: PathBuf) -> Result<Loaded, String> {
    let text_path = path.to_str().ok_or("Script path must be valid Unicode")?;
    let inspection = nbcad_mcp::inspect_script(json!({"path":text_path}))?;
    let name = inspection["name"]
        .as_str()
        .ok_or("Script name is missing")?
        .to_owned();
    let steps = inspection["step_count"]
        .as_u64()
        .ok_or("Script step count is missing")?;
    let checks = inspection["check_count"]
        .as_u64()
        .ok_or("Script check count is missing")?;
    inspection["source"]
        .as_str()
        .ok_or("Expanded script source is missing")?;
    Ok(Loaded {
        path,
        name,
        steps,
        checks,
        inspection,
    })
}

fn available(world: &World) -> Result<(), String> {
    let files = world.resource::<Files>();
    if files.lesson.is_some() {
        return Err("Stop the running script or wait for it to finish".into());
    }
    if files.script.loading() {
        return Err("Wait for the script to finish loading".into());
    }
    if awaiting(world) {
        return Err("Finish the current File dialog first".into());
    }
    Ok(())
}

pub(super) fn edit_path(world: &mut World, input: &ControlInput) -> Result<Value, String> {
    available(world)?;
    let ControlInput::SetValue(path) = input else {
        return Err("Script path requires text".into());
    };
    world.resource_mut::<Files>().script.path = path.clone();
    Ok(json!({"changed":true}))
}

pub(super) fn choose(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    receipt: DocumentReceipt,
) -> Result<Value, String> {
    available(world)?;
    let directory = world
        .resource::<Files>()
        .script
        .loaded
        .as_ref()
        .and_then(|loaded| loaded.path.parent())
        .map(std::path::Path::to_path_buf);
    let (send, receive) = mpsc::channel();
    let wake = handle.clone();
    std::thread::Builder::new()
        .name("cad-script-picker".into())
        .spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("noBS CAD command script (.nbcad.jsonc)", &["jsonc"]);
            if let Some(directory) = directory {
                dialog = dialog.set_directory(directory);
            }
            let _ = send.send(dialog.pick_file());
            wake.request_redraw();
        })
        .map_err(|error| format!("Cannot open script chooser: {error}"))?;
    world.resource_mut::<Files>().picker = Some(Picker {
        receipt,
        kind: PickerKind::Script,
        result: Mutex::new(receive),
    });
    Ok(json!({"awaiting_input":true}))
}

pub(super) fn load(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    services: &NativeServices,
    receipt: DocumentReceipt,
    path: PathBuf,
) -> Result<Value, String> {
    available(world)?;
    if services
        .bridge
        .native_document_receipt(&services.engine, &receipt.owner)?
        != receipt
    {
        return Err("The document changed while the script chooser was open".into());
    }
    let (send, receive) = mpsc::channel();
    let wake = handle.clone();
    std::thread::Builder::new()
        .name("cad-script-inspect".into())
        .spawn(move || {
            let inspected = std::panic::catch_unwind(|| inspect(path))
                .unwrap_or_else(|_| Err("Script inspection stopped unexpectedly".into()));
            let _ = send.send(inspected);
            wake.request_redraw();
        })
        .map_err(|error| format!("Cannot inspect script: {error}"))?;
    let mut files = world.resource_mut::<Files>();
    files.script.loading = Some(Mutex::new(receive));
    files.script.status = Some("Loading script; no commands have run".into());
    Ok(json!({"script_loading":true}))
}

pub(super) fn poll(world: &mut World) {
    let result = world
        .resource::<Files>()
        .script
        .loading
        .as_ref()
        .and_then(|loading| {
            Some(match loading.lock() {
                Ok(channel) => match channel.try_recv() {
                    Ok(result) => result,
                    Err(mpsc::TryRecvError::Empty) => return None,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Err("Script inspection worker disconnected".into())
                    }
                },
                Err(_) => Err("Script inspection result could not be read".into()),
            })
        });
    let Some(result) = result else { return };
    let mut files = world.resource_mut::<Files>();
    files.script.loading = None;
    if let Err(error) = result.and_then(|loaded| files.script.accept(loaded)) {
        files.script.status = Some(format!("Script not loaded: {error}"));
    }
}

fn new_document(
    services: &NativeServices,
    workspace: &Mutex<DocumentWorkspace>,
    receipt: &DocumentReceipt,
    validate: impl FnOnce() -> Result<(), String>,
) -> Result<NativeMutationResult, String> {
    let created = workspace
        .lock()
        .map_err(|_| "Document workspace lock poisoned")?
        .new_tab_guarded(&services.bridge, &services.engine, receipt, validate)?;
    Ok(NativeMutationResult {
        context: created.owner,
        engine_revision: created.revision,
        value: json!({"changed":true}),
    })
}

pub(super) fn run(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    receipt: DocumentReceipt,
    generation: u64,
) -> Result<Value, String> {
    available(world)?;
    let loaded = world.resource::<Files>().script.selected(generation)?;
    remember_view(world, &receipt.owner);
    let workspace = world.resource::<Files>().workspace.clone();
    let wake = handle.clone();
    worker::enqueue_transaction(
        world,
        "script_new_document".into(),
        move |services, guard| new_document(services, &workspace, &receipt, || guard.validate()),
        move |world, services, result| {
            let result = result?;
            let owner = result.context.clone();
            let mut output =
                finish_document_transition(world, services, "script_new_document", result);
            if output["render_error"].is_string() {
                return Ok(output);
            }
            // Publication has completed for this exact new tab. The existing
            // runner rechecks blank state and session ownership before calls.
            match lessons::start_source(
                world,
                &wake,
                services,
                &owner,
                &loaded.name,
                loaded.source().to_owned(),
                "Script",
            ) {
                Ok(()) => {
                    output["script_started"] = json!({"name":loaded.name,"path":loaded.path});
                }
                Err(error) => {
                    // New already committed; never invite a duplicate tab by
                    // reporting the whole transition as an unapplied failure.
                    world.resource_mut::<Files>().script.status =
                        Some(format!("Script not started: {error}"));
                    output["script_error"] = json!(error);
                }
            }
            Ok(output)
        },
    )
}

#[cfg(test)]
mod tests;
