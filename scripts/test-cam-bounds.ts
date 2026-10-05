import assert from 'node:assert/strict';
import { modelBoundsOfBodies, resolveStock } from '../src/cam/geometry';

// A Ø49 disk tessellated with 41 segments: a vertex at 0° but none at 180°,
// the shape that once shifted a centred stock (and its WCS) by +0.036 mm.
const radius = 24.5, segments = 41;
const ring = (z: number) => Array.from({ length: segments }, (_, i) => {
  const a = (2 * Math.PI * i) / segments;
  return { x: radius * Math.cos(a), y: radius * Math.sin(a), z };
});
const positions = [...ring(0), ...ring(10.5)].flatMap(p => [p.x, p.y, p.z]);
const circle = (z: number) => ({ center: { x: 0, y: 0, z }, normal: { x: 0, y: 0, z: 1 }, reference: { x: 1, y: 0, z: 0 }, radius, closed: true });
const meshOnly = { bodies: [{ id: 1, mesh: { positions } }] };
const exact = { bodies: [{ id: 1, mesh: { positions }, edges: [
  { points: ring(0), circle: circle(0) },
  { points: ring(10.5), circle: circle(10.5) },
] }] };

const coarse = modelBoundsOfBodies(meshOnly, [1])!;
assert.ok(coarse.min.x > -radius + 0.05, 'the bare mesh really is short on -X');
const bounds = modelBoundsOfBodies(exact, [1])!;
for (const axis of ['x', 'y'] as const) {
  assert.ok(Math.abs(bounds.min[axis] + radius) < 1e-9 && Math.abs(bounds.max[axis] - radius) < 1e-9, `exact ${axis} extent`);
}
assert.equal(bounds.min.z, 0); assert.equal(bounds.max.z, 10.5);

const stock = resolveStock(
  { mode: 'fixed', shape: 'cylinder', size: { x: 50.8, y: 0, z: 22 }, placement: { center: false, face: 'z_min', offset: 4.5 } },
  bounds, null, 0,
);
assert.ok(Math.abs((stock.modelBox.min.x + stock.modelBox.max.x) / 2) < 1e-9, 'cylinder stock is centred on the disk');

// An open quarter arc (0°..90°) must not claim the -X/-Y extremes.
const quarter = Array.from({ length: 9 }, (_, i) => {
  const a = (Math.PI / 2) * (i / 8);
  return { x: radius * Math.cos(a), y: radius * Math.sin(a), z: 0 };
});
const arc = modelBoundsOfBodies({ bodies: [{ id: 1, mesh: { positions: [0, 0, 0] }, edges: [
  { points: quarter, circle: { ...circle(0), closed: false } },
] }] }, [1])!;
assert.equal(arc.min.x, 0); assert.equal(arc.min.y, 0);
assert.ok(Math.abs(arc.max.x - radius) < 1e-9 && Math.abs(arc.max.y - radius) < 1e-9);

console.log('PASS: model bounds use exact circle extremes (closed and arcs), centred cylinder stock');
