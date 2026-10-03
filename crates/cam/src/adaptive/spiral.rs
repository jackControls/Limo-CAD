//! Continuous exterior clearing of a circular protected section.
//!
//! Alternate tangent half-circles about C and C + (e/2, 0). Their radii
//! decrease by e/2 per half turn. For every cutter section of radius s, the
//! preceding half-turns bound remaining material by a disk whose radius is
//! at most (current path radius - s + e) about the current arc center.
//! The advancing half of the cutter therefore sees <= acos(1 - e/s).
//! Choosing e <= Ae * floor_radius / outer_radius bounds every axial section.
//!
//! After the first two half-turns, the two half-plane stock disks are nested:
//! their centers are e/2 apart and their radii differ by e/2. The larger disk
//! is the certificate for the next half-turn. No intermediate arc is a
//! separate pass. A final half-circle clears the last asymmetric remainder.
//! For a target-free cap, protected = floor_radius - outer_radius: the last
//! two half-circles are centered on C with radius floor_radius. Their swept
//! union includes the center, so no artificial island or cleanup lap remains.
//!
//! Starting the spiral with the cutter just outside the stock spends its
//! first turn ramping engagement up through air. When shorter, cut one full
//! ring at the deepest radius fresh stock permits instead, then spiral in
//! from that ring. A ring at path radius P removes stock - (P - s) at cutter
//! section s; keeping that <= Ae * s / r bounds its straight-wall engagement
//! angle by acos(1 - Ae/r) at every section, a convex disk only less. The
//! ring then certifies the spiral's first turn exactly as a preceding turn
//! would. Its tangent entry from air is sampled against the same angle.

use super::super::linking_planner;
use super::*;

fn shift(c: Point2Dto, u: Point2Dto, d: f64) -> Point2Dto {
    Point2Dto::new(c.x + u.x * d, c.y + u.y * d)
}

pub(super) fn clear(
    builder: &mut ProgramBuilder,
    footprint: &[Point2Dto],
    center: Point2Dto,
    protected: f64,
    r: f64,
    floor_r: f64,
    depth: f64,
    p: &CamAdaptiveParametersDto,
    feed: f64,
    plunge: f64,
    work: &mut Work,
) -> Result<usize, CamPlanError> {
    let stock = footprint
        .iter()
        .map(|&v| dist(v, center))
        .fold(0.0, f64::max)
        + 1e-4;
    if stock <= protected + EPS {
        return Ok(0);
    }
    let e = p.optimal_load * floor_r / r;
    let air = Plan::new(stock + r, protected + r, e, false);
    let k = 1. - p.optimal_load / r;
    let ring = Plan::new(
        (stock + if k >= 0. { r * k } else { floor_r * k }).max(protected + r),
        protected + r,
        e,
        true,
    );
    let mut plan = if ring.length() < air.length() - EPS {
        ring
    } else {
        air
    };
    if plan.turns > 2048 {
        return Err(CamPlanError("High Speed Roughing spiral exceeds its turn budget; split the stock or increase optimal load.".into()));
    }
    if protected + r + EPS < p.minimum_cutting_radius {
        return Err(CamPlanError(
            "High Speed Roughing exterior cannot meet minimum cutting radius.".into(),
        ));
    }
    ensure_program_budget(
        builder.commands.len(),
        2 * plan.turns + 128,
        "roughing spiral",
    )?;
    work.spend(plan.turns * 16 + footprint.len(), 1)?;
    let u = builder
        .linking
        .as_ref()
        .and_then(|l| l.entry_positions.first())
        .filter(|&&v| dist(v, center) > EPS)
        .map_or(Point2Dto::new(1.0, 0.0), |&v| {
            let d = dist(v, center);
            Point2Dto::new((v.x - center.x) / d, (v.y - center.y) / d)
        });
    let tangent = Point2Dto::new(u.y, -u.x);
    let margin = builder
        .linking
        .as_ref()
        .map_or(1.0, |l| l.safe_distance)
        .max(1e-4);
    let reach = builder.linking.as_ref().map_or(0.0, |l| {
        [&l.lead_in, &l.exit()]
            .into_iter()
            .filter(|s| s.enabled)
            .map(|s| 2.0 * s.horizontal_radius + s.linear_distance + s.vertical_radius)
            .fold(0.0, f64::max)
    });
    // A full disk about each air anchor contains the configured lead plus
    // its vertical projection and the dummy tangent segment used below.
    let lead_length = |path_radius: f64, bound: f64| {
        ((bound + r + margin + reach + 1.0).powi(2) - path_radius.powi(2))
            .max(0.0)
            .sqrt()
            + 1e-4
    };
    let residue = (protected + r - floor_r).max(0.0);
    let incoming = footprint.to_vec();
    let remaining = if residue <= EPS {
        Vec::new()
    } else {
        circle_polygon(center, residue)
    };
    let mut fit_entry = |builder: &mut ProgramBuilder, plan: &Plan| {
        let start = shift(center, u, plan.start);
        let mut distance = lead_length(plan.start, stock);
        if builder.linking.is_some() {
            work.spend(incoming.len() * 32 * 16, 0)?;
            distance = linking_planner::fit_air_lead_distance(
                builder, start, tangent, r, &incoming, true, distance,
            )?;
        }
        Ok::<_, CamPlanError>(distance)
    };
    let mut entry_distance = fit_entry(builder, &plan)?;
    if plan.ring {
        // The tangent entry turns slightly toward the stock center, so its
        // leading half can see more than the ring itself. Sample it.
        let phi = (1. - p.optimal_load / r).clamp(-1., 1.).acos();
        let start = shift(center, u, plan.start);
        if !entry_within_engagement(
            center,
            stock,
            start,
            tangent,
            entry_distance,
            r,
            floor_r,
            phi,
        ) {
            plan = air;
            entry_distance = fit_entry(builder, &plan)?;
        }
    }
    let start = shift(center, u, plan.start);
    let single = plan.turns == 0;
    let finish = shift(
        center,
        u,
        if single {
            protected + r
        } else {
            -(protected + r)
        },
    );
    let exit_tangent = if single {
        tangent
    } else {
        Point2Dto::new(-tangent.x, -tangent.y)
    };
    let mut exit_distance = lead_length(protected + r, residue);
    if builder.linking.is_some() {
        work.spend(remaining.len() * 32 * 16, 0)?;
        exit_distance = linking_planner::fit_air_lead_distance(
            builder,
            finish,
            exit_tangent,
            r,
            &remaining,
            false,
            exit_distance,
        )?;
    }
    let entry = shift(start, tangent, -entry_distance);
    let exit = shift(finish, exit_tangent, exit_distance);
    // Emit configured air leads only at the boundaries of this continuous
    // cutting pass, independently of Keep tool down / Retraction Policy.
    if let Some(link) = builder.linking.clone() {
        let (leads, tin, _) = linking_planner::air_leads_against_stock(
            builder,
            entry,
            shift(entry, tangent, 1.0),
            r,
            &incoming,
        )?;
        linking_planner::entry(
            builder,
            leads.start,
            tin,
            depth,
            if link.lead_in.enabled {
                link.lead_in.vertical_radius
            } else {
                0.0
            },
            plunge,
            link.lead_in_feed,
        )?;
        builder.linear(
            Point3Dto::new(leads.line_end.x, leads.line_end.y, depth),
            link.lead_in_feed,
        );
        if let Some(arc) = leads.start_arc {
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
    if single || plan.ring {
        // One complete ring about C at the start radius.
        let opposite = shift(center, u, -plan.start);
        builder.circular(
            Point3Dto::new(opposite.x, opposite.y, depth),
            center,
            true,
            feed,
        );
        builder.circular(Point3Dto::new(start.x, start.y, depth), center, true, feed);
    }
    if !single {
        let alternate = shift(center, u, plan.advance * 0.5);
        for half in 1..=2 * plan.turns {
            let c = if half % 2 == 1 { alternate } else { center };
            let radius = plan.start - half as f64 * plan.advance * 0.5;
            let end = shift(c, u, if half % 2 == 1 { -radius } else { radius });
            builder.circular(Point3Dto::new(end.x, end.y, depth), c, true, feed);
        }
        builder.circular(
            Point3Dto::new(finish.x, finish.y, depth),
            center,
            true,
            feed,
        );
    }
    let link_feed = builder
        .linking
        .as_ref()
        .map_or(p.linking_feed, |l| l.no_engagement_feed);
    builder.linear(Point3Dto::new(exit.x, exit.y, depth), link_feed);
    if let Some(link) = builder.linking.clone() {
        let (leads, _, tout) = linking_planner::air_leads_against_stock(
            builder,
            shift(exit, exit_tangent, -1.0),
            exit,
            r,
            &remaining,
        )?;
        if let Some(arc) = leads.end_arc {
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
        linking_planner::exit(
            builder,
            leads.end,
            tout,
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
        !l.keep_tool_down && l.retraction_policy == crate::linking::CamRetractionPolicy::Full
    }) {
        builder.retract_to_clearance();
    }
    Ok(1)
}

/// A continuous pass: an optional first ring at `start`, then `turns` spiral
/// turns of `advance` each down to the protected path radius, then one
/// closing half-circle. No turns means one ring at the protected radius.
#[derive(Clone, Copy)]
struct Plan {
    start: f64,
    turns: usize,
    advance: f64,
    ring: bool,
}

impl Plan {
    fn new(start: f64, finish: f64, pitch: f64, ring: bool) -> Self {
        if start <= finish + EPS {
            return Self {
                start: finish,
                turns: 0,
                advance: 0.,
                ring: false,
            };
        }
        let mut turns = ((start - finish) / pitch).ceil().max(1.) as usize;
        if !ring && turns == 1 {
            // A band no wider than Ae needs just one circle, not a spiral
            // plus cleanup half-turn. This is the common case on narrow
            // shoulders.
            turns = 0;
            return Self {
                start: finish,
                turns,
                advance: 0.,
                ring: false,
            };
        }
        Self {
            start,
            turns,
            advance: (start - finish) / turns as f64,
            ring,
        }
    }

    /// Cutting length of the pass, excluding leads.
    fn length(&self) -> f64 {
        let finish = self.start - self.turns as f64 * self.advance;
        if self.turns == 0 {
            return TAU * finish;
        }
        let spiral = (1..=2 * self.turns)
            .map(|half| PI * (self.start - half as f64 * self.advance * 0.5))
            .sum::<f64>();
        spiral + PI * finish + if self.ring { TAU * self.start } else { 0. }
    }
}

/// The leading half of each sampled cutter section along a straight tangent
/// entry into a ring sees at most `phi` of fresh stock (a disk of radius
/// `stock` about `center`, which contains the convex stock footprint).
#[allow(clippy::too_many_arguments)]
fn entry_within_engagement(
    center: Point2Dto,
    stock: f64,
    start: Point2Dto,
    tangent: Point2Dto,
    length: f64,
    r: f64,
    floor_r: f64,
    phi: f64,
) -> bool {
    if phi >= PI - 1e-9 {
        return true;
    }
    let heading = tangent.y.atan2(tangent.x);
    (0..=64).all(|i| {
        let x = shift(start, tangent, -length * i as f64 / 64.);
        let d = dist(x, center);
        let toward = (center.y - x.y).atan2(center.x - x.x);
        (0..=4).all(|j| {
            let s = floor_r + (r - floor_r) * j as f64 / 4.;
            let contact = if d + s <= stock {
                PI
            } else if d >= s + stock || s <= EPS {
                0.
            } else {
                // Arc of the section inside the disk, clipped to the half
                // facing the direction of travel.
                let alpha = ((s * s + d * d - stock * stock) / (2. * s * d))
                    .clamp(-1., 1.)
                    .acos();
                let mid = (toward - heading + PI).rem_euclid(TAU) - PI;
                (-1..=1)
                    .map(|k| {
                        let shifted = mid + k as f64 * TAU;
                        ((shifted + alpha).min(PI / 2.) - (shifted - alpha).max(-PI / 2.)).max(0.)
                    })
                    .sum()
            };
            contact <= phi + 1e-9
        })
    })
}

fn circle_polygon(c: Point2Dto, radius: f64) -> Vec<Point2Dto> {
    const N: usize = 128;
    let outer = radius / (PI / N as f64).cos();
    (0..N)
        .map(|i| polar(c, outer, TAU * i as f64 / N as f64))
        .collect()
}
