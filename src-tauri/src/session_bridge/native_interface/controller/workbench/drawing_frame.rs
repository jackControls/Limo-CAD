//! Presentation of the shared sheet metadata, matching SheetFrame and
//! drawing/titleBlock.ts. These primitives never change the saved document.
use super::{Fill, Ink, Label, LabelAlign, Segment, annotations::Art};
use nbcad_sketch::*;

const MIN_TEXT: f64 = 1.8;

fn stroke(art: &mut Art, a: [f64; 2], b: [f64; 2], width: f64) {
    art.segments.push(Segment {
        x1: a[0] as f32,
        y1: a[1] as f32,
        x2: b[0] as f32,
        y2: b[1] as f32,
        hidden: false,
        width_mm: width as f32,
        arrow: false,
        ink: Ink::Frame,
    });
}
fn rectangle(art: &mut Art, x: f64, y: f64, w: f64, h: f64, width: f64) {
    let points = [[x, y], [x + w, y], [x + w, y + h], [x, y + h], [x, y]];
    for pair in points.windows(2) {
        stroke(art, pair[0], pair[1], width);
    }
}
fn label(art: &mut Art, x: f64, baseline: f64, text: String, size: f64, overflow: bool) {
    let width = (advance(&text) * size).max(size);
    art.labels.push(Label {
        x: (x + width * 0.5) as f32,
        y: (baseline - size * 0.4) as f32,
        text,
        width_mm: width as f32,
        height_mm: (size * 1.18) as f32,
        text_height_mm: size as f32,
        align: LabelAlign::Start,
        ink: if overflow {
            Ink::Overflow
        } else {
            Ink::FrameText
        },
        ..Default::default()
    });
}

/// Conservative font-independent advances used by the React title layout.
fn advance(text: &str) -> f64 {
    text.chars()
        .map(|c| {
            if c.is_whitespace() {
                0.33
            } else if "ilI.,:;!|'`".contains(c) {
                0.32
            } else if "MW@%".contains(c) {
                0.95
            } else if c.is_ascii_uppercase() {
                0.75
            } else if c.is_ascii() {
                0.65
            } else {
                1.
            }
        })
        .sum()
}
fn wrap(text: &str, max_advance: f64) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = Vec::new();
    for paragraph in normalized.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && advance(&format!("{line} {word}")) <= max_advance {
                line.push(' ');
                line.push_str(word);
                continue;
            }
            if !line.is_empty() {
                lines.push(std::mem::take(&mut line));
            }
            for c in word.chars() {
                if !line.is_empty() && advance(&line) + advance(&c.to_string()) > max_advance {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(c);
            }
        }
        lines.push(line);
    }
    lines
}
fn cell(art: &mut Art, text: String, bounds: [f64; 4], requested_size: f64) {
    let [x, y, w, h] = bounds;
    let maximum = if requested_size.is_finite() {
        requested_size.clamp(MIN_TEXT, 5.)
    } else {
        MIN_TEXT
    };
    let mut size = maximum;
    loop {
        let lines = wrap(&text, (w - 3.) / size);
        if size + (lines.len().saturating_sub(1) as f64) * size * 1.2 <= h - 2. + 1e-8
            && lines
                .iter()
                .all(|line| advance(line) * size <= w - 3. + 1e-8)
        {
            for (index, text) in lines.into_iter().enumerate() {
                label(
                    art,
                    x + 1.5,
                    y + 1. + size * 0.85 + index as f64 * size * 1.2,
                    text,
                    size,
                    false,
                );
            }
            return;
        }
        if size <= MIN_TEXT {
            break;
        }
        size = (size - 0.1).max(MIN_TEXT);
    }
    // Full source text remains in Sheet setup. Never silently discard suffixes
    // or make engineering text smaller than the existing readable minimum.
    label(
        art,
        x + 1.5,
        y + 1. + MIN_TEXT,
        "! TEXT TOO LONG".into(),
        MIN_TEXT,
        true,
    );
}
fn or_dash(text: &str) -> &str {
    if text.is_empty() { "—" } else { text }
}
fn tolerance(note: &DrawingToleranceNoteDto) -> &str {
    match note.preset {
        DrawingTolerancePreset::None => "TOLERANCES: AS SPECIFIED",
        DrawingTolerancePreset::Iso2768Fine => "GENERAL TOLERANCES ISO 2768-f",
        DrawingTolerancePreset::Iso2768Medium => "GENERAL TOLERANCES ISO 2768-m",
        DrawingTolerancePreset::Iso2768Coarse => "GENERAL TOLERANCES ISO 2768-c",
        DrawingTolerancePreset::Iso2768VeryCoarse => "GENERAL TOLERANCES ISO 2768-v",
        DrawingTolerancePreset::AnsiDecimal => {
            "UNLESS OTHERWISE SPECIFIED: .X ±.1  .XX ±.01  .XXX ±.005"
        }
        DrawingTolerancePreset::Custom => {
            if note.custom.trim().is_empty() {
                "TOLERANCES: AS SPECIFIED"
            } else {
                note.custom.trim()
            }
        }
    }
}
fn format_label(format: DrawingSheetFormat) -> &'static str {
    match format {
        DrawingSheetFormat::A0 => "ISO A0",
        DrawingSheetFormat::A1 => "ISO A1",
        DrawingSheetFormat::A2 => "ISO A2",
        DrawingSheetFormat::A3 => "ISO A3",
        DrawingSheetFormat::A4 => "ISO A4",
        DrawingSheetFormat::Letter => "ANSI A",
        DrawingSheetFormat::AnsiB => "ANSI B",
        DrawingSheetFormat::AnsiC => "ANSI C",
        DrawingSheetFormat::AnsiD => "ANSI D",
        DrawingSheetFormat::AnsiE => "ANSI E",
    }
}
fn grid(art: &mut Art, position: [f64; 2], width: f64, rows: usize, columns: &[f64], line: f64) {
    let [x, y] = position;
    let height = rows as f64 * 6.;
    art.fills.push(Fill {
        x: x as f32,
        y: y as f32,
        width: width as f32,
        height: height as f32,
        round: false,
    });
    rectangle(art, x, y, width, height, line);
    for row in 1..rows {
        stroke(
            art,
            [x, y + row as f64 * 6.],
            [x + width, y + row as f64 * 6.],
            line,
        );
    }
    for column in columns {
        stroke(art, [x + column, y], [x + column, y + height], line);
    }
}

pub(super) fn render(sheet: &DrawingSheetDto, paper_width: f64, paper_height: f64) -> Art {
    let mut art = Art::default();
    rectangle(
        &mut art,
        5.,
        5.,
        paper_width - 10.,
        paper_height - 10.,
        sheet.style.visible.width_mm,
    );
    let width = 180_f64.min(paper_width - 10.);
    let x = paper_width - width - 5.;
    let y = paper_height - 49.;
    rectangle(&mut art, x, y, width, 44., sheet.style.dimension.width_mm);
    for [x1, y1, x2, y2] in [
        [0., 14., 1., 14.],
        [0., 22., 1., 22.],
        [0., 28., 1., 28.],
        [0., 38., 1., 38.],
        [0.64, 0., 0.64, 14.],
        [0.7, 22., 0.7, 28.],
        [0.45, 28., 0.45, 38.],
        [1. / 3., 38., 1. / 3., 44.],
        [2. / 3., 38., 2. / 3., 44.],
    ] {
        stroke(
            &mut art,
            [x + x1 * width, y + y1],
            [x + x2 * width, y + y2],
            sheet.style.dimension.width_mm,
        );
    }
    let title = &sheet.title_block;
    let small = sheet.style.small_text_height_mm;
    let mut add = |text: String, left: f64, top: f64, w: f64, h: f64, size: f64| {
        cell(
            &mut art,
            text,
            [x + left * width, y + top, w * width, h],
            size,
        );
    };
    add(
        if title.title.is_empty() {
            sheet.name.clone()
        } else {
            title.title.clone()
        },
        0.,
        0.,
        0.64,
        9.,
        sheet.style.text_height_mm.min(3.5),
    );
    add(
        format!("DRAWING: {}", or_dash(&title.drawing_number)),
        0.,
        9.,
        0.64,
        5.,
        small,
    );
    add(format!("SHEET: {}", sheet.name), 0.64, 0., 0.36, 9., small);
    add(
        format!(
            "{} · {}",
            format_label(sheet.format),
            if sheet.projection_method == DrawingProjectionMethod::FirstAngle {
                "1ST ANGLE"
            } else {
                "3RD ANGLE"
            }
        ),
        0.64,
        9.,
        0.36,
        5.,
        small,
    );
    add(
        tolerance(&sheet.tolerance_note).into(),
        0.,
        14.,
        1.,
        8.,
        small,
    );
    add(
        format!("COMPANY: {}", or_dash(&title.company)),
        0.,
        22.,
        0.7,
        6.,
        small,
    );
    add(
        format!("REV {}", or_dash(&title.revision)),
        0.7,
        22.,
        0.3,
        6.,
        small,
    );
    add(
        format!("MATERIAL: {}", or_dash(&title.material)),
        0.,
        28.,
        0.45,
        10.,
        small,
    );
    add(
        format!("FINISH: {}", or_dash(&title.finish)),
        0.45,
        28.,
        0.55,
        10.,
        small,
    );
    add(
        format!("DRAWN: {}", or_dash(&title.author)),
        0.,
        38.,
        1. / 3.,
        6.,
        small,
    );
    add(
        format!("CHECKED: {}", or_dash(&title.checked_by)),
        1. / 3.,
        38.,
        1. / 3.,
        6.,
        small,
    );
    add(
        format!("APPROVED: {}", or_dash(&title.approved_by)),
        2. / 3.,
        38.,
        1. / 3.,
        6.,
        small,
    );

    if let Some([x, y]) = sheet.revision_table_position {
        grid(
            &mut art,
            [x, y],
            112.,
            sheet.revisions.len() + 1,
            &[12., 28., 82.],
            sheet.style.dimension.width_mm,
        );
        for (dx, text) in [(2., "REV"), (14., "DATE"), (30., "DESCRIPTION / APPROVAL")] {
            label(&mut art, x + dx, y + 4.2, text.into(), small, false);
        }
        for (i, revision) in sheet.revisions.iter().enumerate() {
            let baseline = y + (i + 1) as f64 * 6. + 4.2;
            let description = if !revision.description.is_empty() {
                &revision.description
            } else {
                or_dash(&revision.change_order)
            };
            let description = if revision.approved_by.is_empty() {
                description.into()
            } else {
                format!("{description} · {}", revision.approved_by)
            };
            for (dx, text) in [
                (2., revision.revision.clone()),
                (14., revision.date.clone()),
                (30., description),
            ] {
                label(&mut art, x + dx, baseline, text, small, false);
            }
        }
    }
    if let Some([x, y]) = sheet.bom_table_position {
        grid(
            &mut art,
            [x, y],
            132.,
            sheet.bom.len() + 1,
            &[12., 40., 100., 112.],
            sheet.style.dimension.width_mm,
        );
        for (dx, text) in [
            (2., "ITEM"),
            (14., "PART"),
            (42., "DESCRIPTION"),
            (102., "QTY"),
            (114., "MATERIAL"),
        ] {
            label(&mut art, x + dx, y + 4.2, text.into(), small, false);
        }
        for (i, item) in sheet.bom.iter().enumerate() {
            let quantity = format!("{:.3}", item.quantity)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned();
            for (dx, text) in [
                (2., item.item_number.clone()),
                (14., or_dash(&item.part_number).into()),
                (42., item.description.clone()),
                (102., quantity),
                (114., or_dash(&item.material).into()),
            ] {
                label(
                    &mut art,
                    x + dx,
                    y + (i + 1) as f64 * 6. + 4.2,
                    text,
                    small,
                    false,
                );
            }
        }
    }
    art
}

#[cfg(test)]
#[path = "drawing_frame/tests.rs"]
mod tests;
