//! Sheet paper for the native drawing workspace. Projected edges, notes, and
//! view names share one layout: paper millimetres, origin at the upper left.
use super::*;
use bevy::ui::UiTransform;
use nbcad_occt::DrawingProjectionDto;
use nbcad_sketch::{DrawingAnnotationDto, DrawingSheetDto, DrawingViewDto};
#[path = "drawing_dimensions.rs"]
mod dimensions;

#[derive(Clone, Default)]
pub(super) struct Label {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub width_mm: f32,
    pub height_mm: f32,
    pub text_height_mm: f32,
    pub mask: bool,
}

#[derive(Clone, Copy)]
pub(super) struct Segment {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub hidden: bool,
    pub width_mm: f32,
    pub arrow: bool,
}

#[derive(Resource)]
struct ArrowTexture(Handle<Image>);

fn raster_stroke_width(width_mm: f32, paper_scale: f32, render_scale: f32) -> f32 {
    // Solid UI quads need to cover a pixel center to rasterize. Keep the
    // drawing's millimetre width intact, but give its screen representation
    // at least one physical pixel (including window DPI and Bevy UiScale).
    (width_mm * paper_scale).max(0.5).max(render_scale.recip())
}

fn arrow_texture(world: &mut World) -> Handle<Image> {
    use bevy::{
        asset::RenderAssetUsages,
        render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    };
    if let Some(texture) = world.get_resource::<ArrowTexture>() {
        return texture.0.clone();
    }
    // Same triangle as drawing/annotations.ts::arrowPolygon: its base is
    // one arrow size from the tip and has half-width 0.38 times that size.
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><polygon points="0,32 64,7.68 64,56.32" fill="white"/></svg>"#;
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).unwrap();
    let mut pixels = resvg::tiny_skia::Pixmap::new(64, 64).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixels.as_mut(),
    );
    let rgba = pixels
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let pixel = pixel.demultiply();
            [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
        })
        .collect();
    let image = Image::new(
        Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let handle = world.resource_mut::<Assets<Image>>().add(image);
    world.insert_resource(ArrowTexture(handle.clone()));
    handle
}

pub(super) fn paint(
    world: &mut World,
    camera: Entity,
    services: &NativeServices,
    state: &mut Workbench,
    width: f32,
    height: f32,
    side: f32,
    controls: &HashMap<String, Entity>,
) -> Result<(), String> {
    let theme = ViewportUiTheme::from_palette(&ViewportPalette::default());
    // The drawing owns the entire content area, including the space around
    // its paper. This opaque occluder also stops picking the hidden 3D model.
    state.widgets.panel(
        world,
        camera,
        "drawing-backdrop",
        rect(side, 120., width - side, height - 168.),
        theme.viewport.with_alpha(1.),
        7,
    );
    let nav_x = side + (width - side - 64.) * 0.5;
    let nav_y = height - 94.;
    card(
        &mut state.widgets,
        world,
        camera,
        "navigation",
        rect(nav_x, nav_y, 64., 34.),
        theme.header,
        5.,
        25,
    );
    for (index, (key, icon)) in [("undo", Icon::Undo), ("redo", Icon::Redo)]
        .into_iter()
        .enumerate()
    {
        if let Some(&entity) = controls.get(key) {
            world.entity_mut(entity).insert((
                rect(nav_x + 6. + index as f32 * 26., nav_y + 5., 24., 24.),
                interface_shell::InterfaceFlat,
                interface_shell::InterfaceCaption(String::new()),
            ));
            state.widgets.glyph(
                world,
                camera,
                &format!("nav-{key}-glyph"),
                rect(nav_x + 10. + index as f32 * 26., nav_y + 9., 16., 16.),
                icon,
                if world.get::<InterfaceControl>(entity).unwrap().disabled {
                    theme.edge
                } else {
                    theme.mute
                },
                31,
            );
        }
    }
    let drawing = services.engine.drawing_snapshot();
    let Some(sheet) = drawing.sheets.iter().find(|sheet| {
        drawing.active_sheet_id == Some(sheet.id)
            || (drawing.active_sheet_id.is_none()
                && drawing.sheets.last().map(|s| s.id) == Some(sheet.id))
    }) else {
        state.paper_key = None;
        state.paper.clear();
        state.paper_labels.clear();
        return Ok(());
    };
    let revision = services.engine.geometry_revision();
    // Drawing commands do not change the solid geometry revision. Retain the
    // complete drawing intent so annotation and view edits repaint immediately.
    if state
        .paper_key
        .as_ref()
        .is_none_or(|(cached_revision, cached_sheet)| {
            *cached_revision != revision || cached_sheet != sheet
        })
    {
        let (segments, labels) = project_sheet(services, sheet);
        state.paper = segments;
        state.paper_labels = labels;
        state.paper_key = Some((revision, sheet.clone()));
    }
    let (sheet_w, sheet_h) = sheet_size(sheet);
    let (origin_x, origin_y, scale) = sheet_layout(width, height, side, sheet_w, sheet_h);
    let ink = Color::srgb_u8(36, 40, 45);
    state.widgets.panel(
        world,
        camera,
        "drawing-paper",
        rect(origin_x, origin_y, sheet_w * scale, sheet_h * scale),
        Color::WHITE,
        8,
    );
    let paper = state.widgets.entity("drawing-paper").unwrap();
    // Paper strokes are measured in millimetres and can be thinner than a
    // physical pixel. Rounding their unrotated layout can collapse the two
    // sides to the same coordinate, erasing an entire extension line.
    world.entity_mut(paper).insert(bevy::ui::LayoutConfig {
        use_rounding: false,
    });
    let render_scale = world
        .get::<Camera>(camera)
        .and_then(Camera::target_scaling_factor)
        .unwrap_or(1.)
        * world
            .get_resource::<bevy::ui::UiScale>()
            .map_or(1., |s| s.0);
    for (index, segment) in state.paper.iter().enumerate() {
        let (x1, y1) = place(0., 0., scale, segment.x1, segment.y1);
        let (x2, y2) = place(0., 0., scale, segment.x2, segment.y2);
        let delta = Vec2::new(x2 - x1, y2 - y1);
        let length = delta.length().max(0.5);
        let midpoint = Vec2::new((x1 + x2) * 0.5, (y1 + y2) * 0.5);
        let key = format!("drawing-edge-{index}");
        let thickness = if segment.arrow {
            length
        } else {
            raster_stroke_width(segment.width_mm, scale, render_scale)
        };
        state.widgets.panel(
            world,
            camera,
            &key,
            Node {
                position_type: PositionType::Absolute,
                left: px(midpoint.x - length * 0.5),
                top: px(midpoint.y - thickness * 0.5),
                width: px(length),
                height: px(thickness),
                ..default()
            },
            if segment.arrow {
                Color::NONE
            } else if segment.hidden {
                Color::srgb_u8(132, 138, 146)
            } else {
                ink
            },
            9,
        );
        state.widgets.parent(world, &key, paper);
        if let Some(entity) = state.widgets.entity(&key) {
            if segment.arrow {
                let texture = arrow_texture(world);
                world.entity_mut(entity).insert(ImageNode {
                    image: texture,
                    color: ink,
                    ..default()
                });
            } else {
                world.entity_mut(entity).remove::<ImageNode>();
            }
            world
                .entity_mut(entity)
                .insert(UiTransform::from_rotation(Rot2::radians(
                    delta.y.atan2(delta.x),
                )));
        }
    }
    for (index, view) in sheet.views.iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-view-name-{index}"),
            rect(
                view.position[0] as f32 * scale,
                view.position[1] as f32 * scale,
                80.,
                14.,
            ),
            &view.name,
            10.,
            11,
        );
        let key = format!("drawing-view-name-{index}");
        state.widgets.parent(world, &key, paper);
        world
            .entity_mut(state.widgets.entity(&key).unwrap())
            .insert(TextColor(ink));
    }
    for (index, label) in state.paper_labels.iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-dimension-{index}"),
            rect(
                (label.x - label.width_mm * 0.5) * scale,
                (label.y - label.height_mm * 0.5) * scale,
                label.width_mm * scale,
                label.height_mm * scale,
            ),
            &label.text,
            label.text_height_mm * scale,
            12,
        );
        let key = format!("drawing-dimension-{index}");
        state.widgets.parent(world, &key, paper);
        world
            .entity_mut(state.widgets.entity(&key).unwrap())
            .insert((
                TextColor(ink),
                TextLayout::justify(Justify::Center),
                BackgroundColor(if label.mask {
                    Color::WHITE
                } else {
                    Color::NONE
                }),
                UiTransform::from_rotation(Rot2::radians(label.angle)),
            ));
        world
            .get_mut::<TextFont>(state.widgets.entity(&key).unwrap())
            .unwrap()
            .font_size = bevy::text::FontSize::Px(label.text_height_mm * scale);
    }
    for (index, (text, position)) in notes(sheet).into_iter().enumerate() {
        state.widgets.text(
            world,
            camera,
            &format!("drawing-note-{index}"),
            rect(
                position[0] as f32 * scale,
                position[1] as f32 * scale,
                160.,
                16.,
            ),
            &text,
            11.,
            12,
        );
        let key = format!("drawing-note-{index}");
        state.widgets.parent(world, &key, paper);
        world
            .entity_mut(state.widgets.entity(&key).unwrap())
            .insert(TextColor(ink));
    }
    Ok(())
}

fn sheet_layout(width: f32, height: f32, side: f32, sheet_w: f32, sheet_h: f32) -> (f32, f32, f32) {
    let scale = ((width - side - 32.).max(1.) / sheet_w).min((height - 232.).max(1.) / sheet_h);
    (side + (width - side - sheet_w * scale) * 0.5, 132., scale)
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
        push_polylines(
            &mut segments,
            view,
            &projection,
            &projection.visible,
            false,
            sheet.style.visible.width_mm,
        );
        push_polylines(
            &mut segments,
            view,
            &projection,
            &projection.hidden,
            true,
            sheet.style.hidden.width_mm,
        );
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
        let Some((value, _, _, c, d)) = dimension_span(*mode, a, b, *offset, view.scale) else {
            continue;
        };
        let text = format!("{prefix}{value:.prec$}{suffix}", prec = *precision as usize);
        let (dimension, label) = dimensions::layout(a, b, c, d, text, &sheet.style, sheet.standard);
        segments.extend(dimension);
        labels.push(label);
    }
    (segments, labels)
}

/// Measure in model millimetres while placing the lines in paper millimetres.
fn dimension_span(
    mode: nbcad_sketch::DrawingLinearDimensionMode,
    first: [f64; 2],
    second: [f64; 2],
    offset: f64,
    view_scale: f64,
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
        nbcad_sketch::DrawingLinearDimensionMode::Horizontal => (
            [first[0], first[1] + offset],
            [second[0], first[1] + offset],
        ),
        nbcad_sketch::DrawingLinearDimensionMode::Vertical => (
            [first[0] + offset, first[1]],
            [first[0] + offset, second[1]],
        ),
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
    Some((value / view_scale, first, second, c, d))
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
    width_mm: f64,
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
                    width_mm: width_mm as f32,
                    arrow: false,
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
pub(super) fn paper_point(
    view: &DrawingViewDto,
    point: [f64; 2],
    projection: &DrawingProjectionDto,
) -> [f64; 2] {
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
    fn paper_strokes_cover_pixels_at_fractional_positions_and_dpi() {
        fn covers_pixel(center: f32, width: f32) -> bool {
            let left = f64::from(center) - f64::from(width) * 0.5;
            let right = f64::from(center) + f64::from(width) * 0.5;
            (left - 0.5).ceil() + 0.5 <= right
        }
        let (x, _, paper_scale) = sheet_layout(1360., 860., 240., 297., 210.);
        let top_right = x + 100. * paper_scale;
        // The live Top dimension's right extension falls between both
        // pixel centers, even though its unrounded layout is nonzero.
        assert!(!covers_pixel(top_right, 0.25 * paper_scale));
        for dpi in [1., 1.25, 1.5, 2.] {
            for ui_scale in [0.75, 1., 1.5] {
                let render_scale = dpi * ui_scale;
                let width = raster_stroke_width(0.25, paper_scale, render_scale);
                assert!(width * render_scale >= 1.);
                assert!(covers_pixel(top_right * render_scale, width * render_scale));
                // The floor affects display only; sufficiently wide shared
                // paper-mm styles retain their exact scaled thickness.
                assert_eq!(
                    raster_stroke_width(2., paper_scale, render_scale),
                    2. * paper_scale
                );
            }
        }
    }

    #[test]
    fn paper_subpixel_extensions_survive_actual_ui_layout() {
        use bevy::ui::{LayoutConfig, ui_layout_system, ui_surface::UiSurface};

        let mut world = World::new();
        world.init_resource::<UiSurface>();
        world.init_resource::<bevy::text::FontCx>();
        world.init_resource::<bevy::text::RemSize>();
        let (_, _, scale) = sheet_layout(1360., 860., 240., 297., 210.);
        let thickness = 0.25 * scale;
        let mut extensions = Vec::new();
        // The Top view's horizontal dimension in the live fixture: the
        // extension goes from paper y=68.5 to 76.7 mm. Before rotation its
        // top/bottom both round to 217 px with Bevy's default layout policy.
        for use_rounding in [true, false] {
            let paper = world
                .spawn((
                    rect(356., 132., 297. * scale, 210. * scale),
                    LayoutConfig { use_rounding },
                ))
                .id();
            let length = (76.7 - 68.5) * scale;
            let midpoint = Vec2::new(60. * scale, 72.6 * scale);
            let extension = world
                .spawn((
                    rect(
                        midpoint.x - length * 0.5,
                        midpoint.y - thickness * 0.5,
                        length,
                        thickness,
                    ),
                    UiTransform::from_rotation(Rot2::radians(std::f32::consts::FRAC_PI_2)),
                    ChildOf(paper),
                ))
                .id();
            extensions.push(extension);
        }
        world.run_system_cached(ui_layout_system).unwrap();
        let rounded = world.get::<ComputedNode>(extensions[0]).unwrap();
        assert_eq!(
            rounded.size().y,
            0.,
            "fixture must reproduce the lost stroke"
        );
        let unrounded = world.get::<ComputedNode>(extensions[1]).unwrap();
        assert!((unrounded.size().y - thickness).abs() < 0.0001);
        assert!(unrounded.size().x > 24.);
    }

    #[test]
    fn paper_fit_reserves_browser_ribbon_and_history_controls() {
        for (width, height, side) in [
            (800., 600., 240.),
            (1360., 860., 240.),
            (1920., 1080., 280.),
        ] {
            for (sheet_w, sheet_h) in [(297., 210.), (210., 297.), (1189., 841.)] {
                let (x, y, scale) = sheet_layout(width, height, side, sheet_w, sheet_h);
                assert!(scale > 0.);
                assert!(x >= side + 15.9);
                assert!(x + sheet_w * scale <= width - 15.9);
                assert!(y >= 132.);
                assert!(y + sheet_h * scale <= height - 99.9);
            }
        }
    }

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
            1.,
        )
        .unwrap();
        assert_eq!(value, 30.);
        assert_eq!(c, [10., 28.]);
        assert_eq!(d, [40., 28.]);
    }

    #[test]
    fn scaled_dimensions_keep_model_measurement_and_paper_offset() {
        use nbcad_sketch::DrawingLinearDimensionMode::{Aligned, Horizontal, Vertical};
        for (mode, second, expected) in [
            (Horizontal, [40., 20.], 30.),
            (Vertical, [10., 60.], 40.),
            (Aligned, [40., 60.], 50.),
        ] {
            for scale in [0.5, 1., 2.] {
                let first = [10. * scale, 20. * scale];
                let second = [second[0] * scale, second[1] * scale];
                let (value, _, _, c, _) = dimension_span(mode, first, second, 8., scale).unwrap();
                assert_eq!(value, expected);
                assert!(((c[0] - first[0]).hypot(c[1] - first[1]) - 8.).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn drawing_annotation_repaints_without_a_solid_revision() {
        use crate::session_bridge::native_interface::tests::Fixture;
        let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
        let fixture = Fixture::new();
        let owner = fixture.owner();
        let mutate = |operation, arguments| {
            fixture
                .bridge
                .apply_native_mutation(&fixture.engine, &owner, operation, &arguments, || Ok(()))
                .unwrap();
        };
        for (operation, arguments) in [
            (
                "sketch_begin",
                json!({"plane":{"type":"origin_plane","plane":"xy"}}),
            ),
            (
                "sketch_add_rectangle",
                json!({"mode":"two_point","p1":{"x":0.,"y":0.},"p2":{"x":40.,"y":25.},"ctrl_held":true}),
            ),
            ("sketch_finish", json!({})),
            (
                "solid_extrude",
                json!({"sketch_name":"Sketch1","profile_indices":[0],"extent":{"type":"distance","distance":6.}}),
            ),
            (
                "drawing_create_sheet",
                json!({"name":"Drawing test","format":"a4","orientation":"landscape"}),
            ),
            (
                "drawing_add_view",
                json!({"sheet_id":1,"view":{"name":"Front 2:1","kind":"front","direction":[0.,-1.,0.],"up":[0.,0.,1.],"position":[100.,100.],"scale":2.}}),
            ),
        ] {
            mutate(operation, arguments);
        }
        let services = NativeServices {
            engine: fixture.engine.clone(),
            bridge: fixture.bridge.clone(),
        };
        let mut app = native_viewport::interface_scene_fixture();
        let world = app.world_mut();
        world.init_resource::<Assets<Image>>();
        world.init_resource::<ViewportUiAssets>();
        let camera = world.spawn(InterfaceCamera).id();
        let mut state = Workbench::default();
        state.widgets.begin();
        paint(
            world,
            camera,
            &services,
            &mut state,
            1200.,
            800.,
            240.,
            &HashMap::new(),
        )
        .unwrap();
        state.widgets.finish(world);
        assert!(state.paper_labels.is_empty());
        let paper = state.widgets.entity("drawing-paper").unwrap();
        assert!(
            !world
                .get::<bevy::ui::LayoutConfig>(paper)
                .unwrap()
                .use_rounding
        );
        assert_eq!(
            world.get::<BackgroundColor>(paper),
            Some(&BackgroundColor(Color::WHITE))
        );
        assert_eq!(world.get::<Node>(paper).unwrap().overflow, Overflow::clip());
        assert!(
            world
                .get::<interface_shell::InterfaceOccluder>(
                    state.widgets.entity("drawing-backdrop").unwrap()
                )
                .is_some()
        );
        let geometry_revision = services.engine.geometry_revision();
        let drawing = services.engine.drawing_snapshot();
        let projection = services
            .engine
            .project_sheet_view(&drawing.sheets[0].views[0])
            .unwrap();
        let (first, second) = projection
            .anchors
            .iter()
            .find_map(|first| {
                projection
                    .anchors
                    .iter()
                    .find(|second| {
                        first.edge_id == second.edge_id
                            && first.endpoint != second.endpoint
                            && ((first.point[0] - second.point[0]).abs() - 40.).abs() < 1e-8
                    })
                    .map(|second| (first, second))
            })
            .unwrap();
        let anchor = |row: &nbcad_occt::DrawingProjectionAnchorDto| {
            json!({
                "body_id":row.body_id,"edge_id":row.edge_id,"edge_key":row.edge_key,
                "endpoint":row.endpoint,"fallback_point":row.model_point,
            })
        };
        let export = || {
            crate::session_bridge::parse_engine_envelope(
                fixture.engine.engine_call("project_export_model", ""),
            )
            .unwrap()
        };
        let before_dimension = export();
        let solid = serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap();
        let inbox = |seq, operation, arguments| {
            let session = fixture
                .bridge
                .session_id_for_window("main")
                .unwrap()
                .unwrap();
            let revision = fixture
                .bridge
                .engine_revision_for_window("main")
                .unwrap()
                .unwrap();
            let directory = crate::session_bridge::inbox_dir(&session);
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                directory.join(format!("{seq}.json")),
                json!({"name":operation,"arguments":arguments,"base_generation":revision,
                    "session_id":session,"window_id":"main"})
                .to_string(),
            )
            .unwrap();
            crate::session_bridge::apply_or_reject_one_inbox_op(
                &fixture.bridge,
                "main",
                &fixture.engine,
                None,
                Some((&owner.document_id, &session)),
            )
            .unwrap()
        };
        assert_eq!(
            inbox(
                1,
                "drawing_add_linear_dimension",
                json!({
                    "sheet_id":1,"view_id":1,"first":anchor(first),"second":anchor(second),
                    "mode":"horizontal","offset":12.,"precision":2,
                }),
            )["applied"],
            true
        );
        let with_dimension = export();
        assert_eq!(services.engine.geometry_revision(), geometry_revision);
        state.widgets.begin();
        paint(
            world,
            camera,
            &services,
            &mut state,
            1200.,
            800.,
            240.,
            &HashMap::new(),
        )
        .unwrap();
        state.widgets.finish(world);
        assert_eq!(state.paper_labels.len(), 1);
        assert_eq!(state.paper_labels[0].text, "40.00");
        let arrows: Vec<_> = state
            .paper
            .iter()
            .enumerate()
            .filter(|(_, segment)| segment.arrow)
            .map(|(index, _)| {
                state
                    .widgets
                    .entity(&format!("drawing-edge-{index}"))
                    .unwrap()
            })
            .collect();
        assert_eq!(arrows.len(), 2);
        assert!(
            arrows
                .iter()
                .all(|entity| world.get::<ImageNode>(*entity).is_some())
        );
        assert_eq!(
            world.get::<ImageNode>(arrows[0]).unwrap().image,
            world.get::<ImageNode>(arrows[1]).unwrap().image,
            "Both arrowheads reuse one rasterized triangle"
        );
        let label = state.widgets.entity("drawing-dimension-0").unwrap();
        assert_eq!(world.get::<ChildOf>(label).unwrap().parent(), paper);
        assert!(
            world
                .query::<&Text>()
                .iter(world)
                .any(|text| text.0 == "40.00")
        );

        // A rejected command must retain the dimension's Undo snapshot.
        let rejected = inbox(
            2,
            "drawing_add_note",
            json!({"sheet_id":999,"text":"Rejected","position":[20.,20.]}),
        );
        assert_eq!(rejected["applied"], false);
        assert_eq!(rejected["dead_lettered"], true);
        assert_eq!(export(), with_dimension);
        let revision = fixture
            .bridge
            .engine_revision_for_window("main")
            .unwrap()
            .unwrap();
        fixture
            .bridge
            .publishers
            .lock()
            .unwrap()
            .get_mut("main")
            .unwrap()
            .active_mut()
            .engine_revision = u64::MAX;
        let exhausted = inbox(
            3,
            "drawing_add_note",
            json!({"sheet_id":1,"text":"Must not dispatch","position":[20.,20.]}),
        );
        assert_eq!(exhausted["applied"], false);
        assert_eq!(exhausted["dead_lettered"], true);
        assert!(
            exhausted["error"]
                .as_str()
                .unwrap()
                .contains("revision exhausted")
        );
        assert_eq!(export(), with_dimension);
        fixture
            .bridge
            .publishers
            .lock()
            .unwrap()
            .get_mut("main")
            .unwrap()
            .active_mut()
            .engine_revision = revision;

        fixture
            .bridge
            .apply_native_history(&fixture.engine, &fixture.owner(), false, || Ok(()))
            .unwrap();
        assert_eq!(export(), before_dimension);
        assert_eq!(
            serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap(),
            solid
        );
        fixture
            .bridge
            .apply_native_history(&fixture.engine, &fixture.owner(), true, || Ok(()))
            .unwrap();
        assert_eq!(export(), with_dimension);
        assert_eq!(
            serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap(),
            solid
        );
    }

    #[test]
    fn drawing_edits_and_sheet_selection_undo_without_deleting_the_solid() {
        use crate::session_bridge::{native_interface::tests::Fixture, parse_engine_envelope};
        let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
        let fixture = Fixture::new();
        let mutate = |operation, arguments| {
            fixture
                .bridge
                .apply_native_mutation(
                    &fixture.engine,
                    &fixture.owner(),
                    operation,
                    &arguments,
                    || Ok(()),
                )
                .unwrap();
        };
        let export = || {
            parse_engine_envelope(fixture.engine.engine_call("project_export_model", "")).unwrap()
        };
        for (operation, arguments) in [
            (
                "sketch_begin",
                json!({"plane":{"type":"origin_plane","plane":"xy"}}),
            ),
            (
                "sketch_add_rectangle",
                json!({"mode":"two_point","p1":{"x":0.,"y":0.},"p2":{"x":40.,"y":25.},"ctrl_held":true}),
            ),
            ("sketch_finish", json!({})),
            (
                "solid_extrude",
                json!({"sketch_name":"Sketch1","profile_indices":[0],"extent":{"type":"distance","distance":6.}}),
            ),
        ] {
            mutate(operation, arguments);
        }
        let solid = serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap();
        let mut snapshots = vec![export()];
        for (operation, arguments) in [
            (
                "drawing_create_sheet",
                json!({"name":"Original sheet","format":"a4","orientation":"landscape"}),
            ),
            (
                "drawing_add_view",
                json!({"sheet_id":1,"view":{"name":"Front","kind":"front","direction":[0.,-1.,0.],"up":[0.,0.,1.],"position":[100.,100.],"scale":1.}}),
            ),
            (
                "drawing_add_note",
                json!({"sheet_id":1,"text":"Preserve the solid","position":[20.,20.]}),
            ),
            (
                "drawing_create_sheet",
                json!({"name":"Scratch sheet","format":"a4","orientation":"portrait"}),
            ),
            ("drawing_select_sheet", json!({"sheet_id":1})),
            ("drawing_delete_sheet", json!({"sheet_id":2})),
        ] {
            mutate(operation, arguments);
            snapshots.push(export());
        }
        for expected in snapshots[..snapshots.len() - 1].iter().rev() {
            fixture
                .bridge
                .apply_native_history(&fixture.engine, &fixture.owner(), false, || Ok(()))
                .unwrap();
            assert_eq!(export(), *expected);
            assert_eq!(
                serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap(),
                solid
            );
        }
        for expected in &snapshots[1..] {
            fixture
                .bridge
                .apply_native_history(&fixture.engine, &fixture.owner(), true, || Ok(()))
                .unwrap();
            assert_eq!(export(), *expected);
            assert_eq!(
                serde_json::to_value(fixture.engine.viewport_snapshot().2).unwrap(),
                solid
            );
        }
    }
}
