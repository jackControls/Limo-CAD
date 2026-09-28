//! General-file Scripts controls must inspect without modifying a nonblank
//! design, run the inspected source in another tab, and preserve both models.
use super::*;
use crate::replay::Client;
use std::path::Path;

pub(super) fn exercise(c: &mut Client, out: &Path) -> Result<Value> {
    ui(
        c,
        json!({"action":"file","command":"rename","name":"Retained lesson"}),
    )?;
    let original = c.call("cad_project_model", json!({}))?;
    let original_ui = ui(c, json!({"action":"inspect"}))?;
    let original_session = original_ui["active_session_id"].clone();
    let source_path = out.join("opened-script.nbcad.jsonc");
    let fragment_path = out.join("opened-profile.collection.jsonc");
    let fragment = json!({"steps":[
        {"call":{"group":"sketch/draw","operation":"sketch_begin","arguments":{"name":"Imported profile","plane":{"type":"origin_plane","plane":"xy"}}}},
        {"call":{"group":"sketch/draw","operation":"sketch_add_rectangle_locked","arguments":{"mode":"two_point","anchor":{"x":0.,"y":0.},"corner_hint":{"x":16.,"y":9.},"width_mm":16.,"height_mm":9.,"ctrl_held":true}}},
        {"call":{"group":"sketch/draw","operation":"sketch_finish","arguments":{}}},
        {"call":{"group":"solid/build","operation":"solid_extrude","arguments":{"sketch_name":"Imported profile","profile_indices":[0],"operation":"new_body","extent":{"type":"distance","distance":5.}}}}
    ]});
    let source = json!({"version":1,"name":"Opened file fixture","includes":["opened-profile.collection.jsonc"],"steps":[
        {"view":"isometric","fit":true,"duration_ms":1},
        {"chapter":"Imported source complete","note":"The inspected source ran in its own design","duration_ms":1}
    ],"checks":[{"call":{"group":"solid/check","operation":"solid_scene","arguments":{}}}]});
    std::fs::write(&source_path, serde_json::to_vec_pretty(&source)?)?;
    std::fs::write(&fragment_path, serde_json::to_vec_pretty(&fragment)?)?;
    std::fs::write(
        out.join("opened-script-inspected-source.json"),
        serde_json::to_vec_pretty(&json!({"root":source,"fragment":fragment}))?,
    )?;
    control(c, "Scripts", None)?;
    control(
        c,
        "Script path",
        Some(source_path.to_str().context("Fixture path Unicode")?),
    )?;
    control(c, "Load script", None)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let loaded = loop {
        let state = ui(c, json!({"action":"inspect"}))?;
        if controls(&state)
            .any(|row| row["label"] == "Run in new design" && row["disabled"] == false)
        {
            break state;
        }
        ensure!(
            Instant::now() < deadline,
            "Imported script did not become runnable: {state}"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    ensure!(
        controls(&loaded).any(|row| row["label"] == "Loaded script path"
            && row["value"] == source_path.to_string_lossy().as_ref()
            && row["read_only"] == true),
        "Loaded script lost its inspected file identity: {loaded}"
    );
    ensure!(
        c.call("cad_project_model", json!({}))? == original,
        "Inspecting a script changed the nonblank design"
    );
    capture(c, out, "scripts-imported-ready")?;
    // Execute the frozen inspected snapshot, not changed includes or a later
    // replacement of the source file under the same path.
    std::fs::write(&source_path, "invalid root changed after inspection")?;
    std::fs::write(&fragment_path, "invalid include changed after inspection")?;
    let started = control(c, "Run in new design", None)?;
    ensure!(
        started["value"]["script_started"]["path"] == source_path.to_string_lossy().as_ref()
            && started["value"]["script_error"].is_null(),
        "Imported script did not start: {started}"
    );
    let session = started["active_session_id"]
        .as_str()
        .context("Script new tab session")?;
    ensure!(
        started["active_session_id"] != original_session,
        "Script reused the original document session"
    );
    c.call("cad_attach", json!({"session_id":session}))?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let presentation = loop {
        let response = ui(c, json!({"action":"presentation","command":"status"}))?;
        let state = &response["presentation"];
        ensure!(state["stopped"] != true, "Imported script stopped: {state}");
        if state["finished"] == true && state["chapter"] == "Imported source complete" {
            break state.clone();
        }
        ensure!(
            Instant::now() < deadline,
            "Imported script did not finish: {state}"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    let exported = c.call("cad_project_model", json!({}))?;
    let model: Value = serde_json::from_str(exported.as_str().context("Imported model export")?)?;
    ensure!(
        model["extrudes"]
            .as_array()
            .is_some_and(|rows| rows.len() == 1)
            && model["extrudes"][0]["extent"]["distance"] == 5.
            && model["fillets"].as_array().is_some_and(Vec::is_empty),
        "Imported source did not create its own editable extrusion: {model}"
    );
    let scene = c.call("solid_scene", json!({}))?;
    ensure!(
        scene["errors"].as_array().is_some_and(Vec::is_empty)
            && scene["bodies"]
                .as_array()
                .is_some_and(|rows| rows.len() == 1),
        "Imported source did not produce one valid solid: {scene}"
    );
    capture(c, out, "scripts-imported-complete")?;
    let archive_path = out.join("opened-script-result.nbcad");
    ui(
        c,
        json!({"action":"file","command":"save","path":archive_path}),
    )?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&archive_path)?)?;
    let saved: Value = serde_json::from_reader(archive.by_name("model.json")?)?;
    ensure!(
        saved == model,
        "Imported source archive lost editable intent"
    );
    let restored = control(c, "Retained lesson", None)?;
    c.call(
        "cad_attach",
        json!({"session_id":restored["active_session_id"]}),
    )?;
    ensure!(
        c.call("cad_project_model", json!({}))? == original,
        "Imported source modified the retained original lesson"
    );
    capture(c, out, "scripts-retained-original")?;
    Ok(
        json!({"state_checks_passed":true,"pixel_review":"required","loaded_ui":loaded,
        "started":started,"presentation":presentation,"model":model,"original":original,
        "checks":["inspect-does-not-run","frozen-source-and-includes","explicit-retained-new-tab",
            "real-shared-runner-solid","exact-saved-model","original-tab-unchanged"],
        "not_proven":["OS script chooser interaction","Source editing","General example browser","Script preview","Physical keyboard path entry"]}),
    )
}
