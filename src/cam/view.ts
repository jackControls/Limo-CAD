import type { CamDocumentDto, CamSetupDto, CamSimulationResultDto } from '../engine/types';
import { modeledStockBodyId } from './geometry';

export type CamWorkpieceView = 'model' | 'stock' | 'compare';

interface CamStageState {
  camSimulation: CamSimulationResultDto | null;
  camSimulationTimeline: CamSimulationResultDto | null;
  selectedCamOperationId: number | null;
}

export function simulationHasStockSurface(simulation: CamSimulationResultDto): boolean {
  return simulation.stock_mesh !== null || simulation.native_stock_present === true;
}

/** Selection/source freshness is shared by the native and overlay renderers. */
export function currentStageSimulation(state: CamStageState, setup: CamSetupDto): CamSimulationResultDto | null {
  const simulation = state.camSimulation;
  if (!simulation || !simulationHasStockSurface(simulation) || simulation.setup_id !== setup.id) return null;
  if (simulation.source === 'cam_toolpath' && simulation.through_operation_id !== state.selectedCamOperationId) return null;
  if (state.camSimulationTimeline && simulation.source !== state.camSimulationTimeline.source) return null;
  return simulation;
}

/** Presentation only: never touches CAM inputs, generations, or retained stock. */
export function camWorkpiecePresentation(state: CamStageState & {
  activeTab: string;
  camDocument: CamDocumentDto;
  camDialogOpen: boolean;
  camWorkpieceView?: CamWorkpieceView;
  camPointPick?: unknown;
}) {
  const result = { stockVisible: false, hideSketches: false, hiddenBodyIds: [] as number[], ghostedBodyIds: [] as number[] };
  if (state.activeTab !== 'cam') return result;
  const setup = state.camDocument.setups.find(candidate => candidate.id === state.camDocument.active_setup_id);
  if (!setup) return result;
  if (state.camDialogOpen) {
    // A dialog's viewport pick targets the part; an opaque modeled stock
    // body would occlude its faces, edges and vertices.
    const stockBodyId = state.camPointPick ? modeledStockBodyId(setup, state.camDocument) : null;
    if (stockBodyId !== null) result.hiddenBodyIds.push(stockBodyId);
    return result;
  }
  const mode = state.camWorkpieceView ?? 'stock';
  const stockBodyId = modeledStockBodyId(setup, state.camDocument);
  // In Model mode, raw stock must stay hidden even before a simulation exists.
  result.stockVisible = mode !== 'model' && currentStageSimulation(state, setup) !== null;
  if (mode === 'model' || result.stockVisible) {
    if (stockBodyId !== null) result.hiddenBodyIds.push(stockBodyId);
  }
  if (result.stockVisible) {
    // Sketch curves draw through solids; over the simulated stock they read
    // as phantom edges of material that is no longer the part.
    result.hideSketches = true;
    const targets = setup.body_ids.filter(id => id !== stockBodyId);
    if (mode === 'compare') result.ghostedBodyIds = targets;
    else result.hiddenBodyIds.push(...targets);
  }
  return result;
}
