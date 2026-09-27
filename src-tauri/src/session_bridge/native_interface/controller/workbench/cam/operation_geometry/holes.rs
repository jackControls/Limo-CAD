use super::*;

const POINTS: &str = "/native/geometry/centers";
const COUNT: &str = "/native/geometry/hole_count";
const CURRENT: &str = "/native/ui/geometry_hole";

pub(super) fn candidates(setup: &CamSetupDto, scene: &SolidSceneDto) -> Vec<(String, CamHoleDto)> {
    let mut result = Vec::new();
    for body in scene
        .bodies
        .iter()
        .filter(|body| setup.body_ids.contains(&body.id))
    {
        for face in body.faces.iter().filter(|face| face.cylinder.is_some()) {
            let reference = format!("{}:{}", body.id.0, face.id.0);
            let mut hole = CamHoleDto {
                point: Point2Dto::new(0., 0.),
                top_z: 0.,
                bottom_z: 0.,
                axis: [0., 0., 1.],
                face_key: Some(reference.clone()),
            };
            if nbcad_sketch::resolve_cam_hole_reference(&reference, &mut hole, setup, scene).is_ok()
            {
                let diameter = face.cylinder.unwrap().radius * 2.;
                result.push((
                    format!("{} · cylinder {} · Ø{diameter:.3} mm", body.name, face.id.0),
                    hole,
                ));
            }
        }
    }
    result
}
fn prefix(index: usize) -> String {
    format!("{PREFIX}holes/{index}")
}
fn stored_points(record: &Value) -> &[Value] {
    record["points"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
}
fn stored_holes(record: &Value) -> &[Value] {
    record["holes"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
}
fn active(draft: &Draft) -> Option<usize> {
    let index = form::text(draft, CURRENT)
        .ok()?
        .parse::<usize>()
        .ok()?
        .checked_sub(1)?;
    (index < count(draft, COUNT, 250_000).ok()?).then_some(index)
}
pub(super) fn extend(
    draft: &mut Draft,
    cam: &CamDocumentDto,
    context: &Context,
) -> Result<(), String> {
    let points = stored_points(&draft.record).to_vec();
    points::extend(draft, POINTS, "Manual center", &points, cam.units);
    form::push(
        draft,
        COUNT,
        "Hole count",
        InputKind::Integer,
        json!(stored_holes(&draft.record).len()),
        cam.units,
        None,
    );
    form::push(
        draft,
        CURRENT,
        "Hole number",
        InputKind::Integer,
        json!(1),
        cam.units,
        None,
    );
    extend_active(draft, cam.units, context)
}
fn extend_active(draft: &mut Draft, units: CamUnits, context: &Context) -> Result<(), String> {
    let Some(index) = active(draft) else {
        return Ok(());
    };
    let prefix = prefix(index);
    if draft
        .fields
        .iter()
        .any(|field| field.path == format!("{prefix}/source"))
    {
        return Ok(());
    }
    let original = stored_holes(&draft.record)
        .get(index)
        .cloned()
        .unwrap_or(Value::Null);
    let source = if original.is_null() || original["face_key"].is_string() {
        "face"
    } else {
        "manual"
    };
    form::push(
        draft,
        &format!("{prefix}/source"),
        "Hole source",
        InputKind::Choice,
        json!(source),
        units,
        Some(form::options(&[
            ("face", "Picked cylindrical face"),
            ("manual", "Explicit hole span"),
        ])),
    );
    let reference = original["face_key"].as_str().unwrap_or("");
    let mut options = context
        .holes
        .iter()
        .map(|(label, hole)| ChoiceOption {
            value: hole.face_key.clone().unwrap(),
            label: label.clone(),
            disabled: false,
        })
        .collect::<Vec<_>>();
    if !reference.is_empty() && !options.iter().any(|option| option.value == reference) {
        options.push(ChoiceOption {
            value: reference.into(),
            label: "Saved cylindrical face is unavailable".into(),
            disabled: true,
        });
    }
    form::push(
        draft,
        &format!("{prefix}/face"),
        "Cylindrical face",
        InputKind::Choice,
        json!(reference),
        units,
        Some(options),
    );
    for (path, label, value) in [
        ("x", "Hole center X", original["point"]["x"].clone()),
        ("y", "Hole center Y", original["point"]["y"].clone()),
        ("top_z", "Hole top Z", original["top_z"].clone()),
        ("bottom_z", "Hole bottom Z", original["bottom_z"].clone()),
    ] {
        form::push(
            draft,
            &format!("{prefix}/{path}"),
            label,
            InputKind::Length,
            value,
            units,
            None,
        );
    }
    form::push(
        draft,
        &format!("{prefix}/axis"),
        "Hole axis",
        InputKind::Choice,
        json!(if original["axis"][2].as_f64().unwrap_or(1.) < 0. {
            "down"
        } else {
            "up"
        }),
        units,
        Some(form::options(&[("up", "Setup +Z"), ("down", "Setup −Z")])),
    );
    Ok(())
}
pub(super) fn changed(
    draft: &mut Draft,
    cam: &CamDocumentDto,
    path: &str,
    context: &Context,
) -> Result<(), String> {
    if path.starts_with(POINTS) || path == points::cursor(POINTS) {
        let original = stored_points(&draft.record).to_vec();
        return points::changed(draft, POINTS, "Manual center", path, &original, cam.units);
    }
    let length = count(draft, COUNT, 250_000)?;
    if path == COUNT {
        let next = if length > stored_holes(&draft.record).len() {
            length
        } else {
            active(draft).map_or(1, |n| n + 1).min(length.max(1))
        };
        form::set(draft, CURRENT, &next.to_string());
    }
    if length > 0 && active(draft).is_none() {
        return Err(format!("Choose a hole number from 1 to {length}"));
    }
    extend_active(draft, cam.units, context)
}
pub(super) fn visible(draft: &Draft, path: &str) -> bool {
    if path.starts_with(POINTS) || path == points::cursor(POINTS) {
        return points::visible(draft, POINTS, path);
    }
    if path == COUNT {
        return true;
    }
    if path == CURRENT {
        return count(draft, COUNT, 250_000).unwrap_or(0) > 0;
    }
    let Some(index) = active(draft) else {
        return false;
    };
    let prefix = prefix(index);
    let Some(suffix) = path.strip_prefix(&format!("{prefix}/")) else {
        return false;
    };
    let picked = form::text(draft, &format!("{prefix}/source")).is_ok_and(|s| s == "face");
    match suffix {
        "source" => true,
        "face" => picked,
        "x" | "y" | "top_z" | "bottom_z" | "axis" => !picked,
        _ => false,
    }
}
pub(super) fn apply(
    draft: &Draft,
    record: &mut Value,
    units: CamUnits,
    context: &Context,
) -> Result<(), String> {
    if form::changed(draft, POINTS) {
        record["points"] =
            serde_json::to_value(points::read(draft, POINTS, stored_points(record), units)?)
                .map_err(|e| e.to_string())?;
    }
    let length = count(draft, COUNT, 250_000)?;
    let original = stored_holes(record);
    let mut holes = Vec::with_capacity(length);
    for index in 0..length {
        let prefix = prefix(index);
        if !form::changed(draft, &format!("{prefix}/")) {
            if let Some(original) = original.get(index) {
                holes.push(original.clone());
                continue;
            }
        }
        match form::text(draft, &format!("{prefix}/source"))? {
            "face" => {
                let reference = form::text(draft, &format!("{prefix}/face"))?;
                let hole = context
                    .holes
                    .iter()
                    .find(|(_, hole)| hole.face_key.as_deref() == Some(reference))
                    .map(|(_, hole)| hole)
                    .ok_or("Choose an available cylindrical face aligned with setup Z")?;
                holes.push(serde_json::to_value(hole).map_err(|e| e.to_string())?);
            }
            "manual" => {
                let original = original.get(index).cloned().unwrap_or(Value::Null);
                let value = |key, old| number(draft, &format!("{prefix}/{key}"), old, units);
                let axis = if !form::changed(draft, &format!("{prefix}/axis"))
                    && original["axis"].is_array()
                {
                    original["axis"].clone()
                } else {
                    match form::text(draft, &format!("{prefix}/axis"))? {
                        "up" => json!([0., 0., 1.]),
                        "down" => json!([0., 0., -1.]),
                        _ => return Err("Choose a hole axis".into()),
                    }
                };
                holes.push(json!({"point":{"x":value("x",original["point"]["x"].as_f64())?,"y":value("y",original["point"]["y"].as_f64())?},
                    "top_z":value("top_z",original["top_z"].as_f64())?,"bottom_z":value("bottom_z",original["bottom_z"].as_f64())?,"axis":axis,"face_key":null}));
            }
            _ => return Err("Choose a hole source".into()),
        }
    }
    record["holes"] = json!(holes);
    Ok(())
}
