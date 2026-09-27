//! Native inspector strings are transient; values are applied to typed shared
//! annotations and validated by DrawingDocumentDto before a commit.
use super::{draft::Draft, *};
use nbcad_sketch::DrawingAnnotationDto;

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Text,
    Multiline,
    Number,
    Mode,
}
pub(super) struct Field {
    pub label: &'static str,
    pub kind: Kind,
    pub text: String,
    pub original: String,
}
fn field(label: &'static str, kind: Kind, value: impl ToString) -> Field {
    let text = value.to_string();
    Field {
        label,
        kind,
        original: text.clone(),
        text,
    }
}
pub(super) fn note_creation(position: [f64; 2]) -> Vec<Field> {
    vec![
        field("Note text", Kind::Multiline, "NOTE"),
        field("Paper X (mm)", Kind::Number, position[0]),
        field("Paper Y (mm)", Kind::Number, position[1]),
    ]
}
pub(super) fn from_annotation(annotation: &DrawingAnnotationDto) -> Vec<Field> {
    match annotation {
        DrawingAnnotationDto::Note { text, position, .. } => vec![
            field("Note text", Kind::Multiline, text),
            field("Paper X (mm)", Kind::Number, position[0]),
            field("Paper Y (mm)", Kind::Number, position[1]),
        ],
        DrawingAnnotationDto::LinearDimension {
            mode,
            offset,
            precision,
            prefix,
            suffix,
            ..
        } => vec![
            field(
                "Dimension mode",
                Kind::Mode,
                match mode {
                    DrawingLinearDimensionMode::Aligned => "aligned",
                    DrawingLinearDimensionMode::Horizontal => "horizontal",
                    DrawingLinearDimensionMode::Vertical => "vertical",
                },
            ),
            field("Offset (paper mm)", Kind::Number, offset),
            field("Precision", Kind::Number, precision),
            field("Prefix", Kind::Text, prefix),
            field("Suffix", Kind::Text, suffix),
        ],
        _ => vec![],
    }
}
pub(super) fn dirty(fields: &[Field]) -> bool {
    fields.iter().any(|f| f.text != f.original)
}
fn number(fields: &[Field], index: usize) -> Result<f64, String> {
    let f = fields.get(index).ok_or("Drawing field was removed")?;
    let value = f
        .text
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("Enter a number for {}", f.label))?;
    if !value.is_finite() {
        return Err(format!("{} must be finite", f.label));
    }
    Ok(value)
}
pub(super) fn note_request(sheet_id: u64, fields: &[Field]) -> Result<AddNote, String> {
    Ok(AddNote {
        sheet_id,
        text: fields.first().ok_or("Enter note text")?.text.clone(),
        position: [number(fields, 1)?, number(fields, 2)?],
    })
}
pub(super) fn apply(draft: &mut Draft, fields: &[Field]) -> Result<(), String> {
    match draft.annotation() {
        DrawingAnnotationDto::Note { .. } => {
            let note = note_request(0, fields)?;
            draft.note(note.text)?;
            draft.set_note_position(note.position)?;
        }
        DrawingAnnotationDto::LinearDimension { presentation, .. } => {
            let presentation = presentation.clone();
            let mode = match fields
                .first()
                .ok_or("Choose a dimension mode")?
                .text
                .as_str()
            {
                "aligned" => DrawingLinearDimensionMode::Aligned,
                "horizontal" => DrawingLinearDimensionMode::Horizontal,
                "vertical" => DrawingLinearDimensionMode::Vertical,
                _ => return Err("Choose a dimension mode".into()),
            };
            let precision = fields
                .get(2)
                .ok_or("Enter precision")?
                .text
                .trim()
                .parse::<u8>()
                .map_err(|_| "Precision must be an integer from 0 to 6")?;
            draft.linear(
                mode,
                number(fields, 1)?,
                precision,
                fields.get(3).ok_or("Prefix field missing")?.text.clone(),
                fields.get(4).ok_or("Suffix field missing")?.text.clone(),
                presentation,
            )?;
        }
        _ => return Err("This annotation's inspector has not been migrated yet".into()),
    }
    Ok(())
}
