//! Paper-space geometry and exact topology resolution shared by all annotations.
//! Corresponds to the existing drawing/annotations.ts renderer; fallback model
//! coordinates are diagnostic data and never substitute for missing topology.
use super::super::paper_point;
use nbcad_occt::{DrawingProjectionAnchorEndpoint as Endpoint, DrawingProjectionDto};
use nbcad_sketch::*;

pub(super) type P = [f64; 2];
pub(super) fn add(a: P, b: P) -> P {
    [a[0] + b[0], a[1] + b[1]]
}
pub(super) fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}
pub(super) fn scale(a: P, v: f64) -> P {
    [a[0] * v, a[1] * v]
}
pub(super) fn dot(a: P, b: P) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
pub(super) fn cross(a: P, b: P) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
pub(super) fn length(a: P) -> f64 {
    a[0].hypot(a[1])
}
pub(super) fn unit(a: P) -> Option<P> {
    let l = length(a);
    (l >= 1e-7 && l.is_finite()).then(|| scale(a, 1. / l))
}
pub(super) fn midpoint(a: P, b: P) -> P {
    scale(add(a, b), 0.5)
}
pub(super) fn normal(a: P) -> P {
    [-a[1], a[0]]
}
pub(super) fn arc(center: P, radius: f64, start: f64, sweep: f64) -> Vec<P> {
    let count = ((sweep.abs() * radius.max(1.) / 0.4).ceil() as usize).clamp(8, 512);
    (0..=count)
        .map(|i| {
            let a = start + sweep * i as f64 / count as f64;
            add(center, [a.cos() * radius, a.sin() * radius])
        })
        .collect()
}

#[derive(Clone, Copy)]
pub(super) struct Circle {
    pub center: P,
    pub radius: f64,
    pub model_radius: f64,
}
pub(super) struct Resolver<'a> {
    pub view: &'a DrawingViewDto,
    pub projection: &'a DrawingProjectionDto,
}
impl Resolver<'_> {
    fn signature(&self, body: nbcad_core::BodyId, expected: &Option<String>) -> Option<()> {
        (self.projection.topology_signatures.get(&body.0.to_string()) == expected.as_ref())
            .then_some(())
    }
    pub fn anchor(&self, r: &DrawingTopologyAnchorRefDto) -> Option<P> {
        self.signature(r.body_id, &r.topology_signature)?;
        if r.circle_center {
            let matches = |c: &&nbcad_occt::DrawingProjectedCircleDto| {
                c.occurrence_id == r.occurrence_id
                    && c.body_id == r.body_id
                    && c.edge_key == r.edge_key
            };
            let c = self
                .projection
                .circles
                .iter()
                .filter(matches)
                .find(|c| c.edge_id == r.edge_id)
                .or_else(|| self.projection.circles.iter().find(matches))?;
            return Some(paper_point(self.view, c.center, self.projection));
        }
        let endpoint = match r.endpoint {
            DrawingEdgeEndpoint::Start => Endpoint::Start,
            DrawingEdgeEndpoint::End => Endpoint::End,
        };
        let matches = |c: &&nbcad_occt::DrawingProjectionAnchorDto| {
            c.occurrence_id == r.occurrence_id
                && c.body_id == r.body_id
                && c.edge_key == r.edge_key
                && c.endpoint == endpoint
        };
        let c = self
            .projection
            .anchors
            .iter()
            .filter(matches)
            .find(|c| c.edge_id == r.edge_id)
            .or_else(|| self.projection.anchors.iter().find(matches))?;
        Some(paper_point(self.view, c.point, self.projection))
    }
    pub fn line(&self, r: &DrawingLineRefDto) -> Option<[P; 2]> {
        self.signature(r.body_id, &r.topology_signature)?;
        let endpoint = |endpoint| {
            let matches = |c: &&nbcad_occt::DrawingProjectionAnchorDto| {
                c.occurrence_id == r.occurrence_id
                    && c.body_id == r.body_id
                    && c.edge_key == r.edge_key
                    && c.endpoint == endpoint
            };
            let c = self
                .projection
                .anchors
                .iter()
                .filter(matches)
                .find(|c| c.edge_id == r.edge_id)
                .or_else(|| self.projection.anchors.iter().find(matches))?;
            Some(paper_point(self.view, c.point, self.projection))
        };
        Some([endpoint(Endpoint::Start)?, endpoint(Endpoint::End)?])
    }
    pub fn circle(&self, r: &DrawingCircularRefDto) -> Option<Circle> {
        self.signature(r.body_id, &r.topology_signature)?;
        let matches = |c: &&nbcad_occt::DrawingProjectedCircleDto| {
            c.occurrence_id == r.occurrence_id && c.body_id == r.body_id && c.edge_key == r.edge_key
        };
        let c = self
            .projection
            .circles
            .iter()
            .filter(matches)
            .find(|c| c.edge_id == r.edge_id)
            .or_else(|| self.projection.circles.iter().find(matches))?;
        Some(Circle {
            center: paper_point(self.view, c.center, self.projection),
            radius: c.radius * self.view.scale,
            model_radius: c.radius,
        })
    }
    pub fn attachment(&self, r: &DrawingAttachmentRefDto) -> Option<P> {
        match r {
            DrawingAttachmentRefDto::Anchor { reference } => self.anchor(reference),
            DrawingAttachmentRefDto::Circle { reference } => {
                self.circle(reference).map(|c| c.center)
            }
            DrawingAttachmentRefDto::Line { reference } => {
                self.line(reference).map(|[a, b]| midpoint(a, b))
            }
        }
    }
}

pub(super) struct Linear {
    pub first: P,
    pub second: P,
    pub start: P,
    pub end: P,
    pub value: f64,
}
pub(super) struct Angular {
    pub vertex: P,
    pub first: P,
    pub second: P,
    pub points: Vec<P>,
    pub text: P,
    pub value: f64,
}
pub(super) enum LineDimension {
    Linear(Linear),
    Angular(Angular),
}
pub(super) fn angular(vertex: P, first: P, second: P, radius: f64) -> Option<Angular> {
    let a = unit(sub(first, vertex))?;
    let b = unit(sub(second, vertex))?;
    let angle = dot(a, b).clamp(-1., 1.).acos();
    if angle < 1e-7 {
        return None;
    }
    let bisector = unit(add(a, b)).unwrap_or(normal(a));
    let sweep = angle * if cross(a, b) >= 0. { 1. } else { -1. };
    Some(Angular {
        vertex,
        first: add(vertex, scale(a, radius + 3.)),
        second: add(vertex, scale(b, radius + 3.)),
        points: arc(vertex, radius, a[1].atan2(a[0]), sweep),
        text: add(vertex, scale(bisector, radius + 4.)),
        value: angle.to_degrees(),
    })
}
fn closest(point: P, line: [P; 2]) -> P {
    let v = sub(line[1], line[0]);
    let denominator = dot(v, v);
    if denominator < 1e-14 {
        line[0]
    } else {
        add(
            line[0],
            scale(v, (dot(sub(point, line[0]), v) / denominator).clamp(0., 1.)),
        )
    }
}
pub(super) fn point_line(point: P, line: [P; 2], position: P, view_scale: f64) -> Option<Linear> {
    let direction = unit(sub(line[1], line[0]))?;
    let foot = add(
        line[0],
        scale(direction, dot(sub(point, line[0]), direction)),
    );
    let value = length(sub(foot, point)) / view_scale;
    if value < 1e-7 {
        return None;
    }
    let offset = scale(direction, dot(sub(position, point), direction));
    Some(Linear {
        first: point,
        second: foot,
        start: add(point, offset),
        end: add(foot, offset),
        value,
    })
}
pub(super) fn line_dimension(
    first: [P; 2],
    second: Option<[P; 2]>,
    mode: DrawingLineDimensionMode,
    position: P,
    view_scale: f64,
) -> Option<LineDimension> {
    let direction = unit(sub(first[1], first[0]))?;
    let mid = midpoint(first[0], first[1]);
    if mode == DrawingLineDimensionMode::Length {
        let normal = normal(direction);
        let offset = scale(normal, dot(sub(position, mid), normal));
        return Some(LineDimension::Linear(Linear {
            first: first[0],
            second: first[1],
            start: add(first[0], offset),
            end: add(first[1], offset),
            value: length(sub(first[1], first[0])) / view_scale,
        }));
    }
    let second = second?;
    let other = unit(sub(second[1], second[0]))?;
    let parallel_tolerance = 1_f64.to_radians().sin();
    if mode == DrawingLineDimensionMode::Distance {
        if cross(direction, other).abs() > parallel_tolerance {
            return None;
        }
        let normal = normal(direction);
        let separation = dot(sub(midpoint(second[0], second[1]), mid), normal);
        if separation.abs() < 1e-7 {
            return None;
        }
        let start = add(mid, scale(direction, dot(sub(position, mid), direction)));
        let end = add(start, scale(normal, separation));
        return Some(LineDimension::Linear(Linear {
            first: closest(start, first),
            second: closest(end, second),
            start,
            end,
            value: separation.abs() / view_scale,
        }));
    }
    let denominator = cross(direction, other);
    if denominator.abs() <= parallel_tolerance {
        return None;
    }
    let vertex = add(
        first[0],
        scale(
            direction,
            cross(sub(second[0], first[0]), other) / denominator,
        ),
    );
    let toward = sub(position, vertex);
    let a = scale(
        direction,
        if dot(toward, direction) < 0. { -1. } else { 1. },
    );
    let b = scale(other, if dot(toward, other) < 0. { -1. } else { 1. });
    angular(
        vertex,
        add(vertex, a),
        add(vertex, b),
        length(toward).max(4.),
    )
    .map(LineDimension::Angular)
}
pub(super) fn center_between(first: [P; 2], second: [P; 2], extension: f64) -> Option<[P; 2]> {
    let a = sub(first[1], first[0]);
    let b = sub(second[1], second[0]);
    let dir = unit(a)?;
    let other = unit(b)?;
    if cross(dir, other).abs() > 0.75_f64.to_radians().sin() {
        return None;
    }
    let normal = normal(dir);
    let first_mid = midpoint(first[0], first[1]);
    let second_mid = midpoint(second[0], second[1]);
    if dot(sub(second_mid, first_mid), normal).abs() < 1e-4 {
        return None;
    }
    let values = [
        dot(first[0], dir),
        dot(first[1], dir),
        dot(second[0], dir),
        dot(second[1], dir),
    ];
    if values[0].max(values[1]).min(values[2].max(values[3]))
        - values[0].min(values[1]).max(values[2].min(values[3]))
        < length(a).min(length(b)) * 0.05
    {
        return None;
    }
    let offset = dot(midpoint(first_mid, second_mid), normal);
    let low = values.iter().copied().fold(f64::INFINITY, f64::min) - extension.max(0.);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max) + extension.max(0.);
    Some([
        add(scale(dir, low), scale(normal, offset)),
        add(scale(dir, high), scale(normal, offset)),
    ])
}
