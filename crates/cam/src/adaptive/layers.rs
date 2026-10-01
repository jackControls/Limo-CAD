//! Height-qualified removal certificates: a cut proves empty space upward
//! through the cutting length, never below its floor. This planner already
//! requires flute length spanning the incoming top through operation bottom.
use super::*;

pub(super) struct Removal {
    pub depth: f64,
    pub exterior: Option<ConvexStock>,
    pub centers: Vec<Point2Dto>,
}

pub(super) fn restore(
    history: &[Removal],
    depth: f64,
    cleared: &mut Cleared,
    work: &mut Work,
) -> Result<(), CamPlanError> {
    for cut in history.iter().filter(|cut| cut.depth <= depth + EPS) {
        work.spend(cut.centers.len() * 81 + 1, 0)?;
        for &c in &cut.centers {
            cleared.add(c);
        }
        if let Some(bound) = &cut.exterior {
            // Either certificate is valid. Prefer the tighter one when
            // nested; never turn disjoint cavity disks into a convex void.
            if cleared
                .exterior
                .as_ref()
                .is_none_or(|old| old.contains_bound(bound))
            {
                cleared.exterior = Some(bound.clone());
            }
        }
    }
    Ok(())
}

pub(super) fn depth_order(
    setup: &CamSetupDto,
    meshes: &[CamStockMeshDto],
    top: f64,
    bottom: f64,
    p: &CamAdaptiveParametersDto,
    corner_height: f64,
) -> Result<Vec<f64>, CamPlanError> {
    // The first band may use full Ap. Subsequent bands overlap the corner's
    // height, so the preceding full-diameter sweep is available at the new
    // cut's Ap ceiling. The planner must still prove actual cleared stock.
    if corner_height + EPS >= p.maximum_stepdown && top - bottom > p.maximum_stepdown + EPS {
        return roughing_depth_levels(setup, meshes, top, bottom, p);
    }
    let terraces = roughing_terraces(setup, meshes, top, bottom, p);
    let cap = meshes
        .iter()
        .flat_map(|mesh| mesh.positions.chunks_exact(3))
        .map(|v| {
            (v[0] - setup.wcs.origin.x) * setup.wcs.z_axis[0]
                + (v[1] - setup.wcs.origin.y) * setup.wcs.z_axis[1]
                + (v[2] - setup.wcs.origin.z) * setup.wcs.z_axis[2]
                + p.axial_stock_to_leave
        })
        .fold(f64::NEG_INFINITY, f64::max);
    let mut ordered = Vec::new();
    let mut upper = top;
    loop {
        let step = if ordered.is_empty() {
            p.maximum_stepdown
        } else {
            p.maximum_stepdown - corner_height
        };
        let limit = (upper - step).max(bottom);
        // Prefer the deepest reachable shoulder/pocket floor. Cutting an
        // arbitrary plane below that terrace would retain its larger target
        // envelope and require a second exterior pass. The highest part cap
        // is cleanup, not a reason to abandon a deeper usable cut.
        let lower = terraces
            .iter()
            .rev()
            .copied()
            .find(|z| {
                *z >= limit - EPS
                    && *z < upper - EPS
                    && (*z < cap - EPS || (*z - bottom).abs() <= EPS)
            })
            .unwrap_or(limit);
        ordered.push(lower);
        // The major cut removes stock above it. Step upward through only
        // the intervening terraces, using that updated stock at each level.
        ordered.extend(
            terraces
                .iter()
                .rev()
                .copied()
                .filter(|z| *z > lower + EPS && *z < upper - EPS),
        );
        upper = lower;
        if ordered.len() > 512 {
            return Err(CamPlanError(
                "High Speed Roughing is limited to 512 depth levels per operation.".into(),
            ));
        }
        if lower <= bottom + EPS {
            break;
        }
    }
    Ok(ordered)
}
