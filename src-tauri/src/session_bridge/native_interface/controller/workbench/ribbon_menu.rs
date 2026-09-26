use super::*;
use std::sync::OnceLock;

fn catalog() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../../../interface/catalog.json")).unwrap()
    })
}
fn label(entry: &Value) -> String {
    static ENGLISH: OnceLock<Value> = OnceLock::new();
    let english = ENGLISH.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../../../src/i18n/en.json")).unwrap()
    });
    entry["labelKey"]
        .as_str()
        .unwrap_or_default()
        .split('.')
        .fold(english, |value, key| &value[key])
        .as_str()
        .unwrap_or("")
        .to_owned()
}
fn key(id: &str) -> &str {
    match id {
        "fillet" => "solid-fillet",
        "chamfer" => "solid-chamfer",
        "shell" => "solid-shell",
        "externalThread" => "external-thread",
        "offsetPlane" => "offset-plane",
        "planeAtAngle" => "angle-plane",
        "mirror" => "solid-mirror",
        "patternRectangular" => "solid-rectangular-pattern",
        "patternCircular" => "solid-circular-pattern",
        "moveCopy" => "move-copy",
        "splitBody" => "split-body",
        "assemblyBrowser" => "assembly",
        "createJoint" => "joint",
        "select" => "clear",
        other => other,
    }
}
fn icon(id: &str) -> Icon {
    match id {
        "createSketch" => Icon::Sketch,
        "extrude" => Icon::Extrude,
        "revolve" => Icon::Revolve,
        "sweep" => Icon::Sweep,
        "loft" => Icon::Loft,
        "rib" => Icon::Rib,
        "hole" => Icon::Hole,
        "externalThread" => Icon::ExternalThread,
        "fillet" => Icon::Fillet,
        "chamfer" => Icon::Chamfer,
        "shell" => Icon::Shell,
        "mirror" => Icon::Mirror,
        "patternRectangular" => Icon::RectangularPattern,
        "patternCircular" => Icon::CircularPattern,
        "moveCopy" => Icon::MoveCopy,
        "combine" => Icon::Combine,
        "splitBody" => Icon::SplitBody,
        "constructionVisibility" | "offsetPlane" => Icon::OffsetPlane,
        "midplane" => Icon::Midplane,
        "planeAtAngle" => Icon::AnglePlane,
        "measure" => Icon::Measure,
        "sectionAnalysis" => Icon::Layers,
        "assemblyBrowser" => Icon::Boxes,
        "createJoint" => Icon::Joint,
        "select" => Icon::Select,
        _ => Icon::Square,
    }
}
fn source(world: &mut World, controls: &HashMap<String, Entity>, id: &str) -> Option<Entity> {
    if id == "createSketch" {
        world
            .query::<(Entity, &InterfaceControl)>()
            .iter(world)
            .find(|(_, c)| c.label == "Create Sketch")
            .map(|(e, _)| e)
    } else {
        controls.get(key(id)).copied()
    }
}

/// Keep each group's primary command visible, then shed secondary commands
/// in the same round-robin order as React. Every hidden command stays in its menu.
fn visible_counts(panels: &[Value], available: f32) -> Vec<usize> {
    let mut counts: Vec<_> = panels
        .iter()
        .map(|p| p["buttons"].as_array().unwrap().len())
        .collect();
    let total = |counts: &[usize]| {
        counts
            .iter()
            .map(|n| 9. + (*n as f32 * 50. - 2.).max(48.))
            .sum::<f32>()
    };
    let max = counts.iter().copied().max().unwrap_or(0);
    for slot in (1..max).rev() {
        for index in (0..counts.len()).rev() {
            if total(&counts) <= available {
                return counts;
            }
            if counts[index] > slot {
                counts[index] -= 1;
            }
        }
    }
    counts
}
fn references(world: &World, services: &NativeServices) -> Result<(NativeCommand, bool), String> {
    let mut visibility = crate::session_bridge::parse_engine_envelope(
        services.engine.engine_call("project_visibility", ""),
    )?;
    let document = services.engine.document_snapshot();
    let mut names = Vec::new();
    let mut planes = Vec::new();
    fn collect(nodes: &[nbcad_core::BrowserNode], names: &mut Vec<String>, planes: &mut Vec<u64>) {
        for node in nodes {
            match node.kind {
                nbcad_core::BrowserNodeKind::Sketch => {
                    if let Some(name) = &node.name {
                        names.push(name.clone());
                    }
                }
                nbcad_core::BrowserNodeKind::ConstructionPlane => {
                    if let Some(id) = node.reference_id {
                        planes.push(id);
                    }
                }
                _ => {}
            }
            collect(&node.children, names, planes);
        }
    }
    collect(&document.browser, &mut names, &mut planes);
    let (_, _, view, _) = native_viewport::interface_view_snapshot(world);
    let showing = names
        .iter()
        .any(|name| !view.hidden_sketch_names.contains(name))
        || planes
            .iter()
            .any(|id| !view.hidden_datum_plane_ids.contains(id));
    let disabled = names.is_empty() && planes.is_empty();
    visibility["hidden_sketch_names"] = if showing { json!(names) } else { json!([]) };
    visibility["hidden_datum_plane_ids"] = if showing { json!(planes) } else { json!([]) };
    Ok((
        NativeCommand::Mutation {
            operation: "project_set_visibility".into(),
            arguments: visibility,
        },
        disabled,
    ))
}

pub(super) fn synchronize(
    world: &mut World,
    camera: Entity,
    controls: &HashMap<String, Entity>,
    width: f32,
    sketch: bool,
    services: &NativeServices,
    state: &mut Workbench,
) -> Result<(), String> {
    let theme = ViewportUiTheme::from_palette(&ViewportPalette::default());
    let workspace_label = match state.workspace {
        Workspace::Solid => "Solid Modeling",
        Workspace::Drawing => "Drawing",
    };
    let workspace_width = if width > 1400. { 108. } else { 56. };
    // The workspace cell also exists in sketch mode; the editor owns the
    // sketch ribbon to its right, including its retained dropdowns.
    let workspace = centered_button(
        &mut state.widgets,
        world,
        camera,
        "workspace",
        "Switch workspace",
        workspace_label,
        NativeCommand::Workbench(Command::Menu("workspace".into())),
        ribbon::node(4., 34., workspace_width - 8.),
        Some(state.menu.as_deref() == Some("workspace")),
        false,
        42,
    )?;
    ribbon::decorate(world, workspace, Icon::Box);
    ribbon::caption(
        world,
        workspace,
        if width > 1400. { workspace_label } else { "" },
    );
    state.widgets.glyph(
        world,
        camera,
        "workspace-chevron",
        rect(workspace_width / 2. - 4., 77., 8., 8.),
        Icon::ChevronDown,
        theme.mute,
        43,
    );
    if let Some(mut c) = world.get_mut::<InterfaceControl>(workspace) {
        c.expanded = Some(state.menu.as_deref() == Some("workspace"));
        c.modal_scope = state.menu.as_ref().map(|_| "workbench-menu".into());
    }
    state.widgets.text(
        world,
        camera,
        "workspace-title",
        rect(0., 96., workspace_width, 16.),
        "WORKSPACE",
        8.,
        30,
    );
    if let Some(e) = state.widgets.entity("workspace-title") {
        world
            .entity_mut(e)
            .insert((TextLayout::justify(Justify::Center), TextColor(theme.mute)));
    }
    state.widgets.panel(
        world,
        camera,
        "workspace-divider",
        rect(workspace_width, 28., 1., 92.),
        theme.edge,
        30,
    );
    if sketch {
        if state.menu.as_deref() == Some("workspace") {
            menu(world, camera, width, 4., &[], controls, services, state)?;
        }
        return Ok(());
    }
    if state.workspace == Workspace::Drawing {
        drawing_ribbon(world, camera, workspace_width, services, state)?;
        if state.menu.as_deref() == Some("workspace") {
            menu(world, camera, width, 4., &[], controls, services, state)?;
        }
        return Ok(());
    }
    let panels = catalog()["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["id"] == "solid")
        .unwrap()["panels"]
        .as_array()
        .unwrap();
    let counts = visible_counts(panels, width - workspace_width - 6.);
    for entity in controls.values() {
        if matches!(
            world
                .get::<NativeCommandBinding>(*entity)
                .map(|b| &b.command),
            Some(
                NativeCommand::Feature(_)
                    | NativeCommand::Assembly(_)
                    | NativeCommand::ClearSelection
            )
        ) {
            if let Some(mut c) = world.get_mut::<InterfaceControl>(*entity) {
                c.visible = false;
            }
        }
    }
    let mut x = workspace_width;
    let mut open_entries = vec![];
    for (panel, count) in panels.iter().zip(counts) {
        let id = panel["id"].as_str().unwrap();
        let buttons = panel["buttons"].as_array().unwrap();
        let group_width = 9. + (count as f32 * 50. - 2.).max(48.);
        let group_label = label(panel);
        let mut entries = panel["menu"].as_array().cloned().unwrap_or_else(|| {
            buttons
                .iter()
                .map(|b| {
                    let mut b = b.clone();
                    b["type"] = json!("item");
                    b
                })
                .collect()
        });
        // Show refs is the primary button, but must remain reachable in the menu.
        if id == "reference" {
            let mut item = buttons[0].clone();
            item["type"] = json!("item");
            entries.insert(0, item);
        }
        let has_menu = id != "profile" && id != "selection";
        for (i, button) in buttons.iter().enumerate() {
            let bid = button["id"].as_str().unwrap();
            let existing = source(world, controls, bid);
            if let Some(entity) = existing {
                let visible = i < count;
                world.get_mut::<InterfaceControl>(entity).unwrap().visible = visible;
                if visible {
                    world.entity_mut(entity).insert(ribbon::node(
                        x + 4. + i as f32 * 50.,
                        34.,
                        48.,
                    ));
                    if bid == "select" {
                        ribbon::decorate(world, entity, Icon::Select);
                        ribbon::caption(world, entity, "Select");
                    }
                }
            } else if i < count {
                let (command, disabled) = if bid == "constructionVisibility" {
                    references(world, services)?
                } else {
                    (NativeCommand::Workbench(Command::Dismiss), true)
                };
                let entity = centered_button(
                    &mut state.widgets,
                    world,
                    camera,
                    &format!("tool-{bid}"),
                    &label(button),
                    &label(button),
                    command,
                    ribbon::node(x + 4. + i as f32 * 50., 34., 48.),
                    None,
                    disabled,
                    30,
                )?;
                ribbon::decorate(world, entity, icon(bid));
            }
        }
        let selected = state.menu.as_deref() == Some(id);
        let caption = group_label.clone();
        let entity = centered_button(
            &mut state.widgets,
            world,
            camera,
            &format!("group-{id}"),
            &format!("{group_label} tools"),
            &caption,
            NativeCommand::Workbench(Command::Menu(id.into())),
            rect(x + 4., 90., group_width - 9., 20.),
            Some(selected),
            !has_menu,
            42,
        )?;
        if let Some(mut c) = world.get_mut::<InterfaceControl>(entity) {
            c.expanded = has_menu.then_some(selected);
            c.modal_scope = state.menu.as_ref().map(|_| "workbench-menu".into());
        }
        interface_shell::caption_size(world, entity, 10.);
        interface_shell::caption_tracking(world, entity, 0.5);
        if has_menu {
            let chevron_key = format!("chevron-{id}");
            state.widgets.glyph(
                world,
                camera,
                &chevron_key,
                Node {
                    width: px(10.),
                    height: px(10.),
                    margin: UiRect::left(px(2.)),
                    flex_shrink: 0.,
                    ..default()
                },
                Icon::ChevronDown,
                theme.mute,
                1,
            );
            state.widgets.parent(world, &chevron_key, entity);
        }
        state.widgets.panel(
            world,
            camera,
            &format!("divider-{id}"),
            rect(x + group_width - 1., 28., 1., 92.),
            theme.edge,
            30,
        );
        if selected {
            state.menu_x = x;
            open_entries = std::mem::take(&mut entries);
        }
        x += group_width;
    }
    if state.menu.is_some() {
        menu(
            world,
            camera,
            width,
            state.menu_x,
            &open_entries,
            controls,
            services,
            state,
        )?;
    }
    Ok(())
}
fn menu(
    world: &mut World,
    camera: Entity,
    width: f32,
    anchor: f32,
    entries: &[Value],
    controls: &HashMap<String, Entity>,
    services: &NativeServices,
    state: &mut Workbench,
) -> Result<(), String> {
    let theme = ViewportUiTheme::from_palette(&ViewportPalette::default());
    let workspace = state.menu.as_deref() == Some("workspace");
    let x = if workspace {
        4.
    } else {
        anchor.min(width - 264.).max(4.)
    };
    let menu_height = if workspace {
        104.
    } else {
        8. + entries
            .iter()
            .map(|e| if e["type"] == "separator" { 9. } else { 30. })
            .sum::<f32>()
    };
    let window_height = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world)
        .map_or(860., |w| w.height());
    state.widgets.backdrop(
        world,
        camera,
        "menu-dismiss",
        "workbench-menu",
        NativeCommand::Workbench(Command::Dismiss),
        rect(0., 120., width, window_height - 120.),
        59,
    )?;
    card(
        &mut state.widgets,
        world,
        camera,
        "menu-card",
        rect(x, 120., 256., menu_height),
        theme.panel.with_alpha(1.),
        5.,
        60,
    );
    let e = state.widgets.entity("menu-card").unwrap();
    world.entity_mut(e).insert(bevy::ui::BoxShadow::new(
        theme.shadow,
        px(0),
        px(8),
        px(0),
        px(24),
    ));
    let mut y = 124.;
    let workspace_entries: Vec<Value> = ["Solid Modeling", "Drawing", "CAM"]
        .into_iter()
        .map(|name| json!({"id":name,"name":name}))
        .collect();
    for (index, item) in if workspace {
        &workspace_entries[..]
    } else {
        entries
    }
    .iter()
    .enumerate()
    {
        if item["type"] == "separator" {
            state.widgets.panel(
                world,
                camera,
                &format!("menu-separator-{index}"),
                rect(x + 8., y + 4., 240., 1.),
                theme.edge,
                61,
            );
            y += 9.;
            continue;
        }
        let id = item["id"].as_str().unwrap();
        let name = if workspace {
            id.to_owned()
        } else {
            label(item)
        };
        let source = source(world, controls, id);
        let (command, disabled) = if workspace {
            match id {
                "Solid Modeling" => (
                    NativeCommand::Workbench(Command::Workspace(Workspace::Solid)),
                    false,
                ),
                "Drawing" => (
                    NativeCommand::Workbench(Command::Workspace(Workspace::Drawing)),
                    false,
                ),
                _ => (NativeCommand::Workbench(Command::Dismiss), true),
            }
        } else if let Some(entity) = source {
            (
                world
                    .get::<NativeCommandBinding>(entity)
                    .unwrap()
                    .command
                    .clone(),
                world.get::<InterfaceControl>(entity).unwrap().disabled,
            )
        } else if id == "constructionVisibility" {
            references(world, services)?
        } else {
            (NativeCommand::Workbench(Command::Dismiss), true)
        };
        let mut control = InterfaceControl::button("workbench-menu", &name);
        control.modal_scope = Some("workbench-menu".into());
        control.role = "menuitem".into();
        control.owned_keys = ["ArrowUp", "ArrowDown", "Home", "End"]
            .map(nbcad_interface::KeyChord::plain)
            .into();
        control.disabled = disabled;
        control.selected = (workspace && id == workspace_name(state.workspace)).then_some(true);
        let entity = state.widgets.button(
            world,
            camera,
            &format!("menu-row-{index}"),
            control,
            None,
            command,
            rect(x + 4., y, 248., 28.),
            Some(if workspace { Icon::Box } else { icon(id) }),
            62,
        )?;
        interface_shell::caption_size(world, entity, 11.);
        if workspace && disabled {
            state.widgets.text(
                world,
                camera,
                &format!("pending-{index}"),
                rect(x + 140., y + 7., 106., 14.),
                "Unavailable",
                9.,
                63,
            );
        }
        y += 30.;
    }
    Ok(())
}

fn workspace_name(workspace: Workspace) -> &'static str {
    match workspace {
        Workspace::Solid => "Solid Modeling",
        Workspace::Drawing => "Drawing",
    }
}

fn drawing_ribbon(
    world: &mut World,
    camera: Entity,
    workspace_width: f32,
    services: &NativeServices,
    state: &mut Workbench,
) -> Result<(), String> {
    let drawing = services.engine.drawing_snapshot();
    let active_id = drawing
        .active_sheet_id
        .or_else(|| drawing.sheets.last().map(|sheet| sheet.id));
    let active = drawing
        .sheets
        .iter()
        .find(|sheet| Some(sheet.id) == active_id);
    let new_sheet = centered_button(
        &mut state.widgets,
        world,
        camera,
        "drawing-new-sheet",
        "New sheet",
        "New sheet",
        NativeCommand::Mutation {
            operation: "drawing_create_sheet".into(),
            arguments: json!({
                "name": format!("Sheet {}", drawing.sheets.len() + 1),
                "format": "a4",
                "orientation": "landscape"
            }),
        },
        ribbon::node(workspace_width + 4., 34., 48.),
        None,
        false,
        30,
    )?;
    ribbon::decorate(world, new_sheet, Icon::Rectangle);
    let delete = match active_id {
        Some(sheet_id) => NativeCommand::Mutation {
            operation: "drawing_delete_sheet".into(),
            arguments: json!({ "sheet_id": sheet_id }),
        },
        None => NativeCommand::Workbench(Command::Dismiss),
    };
    let delete_button = centered_button(
        &mut state.widgets,
        world,
        camera,
        "drawing-delete-sheet",
        "Delete sheet",
        "Delete",
        delete,
        ribbon::node(workspace_width + 54., 34., 48.),
        None,
        active_id.is_none(),
        30,
    )?;
    ribbon::decorate(world, delete_button, Icon::Cancel);
    let status = match active {
        Some(sheet) => format!("{} · {} views", sheet.name, sheet.views.len()),
        None => "No sheet".into(),
    };
    centered_button(
        &mut state.widgets,
        world,
        camera,
        "drawing-status",
        &status,
        &status,
        NativeCommand::Workbench(Command::Dismiss),
        rect(workspace_width + 4., 90., 160., 18.),
        None,
        true,
        30,
    )?;
    for (index, sheet) in drawing.sheets.iter().take(6).enumerate() {
        let selected = Some(sheet.id) == active_id;
        centered_button(
            &mut state.widgets,
            world,
            camera,
            &format!("drawing-sheet-{}", sheet.id),
            &sheet.name,
            &sheet.name,
            NativeCommand::Mutation {
                operation: "drawing_select_sheet".into(),
                arguments: json!({ "sheet_id": sheet.id }),
            },
            rect(workspace_width + 170. + index as f32 * 78., 90., 74., 18.),
            Some(selected),
            false,
            30,
        )?;
    }
    for (index, (key, name, kind, direction, up, position)) in [
        ("drawing-front", "Front", "front", [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [110.0, 120.0]),
        ("drawing-top", "Top", "top", [0.0, 0.0, 1.0], [0.0, 1.0, 0.0], [110.0, 50.0]),
        ("drawing-bottom", "Bottom", "bottom", [0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [110.0, 175.0]),
        ("drawing-left", "Left", "left", [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [40.0, 120.0]),
        ("drawing-right", "Right", "right", [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [190.0, 120.0]),
        ("drawing-iso", "Isometric", "isometric", [1.0, -1.0, 1.0], [0.0, 0.0, 1.0], [230.0, 55.0]),
    ]
    .into_iter()
    .enumerate()
    {
        let command = match active_id {
            Some(sheet_id) => NativeCommand::Mutation {
                operation: "drawing_add_view".into(),
                arguments: json!({
                    "sheet_id": sheet_id,
                    "view": {
                        "name": name,
                        "kind": kind,
                        "direction": direction,
                        "up": up,
                        "position": position,
                        "scale": 1.0
                    }
                }),
            },
            None => NativeCommand::Workbench(Command::Dismiss),
        };
        let entity = centered_button(
            &mut state.widgets,
            world,
            camera,
            key,
            name,
            name,
            command,
            ribbon::node(workspace_width + 108. + index as f32 * 50., 34., 48.),
            None,
            active_id.is_none(),
            30,
        )?;
        ribbon::decorate(world, entity, Icon::Box);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn responsive_groups_preserve_primaries_and_restore_commands_as_space_returns() {
        let panels = catalog()["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| w["id"] == "solid")
            .unwrap()["panels"]
            .as_array()
            .unwrap();
        let compact = visible_counts(panels, 1140.);
        let roomy = visible_counts(panels, 1850.);
        assert_eq!(compact.len(), 9);
        assert!(compact.iter().all(|n| *n >= 1));
        assert!(compact.iter().sum::<usize>() < roomy.iter().sum::<usize>());
        for (panel, count) in panels.iter().zip(roomy) {
            assert_eq!(count, panel["buttons"].as_array().unwrap().len());
        }
        // The overflow list comes from the shared catalog, including shell and
        // the angled plane which are not necessarily direct ribbon buttons.
        assert!(panels.iter().any(|p| p["menu"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|r| r["id"] == "shell"))));
        assert!(panels.iter().any(|p| p["menu"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|r| r["id"] == "planeAtAngle"))));
    }
}
