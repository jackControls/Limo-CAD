//! Existing revision-cloud scallops over saved paper vertices. No document edits.
use super::geometry::{add, arc, length, midpoint, normal, scale, sub, unit, P};

pub const STROKE_MM: f64 = 0.45;
pub const TEXT_HEIGHT_MM: f64 = 3.2;
pub const COLOR: &str = "#c43b4d";
pub const MAX_ARC_POINTS: u64 = 513;

/// Preflight is O(vertices), allocation-free. Callers must charge `work()` to
/// their sheet budget before iterating arcs or allocating presentation vectors.
pub struct Cloud<'a> {
    points: &'a [P],
    count: u64,
    pub label: P,
}
#[derive(Clone, Copy, Debug)]
pub struct Scallop {
    pub center: P,
    pub radius: f64,
    pub start: f64,
    pub sweep: f64,
}
impl Scallop {
    pub fn points(self) -> Vec<P> {
        arc(self.center, self.radius, self.start, self.sweep)
    }
}
impl<'a> Cloud<'a> {
    pub fn new(points: &'a [P]) -> Result<Self, String> {
        if !(3..=4096).contains(&points.len()) || points.iter().flatten().any(|n| !n.is_finite()) {
            return Err("Revision cloud contains invalid paper vertices".into());
        }
        let mut count = 0_u64;
        let mut min = [f64::INFINITY; 2];
        for (i, a) in points.iter().enumerate() {
            min = [min[0].min(a[0]), min[1].min(a[1])];
            let distance = length(sub(points[(i + 1) % points.len()], *a));
            if !distance.is_finite() {
                return Err("Revision cloud contains non-finite geometry".into());
            }
            let steps = (distance / 5.).ceil();
            if steps >= u64::MAX as f64 / MAX_ARC_POINTS as f64 {
                return Err("Revision cloud exceeds the generation work limit".into());
            }
            count = count
                .checked_add(steps as u64)
                .ok_or("Revision cloud work limit overflow")?;
        }
        count
            .checked_mul(MAX_ARC_POINTS)
            .ok_or("Revision cloud work limit overflow")?;
        let label = add(min, [0., -2.]);
        if label.iter().any(|n| !n.is_finite()) {
            return Err("Revision cloud contains non-finite label coordinates".into());
        }
        Ok(Self {
            points,
            count,
            label,
        })
    }
    pub fn work(&self) -> u64 {
        self.count * MAX_ARC_POINTS
    }
    pub fn arcs(&self) -> impl Iterator<Item = Scallop> + '_ {
        self.points.iter().enumerate().flat_map(|(index, &start)| {
            let end = self.points[(index + 1) % self.points.len()];
            let delta = sub(end, start);
            let direction = unit(delta);
            let distance = length(delta);
            // Preserve native handling of repeated or sub-tolerance edges.
            let count = if direction.is_some() {
                (distance / 5.).ceil().max(1.) as u64
            } else {
                0
            };
            let step = distance / count.max(1) as f64;
            let radius = (step * 0.58).max(1.4);
            (0..count).map(move |i| {
                let direction = direction.unwrap();
                let a = add(start, scale(direction, i as f64 * step));
                let b = add(start, scale(direction, (i + 1) as f64 * step));
                let center = add(
                    midpoint(a, b),
                    scale(
                        normal(direction),
                        (radius * radius - step * step * 0.25).sqrt(),
                    ),
                );
                let from = sub(a, center);
                Scallop {
                    center,
                    radius,
                    start: from[1].atan2(from[0]),
                    sweep: 2. * (step / (2. * radius)).asin(),
                }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clockwise_scallops_retain_polygon_order_and_actual_native_curve() {
        let points = [[20., 20.], [50., 20.], [50., 40.], [20., 40.]];
        let plan = Cloud::new(&points).unwrap();
        assert_eq!(plan.label, [20., 18.]);
        assert_eq!(plan.work(), 20 * 513);
        let curves: Vec<_> = plan.arcs().map(Scallop::points).collect();
        assert_eq!(curves.len(), 20);
        for (i, curve) in curves.iter().enumerate() {
            assert!(
                length(sub(
                    *curve.last().unwrap(),
                    curves[(i + 1) % curves.len()][0]
                )) < 1e-12
            );
        }
        assert!(curves[0]
            .iter()
            .any(|p| p[1] < 19. && p[0] > 20. && p[0] < 25.));
        let reversed: Vec<_> = points.into_iter().rev().collect();
        assert!(Cloud::new(&reversed)
            .unwrap()
            .arcs()
            .next()
            .unwrap()
            .points()
            .iter()
            .any(|p| p[1] < 40.));
    }

    #[test]
    fn arbitrary_loaded_vertices_and_duplicates_survive_unchanged() {
        let points = [
            [20., 20.],
            [40., 18.],
            [60., 30.],
            [60., 30.],
            [52., 50.],
            [30., 55.],
            [14., 37.],
        ];
        let before = points;
        let plan = Cloud::new(&points).unwrap();
        assert!(plan
            .arcs()
            .all(|a| a.points().iter().flatten().all(|n| n.is_finite())));
        assert_eq!(points, before);
        let collapsed = [[10., 10.]; 3];
        assert_eq!(Cloud::new(&collapsed).unwrap().arcs().count(), 0);
        assert!(Cloud::new(&[[0., 0.], [1., 0.]]).is_err());
        assert!(Cloud::new(&[[0., 0.], [f64::NAN, 1.], [2., 0.]]).is_err());
        assert!(Cloud::new(&[[f64::MAX, 0.], [-f64::MAX, 1.], [0., 2.]]).is_err());
        assert!(Cloud::new(&vec![[0., 0.]; 4097]).is_err());
    }
}
