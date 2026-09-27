//! Native inspector strings are transient; values are applied to typed shared
//! annotations and validated by DrawingDocumentDto before a commit.
use super::{draft::Draft, *};
use nbcad_interface::{ChoiceOption, ControlInput};
use nbcad_sketch::*;

/// Identity is independent of pagination and conditional presentation fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Id {
    Note,
    X,
    Y,
    Mode,
    Offset,
    Precision,
    Prefix,
    Suffix,
    Tolerance,
    Upper,
    Lower,
    Basic,
    Reference,
    Fit,
    Dual,
    DualUnit,
    DualPrecision,
    DualPlacement,
}
#[derive(Clone, Copy)]
pub(super) enum Choice {
    Mode,
    Tolerance,
    Unit,
    Placement,
}
#[derive(Clone, Copy)]
pub(super) enum Kind {
    Text,
    Multiline,
    Number,
    Choice(Choice),
    Toggle,
}
pub(super) struct Field {
    pub id: Id,
    pub label: &'static str,
    pub kind: Kind,
    pub text: String,
    pub original: String,
}
impl Field {
    pub fn options(&self) -> Option<Vec<ChoiceOption>> {
        let pairs: &[(&str, &str)] = match self.kind {
            Kind::Choice(Choice::Mode) => &[
                ("aligned", "Aligned"),
                ("horizontal", "Horizontal"),
                ("vertical", "Vertical"),
            ],
            Kind::Choice(Choice::Tolerance) => &[
                ("none", "None"),
                ("symmetric", "Plus/minus"),
                ("deviation", "Unequal deviation"),
                ("limits", "Limit dimensions"),
            ],
            Kind::Choice(Choice::Unit) => &[
                ("millimetre", "Millimetre"),
                ("centimetre", "Centimetre"),
                ("inch", "Inch"),
            ],
            Kind::Choice(Choice::Placement) => {
                &[("bracketed", "Bracketed"), ("stacked", "Stacked")]
            }
            _ => return None,
        };
        Some(
            pairs
                .iter()
                .map(|(value, label)| ChoiceOption {
                    value: (*value).into(),
                    label: (*label).into(),
                    disabled: false,
                })
                .collect(),
        )
    }
    pub fn caption(&self) -> String {
        if matches!(self.kind, Kind::Toggle) {
            return self.label.into();
        }
        self.options()
            .and_then(|options| {
                options
                    .into_iter()
                    .find(|o| o.value == self.text)
                    .map(|o| o.label)
            })
            .unwrap_or_else(|| self.text.clone())
    }
}
fn field(id: Id, label: &'static str, kind: Kind, value: impl ToString) -> Field {
    let text = value.to_string();
    Field {
        id,
        label,
        kind,
        original: text.clone(),
        text,
    }
}
pub(super) fn note_creation(position: [f64; 2]) -> Vec<Field> {
    vec![
        field(Id::Note, "Note text", Kind::Multiline, "NOTE"),
        field(Id::X, "Paper X (mm)", Kind::Number, position[0]),
        field(Id::Y, "Paper Y (mm)", Kind::Number, position[1]),
    ]
}
pub(super) fn from_annotation(annotation: &DrawingAnnotationDto) -> Vec<Field> {
    match annotation {
        DrawingAnnotationDto::Note { text, position, .. } => {
            let mut fields = note_creation(*position);
            fields[0].text = text.clone();
            fields[0].original = text.clone();
            fields
        }
        DrawingAnnotationDto::LinearDimension {
            mode,
            offset,
            precision,
            prefix,
            suffix,
            presentation,
            ..
        } => {
            let mut fields = vec![
                field(
                    Id::Mode,
                    "Dimension mode",
                    Kind::Choice(Choice::Mode),
                    match mode {
                        DrawingLinearDimensionMode::Aligned => "aligned",
                        DrawingLinearDimensionMode::Horizontal => "horizontal",
                        DrawingLinearDimensionMode::Vertical => "vertical",
                    },
                ),
                field(Id::Offset, "Offset (paper mm)", Kind::Number, offset),
                field(Id::Precision, "Precision", Kind::Number, precision),
                field(Id::Prefix, "Prefix", Kind::Text, prefix),
                field(Id::Suffix, "Suffix", Kind::Text, suffix),
            ];
            let p = presentation;
            let dual = p.dual_units.clone().unwrap_or(DrawingDualUnitDto {
                unit: DrawingSecondaryUnit::Inch,
                precision: 3,
                placement: DrawingDualUnitPlacement::Bracketed,
            });
            fields.extend([
                field(
                    Id::Tolerance,
                    "Tolerance mode",
                    Kind::Choice(Choice::Tolerance),
                    match p.tolerance.mode {
                        DrawingDimensionToleranceMode::None => "none",
                        DrawingDimensionToleranceMode::Symmetric => "symmetric",
                        DrawingDimensionToleranceMode::Deviation => "deviation",
                        DrawingDimensionToleranceMode::Limits => "limits",
                    },
                ),
                field(
                    Id::Upper,
                    "Upper tolerance",
                    Kind::Number,
                    p.tolerance.upper,
                ),
                field(
                    Id::Lower,
                    "Lower tolerance",
                    Kind::Number,
                    p.tolerance.lower,
                ),
                field(Id::Basic, "Basic dimension", Kind::Toggle, p.basic),
                field(
                    Id::Reference,
                    "Reference dimension",
                    Kind::Toggle,
                    p.reference,
                ),
                field(Id::Fit, "Fit class", Kind::Text, &p.fit_class),
                field(Id::Dual, "Dual units", Kind::Toggle, p.dual_units.is_some()),
                field(
                    Id::DualUnit,
                    "Secondary unit",
                    Kind::Choice(Choice::Unit),
                    match dual.unit {
                        DrawingSecondaryUnit::Millimetre => "millimetre",
                        DrawingSecondaryUnit::Centimetre => "centimetre",
                        DrawingSecondaryUnit::Inch => "inch",
                    },
                ),
                field(
                    Id::DualPrecision,
                    "Dual precision",
                    Kind::Number,
                    dual.precision,
                ),
                field(
                    Id::DualPlacement,
                    "Dual placement",
                    Kind::Choice(Choice::Placement),
                    match dual.placement {
                        DrawingDualUnitPlacement::Bracketed => "bracketed",
                        DrawingDualUnitPlacement::Stacked => "stacked",
                    },
                ),
            ]);
            fields
        }
        _ => vec![],
    }
}
pub(super) fn dirty(fields: &[Field]) -> bool {
    fields.iter().any(|f| f.text != f.original)
}
fn get(fields: &[Field], id: Id) -> Result<&Field, String> {
    fields
        .iter()
        .find(|f| f.id == id)
        .ok_or("Drawing field was removed".into())
}
fn text(fields: &[Field], id: Id) -> Result<&str, String> {
    Ok(&get(fields, id)?.text)
}
pub(super) fn visible(fields: &[Field]) -> Vec<usize> {
    let tolerance = text(fields, Id::Tolerance).is_ok_and(|v| v != "none");
    let dual = text(fields, Id::Dual) == Ok("true");
    fields
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            // Keep inactive edited values reachable, including invalid text which
            // must remain repairable before shared validation can accept Apply.
            let edited = f.text != f.original;
            match f.id {
                Id::Upper | Id::Lower => tolerance || edited,
                Id::DualUnit | Id::DualPrecision | Id::DualPlacement => dual || edited,
                _ => true,
            }
        })
        .map(|(i, _)| i)
        .collect()
}
pub(super) fn edit(fields: &mut [Field], id: Id, input: &ControlInput) -> Result<bool, String> {
    let index = fields
        .iter()
        .position(|f| f.id == id)
        .ok_or("Drawing field was removed")?;
    if !visible(fields).contains(&index) {
        return Err("Drawing field is hidden; use the refreshed controls".into());
    }
    let f = &fields[index];
    let activation = super::super::super::super::is_activation(input);
    let next = match f.kind {
        Kind::Choice(_) => {
            let navigation = matches!(input, ControlInput::Key(k) if !k.ctrl && !k.meta && !k.alt && !k.shift && matches!(k.key.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "Home" | "End"));
            if !activation && !navigation && !matches!(input, ControlInput::SetValue(_)) {
                return Ok(false);
            }
            super::super::cam::choose(&f.options().unwrap(), &f.text, input)
                .map_err(|_| format!("Choose an available value for {}", f.label))?
        }
        Kind::Toggle => match input {
            ControlInput::SetValue(value) => value
                .parse::<bool>()
                .map_err(|_| format!("Choose true or false for {}", f.label))?
                .to_string(),
            _ if activation => (!boolean(fields, id)?).to_string(),
            _ => return Ok(false),
        },
        _ => match input {
            ControlInput::SetValue(value) => value.clone(),
            _ => return Ok(false),
        },
    };
    let mut changed = fields[index].text != next;
    fields[index].text = next;
    if matches!(id, Id::Basic | Id::Reference) && fields[index].text == "true" {
        let other = if id == Id::Basic {
            Id::Reference
        } else {
            Id::Basic
        };
        let field = fields
            .iter_mut()
            .find(|f| f.id == other)
            .ok_or("Dimension field was removed")?;
        changed |= field.text != "false";
        field.text = "false".into();
    }
    Ok(changed)
}
fn number(fields: &[Field], id: Id) -> Result<f64, String> {
    let f = get(fields, id)?;
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
fn precision(fields: &[Field], id: Id) -> Result<u8, String> {
    let f = get(fields, id)?;
    f.text
        .trim()
        .parse::<u8>()
        .ok()
        .filter(|n| *n <= 6)
        .ok_or_else(|| format!("{} must be an integer from 0 to 6", f.label))
}
fn boolean(fields: &[Field], id: Id) -> Result<bool, String> {
    let f = get(fields, id)?;
    f.text
        .parse()
        .map_err(|_| format!("Choose true or false for {}", f.label))
}
fn presentation(fields: &[Field]) -> Result<DrawingDimensionPresentationDto, String> {
    let mode = match text(fields, Id::Tolerance)? {
        "none" => DrawingDimensionToleranceMode::None,
        "symmetric" => DrawingDimensionToleranceMode::Symmetric,
        "deviation" => DrawingDimensionToleranceMode::Deviation,
        "limits" => DrawingDimensionToleranceMode::Limits,
        _ => return Err("Choose a tolerance mode".into()),
    };
    let fit_class = text(fields, Id::Fit)?.to_owned();
    if fit_class.chars().count() > 64 {
        return Err("Fit class must contain at most 64 characters".into());
    }
    let dual_units = if boolean(fields, Id::Dual)? {
        Some(DrawingDualUnitDto {
            unit: match text(fields, Id::DualUnit)? {
                "millimetre" => DrawingSecondaryUnit::Millimetre,
                "centimetre" => DrawingSecondaryUnit::Centimetre,
                "inch" => DrawingSecondaryUnit::Inch,
                _ => return Err("Choose a secondary unit".into()),
            },
            precision: precision(fields, Id::DualPrecision)?,
            placement: match text(fields, Id::DualPlacement)? {
                "bracketed" => DrawingDualUnitPlacement::Bracketed,
                "stacked" => DrawingDualUnitPlacement::Stacked,
                _ => return Err("Choose dual-unit placement".into()),
            },
        })
    } else {
        None
    };
    Ok(DrawingDimensionPresentationDto {
        tolerance: DrawingDimensionToleranceDto {
            mode,
            upper: number(fields, Id::Upper)?,
            lower: number(fields, Id::Lower)?,
        },
        basic: boolean(fields, Id::Basic)?,
        reference: boolean(fields, Id::Reference)?,
        fit_class,
        dual_units,
    })
}
pub(super) fn note_request(sheet_id: u64, fields: &[Field]) -> Result<AddNote, String> {
    Ok(AddNote {
        sheet_id,
        text: text(fields, Id::Note)?.to_owned(),
        position: [number(fields, Id::X)?, number(fields, Id::Y)?],
    })
}
pub(super) fn apply(draft: &mut Draft, fields: &[Field]) -> Result<(), String> {
    match draft.annotation() {
        DrawingAnnotationDto::Note { .. } => {
            let note = note_request(0, fields)?;
            draft.note(note.text)?;
            draft.set_note_position(note.position)?;
        }
        DrawingAnnotationDto::LinearDimension { .. } => {
            let mode = match text(fields, Id::Mode)? {
                "aligned" => DrawingLinearDimensionMode::Aligned,
                "horizontal" => DrawingLinearDimensionMode::Horizontal,
                "vertical" => DrawingLinearDimensionMode::Vertical,
                _ => return Err("Choose a dimension mode".into()),
            };
            draft.linear(
                mode,
                number(fields, Id::Offset)?,
                precision(fields, Id::Precision)?,
                text(fields, Id::Prefix)?.into(),
                text(fields, Id::Suffix)?.into(),
                presentation(fields)?,
            )?;
        }
        _ => return Err("This annotation's inspector has not been migrated yet".into()),
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests;
