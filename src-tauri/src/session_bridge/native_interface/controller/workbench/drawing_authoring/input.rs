use super::super::*;
use super::runtime::{preview, submit, Command, Drag, Editor, Tool};
use super::{
    draft::{Draft, Selection},
    *,
};
use crate::native_viewport::winit_host::NativeHostInput;
use bevy::{
    input::{keyboard::Key, ButtonState},
    window::WindowEvent,
};

fn annotation_at(world: &World, handle: &NativeInterfaceHandle, cursor: [f64; 2]) -> Option<u64> {
    let key = handle.hit_key(cursor)?;
    let binding = world.get::<NativeCommandBinding>(Entity::from_bits(key.0))?;
    match &binding.command {
        NativeCommand::Drawing(drawing_editor::Command::Annotation(_, Command::Select(id))) => {
            Some(*id)
        }
        _ => None,
    }
}
pub(super) fn claim_radial_target(
    world: &World,
    handle: &NativeInterfaceHandle,
    cursor: [f64; 2],
) -> bool {
    let owned = handle.hit_key(cursor).is_some_and(|key| {
        matches!(
            world
                .get::<NativeCommandBinding>(Entity::from_bits(key.0))
                .map(|b| &b.command),
            Some(NativeCommand::Drawing(drawing_editor::Command::Annotation(
                _,
                Command::Circle(_)
            )))
        )
    });
    if owned {
        // prepare_native_input already captured the rectangular control. The
        // ring test owns this press; no later rectangular release may activate.
        handle.cancel_pointer();
    }
    owned
}
pub(in super::super) fn process(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    services: &NativeServices,
    input: &NativeHostInput,
) -> Result<bool, String> {
    let Some(mut editor) = world.remove_resource::<Editor>() else {
        return Ok(false);
    };
    let result = inner(world, handle, services, input, &mut editor);
    if let Err(error) = &result {
        editor.message = error.clone();
        editor.drag = None;
        editor.pair.cancel();
        editor.angular.cancel();
        editor.series.cancel();
        editor.straight.cancel();
        editor.chamfer.cancel();
    }
    world.insert_resource(editor);
    if result.as_ref().is_ok_and(|handled| *handled) {
        refresh_preview(world)?;
        handle.invalidate_presentation();
    }
    result
}
fn inner(
    world: &mut World,
    handle: &NativeInterfaceHandle,
    services: &NativeServices,
    input: &NativeHostInput,
    e: &mut Editor,
) -> Result<bool, String> {
    if workspace(world) != Workspace::Drawing {
        e.drag = None;
        e.pair.cancel();
        e.angular.cancel();
        e.series.cancel();
        e.straight.cancel();
        e.chamfer.cancel();
        return Ok(false);
    }
    let cancel = matches!(&input.event,WindowEvent::WindowFocused(f) if !f.focused)
        || matches!(
            &input.event,
            WindowEvent::KeyboardFocusLost(_)
                | WindowEvent::CursorLeft(_)
                | WindowEvent::WindowCloseRequested(_)
                | WindowEvent::WindowDestroyed(_)
                | WindowEvent::WindowResized(_)
                | WindowEvent::WindowScaleFactorChanged(_)
                | WindowEvent::WindowBackendScaleFactorChanged(_)
        );
    let escape = matches!(&input.event,WindowEvent::KeyboardInput(k) if k.state==ButtonState::Pressed && k.logical_key==Key::Escape && !input.consumed);
    if cancel || escape {
        let active = e.drag.take().is_some()
            || e.pair.first.is_some()
            || !e.angular.picks.is_empty()
            || !e.series.picks.is_empty()
            || e.straight.active()
            || e.chamfer.active()
            || e.tool.is_some()
            || e.selected.is_some();
        e.pair.cancel();
        e.angular.cancel();
        e.series.cancel();
        e.straight.cancel();
        e.chamfer.cancel();
        if escape && !e.dirty() {
            e.clear();
        }
        return Ok(active);
    }
    let Some(stamp) = e.stamp.clone() else {
        return Ok(false);
    };
    let valid = handle.read_surface(|owner, frame| {
        input.context.as_ref() == Some(owner)
            && owner == &stamp.owner
            && frame.modal_stack.is_empty()
    })?;
    if !valid {
        e.drag = None;
        e.pair.cancel();
        e.angular.cancel();
        e.series.cancel();
        e.straight.cancel();
        e.chamfer.cancel();
        return Ok(false);
    }
    let receipt = services
        .bridge
        .native_document_receipt(&services.engine, &stamp.owner)?;
    if receipt.revision != stamp.revision {
        e.drag = None;
        e.pair.cancel();
        e.angular.cancel();
        e.series.cancel();
        e.straight.cancel();
        e.chamfer.cancel();
        return Ok(false);
    }
    let Some(transform) = world
        .get_resource::<Workbench>()
        .and_then(drawing_paper::transform)
    else {
        return Ok(false);
    };
    let Some(cursor) = input
        .cursor
        .filter(|p| p.is_finite())
        .map(|p| p.as_dvec2().to_array())
    else {
        return Ok(false);
    };
    if let Some(drag) = &mut e.drag {
        if drag.stamp != stamp {
            e.drag = None;
            return Ok(false);
        }
        if matches!(&input.event, WindowEvent::CursorMoved(_)) {
            let point = transform.to_paper(cursor);
            let delta = [point[0] - drag.start[0], point[1] - drag.start[1]];
            if delta[0].hypot(delta[1]) * transform.scale >= 3. {
                drag.moved = true;
            }
            if drag.moved {
                if let Some([a, b]) = drag.linear_points {
                    drag.draft.move_linear(a, b, delta)?;
                } else if drag.ordinate_points.is_some() {
                    drag.draft.move_ordinate(delta)?;
                } else if let Some(g) = &drag.radial {
                    drag.draft
                        .move_radial(g.center, g.paper_radius, g.shoulder, delta)?;
                } else if let Some(g) = &drag.angular {
                    drag.draft.move_angular(g.vertex, g.text, delta)?;
                } else if matches!(drag.draft.annotation(), nbcad_sketch::DrawingAnnotationDto::ChamferNote { .. }) {
                    drag.draft.move_chamfer(delta, transform.sheet_mm)?;
                } else if matches!(drag.draft.annotation(),
                    nbcad_sketch::DrawingAnnotationDto::LineDimension { .. }
                    | nbcad_sketch::DrawingAnnotationDto::PointLineDimension { .. }) {
                    drag.draft.move_straight(delta, transform.sheet_mm)?;
                } else if let nbcad_sketch::DrawingAnnotationDto::Note { position, .. } =
                    Draft::new(&e.document, drag.draft.selection())?.annotation()
                {
                    drag.draft.move_note(
                        [position[0] + delta[0], position[1] + delta[1]],
                        transform.sheet_mm,
                    )?;
                }
            }
            return Ok(true);
        }
        if matches!(&input.event,WindowEvent::MouseButtonInput(b) if b.button==MouseButton::Left && b.state==ButtonState::Released)
        {
            let drag = e.drag.take().unwrap();
            if drag.moved && drag.draft.dirty() {
                let next = drag.draft.apply(&e.document)?;
                e.pending_selected = Some(drag.draft.selection().annotation_id);
                submit(
                    world,
                    handle,
                    &services.engine,
                    &services.bridge,
                    &stamp,
                    "drawing_set_document",
                    serde_json::to_value(next).map_err(|x| x.to_string())?,
                )?;
            }
            return Ok(true);
        }
    }
    if e.tool == Some(Tool::Chamfer) && e.chamfer.active()
        && matches!(&input.event, WindowEvent::CursorMoved(_)) && !input.consumed
        && handle.hit_key(cursor).is_none() {
        if let Some(point) = transform.pick(cursor) {
            e.chamfer.move_to(point, transform.sheet_mm)?;
            return Ok(true);
        }
    }
    if e.tool == Some(Tool::Linear) && e.straight.active()
        && matches!(&input.event, WindowEvent::CursorMoved(_)) && !input.consumed
        && handle.hit_key(cursor).is_none() {
        if let Some(point) = transform.pick(cursor) {
            e.straight.move_to(point, transform.sheet_mm)?;
            return Ok(true);
        }
    }
    if !matches!(&input.event,WindowEvent::MouseButtonInput(b) if b.button==MouseButton::Left && b.state==ButtonState::Pressed)
    {
        return Ok(false);
    }
    let Some(point) = transform.pick(cursor) else {
        return Ok(false);
    };
    if e.tool == Some(Tool::Chamfer) && handle.hit_key(cursor).is_some_and(|key| {
        matches!(world.get::<NativeCommandBinding>(Entity::from_bits(key.0)).map(|b| &b.command),
            Some(NativeCommand::Drawing(drawing_editor::Command::Annotation(_, Command::Chamfer(_)))))
    }) {
        handle.cancel_pointer();
        if e.chamfer_source.as_ref().is_none_or(|source| !drawing_paper::same_projection(world.resource::<Workbench>(), source)) {
            return Err("Projection changed; choose refreshed geometry".into());
        }
        if let Some(index) = chamfer::hit(&e.chamfers, point, 1.5_f64.max(4. / transform.scale)) {
            drawing_editor::guard_sheet_edit(world)?;
            e.pick_chamfer(&stamp, index, transform.sheet_mm)?;
        }
        return Ok(true);
    }
    if e.tool == Some(Tool::Linear) && handle.hit_key(cursor).is_some_and(|key| {
        matches!(world.get::<NativeCommandBinding>(Entity::from_bits(key.0)).map(|b| &b.command),
            Some(NativeCommand::Drawing(drawing_editor::Command::Annotation(_, Command::Line(_)))))
    }) {
        handle.cancel_pointer();
        if e.line_source.as_ref().is_none_or(|source| !drawing_paper::same_projection(world.resource::<Workbench>(), source)) {
            return Err("Projection changed; choose refreshed geometry".into());
        }
        if let Some(index) = straight::hit(&e.lines, point, 1.5_f64.max(4. / transform.scale)) {
            drawing_editor::guard_sheet_edit(world)?;
            e.pick_line(&stamp, index)?;
        }
        return Ok(true);
    }
    if let Some(Tool::Radial(mode)) = e.tool {
        if e.circles.len() > 4096 {
            return Err("Too many circular pick targets on this sheet".into());
        }
        // A published circular target must be topmost. In particular, a
        // keyless paper/panel occluder is never treated as an exposed ring.
        if !claim_radial_target(world, handle, cursor) {
            return Ok(false);
        }
        if let Some(index) = radial::hit(&e.circles, point, 2_f64.max(3. / transform.scale)) {
            drawing_editor::guard_sheet_edit(world)?;
            let args = radial::request(&stamp, &e.circles[index], mode)?;
            e.pending_selected = Some(e.document.next_annotation_id);
            submit(
                world,
                handle,
                &services.engine,
                &services.bridge,
                &stamp,
                "drawing_add_radial_dimension",
                serde_json::to_value(args).map_err(|x| x.to_string())?,
            )?;
        }
        // Empty rectangle corners must not fall through to a circular control.
        return Ok(true);
    }
    if let Some(id) = annotation_at(world, handle, cursor) {
        if e.dirty() {
            return Err("Apply or reset the annotation edit first".into());
        }
        drawing_editor::guard_sheet_edit(world)?;
        let mark = drawing_paper::annotation_marks(world, world.resource::<Workbench>())
            .into_iter()
            .find(|m| m.id == id)
            .ok_or("Annotation layout changed")?;
        let draft = Draft::new(
            &e.document,
            Selection {
                sheet_id: stamp.sheet_id,
                annotation_id: id,
            },
        )?;
        let unresolved = match draft.annotation() {
            nbcad_sketch::DrawingAnnotationDto::LinearDimension { .. }
            | nbcad_sketch::DrawingAnnotationDto::ChainDimension { .. } => {
                mark.linear_points.is_none()
            }
            nbcad_sketch::DrawingAnnotationDto::OrdinateDimension { .. } => {
                mark.ordinate_points.is_none()
            }
            nbcad_sketch::DrawingAnnotationDto::RadialDimension { .. } => mark.radial.is_none(),
            nbcad_sketch::DrawingAnnotationDto::AngularDimension { .. } => mark.angular.is_none(),
            nbcad_sketch::DrawingAnnotationDto::LineDimension { .. }
            | nbcad_sketch::DrawingAnnotationDto::PointLineDimension { .. }
            | nbcad_sketch::DrawingAnnotationDto::ChamferNote { .. } => !mark.position_resolved,
            _ => false,
        };
        if unresolved {
            return Err("Repair the dimension's projected references before dragging it".into());
        }
        e.drag = Some(Drag {
            stamp,
            start: point,
            draft,
            linear_points: mark.linear_points,
            radial: mark.radial,
            angular: mark.angular,
            ordinate_points: mark.ordinate_points,
            moved: false,
        });
        return Ok(true);
    }
    if handle.hit_key(cursor).is_some() || handle.has_capture() {
        return Ok(false);
    }
    if e.tool == Some(Tool::Note) {
        drawing_editor::guard_sheet_edit(world)?;
        let mut note = fields::note_request(stamp.sheet_id, &e.fields)?;
        note.position = point;
        e.pending_selected = Some(e.document.next_annotation_id);
        submit(
            world,
            handle,
            &services.engine,
            &services.bridge,
            &stamp,
            "drawing_add_note",
            serde_json::to_value(note).map_err(|x| x.to_string())?,
        )?;
        return Ok(true);
    }
    if e.tool == Some(Tool::Chamfer) && e.chamfer.active() {
        drawing_editor::guard_sheet_edit(world)?;
        if e.chamfer_source.as_ref().is_none_or(|source| !drawing_paper::same_projection(world.resource::<Workbench>(), source)) {
            return Err("Projection changed; choose refreshed geometry".into());
        }
        e.chamfer.move_to(point, transform.sheet_mm)?;
        let next = e.chamfer.create(&e.document, &stamp)?;
        e.pending_selected = Some(e.document.next_annotation_id);
        submit(world, handle, &services.engine, &services.bridge, &stamp,
            "drawing_set_document", serde_json::to_value(next).map_err(|x|x.to_string())?)?;
        e.chamfer.cancel();
        return Ok(true);
    }
    if e.tool == Some(Tool::Linear) && e.straight.active() {
        drawing_editor::guard_sheet_edit(world)?;
        if e.line_source.as_ref().is_none_or(|source| !drawing_paper::same_projection(world.resource::<Workbench>(), source)) {
            return Err("Projection changed; choose refreshed geometry".into());
        }
        e.straight.move_to(point, transform.sheet_mm)?;
        let next = e.straight.create(&e.document, &stamp)?;
        e.pending_selected = Some(e.document.next_annotation_id);
        submit(world, handle, &services.engine, &services.bridge, &stamp,
            "drawing_set_document", serde_json::to_value(next).map_err(|x|x.to_string())?)?;
        e.straight.cancel();
        return Ok(true);
    }
    Ok(false)
}

/// Reuse retained projections for drag feedback and cancellation; no OCCT
/// query or shared-document mutation occurs while the pointer moves.
pub(super) fn refresh_preview(world: &mut World) -> Result<(), String> {
    let Some(mut state) = world.remove_resource::<Workbench>() else {
        return Ok(());
    };
    let result = (|| {
        let Some(e) = world.get_resource::<Editor>() else {
            return Ok(());
        };
        let Some(stamp) = &e.stamp else { return Ok(()) };
        if state.owner.as_ref() != Some(&stamp.owner) {
            return Ok(());
        }
        let Some(sheet) = e.document.sheets.iter().find(|s| s.id == stamp.sheet_id) else {
            return Ok(());
        };
        let sheet = preview(world, sheet, &stamp.owner, stamp.revision);
        drawing_paper::annotation_preview(world, &mut state, &sheet)
    })();
    world.insert_resource(state);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::keyboard::{KeyCode, KeyboardInput};
    #[test]
    fn escape_belongs_only_to_active_drawing_authoring() {
        let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
        let fixture = crate::session_bridge::native_interface::tests::Fixture::new();
        let services = NativeServices {
            engine: fixture.engine.clone(),
            bridge: fixture.bridge.clone(),
        };
        let (mut app, handle, _, _) = interface_shell::tests::fixture();
        let event = NativeHostInput {
            context: Some(fixture.owner()),
            cursor: None,
            modifiers: default(),
            actions: vec![],
            consumed: false,
            event: WindowEvent::KeyboardInput(KeyboardInput {
                key_code: KeyCode::Escape,
                logical_key: Key::Escape,
                text: None,
                state: ButtonState::Pressed,
                repeat: false,
                window: Entity::PLACEHOLDER,
            }),
        };
        let world = app.world_mut();
        for workspace in [Workspace::Solid, Workspace::Cam, Workspace::Drawing] {
            world.insert_resource(Workbench {
                workspace,
                ..default()
            });
            let mut editor = Editor::default();
            assert!(
                !inner(world, &handle, &services, &event, &mut editor).unwrap(),
                "Inactive authoring swallowed {workspace:?} Escape"
            );
        }
        let mut editor = Editor {
            tool: Some(Tool::Note),
            ..default()
        };
        assert!(inner(world, &handle, &services, &event, &mut editor).unwrap());
        assert!(editor.tool.is_none());
    }
}
