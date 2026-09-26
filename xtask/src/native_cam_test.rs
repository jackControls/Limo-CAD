//! Shared-document CAM edits through real native retained controls. Captures
//! are evidence for visual review; state assertions do not claim pixel checks.
use crate::native_fixture::{begin_sketch, capture, control, controls, panel_field, start, ui};
use crate::replay::Client;
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

fn document(c: &mut Client) -> Result<Value> {
    let mut value = c.call("cam_get_document", json!({}))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("_disclosure");
    }
    Ok(value)
}
fn scene(c: &mut Client) -> Result<Value> {
    let mut value = c.call("solid_scene", json!({}))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("_disclosure");
    }
    Ok(value)
}
fn field(c: &mut Client, label: &str, value: &str) -> Result<()> {
    panel_field(c, label, Some(value), "Previous fields", "More fields")?;
    Ok(())
}
fn rejected(c: &mut Client, label: &str) -> Result<()> {
    let inspected = ui(c, json!({"action":"inspect"}))?;
    let target = controls(&inspected)
        .find(|r| r["label"] == label && r["disabled"] == false)
        .with_context(|| format!("No enabled {label}"))?["id"]
        .clone();
    let result = c.call("cad_interface", json!({"action":"click","target":target}));
    ensure!(
        result.is_err() || result.as_ref().is_ok_and(|v| v["status"] == "failed"),
        "Expected {label} to reject invalid CAM edit: {result:?}"
    );
    Ok(())
}
fn history(c: &mut Client, before: &Value, after: &Value) -> Result<()> {
    control(c, "Undo", None)?;
    ensure!(
        &c.call("cad_project_model", json!({}))? == before,
        "CAM Undo did not restore the exact model"
    );
    control(c, "Redo", None)?;
    ensure!(
        &c.call("cad_project_model", json!({}))? == after,
        "CAM Redo did not restore the exact model"
    );
    Ok(())
}
fn current(c: &mut Client) -> Result<()> {
    let statuses = c.call("cam_toolpath_statuses", json!({}))?;
    ensure!(
        statuses
            .as_array()
            .is_some_and(|rows| !rows.is_empty() && rows.iter().all(|r| r["state"] == "current")),
        "Generated toolpaths are not current: {statuses}"
    );
    Ok(())
}

fn advanced_setup(c: &mut Client, solid: &Value) -> Result<()> {
    let before = c.call("cad_project_model", json!({}))?;
    let second = solid["bodies"][1]["name"]
        .as_str()
        .context("Second CAM part missing")?;
    field(c, &format!("Model · {second}"), "true")?;
    field(c, "Stock +Z allowance (mm)", "2")?;
    field(c, "WCS orientation", "up90")?;
    control(c, "Apply", None)?;
    let cam = document(c)?;
    let setup = &cam["setups"][0];
    ensure!(
        setup["body_ids"]
            .as_array()
            .is_some_and(|ids| ids.len() == 2),
        "Native setup did not include both selected solids"
    );
    ensure!(
        setup["stock_spec"]["offsets"]["z_max"] == 2. && setup["wcs"]["origin"]["z"] == 8.,
        "Stock allowance did not resolve the WCS from the actual model"
    );
    ensure!(
        setup["wcs"]["x_axis"] == json!([0., 1., 0.])
            && setup["wcs"]["y_axis"] == json!([-1., 0., 0.]),
        "Native WCS rotation missing"
    );
    let after = c.call("cad_project_model", json!({}))?;
    history(c, &before, &after)?;
    Ok(())
}

fn advanced_tool(c: &mut Client) -> Result<()> {
    let before_cam = document(c)?;
    let before = c.call("cad_project_model", json!({}))?;
    field(c, "Cutter type", "bull_nose_end_mill")?;
    field(c, "Corner radius (mm)", "0.5")?;
    field(c, "Default step down (optional) (mm)", "1.5")?;
    field(c, "Default cutting feed (mm/min)", "1200")?;
    control(c, "Apply", None)?;
    let cam = document(c)?;
    ensure!(
        cam["tools"][0]["kind"] == "bull_nose_end_mill" && cam["tools"][0]["corner_radius"] == 0.5,
        "Native cutter type/corner edit missing"
    );
    ensure!(
        cam["tools"][0]["overall_length"] == before_cam["tools"][0]["overall_length"],
        "Corner edit changed the unedited overall length"
    );
    ensure!(
        cam["setups"][0]["operations"] == before_cam["setups"][0]["operations"],
        "Library defaults rewrote operation cutting data"
    );
    let after = c.call("cad_project_model", json!({}))?;
    history(c, &before, &after)?;
    Ok(())
}

pub(super) fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let mut fixture = start(args, "native-cam")?;
    let c = &mut fixture.client;
    ensure!(
        document(c)?["setups"].as_array().is_some_and(Vec::is_empty),
        "Choose a blank CAM document"
    );
    begin_sketch(c, "XY")?;
    c.call(
        "sketch_add_rectangle",
        json!({"mode":"two_point","p1":{"x":0.,"y":0.},"p2":{"x":40.,"y":25.},"ctrl_held":true}),
    )?;
    control(c, "Finish sketch", None)?;
    c.call("solid_extrude", json!({"sketch_name":"Sketch1","profile_indices":[0],"extent":{"type":"distance","distance":6.}}))?;
    begin_sketch(c, "XY")?;
    c.call(
        "sketch_add_rectangle",
        json!({"mode":"two_point","p1":{"x":60.,"y":0.},"p2":{"x":80.,"y":15.},"ctrl_held":true}),
    )?;
    control(c, "Finish sketch", None)?;
    c.call("solid_extrude", json!({"sketch_name":"Sketch2","profile_indices":[0],"extent":{"type":"distance","distance":6.}}))?;
    let solid = scene(c)?;
    let body = solid["bodies"][0]["id"].clone();
    ensure!(body.is_number(), "CAM fixture solid missing");
    control(c, "Switch workspace", None)?;
    control(c, "CAM", None)?;
    let empty = document(c)?;
    control(c, "New setup", None)?;
    rejected(c, "Create")?;
    ensure!(
        document(c)? == empty,
        "Setup creation guessed a body without explicit selection"
    );
    field(c, "Model body · click to cycle", &body.to_string())?;
    field(c, "Name", "Top setup")?;
    capture(c, &fixture.out, "cam-new-setup")?;
    control(c, "Create", None)?;
    let setup_id = document(c)?["setups"][0]["id"].to_string();
    advanced_setup(c, &solid)?;
    control(c, "Project tools", None)?;
    control(c, "New project tool", None)?;
    for (label, value) in [
        ("Name", "6 mm flat end mill"),
        ("Diameter (mm)", "6"),
        ("Flute length (mm)", "20"),
        ("Overall length (mm)", "50"),
        ("Default spindle (rpm)", "12000"),
        ("Default cutting feed (mm/min)", "800"),
        ("Default plunge feed (mm/min)", "200"),
    ] {
        field(c, label, value)?;
    }
    control(c, "Create", None)?;
    let source_tool_id = document(c)?["tools"][0]["id"].to_string();
    control(c, "Toolpaths", None)?;
    control(c, "New face toolpath", None)?;
    field(c, "Setup · click to cycle", &setup_id)?;
    field(c, "Tool · click to cycle", &source_tool_id)?;
    field(c, "Name", "Face stock")?;
    capture(c, &fixture.out, "cam-new-face")?;
    control(c, "Create", None)?;
    control(c, "Setups", None)?;
    let before_name = c.call("cad_project_model", json!({}))?;
    field(c, "Name", "Top finish setup")?;
    field(c, "Work offset · click to cycle", "g55")?;
    control(c, "Apply", None)?;
    let after_name = c.call("cad_project_model", json!({}))?;
    history(c, &before_name, &after_name)?;
    ensure!(
        document(c)?["setups"][0]["name"] == "Top finish setup",
        "Setup edit missing"
    );
    control(c, "Generate", None)?;
    current(c)?;
    capture(c, &fixture.out, "cam-setup")?;
    let generated = document(c)?;

    control(c, "Project tools", None)?;
    field(c, "Diameter (mm)", "6.5")?;
    control(c, "Apply", None)?;
    advanced_tool(c)?;
    let tool_edited = document(c)?;
    ensure!(
        tool_edited["tools"][0]["diameter"] == 6.5,
        "Cutter edit missing"
    );
    ensure!(
        tool_edited["height_expressions"] == generated["height_expressions"]
            && tool_edited["toolpath_generations"] == generated["toolpath_generations"],
        "Tool edit rewrote unrelated CAM intent or freshness evidence"
    );
    field(c, "Diameter (mm)", "-1")?;
    rejected(c, "Apply")?;
    ensure!(
        document(c)? == tool_edited,
        "Invalid cutter edit changed the CAM document"
    );
    capture(c, &fixture.out, "cam-invalid-tool")?;
    control(c, "Reset", None)?;
    control(c, "Duplicate", None)?;
    field(c, "Name", "Finishing mill")?;
    field(c, "Tool number (optional)", "2")?;
    control(c, "Apply", None)?;
    let tools = document(c)?;
    ensure!(
        tools["tools"].as_array().is_some_and(|r| r.len() == 2),
        "Tool duplicate missing"
    );
    ensure!(
        tools["tools"][1]["name"] == "Finishing mill",
        "Copy did not stay selected for editing"
    );
    let copy_tool_id = tools["tools"][1]["id"].to_string();

    control(c, "Toolpaths", None)?;
    field(c, "Name", "Face finish")?;
    field(c, "Tool · click to cycle", &copy_tool_id)?;
    let inspected = ui(c, json!({"action":"inspect"}))?;
    let chooser = controls(&inspected)
        .find(|r| r["label"] == "Tool · click to cycle")
        .context("Selected tool chooser missing")?;
    ensure!(
        chooser["role"] == "combobox",
        "Tool field is not a named choice"
    );
    ensure!(
        chooser["options"]
            .as_array()
            .is_some_and(|rows| rows.iter().any(|r| r["label"] == "T2 · Finishing mill")),
        "Chooser did not offer named project tools"
    );
    field(c, "Cutting feed (mm/min)", "900")?;
    control(c, "Apply", None)?;
    let edited = document(c)?;
    ensure!(
        edited["setups"][0]["operations"][0]["name"] == "Face finish"
            && edited["setups"][0]["operations"][0]["tool_id"] == tools["tools"][1]["id"],
        "Toolpath edits missing"
    );
    ensure!(
        edited["setups"][0]["operations"][0]["cutting"]["feed_xy"] == 900.,
        "Feed edit missing"
    );
    ensure!(
        edited["height_expressions"] == generated["height_expressions"],
        "Cutting edit changed associative heights"
    );
    control(c, "Generate", None)?;
    current(c)?;
    capture(c, &fixture.out, "cam-toolpath")?;
    let stamped = document(c)?;
    control(c, "Duplicate", None)?;
    let copied = document(c)?;
    ensure!(
        copied["setups"][0]["operations"]
            .as_array()
            .is_some_and(|r| r.len() == 2),
        "Toolpath duplicate missing"
    );
    ensure!(
        copied["height_expressions"]
            .as_array()
            .is_some_and(|r| r.len() == 2),
        "Toolpath copy lost associative height intent"
    );
    ensure!(
        copied["toolpath_generations"] == stamped["toolpath_generations"],
        "Duplicate fabricated generation evidence"
    );
    let before_delete = c.call("cad_project_model", json!({}))?;
    control(c, "Delete", None)?;
    let after_delete = c.call("cad_project_model", json!({}))?;
    history(c, &before_delete, &after_delete)?;
    ensure!(
        document(c)?["setups"][0]["operations"]
            .as_array()
            .is_some_and(|r| r.len() == 1),
        "Delete removed wrong toolpath"
    );

    control(c, "Setups", None)?;
    control(c, "Duplicate", None)?;
    let setups = document(c)?;
    ensure!(
        setups["setups"].as_array().is_some_and(|r| r.len() == 2),
        "Setup copy missing"
    );
    ensure!(
        setups["setups"][0]["stock"] == setups["setups"][1]["stock"]
            && setups["setups"][0]["wcs"] == setups["setups"][1]["wcs"],
        "Copy changed stock/WCS"
    );
    control(c, "Delete", None)?;
    ensure!(
        document(c)?["setups"]
            .as_array()
            .is_some_and(|r| r.len() == 1),
        "Setup deletion failed"
    );
    control(c, "Project tools", None)?;
    control(c, "T2 · Finishing mill", None)?;
    let before_rejected_delete = document(c)?;
    rejected(c, "Delete")?;
    ensure!(
        document(c)? == before_rejected_delete,
        "Used tool deletion changed the document"
    );
    control(c, "Toolpaths", None)?;
    field(c, "Tool · click to cycle", &source_tool_id)?;
    control(c, "Apply", None)?;
    control(c, "Generate", None)?;
    current(c)?;
    control(c, "Project tools", None)?;
    control(c, "T2 · Finishing mill", None)?;
    control(c, "Delete", None)?;
    ensure!(
        document(c)?["tools"]
            .as_array()
            .is_some_and(|r| r.len() == 1),
        "Unused tool deletion failed"
    );
    ensure!(scene(c)? == solid, "CAM editing changed the real solid");
    control(c, "Toolpaths", None)?;
    capture(c, &fixture.out, "native-cam")?;
    ui(
        c,
        json!({"action":"file","command":"save","path":fixture.project}),
    )?;
    let exported = c.call("cad_project_model", json!({}))?;
    let model: Value = serde_json::from_str(exported.as_str().context("Model missing")?)?;
    let mut archive = zip::ZipArchive::new(std::fs::File::open(&fixture.project)?)?;
    let saved: Value = serde_json::from_reader(archive.by_name("model.json")?)?;
    ensure!(
        saved == model,
        "Native Save did not preserve CAM and CAD intent"
    );
    for name in [
        "cam-setup",
        "cam-new-setup",
        "cam-new-face",
        "cam-invalid-tool",
        "cam-toolpath",
        "native-cam",
    ] {
        let png = std::fs::read(fixture.out.join(format!("{name}.png")))?;
        ensure!(
            png.starts_with(b"\x89PNG\r\n\x1a\n") && png.len() > 1024,
            "Capture {name} is missing or empty"
        );
    }
    std::fs::write(
        &fixture.report,
        serde_json::to_string_pretty(&json!({
            "state_checks_passed":true,"pixel_review":"required","session":fixture.session,
            "model":model,"cam":document(c)?,"solid":solid,
            "checks":["real-solid","first-setup-tool-and-face-created-natively","explicit-body-setup-and-tool-selection","setup-edit","cutter-edit","toolpath-edit","named-tool-and-work-offset-choices",
                "invalid-edit-no-mutation","duplicate-and-delete-all-record-kinds","generation-evidence-not-copied",
                "used-tool-delete-rejected","explicit-shared-engine-generation","exact-undo-redo","live-window-capture","saved-model-equality"]
        }))?,
    )?;
    println!("PASS native CAM setup/tool/toolpath edits, shared validation, Generate, exact history and Save");
    Ok(())
}
