use super::*;
use crate::session_bridge::native_interface::tests::Fixture;

#[test]
fn native_ribbon_menus_retain_disabled_commands_and_navigation_toggles() {
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let fixture = Fixture::new();
    let services = NativeServices {
        engine: fixture.engine.clone(),
        bridge: fixture.bridge.clone(),
    };
    let mut app = native_viewport::interface_scene_fixture();
    let world = app.world_mut();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<ViewportUiAssets>();
    let camera = world.spawn(InterfaceCamera).id();
    let owner = fixture.owner();
    let mut state = Workbench::default();
    state.widgets.begin();
    ribbon_menu::synchronize(
        world,
        camera,
        &HashMap::new(),
        1200.,
        false,
        &services,
        &mut state,
    )
    .unwrap();
    world.insert_resource(state);
    execute(world, &Command::Menu("refine".into())).unwrap();
    assert_eq!(modal(world), Some("workbench-menu"));
    let mut state = world.remove_resource::<Workbench>().unwrap();
    state.widgets.begin();
    ribbon_menu::synchronize(
        world,
        camera,
        &HashMap::new(),
        1200.,
        false,
        &services,
        &mut state,
    )
    .unwrap();
    let draft = world
        .query::<&InterfaceControl>()
        .iter(world)
        .find(|c| c.label == "Draft")
        .unwrap();
    assert!(draft.disabled);
    assert_eq!(draft.role, "menuitem");
    assert_eq!(draft.modal_scope.as_deref(), Some("workbench-menu"));
    state.owner = Some(owner.clone());
    world.insert_resource(state);
    escape(world);
    assert_eq!(modal(world), None);
    execute(world, &Command::Menu("workspace".into())).unwrap();
    let mut state = world.remove_resource::<Workbench>().unwrap();
    state.widgets.begin();
    ribbon_menu::synchronize(
        world,
        camera,
        &HashMap::new(),
        1200.,
        false,
        &services,
        &mut state,
    )
    .unwrap();
    let drawing = world
        .query::<&InterfaceControl>()
        .iter(world)
        .find(|c| c.label == "Drawing")
        .unwrap();
    assert!(!drawing.disabled);
    assert_eq!(drawing.role, "menuitem");
    let cam = world
        .query::<&InterfaceControl>()
        .iter(world)
        .find(|c| c.label == "CAM")
        .unwrap();
    assert!(cam.disabled);
    world.insert_resource(state);
    execute(world, &Command::Workspace(Workspace::Drawing)).unwrap();
    assert_eq!(workspace(world), Workspace::Drawing);
    let mut state = world.remove_resource::<Workbench>().unwrap();
    state.widgets.begin();
    ribbon_menu::synchronize(
        world,
        camera,
        &HashMap::new(),
        1200.,
        false,
        &services,
        &mut state,
    )
    .unwrap();
    let sheet = world
        .query::<&InterfaceControl>()
        .iter(world)
        .find(|c| c.label == "New sheet")
        .unwrap();
    assert!(!sheet.disabled);
    let front = world
        .query::<&InterfaceControl>()
        .iter(world)
        .find(|c| c.label == "Front")
        .unwrap();
    assert!(front.disabled);
    world.insert_resource(state);
    execute(world, &Command::Navigation(NavigationTool::Pan)).unwrap();
    assert_eq!(navigation(world), NavigationTool::Pan);
    execute(world, &Command::Navigation(NavigationTool::Pan)).unwrap();
    assert_eq!(navigation(world), NavigationTool::Select);
}
