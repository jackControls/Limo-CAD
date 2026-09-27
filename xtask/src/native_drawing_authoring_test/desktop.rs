//! Actual XTEST note placement and annotation drags in the owned Xvfb host.
//! The complete existing fixture survives every operation and history pair.
use super::*;
use crate::{
    native_drawing_navigation_test::{canvas, owned_pid},
    native_platform_test::Driver,
};
use std::{
    fs, thread,
    time::{Duration, Instant},
};
#[path = "desktop_curved.rs"]
mod curved_input;
#[path = "desktop_series.rs"]
mod series_input;

fn inspect(c: &mut Client) -> Result<Value> {
    ui(c, json!({"action":"inspect"}))
}

fn center(state: &Value, label: &str) -> Result<[f64; 2]> {
    let found: Vec<_> = controls(state)
        .filter(|c| c["label"] == label && c["disabled"] == false)
        .collect();
    ensure!(
        found.len() == 1,
        "Expected one owned paper target {label}: {found:?}"
    );
    let bounds = &found[0]["bounds"];
    Ok([
        bounds["x"].as_f64().context("Target x")?
            + bounds["width"].as_f64().context("Target width")? * 0.5,
        bounds["y"].as_f64().context("Target y")?
            + bounds["height"].as_f64().context("Target height")? * 0.5,
    ])
}

struct Paper {
    origin: [f64; 2],
    scale: f64,
}
impl Paper {
    fn fitted(state: &Value) -> Result<Self> {
        ensure!(
            controls(state).any(|c| c["label"] == "Fit sheet" && c["selected"] == true),
            "Physical authoring requires the whole fitted paper"
        );
        let paper = canvas(state)?;
        let scale = paper["width"].as_f64().unwrap() / 297.;
        ensure!(
            (paper["height"].as_f64().unwrap() / 210. - scale).abs() < 1e-6,
            "Expected the fixture's complete landscape A4 paper bounds"
        );
        Ok(Self {
            origin: [paper["x"].as_f64().unwrap(), paper["y"].as_f64().unwrap()],
            scale,
        })
    }
    fn screen(&self, paper: [f64; 2]) -> [f64; 2] {
        [
            self.origin[0] + paper[0] * self.scale,
            self.origin[1] + paper[1] * self.scale,
        ]
    }
    fn paper(&self, screen: [f64; 2]) -> [f64; 2] {
        [
            (screen[0] - self.origin[0]) / self.scale,
            (screen[1] - self.origin[1]) / self.scale,
        ]
    }
}

fn observed_point(evidence: &Value, key: &str) -> Result<[f64; 2]> {
    // Bevy's CursorMoved stores the OS logical position in Vec2 (f32).
    let point = &evidence[key];
    Ok([
        point[0].as_f64().context("Actual OS logical x")? as f32 as f64,
        point[1].as_f64().context("Actual OS logical y")? as f32 as f64,
    ])
}
fn gesture(
    driver: &Driver,
    c: &mut Client,
    out: &Path,
    stage: &str,
    operation: &str,
    start: [f64; 2],
    end: Option<[f64; 2]>,
) -> Result<Value> {
    let state = inspect(c)?;
    let mut request = json!({"client":state["ui"]["client"],"x":start[0],"y":start[1]});
    if let Some(end) = end {
        request["to_x"] = json!(end[0]);
        request["to_y"] = json!(end[1]);
    }
    fs::write(
        out.join(format!("{stage}-surface.json")),
        serde_json::to_vec_pretty(&state)?,
    )?;
    fs::write(
        out.join(format!("{stage}-request.json")),
        serde_json::to_vec_pretty(&request)?,
    )?;
    // No model/inspect RPC may intervene while the physical button is held.
    // All pointer samples and release occur synchronously in the helper.
    let evidence: Value =
        serde_json::from_str(&driver.invoke(operation, Some(&request.to_string()))?)?;
    fs::write(
        out.join(format!("{stage}-input.json")),
        serde_json::to_vec_pretty(&evidence)?,
    )?;
    Ok(evidence)
}
fn changed_model(c: &mut Client, before: &Value) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let current = model(c)?;
        if &current != before {
            return Ok(current);
        }
        ensure!(
            Instant::now() < deadline,
            "Actual OS annotation gesture did not commit a document change"
        );
        thread::sleep(Duration::from_millis(40));
    }
}
fn close_point(actual: &Value, expected: [f64; 2], label: &str) -> Result<()> {
    for i in 0..2 {
        ensure!(
            (actual[i].as_f64().context("Saved paper coordinate")? - expected[i]).abs() < 1e-5,
            "{label}: physical pixel-to-paper position differs: {actual}, expected {expected:?}"
        );
    }
    Ok(())
}

pub(in super::super) fn exercise(c: &mut Client, out: &Path, server: &str) -> Result<Value> {
    ensure!(
        cfg!(target_os = "linux"),
        "OS annotation authoring proof is opt-in disposable Linux Xvfb only"
    );
    let initial = inspect(c)?;
    let session = initial["active_session_id"]
        .as_str()
        .context("Owned drawing session")?;
    let driver = Driver::new(owned_pid(out, session, server)?, out)?;
    let baseline = model(c)?;
    control(c, "Fit sheet", None)?;
    control(c, "Add note", None)?;
    let text = "OS placed Caf\u{e9} \u{96f6}\u{4ef6}";
    field(c, "Note text", text)?;
    let state = inspect(c)?;
    let paper = Paper::fitted(&state)?;
    let placed = gesture(
        &driver,
        c,
        out,
        "author-os-note-place",
        "drawing-click",
        paper.screen([110., 22.]),
        None,
    )?;
    let created = changed_model(c, &baseline)?;
    let note = exact_one_added(&baseline, &created, out, "author-os-note-created")?;
    ensure!(
        note["kind"] == "note" && note["text"] == text,
        "OS note placement lost prepared text"
    );
    close_point(
        &note["position"],
        paper.paper(observed_point(&placed, "logical_start")?),
        "Note placement",
    )?;
    let id = note["id"].as_u64().context("Created note ID")?;
    history(c, &baseline, &created)?; // One Undo must restore the complete pre-gesture model.
    capture(c, out, "author-os-note-created")?;

    let state = inspect(c)?;
    let start = center(&state, &format!("Edit annotation {id}"))?;
    let end = [start[0] + paper.scale * 18., start[1] + paper.scale * 9.];
    let moved = gesture(
        &driver,
        c,
        out,
        "author-os-note-drag",
        "drawing-drag",
        start,
        Some(end),
    )?;
    let moved_start = paper.paper(observed_point(&moved, "logical_start")?);
    let moved_end = paper.paper(observed_point(&moved, "logical_end")?);
    let dragged = changed_model(c, &created)?;
    let dragged_note = annotations(&dragged)?
        .iter()
        .find(|a| a["id"] == id)
        .context("Dragged note")?;
    close_point(
        &dragged_note["position"],
        [
            note["position"][0].as_f64().unwrap() + moved_end[0] - moved_start[0],
            note["position"][1].as_f64().unwrap() + moved_end[1] - moved_start[1],
        ],
        "Note drag",
    )?;
    let expected = replace_expected(&created, id, |a| {
        a["position"] = dragged_note["position"].clone()
    });
    ensure!(
        dragged == expected,
        "OS note drag changed text or unrelated saved intent"
    );
    history(c, &created, &dragged)?;
    capture(c, out, "author-os-note-dragged")?;
    control(c, "Undo", None)?;
    control(c, "Undo", None)?;
    ensure!(
        model(c)? == baseline,
        "OS note gestures did not restore the complete 24-annotation baseline"
    );

    control(c, "Linear dimension", None)?;
    let labels = pair(c)?;
    let state = inspect(c)?;
    let first = center(&state, &labels[0])?;
    let second = center(&state, &labels[1])?;
    gesture(
        &driver,
        c,
        out,
        "author-os-first-anchor",
        "drawing-click",
        first,
        None,
    )?;
    ensure!(
        model(c)? == baseline,
        "First OS anchor click mutated the drawing"
    );
    gesture(
        &driver,
        c,
        out,
        "author-os-repeat-anchor",
        "drawing-click",
        first,
        None,
    )?;
    ensure!(
        model(c)? == baseline,
        "Repeated OS anchor click created a zero-span dimension"
    );
    gesture(
        &driver,
        c,
        out,
        "author-os-second-anchor",
        "drawing-click",
        second,
        None,
    )?;
    let dimensioned = changed_model(c, &baseline)?;
    let dimension = exact_one_added(&baseline, &dimensioned, out, "author-os-linear-created")?;
    ensure!(
        dimension["kind"] == "linear_dimension"
            && dimension["mode"] == "aligned"
            && dimension["offset"] == 12.,
        "Actual OS anchors did not create the shared aligned dimension"
    );
    for anchor in ["first", "second"] {
        ensure!(
            dimension[anchor]["topology_signature"].is_string()
                && dimension[anchor]["fallback_point"][2] == 6.,
            "OS picking lost the exact frontmost anchor {anchor}"
        );
    }
    history(c, &baseline, &dimensioned)?;
    capture(c, out, "author-os-linear-created")?;
    let dim_id = dimension["id"].as_u64().unwrap();
    let state = inspect(c)?;
    let start = center(&state, &format!("Edit annotation {dim_id}"))?;
    let end = [start[0] + paper.scale * 7., start[1] + paper.scale * 6.];
    let moved = gesture(
        &driver,
        c,
        out,
        "author-os-linear-drag",
        "drawing-drag",
        start,
        Some(end),
    )?;
    let dy = (observed_point(&moved, "logical_end")?[1]
        - observed_point(&moved, "logical_start")?[1])
        / paper.scale;
    let dragged_dimension = changed_model(c, &dimensioned)?;
    let row = annotations(&dragged_dimension)?
        .iter()
        .find(|a| a["id"] == dim_id)
        .context("Dragged dimension")?;
    ensure!(
        (row["offset"].as_f64().context("Paper-space offset")? - 12. - dy).abs() < 1e-5,
        "OS aligned dimension drag did not preserve the paper offset: {}, expected {}",
        row["offset"],
        12. + dy
    );
    let expected = replace_expected(&dimensioned, dim_id, |a| {
        a["offset"] = row["offset"].clone()
    });
    ensure!(
        dragged_dimension == expected,
        "OS dimension drag changed projected anchors, presentation, or unrelated saved intent"
    );
    history(c, &dimensioned, &dragged_dimension)?;
    capture(c, out, "author-os-linear-dragged")?;
    control(c, "Undo", None)?;
    control(c, "Undo", None)?;
    ensure!(
        model(c)? == baseline,
        "OS annotation gestures did not restore all saved intent exactly"
    );
    let curved = curved_input::exercise(&driver, c, out, &baseline, &paper)?;
    let series = series_input::exercise(&driver, c, out, &baseline, &paper)?;
    let report = json!({"actual_input":"X11 XTEST through OS Winit events in owned Xvfb", "note_paper_click_passed":true,
        "curved_dimensions":curved,"series_ordinate_dimensions":series,
        "note_drag_passed":true,"projected_anchor_clicks_passed":true,"dimension_drag_passed":true,
        "one_commit_per_gesture_and_exact_history":true,"all_saved_model_intent_restored":true,
        "captures":["author-os-note-created.png","author-os-note-dragged.png","author-os-linear-created.png","author-os-linear-dragged.png"],
        "not_proven":["Windows/macOS annotation gestures","Wayland","monitor DPI transition","touchpad hardware","Note IME composition"]});
    fs::write(
        out.join("authoring-os.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}
