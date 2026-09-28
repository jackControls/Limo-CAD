import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { drawingHoleCalloutText } from '../src/drawing/annotations';
import { applyHoleExtentUpdate, bestHoleDefinitionForCircle } from '../src/drawing/holeReference';
import type { DrawingAnnotationDto, HoleDefinitionDto, DrawingCircularRefDto, DrawingStandard } from '../src/engine/types';
import type { UnitSystem } from '../src/types/document';

const fixture = JSON.parse(readFileSync(new URL('../crates/occt/tests/fixtures/hole-callout.json', import.meta.url), 'utf8'));
for (const sample of fixture.cases) {
  const annotation = { ...fixture.annotation, ...sample.changes } as Extract<DrawingAnnotationDto, { kind: 'hole_note' }>;
  assert.equal(drawingHoleCalloutText(annotation, sample.standard as DrawingStandard, sample.units as UnitSystem), sample.expected, sample.name);
}
const circle = { ...fixture.annotation.feature, fallback_center: [20, 15, 10], fallback_radius: 3 } as DrawingCircularRefDto;
const definition = { feature_id: 3, body_id: 1, diameter: 6, positions: [{ position: { x: 20, y: 15 }, position_reference: null }],
  face_basis: { origin: [0, 0, 10], u: [1, 0, 0], v: [0, 1, 0], normal: [0, 0, 1] } } as unknown as HoleDefinitionDto;
assert.equal(bestHoleDefinitionForCircle([definition], circle), definition);
assert.equal(bestHoleDefinitionForCircle([definition, { ...definition, feature_id: 4 }], circle), null);
for (const changed of [
  { ...circle, occurrence_id: 5 }, { ...circle, body_id: 2 },
  { ...circle, fallback_center: [20, 15, 0] }, { ...circle, fallback_center: [20.02, 15, 10] },
  { ...circle, fallback_radius: 3.02 }, { ...circle, fallback_normal: [1, 0, 0] },
] as DrawingCircularRefDto[]) assert.equal(bestHoleDefinitionForCircle([definition], changed), null);
assert.equal(bestHoleDefinitionForCircle([{ ...definition, positions: [{ position: { x: 20, y: 15 }, position_reference: { sketch_name: 'Sketch1', entity_id: 1, kind: 'point' } }] }], circle), null);
assert.equal(bestHoleDefinitionForCircle([{ ...definition, face_basis: null }], circle), null);
console.log('PASS shared hole-note labels and exact, unambiguous 3D source matching');
const legacy = structuredClone(fixture.annotation);
const original = structuredClone(legacy);
applyHoleExtentUpdate(legacy, {});
assert.deepEqual(legacy, original, 'No-op legacy edits must preserve absent extent');
applyHoleExtentUpdate(legacy, { through_all: true });
assert.equal(legacy.through_all, true);
assert.equal(legacy.depth, null);
applyHoleExtentUpdate(legacy, { depth: 8 });
assert.equal(legacy.depth, 8);
assert.equal(legacy.through_all, false);
applyHoleExtentUpdate(legacy, { depth: 8, through_all: null });
assert.equal(legacy.through_all, false, 'A finite depth takes precedence over an unspecified extent');
const beforeConflict = structuredClone(legacy);
assert.throws(() => applyHoleExtentUpdate(legacy, { depth: 8, through_all: true }));
assert.deepEqual(legacy, beforeConflict, 'A conflicting extent update must be atomic');
assert.equal(legacy.note, original.note, 'Extent edits preserve note text');
