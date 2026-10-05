use super::*;

fn data(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.remove("_disclosure");
    }
    value
}

#[test]
fn print_intent_mcp_persists_sources_and_rejects_stale_edits_atomically() {
    let (mut server, initial) = mcp_box();
    let body = initial["scene"]["bodies"][0]["id"].clone();
    let before_geometry = data(server.call_tool("solid_scene", json!({})).unwrap());
    let before_assembly = data(server.call_tool("assembly_document", json!({})).unwrap());
    let before_appearance = data(server.call_tool("body_appearances", json!({})).unwrap());
    let before = server.manager.export_project_model().unwrap();
    let mut document = data(server.call_tool("print_intent_get", json!({})).unwrap());
    assert_eq!(document["source_document_id"], Value::Null);
    document["defaults"]["wall_count"] = json!(2);
    document["defaults"]["infill_density_percent"] = json!(15);
    server
        .call_tool(
            "print_intent_set_document",
            json!({"document":document,"expected_model_json":before}),
        )
        .unwrap();
    let current = server.manager.export_project_model().unwrap();
    let result = server.call_tool("print_intent_set_part", json!({
        "body_id":body,"settings":{"wall_count":0,"infill_density_percent":40,"infill_pattern":"gyroid"},"expected_model_json":current
    })).unwrap();
    let namespace = result["source_document_id"].clone();
    assert!(namespace.as_str().is_some());
    let saved = server.manager.export_project_model().unwrap();
    assert!(server
        .call_tool(
            "print_intent_reset_part",
            json!({"body_id":body,"expected_model_json":current})
        )
        .is_err());
    assert_eq!(server.manager.export_project_model().unwrap(), saved);
    let effective = server
        .call_tool(
            "print_intent_effective",
            json!({"target":"portable","body_ids":[body]}),
        )
        .unwrap();
    assert_eq!(effective["parts"][0]["settings"]["wall_count"], 0);
    assert_eq!(effective["parts"][0]["sources"]["wall_count"], "part");
    assert!(!effective["parts"][0]["unsupported"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        data(server.call_tool("solid_scene", json!({})).unwrap()),
        before_geometry
    );
    assert_eq!(
        data(server.call_tool("assembly_document", json!({})).unwrap()),
        before_assembly
    );
    assert_eq!(
        data(server.call_tool("body_appearances", json!({})).unwrap()),
        before_appearance
    );
    server
        .call_tool("cad_load_project_model", json!({"model_json":saved}))
        .unwrap();
    assert_eq!(
        server.call_tool("print_intent_get", json!({})).unwrap()["source_document_id"],
        namespace
    );
    server.call_tool("print_intent_reset_part", json!({"body_id":body,"expected_model_json":server.manager.export_project_model().unwrap()})).unwrap();
    let effective = server
        .call_tool("print_intent_effective", json!({"target":"bambu_studio"}))
        .unwrap();
    assert_eq!(effective["parts"][0]["settings"]["wall_count"], 2);
    assert_eq!(
        effective["parts"][0]["sources"]["wall_count"],
        "project_default"
    );
}

#[test]
fn print_intent_discovery_is_typed_owned_and_not_a_geometry_script() {
    let tools = tool_specs();
    for spec in print_intent_tools::specs() {
        assert_eq!(
            interface::group_for(spec.name),
            Some("document/print-intent")
        );
        assert!(!records_in_script(spec.name));
        assert_eq!(tags_for_tool(spec.name).0, FocusPack::Print);
        if matches!(spec.name, "print_intent_get" | "print_intent_effective") {
            assert!(limo_cad_mcp_mutate::is_live_engine_query(
                spec.engine_method
            ));
            assert!(!is_modeling_mutate(spec.name));
        } else {
            assert!(is_modeling_mutate(spec.name));
            assert!(spec.input_schema["required"]
                .as_array()
                .unwrap()
                .contains(&json!("expected_model_json")));
        }
        assert!(tools.iter().any(|tool| tool.name == spec.name));
    }
    let schema = print_intent_tools::specs()
        .into_iter()
        .find(|tool| tool.name == "print_intent_set_part")
        .unwrap()
        .input_schema;
    assert_eq!(
        schema["properties"]["settings"]["additionalProperties"],
        false
    );
    assert_eq!(
        schema["properties"]["settings"]["properties"]["infill_density_percent"]["maximum"],
        100
    );
}
