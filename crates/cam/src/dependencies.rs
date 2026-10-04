//! Generation dependencies, distinct from the ordered simulation/NC program.
//!
//! A reordered operation needs a new generation only when its own inputs or
//! the earlier material-removal evidence it consumes changed. This is not a
//! collision certificate: setup assembly, target verification and rest-stock
//! replay must still use the new sequence.
use crate::{CamLinkingDto, CamOperationDto, CamRampType, CamSetupDto, DrillCycle};

pub const CAM_ORDER_DEPENDENCY_RULES_REVISION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CamOperationDependencyKind {
    IncomingStockHeight,
    PredrilledEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamOperationDependency {
    pub operation_id: u64,
    pub kind: CamOperationDependencyKind,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlanningDependencyPolicy {
    pub incoming_stock_height: bool,
    pub predrilled_entry: bool,
}

/// Keep exhaustive: adding a strategy must explicitly declare every earlier
/// operation fact its planner reads. Rest-from-setup evidence is separately
/// tracked by the host's transitive upstream-setup fingerprint.
pub(crate) fn planning_dependency_policy(
    operation: &CamOperationDto,
    linking: Option<&CamLinkingDto>,
) -> PlanningDependencyPolicy {
    let predrilled_entry = match operation {
        CamOperationDto::Contour2d { .. } => {
            linking.is_some_and(|l| !l.predrill_positions.is_empty())
        }

        CamOperationDto::Adaptive3d { .. } => {
            linking.is_some_and(|l| l.ramp_type == CamRampType::Predrill)
        }
        CamOperationDto::Face { .. }
        | CamOperationDto::Drill { .. }
        | CamOperationDto::Pocket2d { .. }
        | CamOperationDto::Chamfer2d { .. }
        | CamOperationDto::Thread { .. } => false,
    };
    PlanningDependencyPolicy {
        incoming_stock_height: true,
        predrilled_entry,
    }
}

pub fn cam_operation_dependencies(
    setup: &CamSetupDto,
    operation: &CamOperationDto,
    linking: Option<&CamLinkingDto>,
) -> Vec<CamOperationDependency> {
    let policy = planning_dependency_policy(operation, linking);
    let mut dependencies = Vec::new();
    for source in setup
        .operations
        .iter()
        .take_while(|o| o.id() != operation.id())
        .filter(|o| o.enabled())
    {
        let kind = match source {
            CamOperationDto::Face { .. } if policy.incoming_stock_height => {
                Some(CamOperationDependencyKind::IncomingStockHeight)
            }
            CamOperationDto::Drill { cycle, .. }
                if policy.predrilled_entry
                    && matches!(
                        cycle,
                        DrillCycle::Drill | DrillCycle::ChipBreaking | DrillCycle::DeepHole
                    ) =>
            {
                Some(CamOperationDependencyKind::PredrilledEntry)
            }
            _ => None,
        };
        if let Some(kind) = kind {
            dependencies.push(CamOperationDependency {
                operation_id: source.id(),
                kind,
            });
        }
    }

    dependencies.sort_by_key(|d| (d.kind, d.operation_id));
    dependencies
}
