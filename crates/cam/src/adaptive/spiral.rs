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
//!
//! Loops (the pattern Adaptive Clearing in Fusion produces on round stock):
//! complete concentric rings at most e apart, each joined to the next by one
//! filleted straight crossover tangent to the inner ring, instead of
//! half-turn spiral transitions. A ring at most e inside a completed ring is
//! certified like a spiral turn. Each crossover cuts into the disk the
//! completed ring left (radius ring - s at section s) and is sampled against
//! the engagement angle. A target-free cap ends its last ring, at most two
//! flat-land radii, with a straight crossover into the center: the flat land
//! there covers the remaining disk.

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
    let phi = (1. - p.optimal_load / r).clamp(-1., 1.).acos();
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
    // Full loops from the deepest first ring, when every crossover is within
    // the engagement limit and the whole pass is shorter.
    let first_ring = (stock + if k >= 0. { r * k } else { floor_r * k }).max(protected + r);
    let loops = Loops::plan(
        center,
        u,
        first_ring,
        protected + r,
        protected + r <= floor_r + EPS,
        e,
        floor_r,
        r,
        p.minimum_cutting_radius,
        phi,
    )
    .filter(|loops| loops.length < plan.length() - EPS);
    if loops.is_some() {
        plan = Plan {
            start: first_ring,
            turns: 0,
            advance: 0.,
            ring: true,
        };
    }
    let mut entry_distance = fit_entry(builder, &plan)?;
    let mut loops = loops;
    if plan.ring {
        // The tangent entry turns slightly toward the stock center, so its
        // leading half can see more than the ring itself. Sample it.
        let start = shift(center, u, plan.start);
        let samples = (0..=64).map(|i| {
            (
                shift(start, tangent, -entry_distance * i as f64 / 64.),
                tangent,
            )
        });
        if !within_engagement(samples, center, |_| stock, r, floor_r, phi) {
            plan = air;
            loops = None;
            entry_distance = fit_entry(builder, &plan)?;
        }
    }
    let start = shift(center, u, plan.start);
    let single = plan.turns == 0 && loops.is_none();
    let (finish, exit_tangent) = if let Some(loops) = &loops {
        (loops.finish, loops.exit)
    } else if single {
        (shift(center, u, protected + r), tangent)
    } else {
        (
            shift(center, u, -(protected + r)),
            Point2Dto::new(-tangent.x, -tangent.y),
        )
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
    if let Some(loops) = &loops {
        ensure_program_budget(
            builder.commands.len(),
            loops.moves.len() + 64,
            "roughing loops",
        )?;
        for &(end, arc) in &loops.moves {
            let end = Point3Dto::new(end.x, end.y, depth);
            match arc {
                Some(c) => builder.circular(end, c, true, feed),
                None => builder.linear(end, feed),
            }
        }
    } else if single || plan.ring {
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
    if !single && loops.is_none() {
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

/// Complete rings joined by filleted straight crossovers; see module docs.
struct Loops {
    /// Clockwise arcs (end, center) and lines (end, None) after the start.
    moves: Vec<(Point2Dto, Option<Point2Dto>)>,
    finish: Point2Dto,
    exit: Point2Dto,
    length: f64,
}

impl Loops {
    /// Rings step inward from `first` by at most `pitch` down to `last`, or,
    /// for a target-free cap, to at most two flat-land radii before a
    /// crossover into the center. `None` when a fillet of the minimum
    /// cutting radius does not fit or a crossover exceeds `phi`.
    #[allow(clippy::too_many_arguments)]
    fn plan(
        center: Point2Dto,
        u: Point2Dto,
        first: f64,
        last: f64,
        cap: bool,
        pitch: f64,
        floor_r: f64,
        r: f64,
        fillet: f64,
        phi: f64,
    ) -> Option<Self> {
        let fillet = fillet.max(1e-3);
        let mut rings = vec![first];
        loop {
            let ring = *rings.last().unwrap();
            let next = if cap {
                if ring <= 2. * floor_r + EPS {
                    break;
                }
                // The center crossover's fillet needs ring >= 2 fillets.
                (ring - pitch).max(2. * fillet + 1e-3)
            } else {
                if ring <= last + EPS {
                    break;
                }
                (ring - pitch).max(last)
            };
            if next >= ring - EPS || rings.len() > 2048 {
                return None;
            }
            rings.push(next);
        }
        if cap && *rings.last().unwrap() < 2. * fillet + 1e-3 - EPS {
            return None;
        }
        let rotate = |v: Point2Dto, angle: f64| {
            let (sin, cos) = angle.sin_cos();
            Point2Dto::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
        };
        let add = |a: Point2Dto, b: Point2Dto| Point2Dto::new(a.x + b.x, a.y + b.y);
        let mut moves = Vec::new();
        let mut length = 0.;
        let mut a = u;
        let mut finish = shift(center, u, first);
        let mut exit = Point2Dto::new(u.y, -u.x);
        for (i, &ring) in rings.iter().enumerate() {
            moves.push((shift(center, a, -ring), Some(center)));
            moves.push((shift(center, a, ring), Some(center)));
            length += TAU * ring;
            finish = shift(center, a, ring);
            exit = Point2Dto::new(a.y, -a.x);
            let next = match rings.get(i + 1) {
                Some(&next) => next,
                None if cap => 0.,
                None => break,
            };
            // Canonical frame: the crossover ends at (next, 0) heading -y,
            // tangent to the inner ring (or through the center when next is
            // 0); its fillet is tangent to this ring.
            let f = fillet;
            if f > (ring + next) * 0.5 - EPS {
                return None;
            }
            let m = ((ring - f).powi(2) - (next - f).powi(2)).max(0.).sqrt();
            let theta = m.atan2(next - f);
            let turn = a.y.atan2(a.x) - theta;
            let to = |v: Point2Dto| add(center, rotate(v, turn));
            let fillet_center = to(Point2Dto::new(next - f, m));
            let line_start = to(Point2Dto::new(next, m));
            let end = to(Point2Dto::new(next, 0.));
            let heading = rotate(Point2Dto::new(0., -1.), turn);
            // The completed ring leaves a disk of radius ring - s at section s.
            let samples = (0..=32)
                .map(|j| {
                    let t = theta * (1. - j as f64 / 32.);
                    let w = Point2Dto::new(t.cos(), t.sin());
                    (
                        to(Point2Dto::new(next - f + f * w.x, m + f * w.y)),
                        rotate(Point2Dto::new(w.y, -w.x), turn),
                    )
                })
                .chain((0..=32).map(|j| (shift(line_start, heading, m * j as f64 / 32.), heading)));
            if !within_engagement(samples, center, |s| ring - s, r, floor_r, phi) {
                return None;
            }
            moves.push((line_start, Some(fillet_center)));
            moves.push((end, None));
            length += f * theta + m;
            finish = end;
            exit = heading;
            if next > EPS {
                a = rotate(Point2Dto::new(1., 0.), turn);
            }
        }
        Some(Self {
            moves,
            finish,
            exit,
            length,
        })
    }
}

/// The leading half of each sampled cutter section (radius floor_r..r),
/// at each (position, unit heading), sees at most `phi` of material inside a
/// disk about `center` whose radius at section s is `material(s)`.
fn within_engagement(
    samples: impl IntoIterator<Item = (Point2Dto, Point2Dto)>,
    center: Point2Dto,
    material: impl Fn(f64) -> f64,
    r: f64,
    floor_r: f64,
    phi: f64,
) -> bool {
    if phi >= PI - 1e-9 {
        return true;
    }
    samples.into_iter().all(|(x, direction)| {
        let heading = direction.y.atan2(direction.x);
        let d = dist(x, center);
        let toward = (center.y - x.y).atan2(center.x - x.x);
        (0..=4).all(|j| {
            let s = floor_r + (r - floor_r) * j as f64 / 4.;
            let radius = material(s);
            let contact = if radius <= 0. || d >= s + radius || s <= EPS {
                0.
            } else if d + s <= radius {
                PI
            } else if d + radius <= s {
                // Remaining material lies wholly inside this section.
                0.
            } else {
                // Arc of the section inside the disk, clipped to the half
                // facing the direction of travel.
                let alpha = ((s * s + d * d - radius * radius) / (2. * s * d))
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
