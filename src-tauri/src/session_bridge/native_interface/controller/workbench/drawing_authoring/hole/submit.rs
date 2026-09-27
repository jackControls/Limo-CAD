use super::super::super::super::*;
use super::super::runtime::Editor;
use super::*;

fn prepare(
    engine: &AppState,
    bridge: &SessionBridgeState,
    stamp: &Stamp,
    target: &radial::Target,
    validate: impl FnOnce() -> Result<(), String>,
) -> Result<serde_json::Value, String> {
    bridge.with_native_document_receipt(engine, &stamp.owner, |revision| {
        if revision != stamp.revision {
            return Err("Drawing changed; choose the refreshed circle".into());
        }
        validate()?;
        let encoded = engine.engine_call("hole_definitions", "");
        if encoded.len() > 8 * 1024 * 1024 {
            return Err("Modeled hole definitions exceed the callout limit".into());
        }
        let definitions = serde_json::from_value::<Vec<HoleDefinitionDto>>(
            crate::session_bridge::parse_engine_envelope(encoded)?,
        )
        .map_err(|error| error.to_string())?;
        let next = create(&engine.drawing_snapshot(), stamp, target, &definitions)?;
        serde_json::to_value(next).map_err(|error| error.to_string())
    })
}

pub(in super::super) fn submit(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    engine: &AppState,
    bridge: &SessionBridgeState,
    stamp: &Stamp,
    target: &radial::Target,
) -> Result<Value, String> {
    if worker::available(world) {
        let stamp = stamp.clone();
        let target = target.clone();
        let expected = stamp.owner.clone();
        return worker::enqueue_transaction(
            world,
            "drawing_set_document".into(),
            move |services, guard| {
                let args = prepare(&services.engine, &services.bridge, &stamp, &target, || {
                    guard.validate()
                })?;
                services.bridge.apply_native_mutation_at(
                    &services.engine,
                    &stamp.owner,
                    stamp.revision,
                    "drawing_set_document",
                    &args,
                    || guard.validate(),
                )
            },
            move |world, services, result| {
                let result = result.map_err(|error| {
                    if let Some(mut editor) = world.get_resource_mut::<Editor>() {
                        if editor.stamp.as_ref().is_some_and(|s| s.owner == expected) {
                            editor.message = error.clone();
                        }
                    }
                    error
                })?;
                Ok(finish_mutation(
                    &services.engine,
                    &services.bridge,
                    world,
                    "drawing_set_document",
                    result,
                ))
            },
        );
    }
    // Fixtures without a worker exercise the same receipt-fenced preparation.
    let args = prepare(engine, bridge, stamp, target, || {
        handle
            .frame()
            .filter(|frame| frame.context == stamp.owner)
            .map(|_| ())
            .ok_or("Drawing document changed".into())
    })?;
    super::super::runtime::submit(
        world,
        handle,
        engine,
        bridge,
        stamp,
        "drawing_set_document",
        args,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_bridge::{native_interface::tests::Fixture, parse_engine_envelope};

    #[test]
    fn hole_preparation_queries_the_owned_document_without_editing_and_rejects_stale_or_cancelled_work()
     {
        let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
        let f = Fixture::new();
        let document = super::super::super::tests::document();
        let receipt = f
            .bridge
            .native_document_receipt(&f.engine, &f.owner())
            .unwrap();
        f.bridge
            .apply_native_mutation_at(
                &f.engine,
                &receipt.owner,
                receipt.revision,
                "drawing_set_document",
                &serde_json::to_value(&document).unwrap(),
                || Ok(()),
            )
            .unwrap();
        let receipt = f
            .bridge
            .native_document_receipt(&f.engine, &f.owner())
            .unwrap();
        let stamp = Stamp {
            owner: receipt.owner,
            revision: receipt.revision,
            sheet_id: 1,
        };
        let target = super::super::tests::target();
        let exported =
            || parse_engine_envelope(f.engine.engine_call("project_export_model", "")).unwrap();
        let before = exported();
        let prepared = prepare(&f.engine, &f.bridge, &stamp, &target, || Ok(())).unwrap();
        assert_eq!(
            prepared,
            serde_json::to_value(create(&document, &stamp, &target, &[]).unwrap()).unwrap()
        );
        assert_eq!(exported(), before);
        assert_eq!(
            prepare(&f.engine, &f.bridge, &stamp, &target, || Err(
                "Cancelled owned gesture".into()
            ))
            .unwrap_err(),
            "Cancelled owned gesture"
        );
        let mut stale = stamp.clone();
        stale.revision += 1;
        assert!(prepare(&f.engine, &f.bridge, &stale, &target, || Ok(())).is_err());
        stale = stamp;
        stale.owner.epoch += 1;
        assert!(prepare(&f.engine, &f.bridge, &stale, &target, || Ok(())).is_err());
        assert_eq!(exported(), before);
    }
}
