use super::super::*;
use super::fields::Kind;
use super::runtime::{Command, Editor, Tool, native};
use bevy::ui::UiTransform;
use nbcad_interface::{Field as UiField, KeyChord};

fn target(
    world: &mut World,
    camera: Entity,
    e: &mut Editor,
    key: &str,
    mut control: InterfaceControl,
    command: Command,
    bounds: Node,
    color: Color,
    z: i32,
) -> Result<Entity, String> {
    e.widgets.panel(world, camera, key, bounds, color, z);
    let entity = e.widgets.entity(key).unwrap();
    // This painted rectangle is itself the control. A panel occluder at the
    // same stack position would mask its own pointer target.
    world
        .entity_mut(entity)
        .remove::<interface_shell::InterfaceOccluder>();
    let command = native(e.serial, command);
    if let Some(existing) = world.get::<InterfaceControl>(entity) {
        control.binding = existing.binding;
    }
    if world.get::<InterfaceControl>(entity) != Some(&control) {
        world.entity_mut(entity).insert(control);
    }
    if world
        .get::<NativeCommandBinding>(entity)
        .is_none_or(|binding| binding.command != command)
    {
        bind_command(world, entity, command)?;
    }
    Ok(entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
    use interface_shell::{PointerButton, PointerPhase};
    use nbcad_interface::{ControlInput, ControlKey};

    #[test]
    fn paper_targets_receive_pointer_actions_without_masking_themselves() {
        let (mut app, handle, _, _) = interface_shell::tests::fixture();
        let camera = app.world_mut().spawn_empty().id();
        let mut editor = Editor::default();
        for command in [Command::Select(25), Command::Anchor(0)] {
            // Repaint the same retained decoration as well as first creation.
            let entity = target(
                app.world_mut(),
                camera,
                &mut editor,
                "paper-target",
                InterfaceControl::button("Viewport", "Paper annotation"),
                command.clone(),
                rect(180., 190., 40., 20.),
                Color::NONE,
                21,
            )
            .unwrap();
            app.world_mut().entity_mut(entity).insert((
                ComputedNode {
                    size: Vec2::new(40., 20.),
                    inverse_scale_factor: 1.,
                    ..default()
                },
                UiGlobalTransform::from_translation(Vec2::new(200., 200.)),
                ComputedStackIndex(21),
                InheritedVisibility::VISIBLE,
            ));
            app.update();
            assert!(
                app.world()
                    .get::<interface_shell::InterfaceOccluder>(entity)
                    .is_none()
            );
            assert_eq!(
                handle.hit_key([300., 300.]),
                Some(ControlKey(entity.to_bits()))
            );
            assert!(
                handle
                    .pointer(PointerPhase::Down, [300., 300.], PointerButton::Primary)
                    .unwrap()
            );
            assert!(
                handle
                    .pointer(PointerPhase::Up, [300., 300.], PointerButton::Primary)
                    .unwrap()
            );
            let actions = handle.take_actions().unwrap();
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].control.key, ControlKey(entity.to_bits()));
            assert_eq!(actions[0].control.input, ControlInput::Click);
            handle.validate_action(&actions[0]).unwrap();
            assert_eq!(
                app.world()
                    .get::<NativeCommandBinding>(entity)
                    .unwrap()
                    .command,
                native(editor.serial, command)
            );
        }
    }
}

fn button(
    world: &mut World,
    camera: Entity,
    editor: &mut Editor,
    key: &str,
    label: &str,
    command: Command,
    bounds: Node,
    disabled: bool,
) -> Result<Entity, String> {
    let mut control = InterfaceControl::button("drawing/annotation", label);
    control.disabled = disabled;
    editor.widgets.button(
        world,
        camera,
        key,
        control,
        None,
        native(editor.serial, command),
        bounds,
        None,
        46,
    )
}

pub(super) fn paint(
    world: &mut World,
    camera: Entity,
    e: &mut Editor,
    height: f32,
    side: f32,
    state: &Workbench,
) -> Result<(), String> {
    let theme = crate::native_viewport::ui::theme(world);
    let Some(transform) = drawing_paper::transform(state) else {
        return Ok(());
    };
    let Some(paper) = state.widgets.entity("drawing-paper") else {
        return Ok(());
    };
    // Retained semantic targets use the same paper transform and clipping as
    // their rendered labels. Their empty captions never cover technical text.
    if e.tool.is_none() {
        for mark in drawing_paper::annotation_marks(world, state) {
            let center = transform.to_screen(mark.center);
            if center[0] < transform.clip.x - 100.
                || center[1] < transform.clip.y - 100.
                || center[0] > transform.clip.x + transform.clip.width + 100.
                || center[1] > transform.clip.y + transform.clip.height + 100.
            {
                continue;
            }
            let key = format!("drawing-annotation-{}", mark.id);
            let mut control = InterfaceControl::button(
                "drawing/annotation",
                format!("Edit annotation {}", mark.id),
            );
            control.selected = Some(e.selected == Some(mark.id));
            let bounds = rect(
                ((mark.center[0] - mark.size[0] * 0.5) * transform.scale) as f32,
                ((mark.center[1] - mark.size[1] * 0.5) * transform.scale) as f32,
                (mark.size[0] * transform.scale).max(8.) as f32,
                (mark.size[1] * transform.scale).max(8.) as f32,
            );
            let selected = e.selected == Some(mark.id);
            let entity = target(
                world,
                camera,
                e,
                &key,
                control,
                Command::Select(mark.id),
                bounds,
                if selected {
                    theme.accent.with_alpha(0.12)
                } else {
                    Color::NONE
                },
                19,
            )?;
            e.widgets.parent(world, &key, paper);
            world
                .entity_mut(entity)
                .insert(UiTransform::from_rotation(Rot2::radians(mark.angle)));
        }
    }
    if e.tool == Some(Tool::Linear) {
        let visible: Vec<_> = e
            .targets
            .iter()
            .enumerate()
            .filter(|(_, target)| transform.pick(transform.to_screen(target.paper)).is_some())
            .map(|(index, _)| index)
            .collect();
        if visible.len() > 4096 {
            e.message = "Too many visible anchors; zoom into the required view".into();
        } else {
            for index in visible {
                let target_data = &e.targets[index];
                let key = format!("drawing-anchor-{index}");
                let radius = (1.15 * transform.scale).max(3.);
                let selected = e.pair.first.as_ref().is_some_and(|(_, view, a)| {
                    *view == target_data.view_id
                        && super::anchors::same_anchor(a, &target_data.reference)
                });
                let mut control = InterfaceControl::button(
                    "drawing/anchors",
                    format!("View {} anchor {}", target_data.view_id, index + 1),
                );
                control.selected = Some(selected);
                let bounds = Node {
                    border_radius: BorderRadius::all(percent(50.)),
                    ..rect(
                        (target_data.paper[0] * transform.scale - radius) as f32,
                        (target_data.paper[1] * transform.scale - radius) as f32,
                        (radius * 2.) as f32,
                        (radius * 2.) as f32,
                    )
                };
                target(
                    world,
                    camera,
                    e,
                    &key,
                    control,
                    Command::Anchor(index),
                    bounds,
                    theme.accent.with_alpha(if selected { 1. } else { 0.65 }),
                    21,
                )?;
                e.widgets.parent(world, &key, paper);
            }
        }
    }
    if e.tool.is_none() && e.selected.is_none() {
        return Ok(());
    }
    let width = side.max(248.);
    let bottom = (height - 66.).max(280.);
    e.widgets.panel(
        world,
        camera,
        "annotation-panel",
        rect(0., 112., width, bottom - 112.),
        theme.panel,
        44,
    );
    let title = match e.tool {
        Some(Tool::Note) => "Place note",
        Some(Tool::Linear) => "Linear dimension",
        None => "Edit annotation",
    };
    e.widgets.text(
        world,
        camera,
        "annotation-title",
        rect(12., 121., width - 24., 20.),
        title,
        12.,
        45,
    );
    button(
        world,
        camera,
        e,
        "annotation-back",
        "Sheet setup",
        Command::Cancel,
        rect(10., 148., width - 20., 28.),
        false,
    )?;
    if e.tool == Some(Tool::Linear) {
        let message = if e.pair.first.is_some() {
            "Choose the second projected anchor in the same view."
        } else {
            "Choose two projected endpoints or circle centers in one view."
        };
        e.widgets.text(
            world,
            camera,
            "annotation-instruction",
            rect(12., 192., width - 24., 86.),
            message,
            11.,
            45,
        );
    }
    let mut y = 188.;
    let available = (bottom - 130. - y).max(48.);
    let page_size = ((available / 50.).floor() as usize).clamp(1, 5);
    let visible = super::fields::visible(&e.fields);
    e.page = e.page.min(visible.len().saturating_sub(1) / page_size);
    for index in visible
        .iter()
        .skip(e.page * page_size)
        .take(page_size)
        .copied()
    {
        let field = &e.fields[index];
        let field_key = format!("{:?}", field.id);
        let toggle = matches!(field.kind, Kind::Toggle);
        if !toggle {
            e.widgets.text(
                world,
                camera,
                &format!("annotation-label-{field_key}"),
                rect(12., y, width - 24., 16.),
                field.label,
                10.,
                45,
            );
        }
        let mut control = InterfaceControl::button("drawing/annotation", field.label);
        let h = if matches!(field.kind, Kind::Multiline) {
            64.
        } else {
            28.
        };
        if let Some(options) = field.options() {
            control.role = "combobox".into();
            control.owned_keys = [
                "ArrowUp",
                "ArrowDown",
                "ArrowLeft",
                "ArrowRight",
                "Home",
                "End",
            ]
            .map(KeyChord::plain)
            .into();
            control.field = UiField::Choice {
                value: field.text.clone(),
                options,
            };
        } else if toggle {
            control.role = "checkbox".into();
            control.selected = Some(field.text == "true");
            control.field = UiField::Toggle(field.text == "true");
        } else {
            control.field = UiField::Text {
                value: field.text.clone(),
                read_only: false,
                selection: None,
            };
        }
        let entity = e.widgets.button(
            world,
            camera,
            &format!("annotation-field-{field_key}"),
            control,
            Some(&field.caption()),
            native(e.serial, Command::Field(field.id)),
            rect(10., y + 16., width - 20., h),
            None,
            46,
        )?;
        if toggle {
            interface_shell::checkbox_button(world, entity, camera, field.text == "true");
        }
        if matches!(field.kind, Kind::Multiline) {
            interface_shell::fields::multiline::enable(world, entity)?;
        }
        y += h + 22.;
    }
    if visible.len() > page_size {
        button(
            world,
            camera,
            e,
            "annotation-fields-prev",
            "Previous fields",
            Command::Fields(-1),
            rect(10., y, (width - 26.) / 2., 26.),
            e.page == 0,
        )?;
        button(
            world,
            camera,
            e,
            "annotation-fields-next",
            "More fields",
            Command::Fields(1),
            rect(width / 2. + 3., y, (width - 26.) / 2., 26.),
            (e.page + 1) * page_size >= visible.len(),
        )?;
        y += 32.;
    }
    if !e.fields.is_empty() {
        let label = if e.tool == Some(Tool::Note) {
            "Place note"
        } else {
            "Apply annotation"
        };
        button(
            world,
            camera,
            e,
            "annotation-apply",
            label,
            Command::Apply,
            rect(10., y, (width - 26.) / 2., 28.),
            false,
        )?;
        button(
            world,
            camera,
            e,
            "annotation-reset",
            "Reset annotation",
            Command::Reset,
            rect(width / 2. + 3., y, (width - 26.) / 2., 28.),
            false,
        )?;
        y += 34.;
    }
    if e.selected.is_some() {
        button(
            world,
            camera,
            e,
            "annotation-delete",
            "Delete annotation",
            Command::Delete,
            rect(10., y, width - 20., 28.),
            false,
        )?;
        y += 34.;
    }
    let message = if !e.message.is_empty() {
        &e.message
    } else if e.dirty() {
        "Apply or reset before editing another item"
    } else if e.tool == Some(Tool::Note) {
        "Click the paper or enter a paper position, then Place note."
    } else if e.selected.is_some() {
        "Drag the annotation on paper to move it."
    } else {
        ""
    };
    e.widgets.text(
        world,
        camera,
        "annotation-message",
        rect(12., y, width - 24., (bottom - y - 8.).max(32.)),
        message,
        10.,
        45,
    );
    Ok(())
}
