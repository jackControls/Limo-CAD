use super::*;
use crate::session_bridge::parse_engine_envelope as value;
use serde_json::json;

#[test]
fn native_retention_rebuilds_real_occt_geometry_and_preserves_model() {
    let state = AppState::new();
    value(state.bind_project_session("a")).unwrap();
    value(state.engine_call("begin_sketch", r#"{"type":"origin_plane","plane":"xy"}"#)).unwrap();
    value(state.engine_call("add_rectangle", r#"{"mode":"two_point","p1":{"x":-10.0,"y":-10.0},"p2":{"x":10.0,"y":10.0},"ctrl_held":false}"#)).unwrap();
    value(state.engine_call("end_sketch", "")).unwrap();
    value(state.solid_extrude(r#"{"sketch_name":"Sketch1","profile_indices":[0],"operation":"new_body","extent":{"type":"distance","distance":10.0},"taper_angle_deg":0.0,"flip":false,"target_body_ids":[]}"#)).unwrap();
    let model = value(state.engine_call("project_export_model", "")).unwrap();
    let request = json!({"expected_model_json":model}).to_string();
    let mesh = state.export_stl(&request).unwrap();
    assert!(!mesh.is_empty());
    let scene = serde_json::to_value(
        state
            .inner
            .lock()
            .unwrap()
            .active()
            .manager
            .solid_scene_ref(),
    )
    .unwrap();
    let revision = state.geometry_revision();
    assert!(!state.evict_inactive_project_session("a").unwrap());
    value(state.create_project_session("b")).unwrap();
    assert!(state.evict_inactive_project_session("a").unwrap());
    assert_eq!(state.cold_project_sessions(), ["a"]);
    assert_eq!(state.active_project_session_id(), "b");
    value(state.activate_project_session("a")).unwrap();
    assert!(state.cold_project_sessions().is_empty());
    assert_eq!(state.geometry_revision(), revision + 1);
    assert_eq!(
        value(state.engine_call("project_export_model", "")).unwrap(),
        model
    );
    assert_eq!(state.export_stl(&request).unwrap(), mesh);
    assert_eq!(
        serde_json::to_value(
            state
                .inner
                .lock()
                .unwrap()
                .active()
                .manager
                .solid_scene_ref()
        )
        .unwrap(),
        scene
    );
}

#[test]
fn native_retention_failed_reconstruction_keeps_snapshot_and_active_document() {
    let state = AppState::new();
    value(state.bind_project_session("a")).unwrap();
    value(state.create_project_session("b")).unwrap();
    state.evict_inactive_project_session("a").unwrap();
    let active = state.document_snapshot();
    let active_revision = state.geometry_revision();
    let original = {
        let mut workspace = state.inner.lock().unwrap();
        let NativeProject::Cold { model, .. } = workspace.sessions.get_mut("a").unwrap() else {
            panic!();
        };
        std::mem::replace(model, "{damaged".into())
    };
    assert!(value(state.activate_project_session("a")).is_err());
    assert_eq!(state.active_project_session_id(), "b");
    assert_eq!(state.document_snapshot(), active);
    assert_eq!(state.geometry_revision(), active_revision);
    {
        let mut workspace = state.inner.lock().unwrap();
        let NativeProject::Cold { model, .. } = workspace.sessions.get_mut("a").unwrap() else {
            panic!();
        };
        assert_eq!(model, "{damaged");
        *model = original;
    }
    value(state.activate_project_session("a")).unwrap();
    assert_eq!(state.active_project_session_id(), "a");
}

#[test]
fn native_retention_rejects_replay_that_changes_body_identity() {
    let state = AppState::new();
    value(state.bind_project_session("a")).unwrap();
    value(state.create_project_session("b")).unwrap();
    state.evict_inactive_project_session("a").unwrap();
    {
        let mut workspace = state.inner.lock().unwrap();
        let NativeProject::Cold { body_ids, .. } = workspace.sessions.get_mut("a").unwrap() else {
            panic!();
        };
        body_ids.push(nbcad_core::BodyId(999));
    }
    let error = value(state.activate_project_session("a")).unwrap_err();
    assert!(error.contains("bodies or feature errors"));
    assert_eq!(state.active_project_session_id(), "b");
    assert_eq!(state.cold_project_sessions(), ["a"]);
}
