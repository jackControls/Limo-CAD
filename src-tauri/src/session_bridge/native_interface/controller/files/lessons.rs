//! Short installed lessons use the existing live script runner. The runner
//! waits for inbox receipts, so it must never occupy the modeling worker.
use super::*;
use std::sync::OnceLock;

pub(super) struct Lesson {
    pub id: String,
    pub name: String,
    source: String,
}

pub(super) fn catalog() -> &'static [Lesson] {
    static LESSONS: OnceLock<Vec<Lesson>> = OnceLock::new();
    LESSONS.get_or_init(|| {
        nbcad_mcp::script_examples()
            .as_array()
            .into_iter()
            .flatten()
            .filter(|entry| entry["kind"] == "lesson")
            .filter_map(|entry| {
                Some(Lesson {
                    id: entry["id"].as_str()?.into(),
                    name: entry["name"].as_str()?.into(),
                    source: entry["source"].as_str()?.into(),
                })
            })
            .collect()
    })
}

fn lesson(id: &str) -> Result<&'static Lesson, String> {
    catalog()
        .iter()
        .find(|lesson| lesson.id == id)
        .ok_or_else(|| "Choose a short built-in lesson from Scripts".into())
}

pub(super) struct Running {
    owner: DocumentContext,
    result: Mutex<mpsc::Receiver<Result<Value, String>>>,
}

pub(super) fn start(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    services: &NativeServices,
    owner: &DocumentContext,
    id: &str,
) -> Result<Value, String> {
    let lesson = lesson(id)?;
    if world.resource::<Files>().lesson.is_some() {
        return Err(
            "A lesson is already running. Use its playback controls to stop or pause it".into(),
        );
    }
    if awaiting(world) {
        return Err("Finish the current File dialog first".into());
    }
    let receipt =
        services
            .bridge
            .with_native_document_receipt(&services.engine, owner, |revision| {
                if !services.engine.is_blank_for_script() {
                    return Err("Lessons require a blank document. Use New document first".into());
                }
                Ok(DocumentReceipt {
                    owner: owner.clone(),
                    revision,
                })
            })?;
    let session = services
        .bridge
        .session_id_for_window(&owner.window_id)?
        .ok_or("Publish the current document before running a lesson")?;
    let session = services.bridge.active_script_session(
        &owner.window_id,
        &services.engine,
        &owner.document_id,
        &session,
    )?;
    let (send, receive) = mpsc::channel();
    let services = services.clone();
    let wake = handle.clone();
    std::thread::Builder::new()
        .name("cad-native-lesson".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if services
                    .bridge
                    .native_document_receipt(&services.engine, &receipt.owner)?
                    != receipt
                {
                    return Err("The document changed before the lesson started".into());
                }
                nbcad_mcp::run_script(&lesson.source, None, Some(&session), "present", 1.)
            }))
            .unwrap_or_else(|_| {
                Err(
                    "The lesson worker stopped unexpectedly; any completed work is preserved"
                        .into(),
                )
            });
            let _ = send.send(result);
            wake.request_redraw();
        })
        .map_err(|error| format!("Cannot start lesson: {error}"))?;
    let mut files = world.resource_mut::<Files>();
    files.lesson = Some(Running {
        owner: owner.clone(),
        result: Mutex::new(receive),
    });
    files.lesson_status = Some((owner.clone(), format!("Running {}", lesson.name)));
    files.scripts = false;
    Ok(json!({"lesson_started":lesson.id}))
}

pub(super) fn poll(world: &mut World) {
    let result = world
        .resource::<Files>()
        .lesson
        .as_ref()
        .and_then(|running| {
            Some(match running.result.lock() {
                Ok(receiver) => match receiver.try_recv() {
                    Ok(result) => result,
                    Err(mpsc::TryRecvError::Empty) => return None,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Err("The lesson worker disconnected".into())
                    }
                },
                Err(_) => Err("The lesson result could not be read".into()),
            })
        });
    let Some(result) = result else { return };
    let mut files = world.resource_mut::<Files>();
    let running = files.lesson.take().unwrap();
    let status = match result {
        Ok(report) => format!(
            "Lesson complete: {} steps, {} checks",
            report["steps_completed"], report["checks_completed"]
        ),
        Err(error) => format!("Lesson stopped: {error}"),
    };
    files.lesson_status = Some((running.owner, status));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_catalog_excludes_flagship_and_arbitrary_sources() {
        assert_eq!(catalog().len(), 4);
        assert!(lesson("fillet-basics")
            .unwrap()
            .source
            .contains("solid_fillet"));
        for id in [
            "d-screw-vise",
            "garden-bench",
            "vertical-axis-turbine",
            "../lesson.jsonc",
        ] {
            assert!(lesson(id).is_err(), "{id}");
        }
    }

    #[test]
    fn lesson_refuses_work_before_starting_any_worker() {
        let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
        let fixture = super::super::super::super::tests::Fixture::new();
        let services = NativeServices {
            engine: fixture.engine.clone(),
            bridge: fixture.bridge.clone(),
        };
        let mut world = World::new();
        initialize(
            &mut world,
            Arc::new(Mutex::new(DocumentWorkspace::default())),
        );
        let owner = fixture.owner();
        fixture
            .bridge
            .apply_native_mutation(
                &fixture.engine,
                &owner,
                "sketch_begin",
                &json!({"type":"origin_plane","plane":"xy"}),
                || Ok(()),
            )
            .unwrap();
        let before = fixture.engine.engine_call("project_export_model", "");
        let error = start(
            &mut world,
            &NativeInterfaceHandle::new(|| {}),
            &services,
            &owner,
            "fillet-basics",
        )
        .unwrap_err();
        assert!(error.contains("blank"));
        assert!(world.resource::<Files>().lesson.is_none());
        assert_eq!(
            before,
            fixture.engine.engine_call("project_export_model", "")
        );
    }
}
