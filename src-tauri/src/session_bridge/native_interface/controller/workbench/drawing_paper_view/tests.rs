use super::super::super::drawing_navigation_input;
use super::*;
use crate::native_viewport::winit_host::NativeHostInput;
use bevy::{
    input::{
        ButtonState,
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
    },
    window::{CursorMoved, WindowEvent},
};

fn input(owner: &DocumentContext, event: WindowEvent, cursor: [f32; 2]) -> NativeHostInput {
    NativeHostInput {
        context: Some(owner.clone()),
        cursor: Some(Vec2::from_array(cursor)),
        modifiers: default(),
        event,
        consumed: false,
        actions: vec![],
    }
}
fn button(state: ButtonState) -> WindowEvent {
    WindowEvent::MouseButtonInput(MouseButtonInput {
        button: MouseButton::Middle,
        state,
        window: Entity::PLACEHOLDER,
    })
}
fn moved(cursor: [f32; 2]) -> WindowEvent {
    WindowEvent::CursorMoved(CursorMoved {
        window: Entity::PLACEHOLDER,
        position: Vec2::from_array(cursor),
        delta: None,
    })
}

#[test]
fn ordered_paper_gestures_reuse_projection_clip_all_art_and_reject_retired_owners() {
    let (mut app, handle, _, _) = crate::native_viewport::interface_shell::tests::fixture();
    let owner = handle.frame().unwrap().context;
    let world = app.world_mut();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<ViewportUiAssets>();
    let camera = world.spawn(InterfaceCamera).id();
    let sheet: DrawingSheetDto = serde_json::from_value(
        json!({"id":1,"name":"Navigation","format":"a4","orientation":"landscape"}),
    )
    .unwrap();
    let source = edges::SourceKey::new(owner.clone(), 1, 1, &sheet);
    let mut state = Workbench {
        workspace: Workspace::Drawing,
        ..default()
    };
    state.refresh_owner(&owner);
    state.workspace = Workspace::Drawing;
    state.paper_key = Some((1, sheet, Default::default()));
    state.paper_view = Some(PaperView {
        camera,
        source: source.clone(),
        width: 1000.,
        height: 800.,
        side: 240.,
        navigation: Navigation::new(owner.clone(), 1, [297., 210.], pane(1000., 800., 240.))
            .unwrap(),
    });
    let mut cache = edges::EdgeCache::default();
    let key = raster(world, state.paper_view.as_ref().unwrap()).unwrap();
    let (image, region) = {
        let ready = cache
            .prepare(
                &mut world.resource_mut::<Assets<Image>>(),
                source,
                key,
                |_| panic!("Empty sheet must not project"),
            )
            .unwrap();
        (ready.image, ready.region)
    };
    world.insert_resource(cache);
    draw(world, &mut state, image.clone(), region).unwrap();
    let pane_entity = state.widgets.entity("drawing-content-clip").unwrap();
    let paper_entity = state.widgets.entity("drawing-paper").unwrap();
    assert_eq!(
        world.get::<Node>(pane_entity).unwrap().overflow,
        Overflow::clip()
    );
    assert_eq!(
        world.get::<Node>(paper_entity).unwrap().overflow,
        Overflow::clip()
    );
    assert_eq!(
        world.get::<ChildOf>(paper_entity).unwrap().parent(),
        pane_entity
    );
    let cursor = [540., 330.];
    let before = state.paper_view.as_ref().unwrap().navigation.transform();
    let picked = before.pick(cursor.map(f64::from)).unwrap();
    world.insert_resource(state);
    let mut wheel = input(
        &owner,
        WindowEvent::MouseWheel(MouseWheel {
            unit: MouseScrollUnit::Line,
            phase: bevy::input::touch::TouchPhase::Moved,
            x: 0.,
            y: 5.,
            window: Entity::PLACEHOLDER,
        }),
        cursor,
    );
    wheel.modifiers.ctrl = true;
    assert!(drawing_navigation_input::navigate(world, &handle, &wheel).unwrap());
    let after = world
        .resource::<Workbench>()
        .paper_view
        .as_ref()
        .unwrap()
        .navigation
        .transform();
    assert!(after.scale > before.scale);
    let actual = after.pick(cursor.map(f64::from)).unwrap();
    for i in 0..2 {
        assert!(
            (picked[i] - actual[i]).abs() < 1e-8,
            "Paper pick moved during anchored zoom"
        );
    }
    assert_eq!(
        world.resource::<Assets<Image>>().len(),
        1,
        "Navigation leaked a raster asset"
    );
    let state = world.resource::<Workbench>();
    let edge_entity = state.widgets.entity("drawing-projected-edges").unwrap();
    assert_eq!(world.get::<ImageNode>(edge_entity).unwrap().image, image);
    assert_eq!(
        world.get::<ChildOf>(edge_entity).unwrap().parent(),
        paper_entity
    );

    assert!(
        drawing_navigation_input::navigate(
            world,
            &handle,
            &input(&owner, button(ButtonState::Pressed), cursor)
        )
        .unwrap()
    );
    let outside = [100., 80.];
    assert!(
        drawing_navigation_input::navigate(world, &handle, &input(&owner, moved(outside), outside))
            .unwrap(),
        "Captured pan must continue outside the pane"
    );
    assert_ne!(
        world
            .resource::<Workbench>()
            .paper_view
            .as_ref()
            .unwrap()
            .navigation
            .transform()
            .origin,
        after.origin
    );
    assert!(
        drawing_navigation_input::navigate(
            world,
            &handle,
            &input(&owner, button(ButtonState::Released), outside)
        )
        .unwrap()
    );
    assert!(
        !drawing_navigation_input::navigate(world, &handle, &input(&owner, moved(cursor), cursor))
            .unwrap()
    );

    assert!(
        drawing_navigation_input::navigate(
            world,
            &handle,
            &input(&owner, button(ButtonState::Pressed), cursor)
        )
        .unwrap()
    );
    let retained = world
        .resource::<Workbench>()
        .paper_view
        .as_ref()
        .unwrap()
        .navigation
        .transform();
    let mut retired = owner.clone();
    retired.epoch += 1;
    assert!(
        !drawing_navigation_input::navigate(
            world,
            &handle,
            &input(&retired, moved(outside), outside)
        )
        .unwrap()
    );
    let state = world.resource::<Workbench>();
    assert!(!state.paper_view.as_ref().unwrap().navigation.is_panning());
    assert_eq!(
        state
            .paper_view
            .as_ref()
            .unwrap()
            .navigation
            .transform()
            .origin,
        retained.origin
    );
    assert!(
        !drawing_navigation_input::navigate(
            world,
            &handle,
            &input(&owner, moved(outside), outside)
        )
        .unwrap()
    );
}
