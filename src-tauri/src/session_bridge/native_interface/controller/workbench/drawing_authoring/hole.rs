//! Hole callouts use the saved circular association and existing modeled-hole
//! definitions. Selection never introduces another hole or annotation model.
use super::{Stamp, radial};
use nbcad_sketch::*;
use nbcad_solid::{HoleDefinitionDto, HoleExtent, HoleStyle, HoleThreadHand};

mod submit;
pub(super) use submit::submit;

fn best_definition<'a>(
    definitions: &'a [HoleDefinitionDto],
    feature: &DrawingCircularRefDto,
) -> Result<Option<&'a HoleDefinitionDto>, String> {
    // Bound the scan before walking imported pattern catalogs.
    if definitions.len() > 16_384
        || definitions
            .iter()
            .map(|d| d.positions.len().max(1))
            .sum::<usize>()
            > 65_536
    {
        return Err("Too many modeled hole positions to create a callout".into());
    }
    let normal_length = feature
        .fallback_normal
        .iter()
        .map(|v| v * v)
        .sum::<f64>()
        .sqrt();
    let mut best: Option<(&HoleDefinitionDto, f64)> = None;
    for d in definitions {
        let Some(basis) = &d.face_basis else { continue };
        if d.body_id != feature.body_id {
            continue;
        }
        let radius_error = (d.diameter * 0.5 - feature.fallback_radius).abs();
        if !radius_error.is_finite() || radius_error > 0.03_f64.max(d.diameter * 0.015) {
            continue;
        }
        if normal_length >= 1e-9 {
            let alignment = feature
                .fallback_normal
                .iter()
                .zip(basis.normal)
                .map(|(a, b)| a / normal_length * b)
                .sum::<f64>()
                .abs();
            if !alignment.is_finite() || alignment < 0.985 {
                continue;
            }
        }
        let delta: [f64; 3] = std::array::from_fn(|i| feature.fallback_center[i] - basis.origin[i]);
        let projected =
            [basis.u, basis.v].map(|axis| delta.iter().zip(axis).map(|(a, b)| a * b).sum::<f64>());
        let distance = |p: &nbcad_solid::Point2Dto| (projected[0] - p.x).hypot(projected[1] - p.y);
        let center_error = if d.positions.is_empty() {
            distance(&d.position)
        } else {
            d.positions
                .iter()
                .map(|p| distance(&p.position))
                .fold(f64::INFINITY, f64::min)
        };
        if !center_error.is_finite() || center_error > 0.08_f64.max(d.diameter * 0.025) {
            continue;
        }
        let score = center_error + radius_error * 4.;
        if best.is_none_or(|(old, old_score)| {
            score < old_score || (score == old_score && d.feature_id < old.feature_id)
        }) {
            best = Some((d, score));
        }
    }
    Ok(best.map(|(definition, _)| definition))
}

pub(super) fn create(
    document: &DrawingDocumentDto,
    stamp: &Stamp,
    target: &radial::Target,
    definitions: &[HoleDefinitionDto],
) -> Result<DrawingDocumentDto, String> {
    let feature = &target.reference;
    if !feature.closed
        || !feature.fallback_radius.is_finite()
        || feature.fallback_radius <= 0.
        || target.center.iter().any(|v| !v.is_finite())
    {
        return Err("Hole notes require a complete circular edge".into());
    }
    let definition = best_definition(definitions, feature)?;
    let quantity = definition.map_or(1, |d| d.positions.len().max(1)) as u32;
    let style = definition.map_or(HoleStyle::Simple, |d| d.style);
    let thread = definition.and_then(|d| d.thread.as_ref());
    let annotation = DrawingAnnotationDto::HoleNote {
        id: document.next_annotation_id,
        view_id: target.view_id,
        feature: feature.clone(),
        position: [target.center[0] + 20., target.center[1] + 15.],
        quantity,
        diameter: definition.map_or(feature.fallback_radius * 2., |d| d.diameter),
        depth: definition.and_then(|d| match d.extent {
            HoleExtent::Distance { depth } => Some(depth),
            HoleExtent::ThroughAll => None,
        }),
        thread: thread.map_or_else(String::new, |t| {
            format!(
                "{}{}{}",
                t.designation,
                if t.class.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", t.class)
                },
                if t.hand == HoleThreadHand::Left {
                    " LH"
                } else {
                    ""
                }
            )
        }),
        note: if definition.is_some_and(|d| matches!(d.extent, HoleExtent::ThroughAll)) {
            "THRU".into()
        } else {
            String::new()
        },
        source_feature_id: definition.map(|d| d.feature_id.0),
        feature_name: definition.map_or_else(String::new, |d| d.name.clone()),
        hole_style: match style {
            HoleStyle::Simple => DrawingHoleStyle::Simple,
            HoleStyle::Counterbore => DrawingHoleStyle::Counterbore,
            HoleStyle::Countersink => DrawingHoleStyle::Countersink,
        },
        counterbore_diameter: definition
            .filter(|_| style == HoleStyle::Counterbore)
            .map(|d| d.counterbore_diameter),
        counterbore_depth: definition
            .filter(|_| style == HoleStyle::Counterbore)
            .map(|d| d.counterbore_depth),
        countersink_diameter: definition
            .filter(|_| style == HoleStyle::Countersink)
            .map(|d| d.countersink_diameter),
        countersink_angle_deg: definition
            .filter(|_| style == HoleStyle::Countersink)
            .map(|d| d.countersink_angle_deg),
        thread_depth: thread.and_then(|t| t.depth),
        pattern_note: if quantity > 1 {
            format!("{quantity} HOLES")
        } else {
            String::new()
        },
    };
    let mut next = document.clone();
    let sheet = next
        .sheets
        .iter_mut()
        .find(|s| s.id == stamp.sheet_id)
        .ok_or("Drawing sheet changed")?;
    if !sheet.views.iter().any(|v| v.id == target.view_id) {
        return Err("Drawing view changed".into());
    }
    sheet.annotations.push(annotation);
    if sheet.release.status == DrawingReleaseStatus::Released {
        sheet.release.status = DrawingReleaseStatus::Draft;
    }
    next.next_annotation_id = next
        .next_annotation_id
        .checked_add(1)
        .ok_or("Annotation IDs are exhausted")?;
    next.validate()?;
    Ok(next)
}

#[cfg(test)]
mod history_tests;
#[cfg(test)]
mod tests;
