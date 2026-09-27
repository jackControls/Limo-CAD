use super::super::*;
use super::runtime::{Command, Drag, Editor, Tool, preview, submit};
use super::{
    draft::{Draft, Selection},
    *,
};
use crate::native_viewport::winit_host::NativeHostInput;
use bevy::{
    input::{ButtonState, keyboard::Key},
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
            || e.tool.is_some()
            || e.selected.is_some();
        e.pair.cancel();
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
        return Ok(false);
    }
    let receipt = services
        .bridge
        .native_document_receipt(&services.engine, &stamp.owner)?;
    if receipt.revision != stamp.revision {
        e.drag = None;
        e.pair.cancel();
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
    if !matches!(&input.event,WindowEvent::MouseButtonInput(b) if b.button==MouseButton::Left && b.state==ButtonState::Pressed)
    {
        return Ok(false);
    }
    let Some(point) = transform.pick(cursor) else {
        return Ok(false);
    };
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
        if matches!(
            draft.annotation(),
            nbcad_sketch::DrawingAnnotationDto::LinearDimension { .. }
        ) && mark.linear_points.is_none()
        {
            return Err("Repair the dimension's projected anchors before dragging it".into());
        }
        e.drag = Some(Drag {
            stamp,
            start: point,
            draft,
            linear_points: mark.linear_points,
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
