use super::super::{anchors, tests as fixtures};
use super::*;
use nbcad_interface::KeyChord;

pub(in super::super) fn document() -> DrawingDocumentDto {
    let mut document = fixtures::document();
    let p = fixtures::projection();
    document.sheets[0]
        .annotations
        .push(DrawingAnnotationDto::LinearDimension {
            id: 4,
            view_id: 1,
            first: anchors::endpoint_ref(&p.anchors[4], &p),
            second: anchors::endpoint_ref(&p.anchors[5], &p),
            mode: DrawingLinearDimensionMode::Horizontal,
            offset: 18.,
            precision: 3,
            prefix: "Saved ".into(),
            suffix: " exact".into(),
            presentation: DrawingDimensionPresentationDto {
                tolerance: DrawingDimensionToleranceDto {
                    mode: DrawingDimensionToleranceMode::Limits,
                    upper: 0.12,
                    lower: -0.04,
                },
                basic: false,
                reference: true,
                fit_class: "H7".into(),
                dual_units: Some(DrawingDualUnitDto {
                    unit: DrawingSecondaryUnit::Centimetre,
                    precision: 4,
                    placement: DrawingDualUnitPlacement::Stacked,
                }),
            },
        });
    document.next_annotation_id = 5;
    document.validate().unwrap();
    document
}
pub(in super::super) fn linear(document: &DrawingDocumentDto) -> Draft {
    Draft::new(
        document,
        super::super::draft::Selection {
            sheet_id: 1,
            annotation_id: 4,
        },
    )
    .unwrap()
}
pub(in super::super) fn set(fields: &mut [Field], id: Id, value: &str) {
    edit(fields, id, &ControlInput::SetValue(value.into())).unwrap();
}

#[test]
fn loaded_presentation_round_trips_and_changes_only_the_selected_metadata() {
    let before = document();
    let mut draft = linear(&before);
    let mut fields = from_annotation(draft.annotation());
    apply(&mut draft, &fields).unwrap();
    assert_eq!(
        draft.apply(&before).unwrap(),
        before,
        "Opening and applying must preserve all existing metadata and issued state"
    );
    set(&mut fields, Id::Tolerance, "symmetric");
    set(&mut fields, Id::Upper, "0.25");
    set(&mut fields, Id::Basic, "true");
    set(&mut fields, Id::Fit, "h6");
    set(&mut fields, Id::DualUnit, "inch");
    set(&mut fields, Id::DualPrecision, "6");
    set(&mut fields, Id::DualPlacement, "bracketed");
    apply(&mut draft, &fields).unwrap();
    let after = draft.apply(&before).unwrap();
    let mut expected = before.clone();
    if let DrawingAnnotationDto::LinearDimension {
        presentation: p, ..
    } = &mut expected.sheets[0].annotations[2]
    {
        p.tolerance.mode = DrawingDimensionToleranceMode::Symmetric;
        p.tolerance.upper = 0.25;
        p.basic = true;
        p.reference = false;
        p.fit_class = "h6".into();
        p.dual_units = Some(DrawingDualUnitDto {
            unit: DrawingSecondaryUnit::Inch,
            precision: 6,
            placement: DrawingDualUnitPlacement::Bracketed,
        });
    }
    expected.sheets[0].release.status = DrawingReleaseStatus::Draft;
    assert_eq!(
        after, expected,
        "Anchors, offset, untouched tolerance, other annotations and other sheets must remain exact"
    );
}

#[test]
fn conditional_fields_keep_identity_and_checkbox_keyboard_matches_set_value() {
    let before = document();
    let draft = linear(&before);
    let mut fields = from_annotation(draft.annotation());
    let ids: Vec<_> = fields.iter().map(|f| f.id).collect();
    set(&mut fields, Id::Tolerance, "none");
    set(&mut fields, Id::Dual, "false");
    let shown = |fields: &[Field], id| visible(fields).iter().any(|i| fields[*i].id == id);
    assert!(!shown(&fields, Id::Upper));
    assert!(!shown(&fields, Id::DualUnit));
    assert!(
        edit(
            &mut fields,
            Id::DualUnit,
            &ControlInput::SetValue("inch".into())
        )
        .is_err()
    );
    edit(
        &mut fields,
        Id::Dual,
        &ControlInput::Key(KeyChord::plain(" ")),
    )
    .unwrap();
    assert!(shown(&fields, Id::DualUnit));
    assert_eq!(
        text(&fields, Id::DualUnit).unwrap(),
        "centimetre",
        "Uncommitted hide/show preserves loaded values"
    );
    edit(&mut fields, Id::Basic, &ControlInput::Click).unwrap();
    assert_eq!(text(&fields, Id::Basic).unwrap(), "true");
    assert_eq!(text(&fields, Id::Reference).unwrap(), "false");
    edit(
        &mut fields,
        Id::Reference,
        &ControlInput::Key(KeyChord::plain("Enter")),
    )
    .unwrap();
    assert_eq!(text(&fields, Id::Basic).unwrap(), "false");
    assert_eq!(text(&fields, Id::Reference).unwrap(), "true");
    edit(
        &mut fields,
        Id::Tolerance,
        &ControlInput::Key(KeyChord::plain("ArrowDown")),
    )
    .unwrap();
    assert_eq!(text(&fields, Id::Tolerance).unwrap(), "symmetric");
    assert!(shown(&fields, Id::Upper));
    set(&mut fields, Id::Upper, "not a number");
    set(&mut fields, Id::Tolerance, "none");
    assert!(
        shown(&fields, Id::Upper),
        "Inactive invalid edited text must stay reachable for repair"
    );
    assert_eq!(fields.iter().map(|f| f.id).collect::<Vec<_>>(), ids);
    let copy = ControlInput::Key(KeyChord {
        key: "a".into(),
        ctrl: true,
        ..Default::default()
    });
    for id in [Id::Prefix, Id::Tolerance, Id::Basic] {
        assert!(!edit(&mut fields, id, &copy).unwrap());
    }
}

#[test]
fn invalid_presentation_is_rejected_before_editing_the_typed_draft() {
    let before = document();
    for (id, value) in [
        (Id::Upper, "NaN".into()),
        (Id::Lower, "-inf".into()),
        (Id::Precision, "7".into()),
        (Id::DualPrecision, "7".into()),
        (Id::DualPrecision, "2.5".into()),
        (Id::Fit, "\u{96f6}".repeat(65)),
    ] {
        let mut draft = linear(&before);
        let original = draft.annotation().clone();
        let mut fields = from_annotation(draft.annotation());
        set(&mut fields, id, &value);
        assert!(apply(&mut draft, &fields).is_err(), "Rejected field {id:?}");
        assert_eq!(draft.annotation(), &original);
        assert_eq!(draft.apply(&before).unwrap(), before);
    }
    let mut draft = linear(&before);
    let mut fields = from_annotation(draft.annotation());
    set(&mut fields, Id::Fit, &"\u{96f6}".repeat(64));
    apply(&mut draft, &fields).unwrap();
    assert!(
        draft.apply(&before).is_ok(),
        "Fit validation counts characters, not UTF-8 bytes"
    );
    for id in [Id::Tolerance, Id::DualUnit, Id::DualPlacement, Id::Basic] {
        let old = text(&fields, id).unwrap().to_owned();
        assert!(edit(&mut fields, id, &ControlInput::SetValue("unknown".into())).is_err());
        assert_eq!(text(&fields, id).unwrap(), old);
    }
}

#[test]
fn shared_drawing_validation_remains_authoritative_for_presentation_limits() {
    for variant in 0..4 {
        let mut invalid = document();
        if let DrawingAnnotationDto::LinearDimension {
            presentation: p,
            precision,
            ..
        } = &mut invalid.sheets[0].annotations[2]
        {
            match variant {
                0 => p.tolerance.upper = f64::NAN,
                1 => p.fit_class = "x".repeat(65),
                2 => p.dual_units.as_mut().unwrap().precision = 7,
                _ => *precision = 7,
            }
        }
        assert!(invalid.validate().is_err());
    }
}
