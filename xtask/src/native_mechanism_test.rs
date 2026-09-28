//! Component dragging through the rendered canvas and shared joint solver.
use crate::{
    native_fixture::{begin_sketch, capture, control, controls, panel_field, start, ui},
    replay::Client,
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};

fn drag(
    c: &mut Client,
    out: &Path,
    session: &str,
    server: &str,
    stage: &str,
    from: [f64; 2],
    to: [f64; 2],
) -> Result<()> {
    if std::env::var("NBCAD_NATIVE_MECHANISM_INPUT").as_deref() != Ok("1") {
        ui(
            c,
            json!({"action":"viewport","gesture":"drag","point":from,"to":to}),
        )?;
        return Ok(());
    }
    let pid = crate::native_drawing_navigation_test::owned_pid(out, session, server)?;
    let driver = crate::native_platform_test::Driver::new(pid, out)?;
    driver.event("focus")?;
    let snapshot = ui(c, json!({"action":"inspect"}))?;
    ensure!(
        snapshot["active_session_id"] == session && snapshot["attached_session_id"] == session,
        "Mechanism input no longer targets the acknowledged history session"
    );
    let request = json!({"x":from[0],"y":from[1],"to_x":to[0],"to_y":to[1],
        "client":snapshot["ui"]["client"]});
    // The established XTEST helper maps this owned client to physical pixels;
    // it works for the model canvas as well as paper and checks occlusion.
    let reply = driver.invoke("drawing-drag", Some(&request.to_string()))?;
    std::fs::write(out.join(format!("{stage}-os-input.json")), reply)?;
    crate::native_fixture::inspect_after_gesture(c, out, stage, &snapshot["active_session_id"])?;
    Ok(())
}

fn wait_pose(c: &mut Client, expected: impl Fn(f64) -> bool) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let current = assembly(c)?;
        if current["joints"][0]["linear_offset_mm"]
            .as_f64()
            .is_some_and(&expected)
        {
            return Ok(current);
        }
        ensure!(
            Instant::now() < deadline,
            "Mechanism pose did not reach the expected state: {current}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}
fn assembly(c: &mut Client) -> Result<Value> {
    c.call("assembly_document", json!({}))
}
fn frame(c: &mut Client, body: u64) -> Result<[f64; 2]> {
    ui(
        c,
        json!({"action":"view","view":"front","body_id":body,"fit":true,"duration_ms":0}),
    )?;
    let state = ui(c, json!({"action":"inspect"}))?;
    let b = state["ui"]["canvases"]
        .as_array()
        .context("Canvas missing")?
        .iter()
        .find(|c| c["name"] == "viewport")
        .context("Viewport missing")?;
    Ok([
        b["x"].as_f64().unwrap() + b["width"].as_f64().unwrap() / 2.,
        b["y"].as_f64().unwrap() + b["height"].as_f64().unwrap() / 2.,
    ])
}
pub(super) fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let mut f = start(args, "native-mechanism")?;
    let c = &mut f.client;
    if controls(&ui(c, json!({"action":"inspect"}))?).any(|v| v["label"] == "Back to model browser")
    {
        control(c, "Back to model browser", None)?;
    }
    for i in 0..2 {
        begin_sketch(c, "XY")?;
        c.call("sketch_add_rectangle",json!({"mode":"two_point","p1":{"x":i*40,"y":0},"p2":{"x":i*40+20,"y":10},"ctrl_held":true}))?;
        control(c, "Finish sketch", None)?;
        c.call("solid_extrude",json!({"sketch_name":format!("Sketch{}",i+1),"profile_indices":[0],"extent":{"type":"distance","distance":10.}}))?;
    }
    crate::native_joint_test::open(c, "slider")?;
    panel_field(
        c,
        "Limit Slide",
        None,
        "Scroll joint up",
        "Scroll joint down",
    )?;
    panel_field(
        c,
        "Slide Minimum",
        Some("-5"),
        "Scroll joint up",
        "Scroll joint down",
    )?;
    panel_field(
        c,
        "Slide Maximum",
        Some("20"),
        "Scroll joint up",
        "Scroll joint down",
    )?;
    control(c, "Apply joint", None)?;
    let source = c.call("solid_scene", json!({}))?;
    let before = assembly(c)?;
    let point = frame(c, 2)?;
    drag(
        c,
        &f.out,
        &f.session,
        &f.server,
        "first-drag",
        point,
        [point[0], point[1] - 60.],
    )?;
    let moved = wait_pose(c, |offset| offset > 0.05 && offset <= 20.)?;
    let offset = moved["joints"][0]["linear_offset_mm"]
        .as_f64()
        .context("Joint coordinate missing")?;
    ensure!(
        offset > 0.05 && offset <= 20.,
        "Drag did not move the joint within its limits: {offset}"
    );
    ensure!(
        c.call("solid_scene", json!({}))? == source,
        "Drag modified source geometry"
    );
    capture(c, &f.out, "mechanism-first-drag")?;
    let undo = control(c, "Undo", None)?;
    std::fs::write(
        f.out.join("mechanism-first-undo.json"),
        serde_json::to_vec_pretty(&undo)?,
    )?;
    ensure!(
        assembly(c)? == before,
        "Drag Undo did not restore original coordinates"
    );
    let redo = control(c, "Redo", None)?;
    std::fs::write(
        f.out.join("mechanism-first-redo.json"),
        serde_json::to_vec_pretty(&redo)?,
    )?;
    // History restoration replaces the session publisher. Follow only the
    // acknowledged history transition; owned_pid still checks the exact
    // launching process before any subsequent OS input.
    let resumed_session = redo["active_session_id"]
        .as_str()
        .context("Redo session receipt")?;
    ensure!(
        assembly(c)? == moved,
        "Drag Redo did not restore exact coordinates"
    );
    let point = frame(c, 2)?;
    drag(
        c,
        &f.out,
        resumed_session,
        &f.server,
        "second-drag",
        point,
        [point[0], point[1] + 30.],
    )?;
    let second = wait_pose(c, |position| position < offset)?;
    ensure!(
        second["joints"][0]["linear_offset_mm"].as_f64().unwrap() < offset,
        "Consecutive drag did not follow the displayed pose"
    );
    let undo = control(c, "Undo", None)?;
    std::fs::write(
        f.out.join("mechanism-second-undo.json"),
        serde_json::to_vec_pretty(&undo)?,
    )?;
    let resumed_session = undo["active_session_id"]
        .as_str()
        .context("Undo session receipt")?;
    ensure!(
        assembly(c)? == moved,
        "Consecutive drag created multiple history entries"
    );
    let point = frame(c, 1)?;
    if std::env::var("NBCAD_NATIVE_MECHANISM_INPUT").as_deref() == Ok("1") {
        drag(
            c,
            &f.out,
            resumed_session,
            &f.server,
            "grounded-drag",
            point,
            [point[0] + 40., point[1]],
        )?;
        for _ in 0..10 {
            ensure!(assembly(c)? == moved, "Grounded component moved");
            thread::sleep(Duration::from_millis(25));
        }
    } else {
        let _ = c.call("cad_interface", json!({"action":"viewport","gesture":"drag","point":point,"to":[point[0]+40.,point[1]]}));
        ensure!(assembly(c)? == moved, "Grounded component moved");
    }
    ui(
        c,
        json!({"action":"view","view":"isometric","fit":true,"duration_ms":0}),
    )?;
    capture(c, &f.out, "mechanism-final")?;
    ui(
        c,
        json!({"action":"file","command":"save","path":f.project}),
    )?;
    std::fs::write(
        &f.report,
        serde_json::to_string_pretty(
            &json!({"passed":true,"first_travel_mm":offset,"assembly":assembly(c)?,
                "os_input":std::env::var("NBCAD_NATIVE_MECHANISM_INPUT").as_deref() == Ok("1"),
                "pixel_review":"required","not_proven":["physical hardware","focus loss during drag","mixed-monitor DPI","other joint types"]}),
        )?,
    )?;
    println!("PASS native mechanism dragging: shared solver, grounded rejection, joint limits, consecutive poses, unchanged source geometry, exact single-step Undo/Redo");
    Ok(())
}
