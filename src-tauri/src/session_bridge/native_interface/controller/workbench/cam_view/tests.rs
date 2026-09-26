use super::*;
use crate::session_bridge::{native_interface::tests::Fixture, parse_engine_envelope};

fn job() -> CamDocumentDto {
    let mut document: CamDocumentDto = serde_json::from_value(json!({
        "setups":[{"id":1,"name":"Face test","body_ids":[1],
            "wcs":{"origin":{"x":0.,"y":0.,"z":4.},"x_axis":[1.,0.,0.],"y_axis":[0.,1.,0.],"z_axis":[0.,0.,1.]},
            "stock":{"min":{"x":0.,"y":0.,"z":-4.},"max":{"x":12.,"y":8.,"z":0.}},
            "operations":[{"kind":"face","id":1,"name":"Face top","enabled":true,"tool_id":1,
                "bounds":{"min":{"x":0.,"y":0.},"max":{"x":12.,"y":8.}},
                "top_z":0.,"target_z":-1.,"step_over":1.,"step_down":1.,
                "clearance_z":5.,"retract_z":2.,"feed_height_z":1.,
                "cutting":{"spindle_rpm":8000,"feed_xy":600.,"feed_z":150.,"coolant":"off"}}]}],
        "active_setup_id":1,
        "tools":[{"id":1,"number":1,"name":"2 mm end mill","kind":"flat_end_mill",
            "diameter":2.,"flute_length":8.,"overall_length":30.,"flute_count":2}],
        "next_setup_id":2,"next_tool_id":2,"next_operation_id":2
    })).unwrap();
    document.setups[0].machine = Some(nbcad_cam::CamMachineAssignmentDto::three_axis(
        Default::default(),
    ));
    document.validate_for_editing().unwrap();
    document
}

fn fixture_world(fixture: &Fixture) -> (App, NativeServices, Entity, CamDocumentDto) {
    for (operation, arguments) in [
        (
            "sketch_begin",
            json!({"plane":{"type":"origin_plane","plane":"xy"}}),
        ),
        (
            "sketch_add_rectangle",
            json!({"mode":"two_point","p1":{"x":0.,"y":0.},"p2":{"x":12.,"y":8.},"ctrl_held":true}),
        ),
        ("sketch_finish", json!({})),
        (
            "solid_extrude",
            json!({"sketch_name":"Sketch1","profile_indices":[0],"extent":{"type":"distance","distance":3.}}),
        ),
        ("cam_set_document", serde_json::to_value(job()).unwrap()),
    ] {
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
    }
    let services = NativeServices {
        engine: fixture.engine.clone(),
        bridge: fixture.bridge.clone(),
    };
    let mut app = native_viewport::interface_scene_fixture();
    app.world_mut().init_resource::<Assets<Image>>();
    app.world_mut().init_resource::<ViewportUiAssets>();
    let camera = app.world_mut().spawn(InterfaceCamera).id();
    refresh_native_model(&fixture.engine, app.world_mut(), true).unwrap();
    (
        app,
        services,
        camera,
        fixture.engine.cam_document_snapshot(),
    )
}

fn state(services: &NativeServices, owner: &DocumentContext, document: CamDocumentDto) -> State {
    let receipt = services
        .bridge
        .native_document_receipt(&services.engine, owner)
        .unwrap();
    State {
        key: Some(Key {
            owner: owner.clone(),
            revision: receipt.revision,
            selection: None,
        }),
        document: Some(document),
        setup: Some(1),
        generation: 7,
        paths: true,
        ..default()
    }
}

fn prepared(message: &str) -> Prepared {
    Prepared {
        paths: Vec::new(),
        tool: None,
        simulation: None,
        stock: None,
        message: message.into(),
    }
}

fn marker(value: f32) -> ViewportPreview {
    ViewportPreview {
        lines: vec![ViewportLineLayer {
            color: [0.2, 0.7, 0.3, 1.],
            width: 1.,
            segments: vec![value, 0., 0., value, 1., 0.],
            ..default()
        }],
        ..default()
    }
}

#[test]
fn shared_planner_simulator_and_retained_mesh_match_without_mutating_intent() {
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let fixture = Fixture::new();
    let (app, _, _, document) = fixture_world(&fixture);
    let before =
        parse_engine_envelope(fixture.engine.engine_call("project_export_model", "")).unwrap();
    let setup = document.setup(1).unwrap();
    let mut request = simulation_request(app.world(), &document, setup, Some(1)).unwrap();
    request.voxel_size = Some(0.5);
    request.max_voxels = Some(20_000);
    assert_eq!(request.target.as_ref().unwrap().meshes.len(), 1);
    let expected = nbcad_cam::simulate_setup(&document, &request).unwrap();
    let expected_stock = crate::retained_cam_stock(&expected).unwrap();
    let rendered = prepare(
        &document,
        1,
        Some(1),
        Some(request.clone()),
        &CamSimulationCancellation::default(),
        None,
    )
    .unwrap();
    assert!(rendered.paths.iter().any(|p| !p.segments.is_empty()));
    assert!(rendered.tool.is_some());
    let simulation = rendered.simulation.unwrap();
    assert!(simulation.stock_mesh.is_none() && simulation.native_stock_present);
    assert!(simulation.removed_volume_mm3 > 0.);
    assert_eq!(simulation.removed_volume_mm3, expected.removed_volume_mm3);
    assert_eq!(simulation.comparison, expected.comparison);
    let stock = rendered.stock.unwrap();
    assert_eq!(*stock.positions, *expected_stock.positions);
    assert_eq!(*stock.normals, *expected_stock.normals);
    assert_eq!(
        parse_engine_envelope(fixture.engine.engine_call("project_export_model", "")).unwrap(),
        before
    );
    let cancelled = CamSimulationCancellation::default();
    cancelled.cancel();
    assert!(prepare(&document, 1, Some(1), Some(request), &cancelled, None).is_err());
}

#[test]
fn selected_operation_preview_uses_shared_prefix_and_arc_axes() {
    let mut document = job();
    let mut invalid_later = document.setups[0].operations[0].clone();
    if let nbcad_cam::CamOperationDto::Face { id, tool_id, .. } = &mut invalid_later {
        *id = 2;
        *tool_id = 999;
    }
    document.setups[0].operations.push(invalid_later);
    document.next_operation_id = 3;
    assert!(prepare(
        &document,
        1,
        None,
        None,
        &CamSimulationCancellation::default(),
        None
    )
    .is_err());
    assert!(prepare(
        &document,
        1,
        Some(1),
        None,
        &CamSimulationCancellation::default(),
        None
    )
    .is_ok());
    for (plane, from, to) in [
        (nbcad_cam::CamArcPlane::Xy, [1., 0., 0.], [0., 1., 2.]),
        (nbcad_cam::CamArcPlane::Xz, [0., 0., 1.], [1., 2., 0.]),
        (nbcad_cam::CamArcPlane::Yz, [0., 1., 0.], [2., 0., 1.]),
    ] {
        let point = |[x, y, z]: [f64; 3]| nbcad_cam::Point3Dto::new(x, y, z);
        let points = geometry::arc_points(point(from), point(to), point([0.; 3]), plane, false);
        assert_eq!(points.last(), Some(&point(to)));
        assert!(points.len() >= 8);
        let first = points[0];
        let radial = match plane {
            nbcad_cam::CamArcPlane::Xy => first.x.hypot(first.y),
            nbcad_cam::CamArcPlane::Xz => first.z.hypot(first.x),
            nbcad_cam::CamArcPlane::Yz => first.y.hypot(first.z),
        };
        assert!((radial - 1.).abs() < 1e-10);
    }
}

#[test]
fn cancelled_and_replaced_owner_completions_cannot_reinstall_cam_graphics() {
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let fixture = Fixture::new();
    let (mut app, services, camera, document) = fixture_world(&fixture);
    let mut view = state(&services, &fixture.owner(), document);
    let (send, receiver) = mpsc::channel();
    let cancellation = CamSimulationCancellation::default();
    view.pending = Some(Pending {
        key: view.key.clone().unwrap(),
        generation: view.generation,
        cancellation: cancellation.clone(),
        receiver: Mutex::new(receiver),
    });
    app.world_mut().insert_resource(view);
    execute(app.world_mut(), &Command::Cancel).unwrap();
    assert!(cancellation.is_cancelled());
    assert!(send
        .send(Ok(prepared("Cancelled result must not display")))
        .is_ok());
    synchronize(
        app.world_mut(),
        camera,
        &services,
        &fixture.owner(),
        1360.,
        280.,
        true,
    )
    .unwrap();
    assert!(app.world().resource::<State>().prepared.is_none());
    assert!(app.world().resource::<State>().pending.is_none());
    let (send, receiver) = mpsc::channel();
    {
        let mut view = app.world_mut().resource_mut::<State>();
        view.pending = Some(Pending {
            key: view.key.clone().unwrap(),
            generation: view.generation,
            cancellation: CamSimulationCancellation::default(),
            receiver: Mutex::new(receiver),
        });
    }
    let old_owner = fixture.owner();
    fixture
        .bridge
        .apply_native_history(&fixture.engine, &old_owner, false, || Ok(()))
        .unwrap();
    assert_ne!(fixture.owner(), old_owner);
    refresh_native_model(&fixture.engine, app.world_mut(), true).unwrap();
    native_viewport::apply_interface_preview(
        app.world_mut(),
        &fixture.owner().document_id,
        marker(91.),
    )
    .unwrap();
    assert!(send
        .send(Ok(prepared("Retired document result must not display")))
        .is_ok());
    synchronize(
        app.world_mut(),
        camera,
        &services,
        &fixture.owner(),
        1360.,
        280.,
        true,
    )
    .unwrap();
    assert!(app.world().resource::<State>().prepared.is_none());
    assert_eq!(
        native_viewport::interface_preview_snapshot(app.world()).lines[0].segments[0],
        91.
    );
    synchronize(
        app.world_mut(),
        camera,
        &services,
        &fixture.owner(),
        1360.,
        280.,
        false,
    )
    .unwrap();
}

#[test]
fn cam_overlay_restores_its_own_values_and_preserves_a_newer_overlay() {
    let _lock = crate::session_bridge::tests::TEST_LOCK.lock().unwrap();
    let fixture = Fixture::new();
    let (mut app, services, _, document) = fixture_world(&fixture);
    let owner = fixture.owner();
    let initial_stock = ViewportCamStock {
        positions: Arc::new(vec![0., 0., 0., 1., 0., 0., 0., 1., 0.]),
        normals: Arc::new(vec![0., 0., 1., 0., 0., 1., 0., 0., 1.]),
    };
    native_viewport::apply_interface_preview(app.world_mut(), &owner.document_id, marker(12.))
        .unwrap();
    native_viewport::apply_interface_cam_stock(
        app.world_mut(),
        &owner.document_id,
        Some(initial_stock.clone()),
    )
    .unwrap();
    let (_, _, mut initial, _) = native_viewport::interface_view_snapshot(app.world());
    initial.cam_stock_visible = true;
    native_viewport::apply_interface_view(
        app.world_mut(),
        &owner.document_id,
        None,
        Some(initial.clone()),
    )
    .unwrap();
    let mut view = state(&services, &owner, document);
    display(app.world_mut(), &services, &mut view).unwrap();
    restore(app.world_mut(), &services, &mut view).unwrap();
    assert_eq!(
        native_viewport::interface_preview_snapshot(app.world()).lines[0].segments[0],
        12.
    );
    assert_eq!(
        *native_viewport::interface_cam_stock_snapshot(app.world())
            .1
            .unwrap()
            .positions,
        *initial_stock.positions
    );
    assert!(
        native_viewport::interface_view_snapshot(app.world())
            .2
            .cam_stock_visible
    );
    display(app.world_mut(), &services, &mut view).unwrap();
    native_viewport::apply_interface_preview(app.world_mut(), &owner.document_id, marker(52.))
        .unwrap();
    let newer_stock = ViewportCamStock {
        positions: Arc::new(vec![2., 0., 0., 3., 0., 0., 2., 1., 0.]),
        normals: initial_stock.normals.clone(),
    };
    native_viewport::apply_interface_cam_stock(
        app.world_mut(),
        &owner.document_id,
        Some(newer_stock.clone()),
    )
    .unwrap();
    let (_, _, mut newer, _) = native_viewport::interface_view_snapshot(app.world());
    newer.cam_stock_visible = false;
    newer.ghosted_body_ids = vec![1];
    newer.selected_body_ids = vec![1];
    native_viewport::apply_interface_view(app.world_mut(), &owner.document_id, None, Some(newer))
        .unwrap();
    restore(app.world_mut(), &services, &mut view).unwrap();
    assert_eq!(
        native_viewport::interface_preview_snapshot(app.world()).lines[0].segments[0],
        52.
    );
    assert_eq!(
        *native_viewport::interface_cam_stock_snapshot(app.world())
            .1
            .unwrap()
            .positions,
        *newer_stock.positions
    );
    let presentation = native_viewport::interface_view_snapshot(app.world()).2;
    assert!(!presentation.cam_stock_visible);
    assert_eq!(presentation.ghosted_body_ids, vec![1]);
    assert_eq!(presentation.selected_body_ids, vec![1]);
}
