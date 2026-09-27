use super::*;
use crate::session_bridge::{native_interface::tests::Fixture, parse_engine_envelope};
use serde_json::json;

#[test]
fn presentation_apply_validates_shared_metadata_and_restores_exact_issued_history() {
    use fields::{Id, tests as form};
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let f = Fixture::new();
    let seeded = form::document();
    f.bridge
        .apply_native_mutation(
            &f.engine,
            &f.owner(),
            "drawing_set_document",
            &serde_json::to_value(&seeded).unwrap(),
            || Ok(()),
        )
        .unwrap();
    let drawing = f.engine.drawing_snapshot();
    let exported =
        || parse_engine_envelope(f.engine.engine_call("project_export_model", "")).unwrap();
    let before = exported();
    let receipt = f
        .bridge
        .native_document_receipt(&f.engine, &f.owner())
        .unwrap();
    // Exercise engine-side shared validation as well as the form parser.
    let mut invalid = drawing.clone();
    if let nbcad_sketch::DrawingAnnotationDto::LinearDimension { presentation, .. } =
        &mut invalid.sheets[0].annotations[2]
    {
        presentation.dual_units.as_mut().unwrap().precision = 7;
    }
    assert!(
        f.bridge
            .apply_native_mutation_at(
                &f.engine,
                &receipt.owner,
                receipt.revision,
                "drawing_set_document",
                &serde_json::to_value(invalid).unwrap(),
                || Ok(())
            )
            .is_err()
    );
    assert_eq!(exported(), before);
    assert_eq!(
        f.bridge
            .native_document_receipt(&f.engine, &f.owner())
            .unwrap()
            .revision,
        receipt.revision
    );
    let mut draft = form::linear(&drawing);
    let mut inputs = fields::from_annotation(draft.annotation());
    for (id, value) in [
        (Id::Tolerance, "deviation"),
        (Id::Upper, "0.25"),
        (Id::Lower, "-0.1"),
        (Id::Basic, "true"),
        (Id::Fit, "g6"),
        (Id::DualUnit, "inch"),
        (Id::DualPrecision, "3"),
        (Id::DualPlacement, "bracketed"),
    ] {
        form::set(&mut inputs, id, value);
    }
    fields::apply(&mut draft, &inputs).unwrap();
    let next = draft.apply(&drawing).unwrap();
    assert_eq!(
        next.sheets[0].release.status,
        nbcad_sketch::DrawingReleaseStatus::Draft
    );
    assert_eq!(
        next.sheets[0].release.released_revision,
        drawing.sheets[0].release.released_revision
    );
    assert_eq!(next.sheets[1], drawing.sheets[1]);
    assert_eq!(
        &next.sheets[0].annotations[..2],
        &drawing.sheets[0].annotations[..2]
    );
    assert_eq!(
        exported(),
        before,
        "Editing fields cannot mutate the shared document before Apply"
    );
    f.bridge
        .apply_native_mutation_at(
            &f.engine,
            &receipt.owner,
            receipt.revision,
            "drawing_set_document",
            &serde_json::to_value(&next).unwrap(),
            || Ok(()),
        )
        .unwrap();
    let after = exported();
    assert_eq!(f.engine.drawing_snapshot(), next);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), false, || Ok(()))
        .unwrap();
    assert_eq!(
        exported(),
        before,
        "One Undo must restore the full issued state, without a failed-validation history entry"
    );
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), true, || Ok(()))
        .unwrap();
    assert_eq!(exported(), after);
    assert!(
        f.bridge
            .apply_native_mutation_at(
                &f.engine,
                &receipt.owner,
                receipt.revision,
                "drawing_set_document",
                &serde_json::to_value(drawing).unwrap(),
                || Ok(())
            )
            .is_err()
    );
    assert_eq!(exported(), after);
}

#[test]
fn annotation_edits_delete_and_creation_restore_exact_released_history() {
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let f = Fixture::new();
    let seeded = tests::document();
    f.bridge
        .apply_native_mutation(
            &f.engine,
            &f.owner(),
            "drawing_set_document",
            &serde_json::to_value(&seeded).unwrap(),
            || Ok(()),
        )
        .unwrap();
    let drawing = f.engine.drawing_snapshot();
    let exported =
        || parse_engine_envelope(f.engine.engine_call("project_export_model", "")).unwrap();
    let before = exported();
    let receipt = f
        .bridge
        .native_document_receipt(&f.engine, &f.owner())
        .unwrap();
    let selection = draft::Selection {
        sheet_id: 1,
        annotation_id: 1,
    };
    let mut edit = draft::Draft::new(&drawing, selection).unwrap();
    edit.note("Caf\u{e9} \u{96f6}\u{4ef6}\nEdited note".into())
        .unwrap();
    edit.move_note([80., 90.], [297., 210.]).unwrap();
    let next = edit.apply(&drawing).unwrap();
    f.bridge
        .apply_native_mutation_at(
            &f.engine,
            &receipt.owner,
            receipt.revision,
            "drawing_set_document",
            &serde_json::to_value(&next).unwrap(),
            || Ok(()),
        )
        .unwrap();
    let after = exported();
    assert_ne!(after, before);
    assert_eq!(f.engine.drawing_snapshot(), next);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), false, || Ok(()))
        .unwrap();
    assert_eq!(exported(), before);
    assert_eq!(f.engine.drawing_snapshot(), drawing);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), true, || Ok(()))
        .unwrap();
    assert_eq!(exported(), after);
    let stale = f.bridge.apply_native_mutation_at(
        &f.engine,
        &receipt.owner,
        receipt.revision,
        "drawing_set_document",
        &serde_json::to_value(&drawing).unwrap(),
        || Ok(()),
    );
    assert!(stale.is_err());
    assert_eq!(exported(), after);
    let deleted = draft::Draft::new(&next, selection)
        .unwrap()
        .delete(&next)
        .unwrap();
    f.bridge
        .apply_native_mutation(
            &f.engine,
            &f.owner(),
            "drawing_set_document",
            &serde_json::to_value(&deleted).unwrap(),
            || Ok(()),
        )
        .unwrap();
    assert_eq!(f.engine.drawing_snapshot(), deleted);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), false, || Ok(()))
        .unwrap();
    assert_eq!(exported(), after);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), false, || Ok(()))
        .unwrap();
    assert_eq!(exported(), before);
    f.bridge
        .apply_native_mutation(
            &f.engine,
            &f.owner(),
            "drawing_add_note",
            &json!({"sheet_id":1,"text":"Created in native","position":[70.,80.]}),
            || Ok(()),
        )
        .unwrap();
    let created = f.engine.drawing_snapshot();
    assert_eq!(
        created.sheets[0].release.status,
        nbcad_sketch::DrawingReleaseStatus::Draft
    );
    assert_eq!(created.sheets[1], drawing.sheets[1]);
    assert_eq!(
        &created.sheets[0].annotations[..2],
        &drawing.sheets[0].annotations[..]
    );
    let after_create = exported();
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), false, || Ok(()))
        .unwrap();
    assert_eq!(exported(), before);
    f.bridge
        .apply_native_history(&f.engine, &f.owner(), true, || Ok(()))
        .unwrap();
    assert_eq!(exported(), after_create);
}
