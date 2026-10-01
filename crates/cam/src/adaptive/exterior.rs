//! Continuous exterior clearing with a conservative convex remaining-stock
//! certificate. This is a generic convex-envelope fast path, not a rectangle
//! special case. Concavities and cavities are left to the checked fallback.
//!
//! Let H contain the protected target section and stock S be contained in
//! H (+) disk(B). A closed tool-center offset at d = B + R - e sweeps every
//! point with d-R <= distance(point,H) <= d+R. After the loop, remaining
//! stock is therefore contained in H (+) disk(B-e).
//!
//! At every offset point, a supporting normal n bounds old stock by
//! n.(p-center) <= -(R-e). The back half of a moving cutter is already swept
//! by its infinitesimally preceding positions; intersecting the front half
//! with that supporting half-plane bounds engagement by acos(1-e/R).
//! Tangent straight entries from air obey the same half-plane inequality.
//! Convex offset lines/arcs are C1, so the bound holds through corners too.

use super::{
    dist, subtract_arc, CamPlanError, CamSetupDto, Envelope, Point2Dto, Point3Dto, ProgramBuilder,
    Work, EPS,
};
use crate::model::{CamAdaptiveParametersDto, CamResolvedStockDto};

const MAX_HULL_POINTS: usize = 16_384;
const MAX_HULL_VERTICES: usize = 256;
const MAX_PASSES: usize = 2048;
/// Numeric slack, separate from the user's material allowance and grid error.
const CLEARANCE_GUARD: f64 = 1e-4;

fn cross(a: Point2Dto, b: Point2Dto, c: Point2Dto) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}
fn point_segment(p: Point2Dto, a: Point2Dto, b: Point2Dto) -> f64 {
    let (x, y) = (b.x - a.x, b.y - a.y);
    let t = (((p.x - a.x) * x + (p.y - a.y) * y) / (x * x + y * y).max(EPS * EPS)).clamp(0.0, 1.0);
    (p.x - a.x - t * x).hypot(p.y - a.y - t * y)
}
fn segment_distance(a: Point2Dto, b: Point2Dto, c: Point2Dto, d: Point2Dto) -> f64 {
    if cross(a, b, c) * cross(a, b, d) < 0.0 && cross(c, d, a) * cross(c, d, b) < 0.0 {
        return 0.0;
    }
    point_segment(a, c, d)
        .min(point_segment(b, c, d))
        .min(point_segment(c, a, b))
        .min(point_segment(d, a, b))
}

/// A conservative XY boundary shared by pass bounds and lead clearance.
fn stock_footprint(setup: &CamSetupDto) -> Vec<Point2Dto> {
    match &setup.resolved_stock {
        CamResolvedStockDto::Cylinder { center, radius } => {
            const N: usize = 128;
            let outer = radius / (std::f64::consts::PI / N as f64).cos();
            (0..N)
                .map(|i| {
                    let angle = std::f64::consts::TAU * i as f64 / N as f64;
                    Point2Dto::new(
                        center.x + outer * angle.cos(),
                        center.y + outer * angle.sin(),
                    )
                })
                .collect()
        }
        CamResolvedStockDto::Hex {
            center,
            across_flats,
        } => {
            let radius = across_flats / 3.0_f64.sqrt();
            (0..6)
                .map(|i| {
                    let angle = std::f64::consts::PI / 6.0 + std::f64::consts::TAU * i as f64 / 6.0;
                    Point2Dto::new(
                        center.x + radius * angle.cos(),
                        center.y + radius * angle.sin(),
                    )
                })
                .collect()
        }
        _ => vec![
            Point2Dto::new(setup.stock.min.x, setup.stock.min.y),
            Point2Dto::new(setup.stock.max.x, setup.stock.min.y),
            Point2Dto::new(setup.stock.max.x, setup.stock.max.y),
            Point2Dto::new(setup.stock.min.x, setup.stock.max.y),
        ],
    }
}

#[derive(Clone)]
pub(super) struct ConvexStock {
    hull: Vec<Point2Dto>, // CCW; collinear interior points removed.
    normals: Vec<Point2Dto>,
    pub(super) offset: f64,
}

impl ConvexStock {
    fn from_points(mut points: Vec<Point2Dto>, offset: f64) -> Option<Self> {
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
        points.dedup();
        if points.len() < 3 {
            return None;
        }
        let mut hull = Vec::new();
        for &p in &points {
            // Do not use a positive simplification epsilon: that could
            // replace a slightly convex corner by an inward chord.
            while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0 {
                hull.pop();
            }
            hull.push(p);
        }
        let lower = hull.len();
        for &p in points[..points.len() - 1].iter().rev() {
            while hull.len() > lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], p) <= 0.0
            {
                hull.pop();
            }
            hull.push(p);
        }
        hull.pop();
        if hull.len() < 3 || hull.len() > MAX_HULL_VERTICES {
            return None;
        }
        let normals = (0..hull.len())
            .map(|i| {
                let a = hull[i];
                let b = hull[(i + 1) % hull.len()];
                let length = dist(a, b);
                Point2Dto::new((b.y - a.y) / length, (a.x - b.x) / length)
            })
            .collect();
        Some(Self {
            hull,
            normals,
            offset,
        })
    }

    pub(super) fn from_envelope(
        e: &Envelope,
        depth: f64,
        axial: f64,
        allowance: f64,
        work: &mut Work,
    ) -> Result<Option<Self>, CamPlanError> {
        work.spend(e.target.len(), 0)?;
        let mut points = Vec::new();
        for y in 0..e.ny {
            let active = |x: usize| e.target[x + e.nx * y] + axial > depth + EPS;
            let Some(first) = (0..e.nx).find(|&x| active(x)) else {
                continue;
            };
            let last = (first..e.nx).rev().find(|&x| active(x)).unwrap();
            // Hull orientation is computed on integer grid indices so
            // floating point noise cannot create near-zero corner arcs.
            let x0 = first as f64;
            let x1 = (last + 1) as f64;
            let y0 = y as f64;
            let y1 = (y + 1) as f64;
            // Row extrema enclose every protected *whole cell*, not just
            // its center. Taking their hull preserves that containment.
            points.extend([
                Point2Dto::new(x0, y0),
                Point2Dto::new(x1, y0),
                Point2Dto::new(x0, y1),
                Point2Dto::new(x1, y1),
            ]);
            if points.len() > MAX_HULL_POINTS {
                return Ok(None);
            }
        }
        if points.is_empty() {
            // Above all target surfaces, clear the stock cap with the same
            // proved concentric passes. A tiny virtual island supplies a
            // closed convex offset; the checked fallback clears its core.
            let c = Point2Dto::new(
                e.min.x + e.nx as f64 * e.h * 0.5,
                e.min.y + e.ny as f64 * e.h * 0.5,
            );
            let q = e.h.min(1.0) * 0.01;
            return Ok(Self::from_points(
                vec![
                    Point2Dto::new(c.x - q, c.y - q),
                    Point2Dto::new(c.x + q, c.y - q),
                    Point2Dto::new(c.x + q, c.y + q),
                    Point2Dto::new(c.x - q, c.y + q),
                ],
                CLEARANCE_GUARD,
            ));
        }
        work.spend(points.len().saturating_mul(16), 0)?;
        let Some(mut hull) = Self::from_points(points, allowance + CLEARANCE_GUARD) else {
            return Ok(None);
        };
        for p in &mut hull.hull {
            p.x = e.min.x + p.x * e.h;
            p.y = e.min.y + p.y * e.h;
        }
        // The index-space normals are invariant under this uniform scale.
        // Reject numerically ambiguous corners rather than emit a full
        // circle when a tiny intended turn rounds to coincident angles.
        for i in 0..hull.normals.len() {
            let a = hull.normals[i];
            let b = hull.normals[(i + 1) % hull.normals.len()];
            if (a.x * b.y - a.y * b.x) <= 1e-8 {
                return Ok(None);
            }
        }
        Ok(Some(hull))
    }

    /// A descending face-mill layer must protect the entire preceding
    /// remaining-stock bound above the previous floor/corner transition.
    pub(super) fn contains_bound(&self, previous: &Self) -> bool {
        previous
            .hull
            .iter()
            .all(|&v| self.distance(v) + previous.offset <= self.offset + EPS)
    }

    pub(super) fn vertices(&self) -> usize {
        self.hull.len()
    }
    fn inside(&self, p: Point2Dto) -> bool {
        (0..self.hull.len())
            .all(|i| cross(self.hull[i], self.hull[(i + 1) % self.hull.len()], p) >= -EPS)
    }
    pub(super) fn distance(&self, p: Point2Dto) -> f64 {
        if self.inside(p) {
            return 0.0;
        }
        (0..self.hull.len())
            .map(|i| point_segment(p, self.hull[i], self.hull[(i + 1) % self.hull.len()]))
            .fold(f64::INFINITY, f64::min)
    }
    pub(super) fn capsule_clear(&self, a: Point2Dto, b: Point2Dto, r: f64) -> bool {
        if self.inside(a) || self.inside(b) {
            return false;
        }
        (0..self.hull.len()).all(|i| {
            segment_distance(a, b, self.hull[i], self.hull[(i + 1) % self.hull.len()])
                > self.offset + r + EPS
        })
    }
    pub(super) fn point_clear(&self, p: Point2Dto) -> bool {
        self.distance(p) > self.offset + EPS
    }

    /// Supporting half-planes give a superset of the rounded stock bound.
    /// Their angular intersection can overestimate corner engagement, but
    /// never hides remaining stock from the fallback's engagement checks.
    pub(super) fn clip_contact(&self, c: Point2Dto, r: f64, ranges: &mut Vec<(f64, f64)>) {
        for (a, n) in self.hull.iter().zip(&self.normals) {
            let cosine = (n.x * (a.x - c.x) + n.y * (a.y - c.y) + self.offset) / r;
            if cosine <= -1.0 {
                ranges.clear();
                break;
            }
            if cosine < 1.0 {
                subtract_arc(ranges, n.y.atan2(n.x), cosine.acos());
            }
        }
    }

    /// Whole-segment clearance against the conservative stock left by the
    /// preceding loop. The certificate covers stock above this layer only;
    /// it must never authorize a descent below the certified floor.
    fn link_through_cleared_stock(
        &self,
        builder: &mut ProgramBuilder,
        stock: &Self,
        to: Point3Dto,
        floor: f64,
        r: f64,
    ) -> bool {
        let Some(link) = builder.linking.clone().filter(|l| l.keep_tool_down) else {
            return false;
        };
        let Some(from) = builder.position.filter(|p| p.z < builder.retract_z - EPS) else {
            return false;
        };
        let a = Point2Dto::new(from.x, from.y);
        let b = Point2Dto::new(to.x, to.y);
        let lift = from.z.max(to.z) + link.lift_height;
        let radius = r + link.minimum_clearance.max(link.safe_distance);
        if from.z < floor - EPS
            || to.z < floor - EPS
            || lift > builder.feed_height_z
            || dist(a, b) + (lift - from.z) + (lift - to.z) > link.maximum_stay_down
            || !stock.capsule_clear(a, b, radius)
            || !self.capsule_clear(a, b, radius)
        {
            return false;
        }
        builder.linear(Point3Dto::new(a.x, a.y, lift), link.no_engagement_feed);
        builder.linear(Point3Dto::new(b.x, b.y, lift), link.no_engagement_feed);
        builder.linear(to, link.no_engagement_feed);
        true
    }

    fn remaining_footprint(&self, stock: &[Point2Dto], offset: f64) -> Vec<Point2Dto> {
        let mut polygon = stock.to_vec();
        for (&v, &n) in self.hull.iter().zip(&self.normals) {
            let limit = v.x * n.x + v.y * n.y + offset + CLEARANCE_GUARD;
            let side = |p: Point2Dto| p.x * n.x + p.y * n.y - limit;
            let input = std::mem::take(&mut polygon);
            let Some(&last) = input.last() else {
                break;
            };
            let mut a = last;
            for b in input {
                let (da, db) = (side(a), side(b));
                if (da <= 0.0) != (db <= 0.0) {
                    let t = da / (da - db);
                    polygon.push(Point2Dto::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)));
                }
                if db <= 0.0 {
                    polygon.push(b);
                }
                a = b;
            }
        }
        polygon
    }

    pub(super) fn clear_exterior(
        &self,
        builder: &mut ProgramBuilder,
        setup: &CamSetupDto,
        r: f64,
        floor_r: f64,
        depth: f64,
        p: &CamAdaptiveParametersDto,
        feed: f64,
        plunge: f64,
        work: &mut Work,
        envelope: &Envelope,
    ) -> Result<usize, CamPlanError> {
        if depth >= setup.stock.max.z - EPS {
            return Ok(0);
        }
        let mut footprint = if let Some(heights) = &envelope.stock {
            // Whole-cell row bounds contain all incoming material above this
            // layer, including upper sections of a bull/face mill. Convexity
            // may retain air in holes but never assumes an unsafe entry.
            let mut points = Vec::new();
            for y in 0..envelope.ny {
                let row = &heights[y * envelope.nx..(y + 1) * envelope.nx];
                if let (Some(lo), Some(hi)) = (
                    row.iter().position(|z| *z > depth + EPS),
                    row.iter().rposition(|z| *z > depth + EPS),
                ) {
                    let a = envelope.center(lo + y * envelope.nx);
                    let b = envelope.center(hi + y * envelope.nx);
                    for yy in [a.y - envelope.h / 2., a.y + envelope.h / 2.] {
                        points.push(Point2Dto::new(a.x - envelope.h / 2., yy));
                        points.push(Point2Dto::new(b.x + envelope.h / 2., yy));
                    }
                }
            }
            if points.is_empty() {
                return Ok(0);
            }
            Self::from_points(points, 0.)
                .ok_or_else(|| CamPlanError("Remaining stock footprint is unresolved.".into()))?
                .hull
        } else {
            stock_footprint(setup)
        };
        // Distance to a convex set is convex, so the maximum over a
        // containing polygon occurs at a vertex. The cylinder polygon is
        // circumscribed, never inscribed: no real stock escapes this bound.
        let mut bound = footprint
            .iter()
            .map(|&c| self.distance(c))
            .fold(0.0, f64::max)
            + CLEARANCE_GUARD;
        if bound <= self.offset + EPS {
            return Ok(0);
        }
        // All axial sections advance by e between equal-offset loops. The
        // smallest radius sees the largest angle: e <= f*(1-cos(phi_R)).
        let advance = p.optimal_load * (floor_r / r);
        let passes = ((bound - self.offset) / advance).ceil() as usize;
        if passes > MAX_PASSES {
            return Err(CamPlanError("High Speed Roughing exterior exceeds its pass budget; split the stock or increase optimal load.".into()));
        }
        super::ensure_program_budget(
            builder.commands.len(),
            passes.saturating_mul(2 * self.hull.len() + 8),
            "High Speed Roughing exterior",
        )?;
        work.spend(passes.saturating_mul(self.hull.len() * 8), 1)?;
        let preferred_edge = if let Some(hint) = builder
            .linking
            .as_ref()
            .and_then(|l| l.entry_positions.first())
        {
            (0..self.hull.len())
                .min_by(|&a, &b| {
                    point_segment(*hint, self.hull[a], self.hull[(a + 1) % self.hull.len()])
                        .total_cmp(&point_segment(
                            *hint,
                            self.hull[b],
                            self.hull[(b + 1) % self.hull.len()],
                        ))
                })
                .unwrap()
        } else {
            (0..self.hull.len())
                .max_by(|&a, &b| {
                    dist(self.hull[a], self.hull[(a + 1) % self.hull.len()])
                        .total_cmp(&dist(self.hull[b], self.hull[(b + 1) % self.hull.len()]))
                })
                .unwrap()
        };
        let air_margin = builder
            .linking
            .as_ref()
            .map_or(1.0, |l| l.safe_distance)
            .max(CLEARANCE_GUARD);
        // Enclose each user-configured lead in a disk at its tangent anchor.
        // This includes vertical leads' XY projection as well as the arc.
        let lead_reach = builder.linking.as_ref().map_or(0.0, |l| {
            [&l.lead_in, &l.exit()]
                .into_iter()
                .filter(|lead| lead.enabled)
                .map(|lead| {
                    2.0 * lead.horizontal_radius + lead.linear_distance + lead.vertical_radius
                })
                .fold(0.0, f64::max)
        });
        let offset =
            |v: Point2Dto, n: Point2Dto, d: f64| Point2Dto::new(v.x + n.x * d, v.y + n.y * d);
        for _ in 0..passes {
            work.spend(
                (footprint.len() + self.hull.len()) * 220 + footprint.len() * self.hull.len(),
                0,
            )?;
            let stock = Self::from_points(footprint.clone(), 0.0)
                .expect("remaining stock footprint is a bounded convex polygon");
            let next = (bound - advance).max(self.offset);
            let d = r + next;
            if d + EPS < p.minimum_cutting_radius {
                return Err(CamPlanError(
                    "High Speed Roughing exterior cannot meet minimum cutting radius.".into(),
                ));
            }
            let anchors = |edge: usize| {
                let a = self.hull[edge];
                let b = self.hull[(edge + 1) % self.hull.len()];
                let n = self.normals[edge];
                let tangent = Point2Dto::new(n.y, -n.x); // clockwise external climb (M3)
                let start = offset(Point2Dto::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5), n, d);
                let move_along = |amount: f64| {
                    Point2Dto::new(start.x + tangent.x * amount, start.y + tangent.y * amount)
                };
                let anchor = |direction: f64| {
                    let clear = |length: f64| {
                        let candidate = move_along(direction * length);
                        stock.distance(candidate) > r + air_margin + lead_reach
                            && self.distance(candidate)
                                > r + self.offset + lead_reach + CLEARANCE_GUARD
                    };
                    if clear(air_margin) {
                        return move_along(direction * air_margin);
                    }
                    let mut hi = self
                        .hull
                        .iter()
                        .map(|&v| dist(start, v))
                        .chain(footprint.iter().map(|&v| dist(start, v)))
                        .fold(0.0, f64::max)
                        + 2.0 * (r + self.offset + air_margin + lead_reach);
                    let mut lo = 0.0;
                    for _ in 0..48 {
                        let mid = (lo + hi) * 0.5;
                        if clear(mid) {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    move_along(direction * (hi + CLEARANCE_GUARD))
                };
                (start, anchor(-1.0), anchor(1.0))
            };
            // Move the seam toward the current tool instead of crossing the
            // part to the same fixed entry on every loop. User entry hints
            // remain authoritative; clearance is checked on the actual link.
            let choose_nearby = builder
                .linking
                .as_ref()
                .is_some_and(|l| l.keep_tool_down && l.entry_positions.is_empty())
                && builder
                    .position
                    .is_some_and(|p| p.z >= depth - EPS && p.z < builder.retract_z);
            let edge = if choose_nearby {
                work.spend(
                    (footprint.len() + self.hull.len()) * 220 * self.hull.len(),
                    0,
                )?;
                let from = builder.position.unwrap();
                let from = Point2Dto::new(from.x, from.y);
                (0..self.hull.len())
                    .map(|edge| (edge, dist(from, anchors(edge).1)))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .unwrap()
                    .0
            } else {
                preferred_edge
            };
            let (start, entry, exit) = anchors(edge);
            // Axial entry/retraction and complete lead envelopes clear
            // the conservative remaining stock by a full cutter radius.
            let advanced = if builder.linking.is_some() {
                Some(super::super::linking_planner::air_leads_against_stock(
                    builder, entry, exit, r, &footprint,
                )?)
            } else {
                None
            };
            if let Some((leads, tin, _)) = &advanced {
                let link = builder.linking.clone().unwrap();
                let vertical_radius = if link.lead_in.enabled {
                    link.lead_in.vertical_radius
                } else {
                    0.0
                };
                let vertical = super::super::linking_planner::vertical_points(
                    leads.start,
                    *tin,
                    depth,
                    vertical_radius,
                    true,
                )?;
                if depth + vertical_radius > builder.feed_height_z + EPS {
                    return Err(CamPlanError("Vertical lead-in radius reaches above Feed Height. Reduce the radius or raise Feed Height.".into()));
                }
                if self.link_through_cleared_stock(builder, &stock, vertical[0], depth, r) {
                    super::ensure_program_budget(
                        builder.commands.len(),
                        vertical.len(),
                        "roughing vertical entry",
                    )?;
                    for point in vertical.into_iter().skip(1) {
                        builder.linear(point, link.lead_in_feed);
                    }
                } else {
                    super::super::linking_planner::entry(
                        builder,
                        leads.start,
                        *tin,
                        depth,
                        vertical_radius,
                        plunge,
                        link.lead_in_feed,
                    )?;
                }
                builder.linear(
                    Point3Dto::new(leads.line_end.x, leads.line_end.y, depth),
                    link.lead_in_feed,
                );
                if let Some(arc) = &leads.start_arc {
                    builder.circular(
                        Point3Dto::new(entry.x, entry.y, depth),
                        arc.center,
                        arc.clockwise,
                        link.lead_in_feed,
                    );
                }
            } else {
                builder.approach(entry, depth, plunge);
            }
            builder.linear(Point3Dto::new(start.x, start.y, depth), feed);
            for step in 0..self.hull.len() {
                let i = (edge + self.hull.len() - step) % self.hull.len();
                let prev = (i + self.hull.len() - 1) % self.hull.len();
                let v = self.hull[i];
                let from = offset(v, self.normals[i], d);
                let to = offset(v, self.normals[prev], d);
                builder.linear(Point3Dto::new(from.x, from.y, depth), feed);
                // Each convex turn is < pi. Very nearly collinear hull
                // edges still retain their tiny outward arc, not a shortcut.
                builder.circular(Point3Dto::new(to.x, to.y, depth), v, true, feed);
            }
            builder.linear(Point3Dto::new(start.x, start.y, depth), feed);
            builder.linear(Point3Dto::new(exit.x, exit.y, depth), p.linking_feed);
            if let Some((leads, _, tout)) = &advanced {
                let link = builder.linking.clone().unwrap();
                if let Some(arc) = &leads.end_arc {
                    builder.circular(
                        Point3Dto::new(arc.arc_end.x, arc.arc_end.y, depth),
                        arc.center,
                        arc.clockwise,
                        link.lead_out_feed,
                    );
                }
                builder.linear(
                    Point3Dto::new(leads.end.x, leads.end.y, depth),
                    link.lead_out_feed,
                );
                super::super::linking_planner::exit(
                    builder,
                    leads.end,
                    *tout,
                    depth,
                    if link.exit().enabled {
                        link.exit().vertical_radius
                    } else {
                        0.0
                    },
                    link.lead_out_feed,
                )?;
            }
            if builder.linking.as_ref().is_none_or(|l| {
                !l.keep_tool_down
                    && l.retraction_policy == crate::linking::CamRetractionPolicy::Full
            }) {
                builder.retract_to_clearance();
            }
            // Every completed loop cleared the whole axial section outside
            // this certificate. Include the bull-nose floor residue, then
            // intersect its supporting half-planes with the incoming stock.
            // Subsequent descents/leads may use that proved empty space.
            let trimmed = self.remaining_footprint(&footprint, next + r - floor_r);
            if Self::from_points(trimmed.clone(), 0.0).is_some() {
                footprint = trimmed;
            }
            bound = next;
        }
        Ok(passes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    fn square() -> ConvexStock {
        ConvexStock::from_points(
            vec![
                Point2Dto::new(0.0, 0.0),
                Point2Dto::new(4.0, 0.0),
                Point2Dto::new(4.0, 4.0),
                Point2Dto::new(0.0, 4.0),
            ],
            0.2,
        )
        .unwrap()
    }
    #[test]
    fn cleared_links_require_whole_capsule_clearance_and_certified_depth() {
        let target = square();
        let builder = || {
            let mut b = ProgramBuilder::new();
            b.position = Some(Point3Dto::new(-2.0, -2.0, -1.0));
            b.retract_z = 3.0;
            b.feed_height_z = 1.0;
            b.linking = Some(crate::CamLinkingDto {
                keep_tool_down: true,
                maximum_stay_down: 20.0,
                safe_distance: 0.1,
                minimum_clearance: 0.25,
                lift_height: 0.1,
                ..Default::default()
            });
            b
        };
        let to = Point3Dto::new(-2.0, 6.0, -1.0);
        let mut b = builder();
        assert!(target.link_through_cleared_stock(&mut b, &target, to, -1.0, 1.0));
        assert_eq!(b.position, Some(to));
        assert!(b
            .commands
            .iter()
            .all(|c| matches!(c, crate::CamCommandDto::Linear { .. })));
        // Endpoints alone are insufficient: this segment crosses the stock.
        let mut b = builder();
        assert!(!target.link_through_cleared_stock(
            &mut b,
            &target,
            Point3Dto::new(6.0, 6.0, -1.0),
            -1.0,
            1.0
        ));
        assert!(b.commands.is_empty());
        for (from_z, to_z) in [(-1.1, -1.0), (-1.0, -1.1)] {
            let mut b = builder();
            b.position.as_mut().unwrap().z = from_z;
            assert!(!target.link_through_cleared_stock(
                &mut b,
                &target,
                Point3Dto::new(to.x, to.y, to_z),
                -1.0,
                1.0
            ));
            assert!(b.commands.is_empty());
        }
        // A bull-nose floor residue is still material, even if target clears.
        let mut residue = square();
        residue.offset = 0.8;
        assert!(!target.link_through_cleared_stock(&mut builder(), &residue, to, -1.0, 1.0));
        for constraint in 0..4 {
            let mut b = builder();
            let l = b.linking.as_mut().unwrap();
            match constraint {
                0 => l.keep_tool_down = false,
                1 => l.maximum_stay_down = 2.0,
                2 => l.lift_height = 3.0,
                _ => l.minimum_clearance = 2.0,
            }
            assert!(!target.link_through_cleared_stock(&mut b, &target, to, -1.0, 1.0));
            assert!(b.commands.is_empty());
        }
    }

    #[test]
    fn stock_certificate_checks_whole_segments_and_round_corners() {
        let s = square();
        assert!(s.capsule_clear(Point2Dto::new(-2.0, -2.0), Point2Dto::new(-2.0, 6.0), 1.0));
        assert!(!s.capsule_clear(Point2Dto::new(-2.0, 2.0), Point2Dto::new(6.0, 2.0), 1.0));
        assert!(!s.point_clear(Point2Dto::new(2.0, 2.0)));
        assert!(!s.point_clear(Point2Dto::new(-0.1, -0.1)));
        assert!(s.point_clear(Point2Dto::new(-0.15, -0.15)));
    }
    #[test]
    fn remaining_footprint_contains_rounded_floor_stock() {
        let target = square();
        let incoming = vec![
            Point2Dto::new(-10.0, -10.0),
            Point2Dto::new(14.0, -10.0),
            Point2Dto::new(14.0, 14.0),
            Point2Dto::new(-10.0, 14.0),
        ];
        // Include a 1 mm corner-tool floor residue outside the 0.2 allowance.
        let offset = 1.2;
        let remaining =
            ConvexStock::from_points(target.remaining_footprint(&incoming, offset), 0.0).unwrap();
        for x in -30..=70 {
            for y in -30..=70 {
                let p = Point2Dto::new(x as f64 * 0.1, y as f64 * 0.1);
                if target.distance(p) <= offset {
                    assert!(remaining.inside(p));
                }
            }
        }
        assert!(!remaining.inside(Point2Dto::new(9.0, 2.0)));
    }

    #[test]
    fn offset_engagement_bound_includes_only_advancing_material() {
        for r in [1.0_f64, 6.0, 12.0] {
            for fraction in [0.01_f64, 0.2, 0.8, 1.5, 2.0] {
                let e = r * fraction;
                let phi = (1.0 - fraction).acos();
                // At every C1 offset station, n=(1,0), t=(0,1) after rotation.
                // Possible material lies x <= -(R-e); backward y<0 was swept.
                let mut engaged = 0;
                const N: usize = 100_000;
                for i in 0..N {
                    let theta = 2.0 * PI * (i as f64 + 0.5) / N as f64;
                    if r * theta.cos() <= -(r - e) && theta.sin() >= 0.0 {
                        engaged += 1;
                    }
                }
                assert!((engaged as f64 * 2.0 * PI / N as f64 - phi).abs() < 2.0 * PI / N as f64);
            }
        }
    }

    #[test]
    fn corner_exterior_contact_and_advance_bound_every_cutter_section() {
        let mut floor = square();
        floor.offset = 2.2; // nominal allowance .2 plus R-f = 2
        for x in [-2., 2., 6., 9.] {
            for y in [-2., 2., 6., 9.] {
                let c = Point2Dto::new(x, y);
                let mut intervals = vec![(0., 2. * PI)];
                floor.clip_contact(c, 4., &mut intervals);
                let upper = super::super::angle_with_guard(&intervals);
                let n = 2048;
                let count = (0..n)
                    .filter(|&i| {
                        (0..=32).any(|j| {
                            let delta = 2. * j as f64 / 32.;
                            let angle = 2. * PI * (i as f64 + 0.5) / n as f64;
                            let p = Point2Dto::new(
                                c.x + (4. + delta) * angle.cos(),
                                c.y + (4. + delta) * angle.sin(),
                            );
                            floor.distance(p) <= floor.offset - delta
                        })
                    })
                    .count();
                assert!(upper + 0.005 >= count as f64 * 2. * PI / n as f64);
            }
        }
        for f in [0.5_f64, 2., 4., 6.] {
            let (r, ae) = (6., 1.2);
            let advance = ae * f / r;
            for j in 0..=100 {
                let s = f + (r - f) * j as f64 / 100.;
                assert!((1. - advance / s).acos() <= (1. - ae / r).acos() + EPS);
            }
        }
    }
}
