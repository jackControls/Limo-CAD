import { translate } from '../i18n';
import type { CamHeightGeometryDto, CamSetupDto, Point3Dto, SketchDto, SolidSceneDto } from '../engine/types';
import type { CamPointPickCandidate } from '../store/appStore';
import { modelPointToSetup, sketchUvToModel } from './geometry';

const TOLERANCE = 1e-4;
const missing = () => new Error(translate('cam.operation.heightGeometryInvalid'));

/** Resolve from current geometry, never from a cached click position. */
export function resolveHeightGeometry(ref: CamHeightGeometryDto, scene: SolidSceneDto, sketches: SketchDto[], setup: CamSetupDto): number {
  let points: Point3Dto[];
  if (ref.kind === 'sketch_point' || ref.kind === 'sketch_line') {
    const sketch = sketches.find(s => s.name === ref.sketch);
    const entity = sketch?.entities.find(e => e.id === ref.entity_id);
    if (!sketch || !entity) throw missing();
    if (ref.kind === 'sketch_point' && entity.kind === 'point') points = [sketchUvToModel(sketch.basis, entity.position)];
    else if (ref.kind === 'sketch_line' && entity.kind === 'line' && !entity.consumed) points = [entity.start, entity.end].map(p => sketchUvToModel(sketch.basis, p));
    else throw missing();
  } else {
    const body = scene.bodies.find(b => b.id === ref.body_id && setup.body_ids.includes(b.id));
    if (!body) throw missing();
    if (ref.kind === 'face') {
      const plane = body.faces.find(f => f.key === ref.key)?.plane;
      if (!plane || Math.abs(Math.abs(plane.normal.reduce((v, n, i) => v + n * setup.wcs.z_axis[i], 0)) - 1) > 1e-6) throw missing();
      points = [{ x: plane.origin[0], y: plane.origin[1], z: plane.origin[2] }];
    } else {
      points = body.edges.find(e => e.key === ref.key)?.points ?? [];
      if (ref.kind === 'vertex') points = points.length ? [points[ref.end ? points.length - 1 : 0]] : [];
    }
  }
  const levels = points.map(p => modelPointToSetup(p, setup.wcs).z);
  if (!levels.length || levels.some(z => !Number.isFinite(z) || Math.abs(z - levels[0]) > TOLERANCE)) throw missing();
  return levels[0];
}

export function heightGeometryLabel(ref: CamHeightGeometryDto): string {
  if (ref.kind === 'sketch_point' || ref.kind === 'sketch_line') return `${ref.sketch} · ${ref.kind === 'sketch_point' ? 'point' : 'line'} ${ref.entity_id}`;
  return `Body ${ref.body_id} · ${ref.kind} ${ref.key.replace(/^(face|edge):/, '')}${ref.kind === 'vertex' ? (ref.end ? ' end' : ' start') : ''}`;
}

export function heightGeometryCandidates(scene: SolidSceneDto, sketches: SketchDto[], setup: CamSetupDto): CamPointPickCandidate[] {
  const candidates: CamPointPickCandidate[] = [];
  const add = (ref: CamHeightGeometryDto, point: Point3Dto, target?: CamPointPickCandidate['target']) => {
    try { resolveHeightGeometry(ref, scene, sketches, setup); } catch { return; }
    candidates.push({ point, target, key: JSON.stringify(ref), payload: ref, occlude: ref.kind === 'vertex', label: heightGeometryLabel(ref) });
  };
  for (const body of scene.bodies.filter(b => setup.body_ids.includes(b.id))) {
    for (const face of body.faces) {
      if (!face.plane) continue;
      const [x, y, z] = face.plane.origin;
      add({ kind: 'face', body_id: body.id, key: face.key }, { x, y, z }, { kind: 'face', bodyId: body.id, id: face.id });
    }
    for (const edge of body.edges) {
      if (!edge.points.length) continue;
      add({ kind: 'edge', body_id: body.id, key: edge.key }, edge.points[0], { kind: 'edge', bodyId: body.id, id: edge.id });
      const first = edge.points[0], last = edge.points[edge.points.length - 1];
      if (Math.hypot(first.x - last.x, first.y - last.y, first.z - last.z) <= TOLERANCE) continue;
      add({ kind: 'vertex', body_id: body.id, key: edge.key, end: false }, first);
      add({ kind: 'vertex', body_id: body.id, key: edge.key, end: true }, last);
    }
  }
  for (const sketch of sketches) for (const entity of sketch.entities) {
    if (entity.kind === 'point') add({ kind: 'sketch_point', sketch: sketch.name, entity_id: entity.id }, sketchUvToModel(sketch.basis, entity.position));
    if (entity.kind === 'line' && !entity.consumed) {
      const points = [entity.start, entity.end].map(p => sketchUvToModel(sketch.basis, p));
      add({ kind: 'sketch_line', sketch: sketch.name, entity_id: entity.id }, points[0], { kind: 'line', points });
    }
  }
  return candidates;
}
