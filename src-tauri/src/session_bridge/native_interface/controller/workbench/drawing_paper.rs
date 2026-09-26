//! Sheet paper for the native drawing workspace. Projected edges, notes, and
//! view names share one layout: paper millimetres, origin at the upper left.
use super::*;
use bevy::ui::UiTransform;
use nbcad_occt::DrawingProjectionDto;
use nbcad_sketch::{DrawingAnnotationDto, DrawingSheetDto, DrawingViewDto};

#[derive(Clone)]
pub(super) struct Label {
    pub text: String,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub(super) struct Segment {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub hidden: bool,
}

pub(super) fn paint(
    world: &mut World,
    camera: Entity,
    services: &NativeServices,
    state: &mut Workbench,
    width: f32,
    height: f32,
) -> Result<(), String> {
    let drawing = services.engine.drawing_snapshot();
    let Some(sheet) = drawing.sheets.iter().find(|sheet| {
        drawing.active_sheet_id == Some(sheet.id)
            || (drawing.active_sheet_id.is_none() && drawing.sheets.last().map(|s| s.id) == Some(sheet.id))
    }) else {
        state.paper_key.clear();
        state.paper.clear();
        return Ok(());
    };
    let revision = services.engine.geometry_revision();
    let key = format!(
        "{revision}:{}:{}",
        sheet.id,
        sheet
            .views
            .iter()
            .map(|view| format!("{}:{:?}:{}", view.id, view.kind, view.scale))
            .collect::<Vec<_>>()
            .join(",")
    );
    if state.paper_key != key {
        let (segments, labels) = project_sheet(services, sheet);
        state.paper = segments;
        state.paper_labels = labels;
        state.paper_key = key;
    }
    let theme = ViewportUiTheme::from_palette(&ViewportPalette::default());
    let (sheet_w, sheet_h) = sheet_size(sheet);
    let scale = ((width - 24.) / sheet_w).min(((height - 188.).max(40.)) / sheet_h);
    let origin_x = (width - sheet_w * scale) * 0.5;
    let origin_y = 128.;
    state.widgets.panel(
        world,
        camera,
        "drawing-paper",
        rect(origin_x, origin_y, sheet_w * scale, sheet_h * scale),
        theme.header.with_alpha(1.),
        8,
    );
    for (index, segment) in state.paper.iter().take(800).enumerate() {
        let (x1, y1) = place(origin_x, origin_y, scale, segment.x1, segment.y1);
        let (x2, y2) = place(origin_x, origin_y, scale, segment.x2, segment.y2);
        let delta = Vec2::new(x2 - x1, y2 - y1);
        let length = delta.length().max(0.5);
        let midpoint = Vec2::new((x1 + x2) * 0.5, (y1 + y2) * 0.5);
        let key = format!("drawing-edge-{index}");
        state.widgets.panel(
            world,
            camera,
            &key,
            Node {
                position_type: PositionType::Absolute,
                left: px(midpoint.x - length * 0.5),
                top: px(midpoint.y - 0.6),
                width: px(length),
                height: px(1.2),
                ..default()
            },
            if segment.hidden { theme.mute } else { theme.ink },
            9,
        );
        if let Some(entity) = state.widgets.entity(&key) {
            world.entity_mut(entity).insert(UiTransform::from_rotation(
                Rot2::radians(delta.y.atan2(delta.x)),
            ));
        }
    }
    for (index, view) in sheet.views.iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-view-name-{index}"),
            rect(
                origin_x + view.position[0] as f32 * scale,
                origin_y + view.position[1] as f32 * scale,
                80.,
                14.,
            ),
            &view.name,
            10.,
            11,
        );
    }
    for (index, label) in state.paper_labels.iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-dimension-{index}"),
            rect(
                origin_x + label.x * scale,
                origin_y + label.y * scale,
                80.,
                14.,
            ),
            &label.text,
            10.,
            12,
        );
    }
    for (index, (text, position)) in notes(sheet).into_iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-note-{index}"),
            rect(
                origin_x + position[0] as f32 * scale,
                origin_y + position[1] as f32 * scale,
                160.,
                16.,
            ),
            &text,
            11.,
            12,
        );
    }
    Ok(())
}

fn place(origin_x: f32, origin_y: f32, scale: f32, x: f32, y: f32) -> (f32, f32) {
    (origin_x + x * scale, origin_y + y * scale)
}

fn notes(sheet: &DrawingSheetDto) -> Vec<(String, [f64; 2])> {
    sheet
        .annotations
        .iter()
        .filter_map(|annotation| match annotation {
            DrawingAnnotationDto::Note { text, position, .. } => Some((text.clone(), *position)),
            _ => None,
        })
        .collect()
}

fn project_sheet(services: &NativeServices, sheet: &DrawingSheetDto) -> (Vec<Segment>, Vec<Label>) {
    let mut segments = Vec::new();
    let mut projections = std::collections::BTreeMap::new();
    for view in &sheet.views {
        let Ok(projection) = services.engine.project_sheet_view(view) else {
            continue;
        };
        push_polylines(&mut segments, view, &projection, &projection.visible, false);
        push_polylines(&mut segments, view, &projection, &projection.hidden, true);
        projections.insert(view.id, (view.clone(), projection));
        if segments.len() >= 800 {
            break;
        }
    }
    let mut labels = Vec::new();
    for annotation in &sheet.annotations {
        let DrawingAnnotationDto::LinearDimension {
            view_id,
            first,
            second,
            mode,
            offset,
            prefix,
            suffix,
            precision,
            ..
        } = annotation
        else {
            continue;
        };
        let Some((view, projection)) = projections.get(view_id) else {
            continue;
        };
        let Ok(first_point) = anchor_point(first, projection) else {
            continue;
        };
        let Ok(second_point) = anchor_point(second, projection) else {
            continue;
        };
        let a = paper_point(view, first_point, projection);
        let b = paper_point(view, second_point, projection);
        let Some((value, _, _, c, d)) = dimension_span(*mode, a, b, *offset) else {
            continue;
        };
        // Extension and dimension lines are already in paper millimetres.
        for (start, end) in [(a, c), (b, d), (c, d)] {
            segments.push(Segment {
                x1: start[0] as f32,
                y1: start[1] as f32,
                x2: end[0] as f32,
                y2: end[1] as f32,
                hidden: false,
            });
        }
        labels.push(Label {
            text: format!("{prefix}{value:.prec$}{suffix}", prec = *precision as usize),
            x: ((c[0] + d[0]) * 0.5) as f32,
            y: ((c[1] + d[1]) * 0.5) as f32,
        });
    }
    (segments, labels)
}

/// Model-millimetre span plus the offset dimension line, before view placement.
fn dimension_span(
    mode: nbcad_sketch::DrawingLinearDimensionMode,
    first: [f64; 2],
    second: [f64; 2],
    offset: f64,
) -> Option<(f64, [f64; 2], [f64; 2], [f64; 2], [f64; 2])> {
    let value = match mode {
        nbcad_sketch::DrawingLinearDimensionMode::Horizontal => (second[0] - first[0]).abs(),
        nbcad_sketch::DrawingLinearDimensionMode::Vertical => (second[1] - first[1]).abs(),
        nbcad_sketch::DrawingLinearDimensionMode::Aligned => {
            (second[0] - first[0]).hypot(second[1] - first[1])
        }
    };
    if value < 1e-9 {
        return None;
    }
    let (c, d) = match mode {
        nbcad_sketch::DrawingLinearDimensionMode::Horizontal => {
            ([first[0], first[1] + offset], [second[0], first[1] + offset])
        }
        nbcad_sketch::DrawingLinearDimensionMode::Vertical => {
            ([first[0] + offset, first[1]], [first[0] + offset, second[1]])
        }
        nbcad_sketch::DrawingLinearDimensionMode::Aligned => {
            let length = (second[0] - first[0]).hypot(second[1] - first[1]);
            let normal = [
                -(second[1] - first[1]) / length,
                (second[0] - first[0]) / length,
            ];
            (
                [first[0] + normal[0] * offset, first[1] + normal[1] * offset],
                [
                    second[0] + normal[0] * offset,
                    second[1] + normal[1] * offset,
                ],
            )
        }
    };
    Some((value, first, second, c, d))
}

fn anchor_point(
    anchor: &nbcad_sketch::DrawingTopologyAnchorRefDto,
    projection: &DrawingProjectionDto,
) -> Result<[f64; 2], String> {
    projection
        .anchors
        .iter()
        .find(|row| {
            row.body_id == anchor.body_id
                && row.edge_id == anchor.edge_id
                && row.edge_key == anchor.edge_key
                && matches!(
                    (row.endpoint, anchor.endpoint),
                    (
                        nbcad_occt::DrawingProjectionAnchorEndpoint::Start,
                        nbcad_sketch::DrawingEdgeEndpoint::Start
                    ) | (
                        nbcad_occt::DrawingProjectionAnchorEndpoint::End,
                        nbcad_sketch::DrawingEdgeEndpoint::End
                    )
                )
        })
        .map(|row| row.point)
        .ok_or_else(|| "Dimension anchor is not in this view".into())
}

fn push_polylines(
    segments: &mut Vec<Segment>,
    view: &DrawingViewDto,
    projection: &DrawingProjectionDto,
    lines: &[nbcad_occt::DrawingPolylineDto],
    hidden: bool,
) {
    for line in lines {
        let mut previous = None;
        for point in &line.points {
            let paper = paper_point(view, *point, projection);
            if let Some((x1, y1)) = previous {
                segments.push(Segment {
                    x1,
                    y1,
                    x2: paper[0] as f32,
                    y2: paper[1] as f32,
                    hidden,
                });
                if segments.len() >= 800 {
                    return;
                }
            }
            previous = Some((paper[0] as f32, paper[1] as f32));
        }
    }
}

/// Same placement as the drawing export: the view position is the projected
/// bounds center, and paper Y grows downward.
pub(super) fn paper_point(view: &DrawingViewDto, point: [f64; 2], projection: &DrawingProjectionDto) -> [f64; 2] {
    let bounds = projection.bounds;
    [
        view.position[0] + (point[0] - (bounds[0] + bounds[2]) * 0.5) * view.scale,
        view.position[1] - (point[1] - (bounds[1] + bounds[3]) * 0.5) * view.scale,
    ]
}

fn sheet_size(sheet: &DrawingSheetDto) -> (f32, f32) {
    let (short, long) = match sheet.format {
        nbcad_sketch::DrawingSheetFormat::A0 => (841., 1189.),
        nbcad_sketch::DrawingSheetFormat::A1 => (594., 841.),
        nbcad_sketch::DrawingSheetFormat::A2 => (420., 594.),
        nbcad_sketch::DrawingSheetFormat::A3 => (297., 420.),
        nbcad_sketch::DrawingSheetFormat::A4 => (210., 297.),
        nbcad_sketch::DrawingSheetFormat::Letter => (215.9, 279.4),
        nbcad_sketch::DrawingSheetFormat::AnsiB => (279.4, 431.8),
        nbcad_sketch::DrawingSheetFormat::AnsiC => (431.8, 558.8),
        nbcad_sketch::DrawingSheetFormat::AnsiD => (558.8, 863.6),
        nbcad_sketch::DrawingSheetFormat::AnsiE => (863.6, 1117.6),
    };
    match sheet.orientation {
        nbcad_sketch::DrawingSheetOrientation::Landscape => (long, short),
        nbcad_sketch::DrawingSheetOrientation::Portrait => (short, long),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbcad_sketch::DrawingViewKind;

    #[test]
    fn paper_point_centers_the_projection_on_the_view() {
        let view = DrawingViewDto {
            scope: Default::default(),
            occurrence_ids: vec![],
            id: 1,
            name: "Front".into(),
            kind: DrawingViewKind::Front,
            direction: [0., -1., 0.],
            up: [0., 0., 1.],
            position: [100., 80.],
            scale: 2.,
            body_ids: vec![],
            show_hidden_lines: false,
            show_tangent_edges: false,
            parent_view_id: None,
            alignment: Default::default(),
            derivation: None,
        };
        let projection = DrawingProjectionDto {
            topology_signatures: Default::default(),
            visible: vec![],
            hidden: vec![],
            anchors: vec![],
            circles: vec![],
            section: vec![],
            bounds: [0., 0., 10., 4.],
        };
        let center = paper_point(&view, [5., 2.], &projection);
        assert_eq!(center, [100., 80.]);
        let corner = paper_point(&view, [10., 4.], &projection);
        assert_eq!(corner, [110., 76.]);
    }

    #[test]
    fn horizontal_dimension_offset_is_paper_millimetres() {
        let (value, _, _, c, d) = dimension_span(
            nbcad_sketch::DrawingLinearDimensionMode::Horizontal,
            [10., 20.],
            [40., 22.],
            8.,
        )
        .unwrap();
        assert_eq!(value, 30.);
        assert_eq!(c, [10., 28.]);
        assert_eq!(d, [40., 28.]);
    }
}
