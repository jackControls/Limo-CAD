//! The release three-endpoint series and origin/target ordinate tools.
use super::*;

pub(super) fn select_tool(c: &mut Client, label: &str) -> Result<()> {
    control(c, "More dimensions", None)?;
    control(c, label, None)?;
    Ok(())
}
pub(super) fn exercise(c: &mut Client, out: &Path, baseline: &Value) -> Result<Value> {
    let projection = curved::projection(c)?;
    let mut images = Vec::new();
    for (label, layout) in [
        ("Chain", Some("chain")),
        ("Baseline", Some("baseline")),
        ("Continued", Some("continued")),
        ("Ordinate", None),
    ] {
        let stage = layout.unwrap_or("ordinate");
        select_tool(c, label)?;
        let labels = curved::angular_triple(c)?;
        let count = if layout.is_some() { 3 } else { 2 };
        for anchor in &labels[..count - 1] {
            control(c, anchor, None)?;
            ensure!(
                &model(c)? == baseline,
                "Partial {stage} selection mutated the model"
            );
            control(c, anchor, None)?;
            ensure!(
                &model(c)? == baseline,
                "Repeated {stage} endpoint advanced the selection"
            );
        }
        capture(c, out, &format!("author-{stage}-targets"))?;
        images.push(format!("author-{stage}-targets.png"));
        control(c, &labels[count - 1], None)?;
        let created = model(c)?;
        let a = exact_one_added(baseline, &created, out, &format!("author-{stage}-created"))?;
        let references = if let Some(layout) = layout {
            ensure!(
                a["kind"] == "chain_dimension"
                    && a["layout"] == layout
                    && a["mode"] == "aligned"
                    && a["spacing"] == 7.
                    && a["offset"] == 12.
                    && a["precision"] == 2,
                "Release series defaults changed"
            );
            a["anchors"]
                .as_array()
                .context("Series references")?
                .clone()
        } else {
            ensure!(
                a["kind"] == "ordinate_dimension"
                    && a["axis"] == "both"
                    && a["offset"] == 10.
                    && a["precision"] == 2,
                "Release ordinate defaults changed"
            );
            vec![a["origin"].clone(), a["target"].clone()]
        };
        ensure!(references.len() == count, "Saved endpoint count changed");
        for reference in &references {
            ensure!(
                reference["circle_center"] == false,
                "Series/ordinate creation exposed circle centers"
            );
            curved::endpoint_ref(&projection, reference)?;
        }
        let id = a["id"].as_u64().unwrap();
        history(c, baseline, &created)?;
        if layout.is_some() {
            // Every span has its own small published target for the same
            // annotation, rather than one rectangle spanning empty paper.
            let state = ui(c, json!({"action":"inspect"}))?;
            let first = controls(&state)
                .find(|a| a["label"] == format!("Edit annotation {id}"))
                .context("First series label")?;
            let second = controls(&state)
                .find(|a| a["label"] == format!("Edit annotation {id} part 2"))
                .context("Second series label")?;
            ensure!(
                first["bounds"] != second["bounds"],
                "Series label parts share one giant target"
            );
            control(c, &format!("Edit annotation {id} part 2"), None)?;
        } else {
            control(c, &format!("Edit annotation {id}"), None)?;
        }
        if layout.is_some() {
            field(c, "Baseline spacing (mm)", "11")?;
            field(c, "Prefix", "SER ")?;
            field(c, "Suffix", " exact")?;
        } else {
            field(c, "Axis", "y")?;
        }
        field(
            c,
            if layout.is_some() {
                "Offset (paper mm)"
            } else {
                "Leader offset (paper mm)"
            },
            "20",
        )?;
        for (name, value) in [
            ("Precision", "3"),
            ("Tolerance mode", "deviation"),
            ("Upper tolerance", "0.2"),
            ("Lower tolerance", "-0.1"),
            ("Reference dimension", "true"),
            ("Fit class", "H7"),
            ("Dual units", "true"),
            ("Secondary unit", "inch"),
            ("Dual precision", "3"),
            ("Dual placement", "bracketed"),
        ] {
            field(c, name, value)?;
        }
        ensure!(model(c)? == created, "{stage} field edits applied early");
        control(c, "Apply annotation", None)?;
        let edited = model(c)?;
        let expected = replace_expected(&created, id, |a| {
            a["offset"] = json!(20.);
            a["precision"] = json!(3);
            if layout.is_some() {
                a["spacing"] = json!(11.);
                a["prefix"] = json!("SER ");
                a["suffix"] = json!(" exact");
            } else {
                a["axis"] = json!("y");
            }
            a["presentation"] = json!({"tolerance":{"mode":"deviation","upper":0.2,"lower":-0.1},"basic":false,"reference":true,"fit_class":"H7","dual_units":{"unit":"inch","precision":3,"placement":"bracketed"}});
        });
        ensure!(
            edited == expected,
            "{stage} inspector lost references, metadata or unrelated intent"
        );
        history(c, &created, &edited)?;
        control(c, &format!("Edit annotation {id}"), None)?;
        capture(c, out, &format!("author-{stage}-edited"))?;
        images.push(format!("author-{stage}-edited.png"));
        curved::save_exact(c, out, stage, &edited)?;
        curved::delete_and_restore(c, id, &created, &edited, baseline)?;
    }
    Ok(
        json!({"endpoint_creation_and_presentation":true,"every_series_label_published":true,"exact_history_archive":true,"all_saved_intent_preserved":true,"captures":images,"not_proven":["Actual OS series/ordinate gestures (separate opt-in fixture)"]}),
    )
}
