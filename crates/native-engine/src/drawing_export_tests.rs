//! Real kernel/engine unit plumbing without creating a window or input events.
use super::*;
use serde_json::{json, Value};

fn value(json: String) -> Value {
    let envelope: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(envelope["ok"], true, "{envelope}");
    envelope["value"].clone()
}

#[test]
fn section_inspection_reads_real_material_and_leaves_model_and_revision_unchanged() {
    let state = NativeEngineHost::new();
    value(state.engine_call("begin_sketch", r#"{"type":"origin_plane","plane":"xy"}"#));
    value(state.engine_call("add_rectangle", r#"{"mode":"two_point","p1":{"x":10.0,"y":20.0},"p2":{"x":50.0,"y":50.0},"ctrl_held":true}"#));
    value(state.engine_call("end_sketch", ""));
    value(state.solid_extrude(r#"{"sketch_name":"Sketch1","profile_indices":[0],"operation":"new_body","extent":{"type":"distance","distance":6.0},"taper_angle_deg":0.0,"flip":false,"target_body_ids":[]}"#));
    let body_id = state.viewport_snapshot().2.bodies[0].id;
    value(state.engine_call("begin_sketch", r#"{"type":"origin_plane","plane":"xy"}"#));
    value(state.engine_call("add_rectangle", r#"{"mode":"two_point","p1":{"x":20.0,"y":30.0},"p2":{"x":40.0,"y":40.0},"ctrl_held":true}"#));
    value(state.engine_call("end_sketch", ""));
    value(state.solid_extrude(&json!({"sketch_name":"Sketch2","profile_indices":[0],"operation":"cut","extent":{"type":"distance","distance":6.},"taper_angle_deg":0.,"flip":false,"target_body_ids":[body_id]}).to_string()));
    let body_id = state.viewport_snapshot().2.bodies[0].id;
    let before = value(state.engine_call("project_export_model", ""));
    let revision = state.geometry_revision();
    for (plane, offset, probe, expected, bounds) in [
        ("xy", 3., 35., 10., [10., 20., 50., 50.]),
        ("xz", 35., 3., 10., [10., 0., 50., 6.]),
        ("yz", 30., 3., 10., [20., 0., 50., 6.]),
    ] {
        let report = value(
            state.engine_call(
                "solid_section_review",
                &json!({"body_id":body_id,"plane":plane,"offset_mm":offset,"probe_mm":probe})
                    .to_string(),
            ),
        );
        assert_eq!(
            report["probe_spans"].as_array().unwrap().len(),
            2,
            "{report}"
        );
        for span in report["probe_spans"].as_array().unwrap() {
            assert!(
                (span["length_mm"].as_f64().unwrap() - expected).abs() < 1e-6,
                "{report}"
            );
        }
        for (i, expected) in bounds.into_iter().enumerate() {
            assert!(
                (report["bounds_mm"][i].as_f64().unwrap() - expected).abs() < 1e-6,
                "{report}"
            );
        }
        assert!(
            report["svg"].as_str().unwrap().contains("material")
                || report["svg"].as_str().unwrap().contains("probe spans")
        );
        if let Ok(path) = std::env::var("LIMO_SECTION_QA_SVG") {
            if plane == "xy" {
                std::fs::write(path, report["svg"].as_str().unwrap()).unwrap();
            }
        }
    }
    let empty = value(state.engine_call(
        "solid_section_review",
        &json!({"body_id":body_id,"plane":"xy","offset_mm":60.}).to_string(),
    ));
    assert!(empty["bounds_mm"].is_null());
    assert_eq!(empty["svg"], "");
    let bad: Value = serde_json::from_str(&state.engine_call(
        "solid_section_review",
        r#"{"body_id":999,"plane":"xy","offset_mm":3}"#,
    ))
    .unwrap();
    assert_eq!(bad["ok"], false);
    assert_eq!(value(state.engine_call("project_export_model", "")), before);
    assert_eq!(state.geometry_revision(), revision);
}

#[test]
fn native_straight_export_uses_loaded_document_units_and_preserves_exact_project() {
    let state = NativeEngineHost::new();
    value(state.engine_call("begin_sketch", r#"{"type":"origin_plane","plane":"xy"}"#));
    value(state.engine_call(
        "add_rectangle",
        r#"{"mode":"two_point","p1":{"x":0.0,"y":0.0},"p2":{"x":40.0,"y":30.0},"ctrl_held":true}"#,
    ));
    value(state.engine_call("end_sketch", ""));
    value(state.solid_extrude(r#"{"sketch_name":"Sketch1","profile_indices":[0],"operation":"new_body","extent":{"type":"distance","distance":6.0},"taper_angle_deg":0.0,"flip":false,"target_body_ids":[]}"#));
    let scene = state.viewport_snapshot().2;
    let body = &scene.bodies[0];
    let edge = body
        .edges
        .iter()
        .find(|edge| {
            let a = edge.points.first().unwrap();
            let b = edge.points.last().unwrap();
            ((a.x - b.x).abs() - 40.).abs() < 1e-6
                && (a.y - b.y).abs() < 1e-6
                && (a.z - b.z).abs() < 1e-6
        })
        .unwrap();
    let mut manager = limo_cad_sketch::SketchManager::new();
    let mut drawing = manager
        .drawing_command(
            serde_json::from_value(json!({"type":"create_sheet","arguments":{
                "name":"Native straight unit QA","format":"a4","orientation":"landscape"
            }}))
            .unwrap(),
        )
        .unwrap();
    drawing.sheets[0].views.push(
        serde_json::from_value(
            json!({"id":1,"name":"Top","kind":"top","body_ids":[body.id],
                "direction":[0.,0.,1.],"up":[0.,1.,0.],"position":[100.,75.],"scale":1.
            }),
        )
        .unwrap(),
    );
    drawing.next_view_id = 2;
    drawing.sheets[0].annotations.push(serde_json::from_value(json!({"kind":"line_dimension","id":1,"view_id":1,
        "first":{"body_id":body.id,"edge_id":edge.id,"edge_key":edge.key,"topology_signature":limo_cad_sketch::drawing_topology::drawing_body_signature(body),
            "fallback_start":[999.,999.,999.],"fallback_end":[998.,999.,999.]},
        "mode":"length","position":[100.,105.],"precision":3,"prefix":"L="
    })).unwrap());
    drawing.next_annotation_id = 2;
    value(state.engine_call(
        "drawing_set_document",
        &serde_json::to_string(&drawing).unwrap(),
    ));
    for (unit, expected) in [
        ("mm", "L=40.000 mm"),
        ("cm", "L=4.000 cm"),
        ("in", "L=1.575 in"),
    ] {
        let model = value(state.engine_call("project_export_model", ""));
        let mut model: Value = serde_json::from_str(model.as_str().unwrap()).unwrap();
        model["document"]["settings"]["units"] = json!(unit);
        value(state.project_load(&serde_json::to_string(&model.to_string()).unwrap()));
        let before = value(state.engine_call("project_export_model", ""));
        let revision = state.geometry_revision();
        for format in ["svg", "dxf"] {
            let exported =
                value(state.drawing_export(&json!({"sheet_id":1,"format":format}).to_string()));
            assert_eq!(exported["format"], format);
            assert!(exported["content"].as_str().unwrap().contains(expected));
            assert!(!exported["content"].as_str().unwrap().contains("999.00000"));
            assert_eq!(value(state.engine_call("project_export_model", "")), before);
            assert_eq!(state.geometry_revision(), revision);
        }
    }
}

#[test]
fn native_center_export_uses_real_circular_edges_without_changing_project_history() {
    let state = NativeEngineHost::new();
    for (index, x) in [60., 100.].into_iter().enumerate() {
        value(state.engine_call("begin_sketch", r#"{"type":"origin_plane","plane":"xy"}"#));
        value(
            state.engine_call(
                "add_circle",
                &json!({"mode":"center_diameter","p1":{"x":x,"y":15.},
            "p2":{"x":x+3.+index as f64,"y":15.},"ctrl_held":true})
                .to_string(),
            ),
        );
        value(state.engine_call("end_sketch", ""));
        value(state.solid_extrude(&json!({"sketch_name":format!("Sketch{}",index+1),"profile_indices":[0],"operation":"new_body",
            "extent":{"type":"distance","distance":10.},"taper_angle_deg":0.,"flip":false,"target_body_ids":[]}).to_string()));
    }
    let scene = state.viewport_snapshot().2;
    assert_eq!(scene.bodies.len(), 2);
    let projection: limo_cad_occt::DrawingProjectionDto =
        serde_json::from_value(value(state.drawing_projection(
            &json!({"direction":[0.,0.,1.],"up":[0.,1.,0.],"include_hidden":true}).to_string(),
        )))
        .unwrap();
    let reference = |body: limo_cad_core::BodyId| {
        let circle = projection
            .circles
            .iter()
            .find(|circle| circle.body_id == body && circle.closed)
            .unwrap();
        json!({"body_id":circle.body_id,"edge_id":circle.edge_id,"edge_key":circle.edge_key,"occurrence_id":circle.occurrence_id,
            "topology_signature":projection.topology_signatures[&body.0.to_string()],"fallback_center":[999.,999.,999.],
            "fallback_normal":[0.,0.,1.],"fallback_radius":999.,"closed":true})
    };
    let mut manager = limo_cad_sketch::SketchManager::new();
    let mut drawing = manager
        .drawing_command(
            serde_json::from_value(json!({"type":"create_sheet","arguments":{
        "name":"Real cylindrical centers","format":"a4","orientation":"landscape"}}))
            .unwrap(),
        )
        .unwrap();
    drawing.sheets[0].views.push(
        serde_json::from_value(
            json!({"id":1,"name":"Top","kind":"top","direction":[0.,0.,1.],
        "up":[0.,1.,0.],"position":[100.,75.],"scale":2.}),
        )
        .unwrap(),
    );
    drawing.sheets[0].annotations=vec![
        serde_json::from_value(json!({"kind":"center_mark","id":1,"view_id":1,"feature":reference(scene.bodies[0].id),"extension":4.})).unwrap(),
        serde_json::from_value(json!({"kind":"center_line","id":2,"view_id":1,"first":reference(scene.bodies[0].id),
            "second":reference(scene.bodies[1].id),"extension":5.5})).unwrap(),
    ];
    drawing.next_view_id = 2;
    drawing.next_annotation_id = 3;
    value(state.engine_call(
        "drawing_set_document",
        &serde_json::to_string(&drawing).unwrap(),
    ));
    let before = value(state.engine_call("project_export_model", ""));
    let revision = state.geometry_revision();
    for format in ["svg", "dxf"] {
        let exported =
            value(state.drawing_export(&json!({"sheet_id":1,"format":format}).to_string()));
        let content = exported["content"].as_str().unwrap();
        assert!(content.contains("CENTER_MARK"));
        assert!(!content.contains("999.00000"));
        assert_eq!(value(state.engine_call("project_export_model", "")), before);
        assert_eq!(state.geometry_revision(), revision);
    }
}
