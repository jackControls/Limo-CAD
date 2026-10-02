import type { DrawingAnnotationDto, DrawingCircularRefDto, HoleDefinitionDto } from '../engine/types';

type HoleNote = Extract<DrawingAnnotationDto, { kind: 'hole_note' }>;
export function applyHoleExtentUpdate(annotation: HoleNote, update: Partial<Pick<HoleNote, 'depth' | 'through_all'>>): void {
  if (update.through_all === true && update.depth != null) throw new Error('A through hole cannot also have a finite depth');
  if (update.depth !== undefined) annotation.depth = update.depth;
  if (update.through_all !== undefined) annotation.through_all = update.through_all;
  if (update.through_all === true) annotation.depth = null;
  if (update.depth != null) annotation.through_all = false;
}

/** Match the full current 3D entry, radius, and body. Sample winding does not
 * establish cutting direction; ambiguous/associative/placed sources stay manual. */
export function bestHoleDefinitionForCircle(definitions: HoleDefinitionDto[], feature: DrawingCircularRefDto): HoleDefinitionDto | null {
  if (feature.occurrence_id != null || definitions.length > 16_384
    || definitions.reduce((n, d) => n + Math.max(1, d.positions.length), 0) > 65_536) return null;
  const normalLength = Math.hypot(...feature.fallback_normal);
  if (!Number.isFinite(normalLength) || normalLength < 1e-9) return null;
  const tolerance = Math.max(feature.fallback_radius * 1e-6, 1e-5);
  let matched: HoleDefinitionDto | null = null;
  for (const d of definitions) {
    if (d.body_id !== feature.body_id) continue;
    const radiusError = Math.abs(d.diameter * 0.5 - feature.fallback_radius);
    if (!Number.isFinite(radiusError) || radiusError > tolerance) continue;
    const associative = d.positions.length ? d.positions.some(p => p.position_reference != null) : d.position_reference != null;
    if (associative || !d.face_basis) return null;
    const basis = d.face_basis;
    const normalSq = basis.normal.reduce((n, v) => n + v * v, 0);
    if (!Number.isFinite(normalSq) || Math.abs(normalSq - 1) > 1e-6) return null;
    const alignment = Math.abs(feature.fallback_normal.reduce((n, v, i) => n + v / normalLength * basis.normal[i], 0));
    if (!Number.isFinite(alignment) || alignment < 1 - 1e-6) continue;
    const positions = d.positions.length ? d.positions.map(p => p.position) : [d.position];
    let error = Infinity;
    for (const point of positions) {
      const delta = basis.origin.map((v, i) => v + basis.u[i] * point.x + basis.v[i] * point.y - feature.fallback_center[i]);
      error = Math.min(error, Math.hypot(...delta));
    }
    if (!Number.isFinite(error) || error > tolerance) continue;
    if (matched) return null;
    matched = d;
  }
  return matched;
}
