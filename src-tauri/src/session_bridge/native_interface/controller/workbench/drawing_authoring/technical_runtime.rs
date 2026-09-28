use super::super::*;
use super::{
    runtime::{Command, Editor, Target, Tool},
    *,
};
use nbcad_sketch::{DrawingDocumentDto, DrawingRadialDimensionMode};

pub(super) fn synchronize(world: &World, state: &Workbench, e: &mut Editor) -> Result<(), String> {
    let Some(Tool::Technical(tool)) = e.tool else {
        return Ok(());
    };
    if e.technical_source
        .as_ref()
        .is_some_and(|s| drawing_paper::same_projection(state, s))
    {
        return Ok(());
    }
    e.technical.cancel();
    e.targets.clear();
    e.circles.clear();
    e.lines.clear();
    e.technical_source = None;
    if let Some(result) = drawing_paper::with_projections(world, state, |projections, bases| {
        let scene = crate::native_viewport::interface_geometry(world).scene;
        let mut targets = Vec::new();
        let mut circles = Vec::new();
        let mut lines = Vec::new();
        for (view, projection) in projections.values() {
            let direction = bases
                .get(&view.id)
                .ok_or("Drawing projection basis is missing")?
                .direction;
            if tool.anchors() {
                for a in anchors::endpoints(view, projection, direction)? {
                    targets.push(Target {
                        view_id: view.id,
                        reference: anchors::endpoint_ref(a, projection),
                        paper: drawing_paper::paper_point(view, a.point, projection),
                    });
                    if targets.len() > 4096 {
                        return Err("Too many annotation anchor targets on this sheet".to_owned());
                    }
                }
            }
            if tool.circles() {
                if projection.circles.len() > 16_384 {
                    return Err("Too many circular targets in this view".to_owned());
                }
                let mode = if tool == technical::Tool::BoltCircle {
                    DrawingRadialDimensionMode::Diameter
                } else {
                    DrawingRadialDimensionMode::Radius
                };
                circles.extend(radial::targets(view, projection, direction, mode)?);
                if circles.len() > 4096 {
                    return Err("Too many annotation circle targets on this sheet".to_owned());
                }
            }
            if tool.lines() {
                lines.extend(straight::targets(scene, view, projection, direction)?);
                if lines.len() > 4096
                    || lines.iter().map(|t| t.pick_segments.len()).sum::<usize>() > 16_384
                {
                    return Err("Too many annotation edge targets on this sheet".to_owned());
                }
            }
        }
        Ok((targets, circles, lines))
    }) {
        (e.targets, e.circles, e.lines) = result?;
        e.technical_source = drawing_paper::projection_stamp(state);
        e.serial = e.serial.wrapping_add(1);
    }
    Ok(())
}
pub(super) fn pick(
    world: &World,
    e: &mut Editor,
    stamp: &Stamp,
    command: &Command,
) -> Result<Option<DrawingDocumentDto>, String> {
    let Some(Tool::Technical(tool)) = e.tool else {
        return Err("Choose an annotation tool first".into());
    };
    if e.technical_source
        .as_ref()
        .is_none_or(|s| !drawing_paper::same_projection(world.resource::<Workbench>(), s))
    {
        return Err("Projection changed; choose refreshed geometry".into());
    }
    let size = drawing_paper::transform(world.resource::<Workbench>())
        .ok_or("Open drawing paper")?
        .sheet_mm;
    let mut next = match command {
        Command::Anchor(i) if tool.anchors() => e.technical.anchor(
            tool,
            stamp,
            e.targets.get(*i).ok_or("Anchor changed")?,
            &e.document,
            size,
        )?,
        Command::Circle(i) if tool.circles() => e.technical.circle(
            tool,
            stamp,
            e.circles.get(*i).ok_or("Circle changed")?,
            &e.document,
            size,
        )?,
        Command::Line(i) if tool.lines() => e.technical.line(
            tool,
            stamp,
            e.lines.get(*i).ok_or("Edge changed")?,
            &e.document,
            size,
        )?,
        _ => return Err("Choose geometry for the active annotation tool".into()),
    };
    // Same BOM defaults as the existing document workflow, with body names from
    // the current owning scene. Existing BOM metadata is never overwritten.
    if let Some(document) = &mut next {
        if document.next_bom_item_id != e.document.next_bom_item_id {
            let scene = crate::native_viewport::interface_geometry(world).scene;
            if let Some(item) = document
                .sheets
                .iter_mut()
                .find(|s| s.id == stamp.sheet_id)
                .and_then(|s| {
                    s.bom
                        .iter_mut()
                        .find(|b| b.id == e.document.next_bom_item_id)
                })
            {
                if let Some(body) = scene.bodies.iter().find(|b| Some(b.id) == item.body_id) {
                    item.description = body.name.clone();
                }
            }
            document.validate()?;
        }
        e.pending_selected = Some(e.document.next_annotation_id);
    }
    Ok(next)
}
