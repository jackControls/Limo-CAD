use super::*;
use serde_json::json;

fn sheet() -> DrawingSheetDto {
    serde_json::from_value(json!({"id":1,"name":"Production","format":"a4","orientation":"landscape",
        "title_block":{"title":"Café 零件","drawing_number":"DWG-7","revision":"B","author":"Engineer","checked_by":"Checker","approved_by":"Approver","company":"Company","material":"Aluminium","finish":"Deburr"},
        "tolerance_note":{"preset":"custom","custom":"  Company tolerance 0.25 mm  "}})).unwrap()
}
fn has(art: &Art, text: &str) -> bool {
    art.labels.iter().any(|label| label.text == text)
}

#[test]
fn sheet_frame_retains_all_title_metadata_and_uses_shared_paper_bounds() {
    let sheet = sheet();
    let original = sheet.clone();
    for (width, height) in [(297., 210.), (279.4, 431.8), (1189., 841.)] {
        let art = render(&sheet, width, height);
        assert_eq!(art.segments.len(), 17);
        let border = &art.segments[..4];
        assert_eq!((border[0].x1, border[0].y1), (5., 5.));
        assert!((border[1].x1 as f64 - (width - 5.)).abs() < 1e-4);
        assert!((border[2].y1 as f64 - (height - 5.)).abs() < 1e-4);
        for text in [
            "Café 零件",
            "DRAWING: DWG-7",
            "SHEET: Production",
            "ISO A4 · 1ST ANGLE",
            "Company tolerance 0.25 mm",
            "COMPANY: Company",
            "REV B",
            "MATERIAL: Aluminium",
            "FINISH: Deburr",
            "DRAWN: Engineer",
            "CHECKED: Checker",
            "APPROVED: Approver",
        ] {
            assert!(has(&art, text), "missing {text}");
        }
        assert!(
            art.labels
                .iter()
                .all(|l| l.align == LabelAlign::Start && l.text_height_mm >= 1.8)
        );
        assert!(art.labels.iter().all(|l| {
            l.x - l.width_mm * 0.5 >= (width - 185.) as f32
                && l.x + l.width_mm * 0.5 <= (width - 5.) as f32
                && l.y >= (height - 49.) as f32
                && l.y <= (height - 5.) as f32
        }));
    }
    assert_eq!(sheet, original);
}

#[test]
fn title_wraps_unicode_and_reports_overflow_without_altering_source() {
    assert_eq!(wrap("one  two\r\n零件", 20.), vec!["one two", "零件"]);
    assert_eq!(wrap("零件加工", 2.), vec!["零件", "加工"]);
    let mut art = Art::default();
    cell(
        &mut art,
        "Long component title split across two rows".into(),
        [10., 20., 65., 9.],
        3.5,
    );
    assert!(art.labels.len() > 1);
    assert!(
        art.labels
            .iter()
            .all(|l| l.ink != Ink::Overflow && l.x + l.width_mm * 0.5 <= 73.51)
    );
    let mut sheet = sheet();
    sheet.title_block.title = "零".repeat(2048);
    let before = sheet.clone();
    let art = render(&sheet, 297., 210.);
    assert!(
        art.labels
            .iter()
            .any(|l| l.text == "! TEXT TOO LONG" && l.ink == Ink::Overflow)
    );
    assert_eq!(sheet, before);
}

#[test]
fn revision_and_bom_tables_keep_saved_positions_complete_rows_and_optional_visibility() {
    let mut sheet = sheet();
    sheet.revisions = serde_json::from_value(json!([
        {"id":1,"revision":"A","date":"2026-09-25","description":"Initial","approved_by":"QA"},
        {"id":2,"revision":"B","date":"2026-09-26","description":"","change_order":"ECO-7"}
    ]))
    .unwrap();
    sheet.bom=serde_json::from_value(json!([
        {"id":1,"item_number":"1","part_number":"P-7","description":"Plate 零件","quantity":2.5,"material":"Al"},
        {"id":2,"item_number":"2","description":"Pin","quantity":10.,"material":"Steel"}
    ])).unwrap();
    assert_eq!(render(&sheet, 297., 210.).fills.len(), 0);
    sheet.revision_table_position = Some([12., 20.]);
    sheet.bom_table_position = Some([150., 20.]);
    let before = sheet.clone();
    let art = render(&sheet, 297., 210.);
    assert_eq!(art.fills.len(), 2);
    assert_eq!(
        (
            art.fills[0].x,
            art.fills[0].y,
            art.fills[0].width,
            art.fills[0].height
        ),
        (12., 20., 112., 18.)
    );
    assert_eq!(
        (
            art.fills[1].x,
            art.fills[1].y,
            art.fills[1].width,
            art.fills[1].height
        ),
        (150., 20., 132., 18.)
    );
    for text in [
        "DESCRIPTION / APPROVAL",
        "Initial · QA",
        "ECO-7",
        "Plate 零件",
        "P-7",
        "2.5",
        "10",
        "MATERIAL",
    ] {
        assert!(has(&art, text), "missing table value {text}");
    }
    let part = art.labels.iter().find(|l| l.text == "P-7").unwrap();
    assert!((part.x - part.width_mm * 0.5 - 164.).abs() < 1e-4);
    assert_eq!(sheet, before);
}

#[test]
fn standard_projection_and_tolerance_text_follow_shared_sheet_settings() {
    let mut sheet = sheet();
    sheet.format = DrawingSheetFormat::Letter;
    sheet.projection_method = DrawingProjectionMethod::ThirdAngle;
    sheet.tolerance_note = DrawingToleranceNoteDto {
        preset: DrawingTolerancePreset::AnsiDecimal,
        custom: String::new(),
    };
    let art = render(&sheet, 279.4, 215.9);
    assert!(has(&art, "ANSI A · 3RD ANGLE"));
    assert!(art.labels.iter().any(|l| l.text.contains(".XXX ±.005")));
    sheet.tolerance_note.preset = DrawingTolerancePreset::None;
    assert!(has(&render(&sheet, 297., 210.), "TOLERANCES: AS SPECIFIED"));
}
