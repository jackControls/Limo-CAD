//! Native workbench chrome follows the same command catalog and proportions
//! as React. Retained controls keep the normal document/binding guards.
use super::*;
use chrome::{rect, Widgets};
use interface_shell::ribbon::{self, Icon};

#[derive(Resource, Default)]
pub(crate) struct NavigationRectangle(pub Option<InterfaceRect>);

mod drawing_paper;
pub(crate) mod cam;
pub(crate) mod cam_export;
pub(crate) mod cam_view;
mod ribbon_menu;
#[cfg(test)]
mod tests;
mod viewport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NavigationTool {
    #[default]
    Select,
    Orbit,
    Pan,
    Zoom,
    ZoomWindow,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Workspace {
    #[default]
    Solid,
    Drawing,
    Cam,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Command {
    Menu(String),
    Dismiss,
    Navigation(NavigationTool),
    Workspace(Workspace),
    CamView(cam_view::Command),
    CamExport(cam_export::Command),
}
#[derive(Resource, Default)]
struct Workbench {
    owner: Option<DocumentContext>,
    menu: Option<String>,
    menu_x: f32,
    navigation: NavigationTool,
    workspace: Workspace,
    sketch: bool,
    dial: Option<InterfaceRect>,
    widgets: Widgets,
    axes: Option<Entity>,
    paper_key: Option<(u64, nbcad_sketch::DrawingSheetDto, nbcad_core::UnitSystem)>,
    paper: Vec<drawing_paper::Segment>,
    paper_labels: Vec<drawing_paper::Label>,
    paper_fills: Vec<drawing_paper::Fill>,
}

fn same_document(previous: Option<&DocumentContext>, current: &DocumentContext) -> bool {
    previous.is_some_and(|previous| previous.window_id == current.window_id
        && previous.document_id == current.document_id)
}

impl Workbench {
    fn refresh_owner(&mut self, owner: &DocumentContext) {
        if self.owner.as_ref() == Some(owner) { return; }
        if !same_document(self.owner.as_ref(), owner) { self.workspace = Workspace::Solid; }
        self.menu = None;
        self.navigation = NavigationTool::Select;
        self.owner = Some(owner.clone());
        self.paper_key = None;
        self.paper.clear();
        self.paper_labels.clear();
        self.paper_fills.clear();
    }
}

pub(crate) fn modal(world: &World) -> Option<&'static str> {
    cam_export::modal(world).or_else(|| cam_view::modal(world)).or_else(|| world
        .get_resource::<Workbench>()
        .and_then(|s| s.menu.as_ref())
        .map(|_| "workbench-menu"))
}
pub(crate) fn escape(world: &mut World) {
    if cam_view::modal(world).is_some() { cam_view::escape(world); return; }
    cam_export::escape(world);
    if let Some(mut state) = world.get_resource_mut::<Workbench>() {
        state.menu = None;
    }
}
pub(crate) fn workspace(world: &World) -> Workspace {
    world
        .get_resource::<Workbench>()
        .map_or(Workspace::Solid, |state| state.workspace)
}
pub(crate) fn navigation(world: &World) -> NavigationTool {
    world
        .get_resource::<Workbench>()
        .map_or(NavigationTool::Select, |s| s.navigation)
}
pub(crate) fn dial(world: &World) -> Option<InterfaceRect> {
    world.get_resource::<Workbench>().and_then(|s| s.dial)
}
pub(crate) fn dial_key(world: &World) -> Option<nbcad_interface::ControlKey> {
    world
        .get_resource::<Workbench>()
        .and_then(|s| s.axes)
        .map(|e| nbcad_interface::ControlKey(e.to_bits()))
}
pub(crate) fn execute(world: &mut World, command: &Command) -> Result<Value, String> {
    if let Command::CamView(command) = command { return cam_view::execute(world, command); }
    world.init_resource::<Workbench>();
    let mut state = world.resource_mut::<Workbench>();
    match command {
        Command::Menu(menu) => {
            state.menu = (state.menu.as_ref() != Some(menu)).then(|| menu.clone())
        }
        Command::Dismiss => state.menu = None,
        Command::Navigation(tool) => {
            state.navigation = if state.navigation == *tool {
                NavigationTool::Select
            } else {
                *tool
            };
            state.menu = None;
        }
        Command::Workspace(workspace) => {
            state.workspace = *workspace;
            state.menu = None;
        }
        Command::CamView(_) => unreachable!(),
        Command::CamExport(_) => return Err("Post controls require their document receipt".into()),
    }
    Ok(json!({"handled":true}))
}

pub(super) fn tool_node() -> Node {
    ribbon::node(0., 0., 48.)
}

fn centered_button(
    widgets: &mut Widgets,
    world: &mut World,
    camera: Entity,
    key: &str,
    label: &str,
    caption: &str,
    command: NativeCommand,
    mut bounds: Node,
    selected: Option<bool>,
    disabled: bool,
    z: i32,
) -> Result<Entity, String> {
    let mut control = InterfaceControl::button("document/session", label);
    control.selected = selected;
    control.disabled = disabled;
    bounds.justify_content = JustifyContent::Center;
    let entity = widgets.button(
        world,
        camera,
        key,
        control,
        Some(caption),
        command,
        bounds,
        None,
        z,
    )?;
    if world.get::<ribbon::RibbonButton>(entity).is_none() {
        interface_shell::center_caption(world, entity);
        interface_shell::caption_size(world, entity, 10.);
    }
    Ok(entity)
}

pub(super) fn card(
    widgets: &mut Widgets,
    world: &mut World,
    camera: Entity,
    key: &str,
    mut bounds: Node,
    fill: Color,
    radius: f32,
    z: i32,
) {
    let theme = ViewportUiTheme::from_palette(&ViewportPalette::default());
    bounds.border = UiRect::all(px(1.));
    bounds.border_radius = BorderRadius::all(px(radius));
    widgets.panel(world, camera, key, bounds, fill, z);
    if let Some(entity) = widgets.entity(key) {
        world
            .entity_mut(entity)
            .insert(BorderColor::all(theme.edge));
    }
}

pub(super) fn synchronize(
    world: &mut World,
    camera: Entity,
    controls: &HashMap<String, Entity>,
    width: f32,
    height: f32,
    side: f32,
    sketch: bool,
    owner: &DocumentContext,
    services: &NativeServices,
) -> Result<(), String> {
    let mut state = world.remove_resource::<Workbench>().unwrap_or_default();
    let result = (|| {
        state.refresh_owner(owner);
        if sketch != state.sketch {
            state.navigation = NavigationTool::Select;
            state.sketch = sketch;
        }
        if sketch
            && state
                .menu
                .as_deref()
                .is_some_and(|menu| menu != "workspace")
        {
            state.menu = None;
        }
        if files::modal(world).is_some() || history::modal(world).is_some() {
            state.menu = None;
        }
        state.widgets.begin();
        ribbon_menu::synchronize(world, camera, controls, width, sketch, services, &mut state)?;
        if state.workspace == Workspace::Drawing && !sketch {
            state.dial = None;
            if let Some(entity) = state.axes.take() { world.despawn(entity); }
            for entity in controls.values() {
                if matches!(world.get::<NativeCommandBinding>(*entity).map(|binding| &binding.command),
                    Some(NativeCommand::Orient(_) | NativeCommand::Fit | NativeCommand::ClearSelection)) {
                    world.get_mut::<InterfaceControl>(*entity).unwrap().visible = false;
                }
            }
            drawing_paper::paint(world, camera, services, &mut state, width, height, side, controls)?;
        } else {
            state.paper_key = None;
            state.paper.clear();
            state.paper_labels.clear();
            viewport::synchronize(world, camera, controls, width, height, side, &mut state)?;
        }
        cam::synchronize(world, camera, services, owner, height, side,
            state.workspace == Workspace::Cam && !sketch)?;
        let cam_visible = state.workspace == Workspace::Cam && !sketch && feature::panel(world).is_none();
        cam_view::synchronize(world, camera, services, owner, width, side,
            cam_visible)?;
        cam_export::synchronize(world, camera, services, owner, width, height, side,
            cam_visible)?;
        state.widgets.finish(world);
        Ok(())
    })();
    world.insert_resource(state);
    result
}
