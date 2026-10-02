import type { Point3Dto } from '../engine/types';
import type { AppState, CamHolePickHole, CamPointPickCandidate } from '../store/appStore';
import {
  pickClipPolylineCandidate, type ClipPoint, type ScreenPoint, type ScreenViewport,
} from '../modeling/screenSpaceEntityPicker';

export interface CamPickPointer { clientX: number; clientY: number; altKey: boolean }

/** Rendering adapter: the picker does not own the camera, renderer or CAD engine. */
export interface CamPickGeometry {
  viewport: ScreenViewport;
  clip(point: Point3Dto): ClipPoint;
  occluded(point: Point3Dto): boolean;
  face(): { bodyId: number; faceId: number } | null;
  edge(): { bodyId: number; edgeId: number } | null;
  hole(): CamHolePickHole | null;
}

export type CamPickerState = Pick<AppState,
  | 'camPointPick' | 'camHolePick' | 'camLoopPick' | 'camChainPick'
  | 'setCamPointPickHover' | 'setCamHolePickHover' | 'setCamLoopPickHover'
  | 'setHoveredFace' | 'setHoveredEdge' | 'toggleCamHolePickHole' | 'selectCamLoopPickLoop'>;

interface PickerActions {
  referenceKey(candidate: CamPointPickCandidate): string;
  completeReference(candidate: CamPointPickCandidate): void;
  hoverChain(key: string | null): void;
  selectChain(key: string, individual: boolean): void;
}

type PickResult =
  | { kind: 'reference'; value: CamPointPickCandidate | null }
  | { kind: 'hole'; value: CamHolePickHole | null }
  | { kind: 'loop' | 'chain'; value: string | null };

/** One picker per viewport, shared by setup origins, every height row, and
 * toolpath geometry. It owns acquisition priority and hover/click routing;
 * callers retain their drafts, validation and single/multiple selection rules.
 * A temporary height/origin session takes precedence without clearing a path.
 */
export class CamGeometryPicker<Pointer extends CamPickPointer = CamPickPointer> {
  constructor(
    private readonly state: () => CamPickerState,
    private readonly geometry: (pointer: Pointer) => CamPickGeometry,
    private readonly actions: PickerActions,
  ) {}

  hover(pointer: Pointer): { handled: boolean; hit: boolean } {
    const state = this.state();
    const result = this.pick(pointer, state);
    if (!result) return { handled: false, hit: false };
    // Always clear the other geometry channel when moving between references.
    state.setHoveredFace(result.kind === 'hole' ? result.value?.faceId ?? null
      : result.kind === 'reference' && result.value?.target?.kind === 'face' ? result.value.target.id : null);
    state.setHoveredEdge(result.kind === 'reference' && result.value?.target?.kind === 'edge' ? result.value.target.id : null);
    switch (result.kind) {
      case 'reference': state.setCamPointPickHover(result.value ? this.actions.referenceKey(result.value) : null); break;
      case 'hole': state.setCamHolePickHover(result.value?.key ?? null); break;
      case 'loop': state.setCamLoopPickHover(result.value); break;
      case 'chain': this.actions.hoverChain(result.value); break;
    }
    return { handled: true, hit: result.value !== null };
  }

  clearHover(): void {
    const state = this.state();
    state.setHoveredFace(null);
    state.setHoveredEdge(null);
    if (state.camPointPick) state.setCamPointPickHover(null);
    if (state.camHolePick) state.setCamHolePickHover(null);
    if (state.camLoopPick) state.setCamLoopPickHover(null);
    if (state.camChainPick) this.actions.hoverChain(null);
  }

  select(pointer: Pointer): boolean {
    const state = this.state();
    // Reacquire against current geometry: never commit a stale hover result.
    const result = this.pick(pointer, state);
    if (!result) return false;
    if (result.value !== null) {
      switch (result.kind) {
        case 'reference': this.actions.completeReference(result.value); break;
        case 'hole': state.toggleCamHolePickHole(result.value); break;
        case 'loop': state.selectCamLoopPickLoop(result.value); break;
        case 'chain': this.actions.selectChain(result.value, pointer.altKey); break;
      }
    }
    return true; // An active picker owns empty-space clicks as well.
  }

  private pick(pointer: Pointer, state: CamPickerState): PickResult | null {
    if (!state.camPointPick && !state.camHolePick && !state.camLoopPick && !state.camChainPick) return null;
    const geometry = this.geometry(pointer);
    const screen = { x: pointer.clientX, y: pointer.clientY };
    if (state.camPointPick) return { kind: 'reference', value: this.reference(state.camPointPick.candidates, geometry, screen) };
    if (state.camHolePick) return { kind: 'hole', value: geometry.hole() };
    if (state.camLoopPick) {
      return { kind: 'loop', value: this.curves(state.camLoopPick.loops.map(loop => ({
        key: loop.key, points: loop.modelPoints, closed: true, interior: true,
      })), geometry, screen, 14) };
    }
    const session = state.camChainPick!;
    return { kind: 'chain', value: this.curves(session.entities.map(entity => ({
      key: entity.key, points: entity.modelPoints, closed: entity.closed || entity.kind === 'circle',
      // At coincident seams, automatic loops favor the setup-planar rim.
      bias: session.mode === 'closed' && !pointer.altKey && entity.planarInSetup === false ? 0.5 : 0,
    })), geometry, screen, 10) };
  }

  private reference(candidates: CamPointPickCandidate[], geometry: CamPickGeometry, pointer: ScreenPoint): CamPointPickCandidate | null {
    let best: CamPointPickCandidate | null = null;
    let distance = candidates.some(c => c.target) ? 8 : 16;
    let depth = Infinity;
    for (const candidate of candidates) {
      if (candidate.target) continue;
      const point = project(candidate.point, geometry);
      if (!point || (candidate.occlude && geometry.occluded(candidate.point))) continue;
      const d = Math.hypot(point.x - pointer.x, point.y - pointer.y);
      if (d < distance - 0.1 || (d <= distance && point.z <= depth)) {
        best = candidate; distance = d; depth = point.z;
      }
    }
    if (best) return best;
    const lines = candidates.flatMap(candidate => candidate.target?.kind === 'line'
      ? [{ key: this.actions.referenceKey(candidate), points: candidate.target.points }] : []);
    const line = this.curves(lines, geometry, pointer, 10);
    if (line !== null) return candidates.find(c => this.actions.referenceKey(c) === line) ?? null;
    if (candidates.some(c => c.target?.kind === 'edge')) {
      const edge = geometry.edge();
      const candidate = edge && candidates.find(c => c.target?.kind === 'edge' && c.target.bodyId === edge.bodyId && c.target.id === edge.edgeId);
      if (candidate) return candidate;
    }
    if (candidates.some(c => c.target?.kind === 'face')) {
      const face = geometry.face();
      return face ? candidates.find(c => c.target?.kind === 'face' && c.target.bodyId === face.bodyId && c.target.id === face.faceId) ?? null : null;
    }
    return null;
  }

  /** Shared projected curve hit test: boundaries for lines/chains, optional
   * interior for pocket loops. Homogeneous clipping keeps partially visible
   * curves selectable without turning an open path into a closed boundary. */
  private curves(candidates: CurveCandidate[], geometry: CamPickGeometry, pointer: ScreenPoint, radius: number): string | null {
    let best: string | null = null;
    let score = Infinity;
    for (const candidate of candidates) {
      let distance = Infinity;
      if (candidate.interior && candidate.closed) {
        const polygon = candidate.points.map(p => project(p, geometry));
        if (polygon.length >= 3 && polygon.every(p => p !== null) && insidePolygon(pointer, polygon)) distance = 0;
      }
      if (distance !== 0) {
        const points = candidate.points.map(p => geometry.clip(p));
        if (candidate.closed && points.length > 1) points.push(points[0]);
        distance = pickClipPolylineCandidate([{ key: candidate.key, value: null, polylines: [points] }], pointer, geometry.viewport, { enterRadiusPx: radius })?.distancePx ?? Infinity;
      }
      const biased = distance + (candidate.bias ?? 0);
      if (distance <= radius && biased <= score) { best = candidate.key; score = biased; }
    }
    return best;
  }
}

interface CurveCandidate {
  key: string;
  points: Point3Dto[];
  closed?: boolean;
  interior?: boolean;
  bias?: number;
}

function project(point: Point3Dto, geometry: CamPickGeometry): (ScreenPoint & { z: number }) | null {
  const p = geometry.clip(point);
  if (p.w <= 1e-7 || !Number.isFinite(p.w)) return null;
  const z = p.z / p.w;
  if (!Number.isFinite(z) || z < -1 || z > 1) return null;
  const { left, top, width, height } = geometry.viewport;
  return { x: left + (p.x / p.w + 1) * width / 2, y: top + (1 - p.y / p.w) * height / 2, z };
}

function insidePolygon(point: ScreenPoint, polygon: ScreenPoint[]): boolean {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const a = polygon[i], b = polygon[j];
    if ((a.y > point.y) !== (b.y > point.y) && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}
