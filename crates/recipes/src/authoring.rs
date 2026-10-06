//! Naming for the feature-creating calls used by the flagship Rust authors.
use serde_json::{json, Value};

/// Label the newly returned history feature without changing the source call,
/// its result binding, geometry arguments or feature identity.
pub fn feature_name_step(id: &str, operation: &str, arguments: &Value) -> Option<Value> {
    let details = match operation {
        "solid_extrude" => {
            let action = match arguments["operation"].as_str().unwrap_or("new_body") {
                "new_body" => "Stock",
                "join" => "Add",
                "cut" => "Cut",
                value => value,
            };
            let extent = &arguments["extent"];
            if extent["type"] == "distance" {
                format!("{action} {} mm", extent["distance"])
            } else {
                format!("{action} {}", extent["type"].as_str()?.replace('_', " "))
            }
        }
        "solid_fillet" => format!("Fillet R{} mm", arguments["radius"]),
        "solid_chamfer" => format!("Chamfer {} mm", arguments["distance"]),
        "solid_hole" => format!("Hole Ø{} mm", arguments["diameter"]),
        "solid_external_thread" => "External thread".into(),
        "solid_circular_pattern" => format!("Circular pattern / {} instances", arguments["count"]),
        "solid_combine" => format!("Combine / {}", arguments["operation"].as_str()?),
        _ => return None,
    };
    let stem = arguments["sketch_name"]
        .as_str()
        .unwrap_or(id)
        .trim_end_matches("_build")
        .replace('_', " ");
    let name = format!("{stem} / {details}");
    assert!(
        (1..=256).contains(&name.chars().count()) && !name.chars().any(char::is_control),
        "Invalid feature name for {id}: {name:?}"
    );
    Some(json!({
        "id":format!("{id}_feature_name"),
        "call":{
            "group":"document/history",
            "operation":"solid_rename_feature",
            "arguments":{
                "feature_id":{"$select":{"from":{"$ref":id},"path":"/document/features","take":"last","pointer":"/id"}},
                "name":name
            }
        }
    }))
}
