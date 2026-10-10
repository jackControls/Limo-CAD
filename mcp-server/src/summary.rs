//! Feature-level feedback for agents: what the current document contains, in the
//! terms an agent reasons in (holes with positions, diameters, depths and
//! faces; bodies with bounding boxes), plus warnings for the mistakes that
//! never raise an error, and a check of an expected feature table against
//! the built model. Everything here reads the same document and scene the
//! desktop shows; nothing is a second modelling path.
use limo_cad_core::PlaneBasis;
use limo_cad_solid::{BodyDto, FaceDto, HoleDefinitionDto, HoleExtent, HoleStyle, SolidSceneDto};
use serde_json::{json, Value};
use std::collections::BTreeMap;

type CoaxialCylinderGroup = (u64, [f64; 3], [f64; 3], Vec<f64>, Vec<u64>);

/// One drilled position, in world coordinates.
#[derive(Clone, Debug)]
pub struct Hole {
    pub feature_id: Option<u64>,
    pub name: String,
    pub body_id: u64,
    pub face_id: Option<u64>,
    pub face_ids: Vec<u64>,
    pub position: [f64; 3],
    /// Outward normal of the face the hole starts from (drilling goes the
    /// other way unless `flip`).
    pub normal: [f64; 3],
    pub flip: bool,
    pub diameter: f64,
    /// Geometry-only inference does not establish whether a cavity breaks through.
    pub through: Option<bool>,
    pub depth: Option<f64>,
    pub depth_evidence: Option<&'static str>,
    pub style: String,
    pub counterbore_diameter: Option<f64>,
    pub counterbore_depth: Option<f64>,
    pub thread: Option<String>,
}

impl Hole {
    fn json(&self) -> Value {
        json!({
            "feature_id": self.feature_id,
            "name": self.name,
            "body_id": self.body_id,
            "face_id": self.face_id,
            "face_ids": self.face_ids,
            "source": if self.feature_id.is_some() { "feature_definition" } else { "geometry" },
            "confidence": if self.feature_id.is_some() { "authored" } else { "candidate" },
            "inference_method": self.feature_id.is_none().then_some("closed_analytic_circle_and_inward_display_normals"),
            "x": round3(self.position[0]),
            "y": round3(self.position[1]),
            "z": round3(self.position[2]),
            "normal": [round3(self.normal[0]), round3(self.normal[1]), round3(self.normal[2])],
            "flip": self.flip,
            "diameter": round3(self.diameter),
            "through": self.through,
            "depth": self.depth.map(round3),
            "depth_evidence": self.depth_evidence,
            "style": self.style,
            "counterbore_diameter": self.counterbore_diameter.map(round3),
            "counterbore_depth": self.counterbore_depth.map(round3),
            "thread": self.thread,
        })
    }

    fn class_key(&self) -> String {
        format!(
            "{:.2}|{}|{}|{}|{}",
            self.diameter,
            self.style,
            self.counterbore_diameter
                .map(|d| format!("{d:.2}"))
                .unwrap_or_default(),
            self.thread.clone().unwrap_or_default(),
            match self.through {
                Some(true) => "through".to_string(),
                Some(false) => format!("depth {:.2}", self.depth.unwrap_or(0.0)),
                None => "unknown extent".to_string(),
            }
        )
    }
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub struct BodyBox {
    pub id: u64,
    pub name: String,
    pub feature_id: u64,
    pub min: [f64; 3],
    pub max: [f64; 3],
    pub faces: usize,
    pub planar_faces: usize,
    pub cylindrical_faces: usize,
}

fn body_box(body: &BodyDto) -> Option<BodyBox> {
    let positions = &body.mesh.positions;
    if positions.len() < 3 {
        return None;
    }
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for chunk in positions.as_chunks::<3>().0 {
        for (i, component) in chunk.iter().enumerate() {
            min[i] = min[i].min(f64::from(*component));
            max[i] = max[i].max(f64::from(*component));
        }
    }
    Some(BodyBox {
        id: body.id.0,
        name: body.name.clone(),
        feature_id: body.feature_id.0,
        min,
        max,
        faces: body.faces.len(),
        planar_faces: body.faces.iter().filter(|f| f.plane.is_some()).count(),
        cylindrical_faces: body.faces.iter().filter(|f| f.cylinder.is_some()).count(),
    })
}

fn face_plane(scene: &SolidSceneDto, body_id: u64, face_id: u64) -> Option<PlaneBasis> {
    scene
        .bodies
        .iter()
        .filter(|b| b.id.0 == body_id)
        .flat_map(|b| b.faces.iter())
        .chain(scene.bodies.iter().flat_map(|b| b.faces.iter()))
        .find(|f| f.id.0 == face_id)
        .and_then(|f| f.plane)
}

/// Holes from the feature history, one entry per drilled position.
pub fn holes_from_definitions(
    definitions: &[HoleDefinitionDto],
    scene: &SolidSceneDto,
) -> Vec<Hole> {
    let mut holes = Vec::new();
    for definition in definitions {
        let Some(basis) = definition
            .face_basis
            .or_else(|| face_plane(scene, definition.body_id.0, definition.face_id.0))
        else {
            continue;
        };
        let (through, depth) = match definition.extent {
            HoleExtent::ThroughAll => (true, None),
            HoleExtent::Distance { depth } => (false, Some(depth)),
        };
        let style = match definition.style {
            HoleStyle::Simple => "simple",
            HoleStyle::Counterbore => "counterbore",
            HoleStyle::Countersink => "countersink",
        };
        let positions: Vec<[f64; 2]> = if definition.positions.is_empty() {
            vec![[definition.position.x, definition.position.y]]
        } else {
            definition
                .positions
                .iter()
                .map(|p| [p.position.x, p.position.y])
                .collect()
        };
        for uv in positions {
            holes.push(Hole {
                feature_id: Some(definition.feature_id.0),
                name: definition.name.clone(),
                body_id: definition.body_id.0,
                face_id: Some(definition.face_id.0),
                face_ids: vec![definition.face_id.0],
                position: basis.to_3d(uv),
                normal: basis.normal,
                flip: definition.flip,
                diameter: definition.diameter,
                through: Some(through),
                depth,
                depth_evidence: depth.map(|_| "authored_feature_extent"),
                style: style.to_string(),
                counterbore_diameter: (definition.style == HoleStyle::Counterbore)
                    .then_some(definition.counterbore_diameter),
                counterbore_depth: (definition.style == HoleStyle::Counterbore)
                    .then_some(definition.counterbore_depth),
                thread: definition.thread.as_ref().map(|t| t.designation.clone()),
            });
        }
    }
    holes
}

/// Infer cylindrical cavities from inward-facing walls with a closed circular
/// boundary. A cylinder alone also describes bosses and fillets. Missing
/// boundary or orientation evidence must not turn those surfaces into holes.
/// Coaxial cavity walls share one entry; distinct radii identify a counterbore.
#[cfg(test)]
pub fn holes_from_scene(scene: &SolidSceneDto) -> Vec<Hole> {
    infer_scene(scene).0
}

fn infer_scene(scene: &SolidSceneDto) -> (Vec<Hole>, Vec<Value>) {
    let mut ambiguous = Vec::new();
    let mut groups: Vec<CoaxialCylinderGroup> = Vec::new();
    for body in &scene.bodies {
        for face in &body.faces {
            let Some(cylinder) = &face.cylinder else {
                continue;
            };
            let axis = normalize([cylinder.axis.x, cylinder.axis.y, cylinder.axis.z]);
            let origin = [cylinder.origin.x, cylinder.origin.y, cylinder.origin.z];
            if !cylinder.radius.is_finite()
                || cylinder.radius <= 0.0
                || !origin
                    .iter()
                    .chain(axis.iter())
                    .all(|value| value.is_finite())
                || norm(axis) < 0.99
            {
                continue;
            }
            match cavity_wall(body, face, origin, axis, cylinder.radius) {
                WallEvidence::Exterior => continue,
                WallEvidence::Ambiguous(reason) => {
                    ambiguous.push(json!({
                        "body_id": body.id.0, "face_ids": [face.id.0],
                        "source": "geometry", "confidence": "ambiguous",
                        "classification": "cylindrical_surface_candidate",
                        "reason": reason,
                        "inference_method": "analytic_cylinder_boundary_and_display_normals",
                        "diameter": round3(2.0 * cylinder.radius),
                        "position": origin, "axis": axis,
                        "through": null, "depth": null,
                    }));
                    continue;
                }
                WallEvidence::Cavity => {}
            }
            let mut joined = Vec::new();
            for (index, group) in groups.iter().enumerate() {
                if group.0 != body.id.0 || dot(group.2, axis).abs() < 0.999999 {
                    continue;
                }
                let delta = sub(origin, group.1);
                let off = sub(delta, scale(group.2, dot(delta, group.2)));
                if norm(off) < 1e-6 && group.4.iter().any(|id| connected_walls(body, face, *id)) {
                    joined.push(index);
                }
            }
            let mut group = (
                body.id.0,
                origin,
                axis,
                vec![cylinder.radius],
                vec![face.id.0],
            );
            // A bridge patch can connect two earlier groups. Merge all of them
            // so splitting a wall does not make recognition depend on face order.
            for index in joined.into_iter().rev() {
                let previous = groups.remove(index);
                group.1 = previous.1;
                group.2 = previous.2;
                group.3.extend(previous.3);
                group.4.extend(previous.4);
            }
            groups.push(group);
        }
    }
    let holes = groups
        .into_iter()
        .map(|(body_id, origin, axis, mut radii, mut face_ids)| {
            radii.sort_by(f64::total_cmp);
            radii.dedup_by(|a, b| (*a - *b).abs() <= 1e-6);
            face_ids.sort_unstable();
            face_ids.dedup();
            // Only a single cylindrical wall bounded by two exact rings and
            // one planar disk establishes a simple blind depth. Multi-radius,
            // split, through and incomplete topology retain unknown extent.
            let extent = (face_ids.len() == 1 && radii.len() == 1)
                .then(|| {
                    let body = scene.bodies.iter().find(|body| body.id.0 == body_id)?;
                    let face = body.faces.iter().find(|face| face.id.0 == face_ids[0])?;
                    blind_depth(body, face, origin, axis, radii[0])
                })
                .flatten();
            let depth = extent.as_ref().map(|extent| extent.depth);
            Hole {
                feature_id: None,
                name: "cylinder".into(),
                body_id,
                face_id: face_ids.first().copied(),
                face_ids,
                position: extent.as_ref().map_or(origin, |extent| extent.mouth),
                normal: extent.as_ref().map_or(axis, |extent| extent.outward),
                flip: false,
                diameter: 2.0 * radii[0],
                through: depth.map(|_| false),
                depth,
                depth_evidence: depth.map(|_| "two_analytic_rings_and_one_planar_disk"),
                style: if radii.len() > 1 {
                    "counterbore"
                } else {
                    "simple"
                }
                .into(),
                counterbore_diameter: (radii.len() > 1).then(|| 2.0 * radii[radii.len() - 1]),
                counterbore_depth: None,
                thread: None,
            }
        })
        .collect();
    (holes, ambiguous)
}

#[derive(Debug, PartialEq)]
enum WallEvidence {
    Cavity,
    Exterior,
    Ambiguous(&'static str),
}

fn cavity_wall(
    body: &BodyDto,
    face: &FaceDto,
    origin: [f64; 3],
    axis: [f64; 3],
    radius: f64,
) -> WallEvidence {
    let closed_boundary = body.edges.iter().any(|edge| {
        if !face.edge_keys.contains(&edge.key) {
            return false;
        }
        let Some(circle) = &edge.circle else {
            return false;
        };
        let center = [circle.center.x, circle.center.y, circle.center.z];
        let normal = normalize([circle.normal.x, circle.normal.y, circle.normal.z]);
        let delta = sub(center, origin);
        circle.closed
            && (circle.radius - radius).abs() <= 1e-6
            && dot(normal, axis).abs() > 0.999999
            && norm(sub(delta, scale(axis, dot(delta, axis)))) <= 1e-6
    });
    let start = face.first_index as usize;
    let Some(end) = start.checked_add(face.index_count as usize) else {
        return WallEvidence::Ambiguous("invalid_face_mesh_range");
    };
    let Some(indices) = body.mesh.indices.get(start..end) else {
        return WallEvidence::Ambiguous("missing_face_mesh");
    };
    let positions = body.mesh.positions.as_chunks::<3>().0;
    let normals = body.mesh.normals.as_chunks::<3>().0;
    let mut inward = false;
    let mut outward = false;
    for index in indices {
        let Some(position) = positions.get(*index as usize) else {
            continue;
        };
        let Some(normal) = normals.get(*index as usize) else {
            continue;
        };
        let normal = normal.map(f64::from);
        let delta = sub(position.map(f64::from), origin);
        let radial = sub(delta, scale(axis, dot(delta, axis)));
        let length = norm(radial) * norm(normal);
        if length.is_finite() && length > 0.0 {
            let direction = dot(radial, normal) / length;
            inward |= direction < -0.5;
            outward |= direction > 0.5;
        }
    }
    match (inward, outward, closed_boundary) {
        (false, true, _) => WallEvidence::Exterior,
        (true, false, true) => WallEvidence::Cavity,
        (true, true, _) => WallEvidence::Ambiguous("inconsistent_wall_orientation"),
        (false, false, _) => WallEvidence::Ambiguous("missing_wall_orientation"),
        (true, false, false) => WallEvidence::Ambiguous("open_or_split_circular_boundary"),
    }
}

// Coaxial but disconnected bores are not a counterbore. Require shared
// topology, either at the wall split or through one planar annular shoulder.
fn connected_walls(body: &BodyDto, face: &FaceDto, other_id: u64) -> bool {
    if face.id.0 == other_id {
        return true;
    }
    let Some(other) = body.faces.iter().find(|other| other.id.0 == other_id) else {
        return false;
    };
    let shares = |a: &FaceDto, b: &FaceDto| a.edge_keys.iter().any(|key| b.edge_keys.contains(key));
    shares(face, other)
        || body.faces.iter().any(|shoulder| {
            shoulder.plane.is_some() && shares(face, shoulder) && shares(other, shoulder)
        })
}

struct BlindExtent {
    depth: f64,
    mouth: [f64; 3],
    outward: [f64; 3],
}

fn blind_depth(
    body: &BodyDto,
    face: &FaceDto,
    origin: [f64; 3],
    axis: [f64; 3],
    radius: f64,
) -> Option<BlindExtent> {
    let rings: Vec<_> = body
        .edges
        .iter()
        .filter_map(|edge| {
            let circle = edge.circle.as_ref()?;
            let center = [circle.center.x, circle.center.y, circle.center.z];
            let delta = sub(center, origin);
            let normal = normalize([circle.normal.x, circle.normal.y, circle.normal.z]);
            (face.edge_keys.contains(&edge.key)
                && circle.closed
                && (circle.radius - radius).abs() <= 1e-6
                && dot(normal, axis).abs() > 0.999999
                && norm(sub(delta, scale(axis, dot(delta, axis)))) <= 1e-6)
                .then_some((edge, center))
        })
        .collect();
    if rings.len() != 2 {
        return None;
    }
    let is_cap = |index: usize| {
        let (edge, _) = rings[index];
        body.faces.iter().any(|cap| {
            cap.id != face.id
                && cap.edge_keys.len() == 1
                && cap.edge_keys[0] == edge.key
                && cap
                    .plane
                    .is_some_and(|plane| dot(plane.normal, axis).abs() > 0.999999)
        })
    };
    let cap = match (is_cap(0), is_cap(1)) {
        (true, false) => 0,
        (false, true) => 1,
        _ => return None,
    };
    let mouth = rings[1 - cap].1;
    let delta = sub(mouth, rings[cap].1);
    let depth = dot(delta, axis).abs();
    (depth.is_finite() && depth > 1e-6).then(|| BlindExtent {
        depth,
        mouth,
        outward: scale(axis, if dot(delta, axis) < 0. { -1. } else { 1. }),
    })
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
fn normalize(a: [f64; 3]) -> [f64; 3] {
    let n = norm(a);
    if n == 0.0 {
        a
    } else {
        scale(a, 1.0 / n)
    }
}

pub struct Summary {
    pub bodies: Vec<BodyBox>,
    pub holes: Vec<Hole>,
    pub source: &'static str,
    pub errors: usize,
    pub display_warnings: Vec<Value>,
    pub ambiguous_hole_candidates: Vec<Value>,
    pub geometry_hole_candidates: Vec<Hole>,
}

pub fn summarize(scene: &SolidSceneDto, definitions: &[HoleDefinitionDto]) -> Summary {
    let bodies = scene.bodies.iter().filter_map(body_box).collect();
    let (inferred, ambiguous_hole_candidates) = infer_scene(scene);
    let (holes, source, geometry_hole_candidates) = if definitions.is_empty() {
        (inferred, "scene", Vec::new())
    } else {
        // Keep the authored count stable without hiding imported geometry in a
        // mixed document. This independent inference layer may overlap authored
        // bores; do not combine the two counts without exact feature provenance.
        (
            holes_from_definitions(definitions, scene),
            "features",
            inferred,
        )
    };
    Summary {
        bodies,
        holes,
        source,
        ambiguous_hole_candidates,
        geometry_hole_candidates,
        errors: scene.errors.len(),
        display_warnings: scene
            .bodies
            .iter()
            .flat_map(|body| {
                body.display_warnings.iter().map(|warning| {
                    json!({
                        "code": "imported_step_face_not_displayed",
                        "body_id": body.id.0,
                        "feature_id": body.feature_id.0,
                        "face_key": warning.face_key,
                        "message": warning.message,
                        "exact_geometry_retained": true,
                    })
                })
            })
            .collect(),
    }
}

impl Summary {
    /// Compact by default: bodies, counts and a tally per hole class. `full`
    /// adds every hole with its position.
    pub fn json(&self, full: bool) -> Value {
        let mut classes: BTreeMap<String, (Value, usize)> = BTreeMap::new();
        for hole in &self.holes {
            let entry = classes.entry(hole.class_key()).or_insert_with(|| {
                (
                    json!({
                        "diameter": round3(hole.diameter),
                        "style": hole.style,
                        "counterbore_diameter": hole.counterbore_diameter.map(round3),
                        "thread": hole.thread,
                        "through": hole.through,
                        "depth": hole.depth.map(round3),
                    }),
                    0,
                )
            });
            entry.1 += 1;
        }
        let mut result = json!({
            "bodies": self.bodies.iter().map(|b| json!({
                "id": b.id, "name": b.name, "feature_id": b.feature_id,
                "bbox_min": [round3(b.min[0]), round3(b.min[1]), round3(b.min[2])],
                "bbox_max": [round3(b.max[0]), round3(b.max[1]), round3(b.max[2])],
                "size": [round3(b.max[0]-b.min[0]), round3(b.max[1]-b.min[1]), round3(b.max[2]-b.min[2])],
                "faces": b.faces, "planar_faces": b.planar_faces, "cylindrical_faces": b.cylindrical_faces,
            })).collect::<Vec<_>>(),
            "scene_errors": self.errors,
            "display_warning_count": self.display_warnings.len(),
            "display_warnings": self.display_warnings,
            "hole_source": self.source,
            "hole_detection_scope": if self.source == "features" {
                "authored_features"
            } else {
                "closed_cylindrical_cavities"
            },
            "hole_count": self.holes.len(),
            "ambiguous_hole_candidate_count": self.ambiguous_hole_candidates.len(),
            "additional_geometry_candidate_count": self.geometry_hole_candidates.len(),
            "additional_geometry_candidate_scope": "independent_inference_may_overlap_authored_features",
            "authored_hole_count": self.holes.iter().filter(|hole| hole.feature_id.is_some()).count(),
            "hole_candidate_count": self.holes.iter().filter(|hole| hole.feature_id.is_none()).count(),
            "holes_by_class": classes.values().map(|(class, count)| {
                let mut class = class.clone();
                class["count"] = json!(count);
                class
            }).collect::<Vec<_>>(),
        });
        if full {
            result["ambiguous_hole_candidates"] = json!(self.ambiguous_hole_candidates);
            result["additional_geometry_candidates"] = self
                .geometry_hole_candidates
                .iter()
                .map(Hole::json)
                .collect();
            result["holes"] = Value::Array(self.holes.iter().map(Hole::json).collect());
        }
        result
    }
}

/// The mistakes that never raise an error: a first point left out of
/// `positions`, holes that merge, holes off the body, blind depths deeper
/// than the body, and bindings a script never used.
pub fn warnings(
    summary: &Summary,
    definitions: &[HoleDefinitionDto],
    unused_bindings: &[String],
) -> Vec<Value> {
    let mut warnings = summary.display_warnings.clone();
    for definition in definitions {
        if !definition.positions.is_empty()
            && !definition.positions.iter().any(|p| {
                (p.position.x - definition.position.x).abs() < 1e-6
                    && (p.position.y - definition.position.y).abs() < 1e-6
            })
        {
            warnings.push(json!({
                "code": "hole_position_ignored",
                "feature_id": definition.feature_id.0,
                "message": format!(
                    "Hole feature {} ({}): position ({}, {}) is not among its {} positions; when positions is given only those points are drilled, so repeat the first point inside positions if it is meant to be a hole",
                    definition.feature_id.0, definition.name, round3(definition.position.x), round3(definition.position.y), definition.positions.len()
                ),
            }));
        }
    }
    let holes = &summary.holes;
    for (i, a) in holes.iter().enumerate() {
        for b in holes.iter().skip(i + 1) {
            if a.body_id != b.body_id || dot(a.normal, b.normal).abs() < 0.999 {
                continue;
            }
            let delta = sub(b.position, a.position);
            let across = sub(delta, scale(a.normal, dot(delta, a.normal)));
            let distance = norm(across);
            let reach = (a.counterbore_diameter.unwrap_or(a.diameter)
                + b.counterbore_diameter.unwrap_or(b.diameter))
                / 2.0;
            if distance < reach - 1e-6 {
                warnings.push(json!({
                    "code": "holes_overlap",
                    "feature_id": a.feature_id,
                    "confirmed": false,
                    "evidence": "projected_axis_distance",
                    "body_id": a.body_id,
                    "face_ids": [&a.face_ids, &b.face_ids],
                    "message": format!(
                        "Hole overlap candidate: axes at ({}, {}, {}) Ø{} and ({}, {}, {}) Ø{} are {} mm apart in projection; verify axial extents and exact geometry before concluding the openings merge",
                        round3(a.position[0]), round3(a.position[1]), round3(a.position[2]), round3(a.diameter),
                        round3(b.position[0]), round3(b.position[1]), round3(b.position[2]), round3(b.diameter),
                        round3(distance)
                    ),
                }));
            }
        }
    }
    for hole in holes {
        let Some(body) = summary.bodies.iter().find(|b| b.id == hole.body_id) else {
            continue;
        };
        let outside = (0..3).any(|i| {
            hole.normal[i].abs() < 0.9
                && (hole.position[i] < body.min[i] - 1e-3 || hole.position[i] > body.max[i] + 1e-3)
        });
        if outside {
            warnings.push(json!({
                "code": "hole_outside_body",
                "feature_id": hole.feature_id,
                "message": format!(
                    "Hole at ({}, {}, {}) Ø{} lies outside body {} (bbox {:?} to {:?})",
                    round3(hole.position[0]), round3(hole.position[1]), round3(hole.position[2]), round3(hole.diameter),
                    body.id, body.min.map(round3), body.max.map(round3)
                ),
            }));
        }
        if let Some(depth) = hole.depth {
            let extent = (0..3)
                .map(|i| (body.max[i] - body.min[i]) * hole.normal[i].abs())
                .sum::<f64>();
            if extent > 0.0 && depth > extent + 1e-3 {
                warnings.push(json!({
                    "code": "blind_depth_exceeds_body",
                    "feature_id": hole.feature_id,
                    "message": format!(
                        "Hole at ({}, {}, {}) Ø{} is blind to {} mm but body {} is only {} mm along its axis; it breaks through",
                        round3(hole.position[0]), round3(hole.position[1]), round3(hole.position[2]), round3(hole.diameter),
                        round3(depth), body.id, round3(extent)
                    ),
                }));
            }
        }
    }
    for name in unused_bindings {
        warnings.push(json!({
            "code": "unused_binding",
            "message": format!("let binding {name} is never referenced"),
        }));
    }
    warnings
}

/// Compare an expected feature table with what was built.
pub fn check(summary: &Summary, expected: &Value, tolerance: f64) -> Result<Value, String> {
    let mut result = json!({"tolerance_mm": tolerance});
    let mut ok = true;
    if let Some(bbox) = expected.get("bbox") {
        let wanted: Vec<f64> = bbox
            .as_array()
            .ok_or("expected.bbox must be [x, y, z] extents")?
            .iter()
            .map(|v| v.as_f64().ok_or("expected.bbox entries must be numbers"))
            .collect::<Result<_, _>>()?;
        if wanted.len() != 3 {
            return Err("expected.bbox must have three extents".into());
        }
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        for body in &summary.bodies {
            for i in 0..3 {
                min[i] = min[i].min(body.min[i]);
                max[i] = max[i].max(body.max[i]);
            }
        }
        let built: Vec<f64> = if summary.bodies.is_empty() {
            vec![0.0; 3]
        } else {
            (0..3).map(|i| max[i] - min[i]).collect()
        };
        let bbox_ok = (0..3).all(|i| (built[i] - wanted[i]).abs() <= tolerance);
        ok &= bbox_ok;
        result["bbox"] = json!({"expected": wanted, "built": built.iter().map(|v| round3(*v)).collect::<Vec<_>>(), "ok": bbox_ok});
    }
    if let Some(holes) = expected.get("holes") {
        let expected_holes = holes.as_array().ok_or("expected.holes must be an array")?;
        let mut used = vec![false; summary.holes.len()];
        let mut matched = Vec::new();
        let mut missing = Vec::new();
        for (index, wanted) in expected_holes.iter().enumerate() {
            let x = wanted["x"].as_f64().ok_or("expected hole needs x")?;
            let y = wanted["y"].as_f64().ok_or("expected hole needs y")?;
            let z = wanted.get("z").and_then(Value::as_f64);
            let distance_to = |hole: &Hole| {
                let dz = z.map_or(0.0, |z| hole.position[2] - z);
                ((hole.position[0] - x).powi(2) + (hole.position[1] - y).powi(2) + dz * dz).sqrt()
            };
            let mut best: Option<(usize, f64)> = None;
            for (i, hole) in summary.holes.iter().enumerate() {
                if used[i] {
                    continue;
                }
                let distance = distance_to(hole);
                if distance <= tolerance && best.is_none_or(|(_, d)| distance < d) {
                    best = Some((i, distance));
                }
            }
            match best {
                Some((i, distance)) => {
                    used[i] = true;
                    let hole = &summary.holes[i];
                    let diameter_ok = wanted
                        .get("diameter")
                        .and_then(Value::as_f64)
                        .is_none_or(|d| (d - hole.diameter).abs() <= 0.05);
                    let counterbore_ok = wanted
                        .get("counterbore_diameter")
                        .and_then(Value::as_f64)
                        .is_none_or(|d| {
                            hole.counterbore_diameter
                                .is_some_and(|built| (built - d).abs() <= 0.05)
                        });
                    let through_ok = wanted
                        .get("through")
                        .and_then(Value::as_bool)
                        .is_none_or(|through| Some(through) == hole.through);
                    let depth_ok =
                        wanted
                            .get("depth")
                            .and_then(Value::as_f64)
                            .is_none_or(|depth| {
                                hole.depth
                                    .is_some_and(|built| (built - depth).abs() <= 0.05)
                            });
                    let all_ok = diameter_ok && counterbore_ok && through_ok && depth_ok;
                    ok &= all_ok;
                    matched.push(json!({
                        "expected_index": index,
                        "built": hole.json(),
                        "offset_mm": round3(distance),
                        "diameter_ok": diameter_ok,
                        "counterbore_ok": counterbore_ok,
                        "through_ok": through_ok,
                        "depth_ok": depth_ok,
                        "ok": all_ok,
                    }));
                }
                None => {
                    ok = false;
                    let nearest = summary
                        .holes
                        .iter()
                        .map(|hole| (distance_to(hole), hole))
                        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    missing.push(json!({
                        "expected_index": index,
                        "expected": wanted,
                        "nearest_built": nearest.map(|(distance, hole)| json!({"offset_mm": round3(distance), "hole": hole.json()})),
                    }));
                }
            }
        }
        let extra: Vec<Value> = summary
            .holes
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(_, hole)| hole.json())
            .collect();
        ok &= extra.is_empty();
        result["holes"] = json!({
            "expected": expected_holes.len(),
            "built": summary.holes.len(),
            "matched": matched,
            "missing": missing,
            "extra": extra,
        });
    }
    result["ok"] = json!(ok);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cylinder_scene() -> SolidSceneDto {
        let point = |x, y, z| json!({"x": x, "y": y, "z": z});
        let edges: Vec<_> = [0., 8.]
            .into_iter()
            .enumerate()
            .map(|(i, z)| {
                json!({
                    "id": i + 1, "key": format!("ring{i}"), "points": [],
                    "circle": {"center": point(0.,0.,z), "normal": point(0.,0.,1.),
                        "reference": point(1.,0.,0.), "radius":2., "closed":true}
                })
            })
            .collect();
        serde_json::from_value(json!({"bodies":[{
            "id":1,"name":"candidate","feature_id":1,
            "mesh":{"positions":[2.,0.,0., 2.,0.,8., 0.,2.,8.],
                "normals":[-1.,0.,0., -1.,0.,0., 0.,-1.,0.],"indices":[0,1,2]},
            "faces":[{"id":1,"key":"wall","first_index":0,"index_count":3,"plane":null,
                "edge_keys":["ring0","ring1"], "cylinder": {
                    "origin":point(0.,0.,0.), "axis":point(0.,0.,1.),
                    "reference":point(1.,0.,0.), "radius":2.}}], "edges":edges
        }],"errors":[]}))
        .unwrap()
    }

    #[test]
    fn ambiguous_split_walls_remain_candidates_without_inflating_bore_count() {
        let mut scene = cylinder_scene();
        let body = &mut scene.bodies[0];
        // Two separately identified half-cylinder patches with open arc
        // boundaries: neither patch has an individual closed circular edge.
        for edge in &mut body.edges {
            edge.circle.as_mut().unwrap().closed = false;
        }
        let mut second = body.faces[0].clone();
        second.id = limo_cad_core::FaceId(2);
        second.key = "second_half".into();
        second.edge_keys = vec!["other_arc0".into(), "other_arc1".into()];
        for i in 0..2 {
            let z = i as f64 * 8.;
            let point = |x, y| limo_cad_solid::Point3Dto { x, y, z };
            body.edges[i].points = vec![point(2., 0.), point(0., 2.), point(-2., 0.)];
            let mut arc = body.edges[i].clone();
            arc.id = limo_cad_core::EdgeId(i as u64 + 3);
            arc.key = format!("other_arc{i}");
            arc.points = vec![point(-2., 0.), point(0., -2.), point(2., 0.)];
            body.edges.push(arc);
        }
        second.first_index = 3;
        body.mesh
            .positions
            .extend([-2., 0., 0., -2., 0., 8., 0., -2., 8.]);
        body.mesh
            .normals
            .extend([1., 0., 0., 1., 0., 0., 0., 1., 0.]);
        body.mesh.indices.extend([3, 4, 5]);
        body.faces.push(second);
        let result = summarize(&scene, &[]).json(true);
        assert_eq!(result["hole_count"], 0);
        assert_eq!(result["ambiguous_hole_candidate_count"], 2);
        assert_eq!(
            result["ambiguous_hole_candidates"][1]["face_ids"],
            json!([2])
        );
        assert_eq!(
            result["ambiguous_hole_candidates"][0]["reason"],
            "open_or_split_circular_boundary"
        );
        scene.bodies[0].mesh.normals.clear();
        let result = summarize(&scene, &[]).json(true);
        assert_eq!(
            result["ambiguous_hole_candidates"][0]["reason"],
            "missing_wall_orientation"
        );
    }

    #[test]
    fn blind_depth_requires_two_exact_rings_and_one_disk_cap() {
        let mut scene = cylinder_scene();
        assert!(holes_from_scene(&scene)[0].depth.is_none());
        let body = &mut scene.bodies[0];
        let mut cap = body.faces[0].clone();
        cap.id = limo_cad_core::FaceId(2);
        cap.key = "bottom".into();
        cap.cylinder = None;
        cap.edge_keys = vec!["ring1".into()];
        cap.plane = Some(PlaneBasis {
            origin: [0., 0., 8.],
            u: [1., 0., 0.],
            v: [0., 1., 0.],
            normal: [0., 0., -1.],
        });
        body.faces.push(cap);
        let hole = &holes_from_scene(&scene)[0];
        assert_eq!(hole.depth, Some(8.));
        assert_eq!(hole.through, Some(false));
        assert_eq!(hole.position, [0., 0., 0.]);
        assert_eq!(hole.normal, [0., 0., -1.]);
        // Cylinder origin and axis sign are parameterization, not the mouth.
        let cylinder = scene.bodies[0].faces[0].cylinder.as_mut().unwrap();
        cylinder.origin.z = 100.;
        cylinder.axis.z = -1.;
        let hole = &holes_from_scene(&scene)[0];
        assert_eq!(hole.depth, Some(8.));
        assert_eq!(hole.position, [0., 0., 0.]);
        assert_eq!(hole.normal, [0., 0., -1.]);
        // An annular shoulder is not a blind bottom.
        scene.bodies[0].faces[1].edge_keys.push("inner_ring".into());
        assert!(holes_from_scene(&scene)[0].depth.is_none());
    }

    #[test]
    fn disconnected_coaxial_cylinders_do_not_form_a_false_counterbore() {
        let mut scene = cylinder_scene();
        let body = &mut scene.bodies[0];
        let mut second = body.faces[0].clone();
        second.id = limo_cad_core::FaceId(2);
        second.key = "other_wall".into();
        second.edge_keys = vec!["other_ring".into()];
        let mut edge = body.edges[0].clone();
        edge.key = "other_ring".into();
        edge.id = limo_cad_core::EdgeId(3);
        edge.circle.as_mut().unwrap().center.z = 20.;
        body.edges.push(edge);
        body.faces.push(second);
        assert_eq!(holes_from_scene(&scene).len(), 2);
        // The shared ring is positive connectivity evidence for an axial split.
        scene.bodies[0].faces[1].edge_keys.push("ring1".into());
        assert_eq!(holes_from_scene(&scene).len(), 1);
        for normal in &mut scene.bodies[0].mesh.normals {
            *normal = -*normal;
        }
        let result = summarize(&scene, &[]).json(true);
        assert_eq!(result["hole_count"], 0);
        assert_eq!(result["ambiguous_hole_candidate_count"], 0);
    }
}
