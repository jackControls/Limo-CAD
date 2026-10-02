import assert from 'node:assert/strict';
import { CamGeometryPicker, type CamPickerState, type CamPickGeometry } from '../src/cam/geometryPicker';
import type { CamHolePickHole, CamPointPickCandidate } from '../src/store/appStore';

const point = (x: number, y: number, z = 0) => ({ x, y, z });
const pointer = (x = 50, y = 50, altKey = false) => ({ clientX: x, clientY: y, altKey });
const references: CamPointPickCandidate[] = [];
const chains: Array<[string, boolean]> = [];
const loops: string[] = [];
const holes: CamHolePickHole[] = [];
let chainHover: string | null = null;
let hoveredFace: number | null = null;
let hoveredEdge: number | null = null;
let blocked = false;
let face: ReturnType<CamPickGeometry['face']> = null;
let edge: ReturnType<CamPickGeometry['edge']> = null;
let hole: CamHolePickHole | null = null;
let samples = 0;
const state: CamPickerState = {
  camPointPick: null, camHolePick: null, camLoopPick: null, camChainPick: null,
  setCamPointPickHover: key => { if (state.camPointPick) state.camPointPick.hoverKey = key; },
  setCamHolePickHover: key => { if (state.camHolePick) state.camHolePick.hoverKey = key; },
  setCamLoopPickHover: key => { if (state.camLoopPick) state.camLoopPick.hoverKey = key; },
  setHoveredFace: value => { hoveredFace = value; },
  setHoveredEdge: value => { hoveredEdge = value; },
  toggleCamHolePickHole: value => { holes.push(value); },
  selectCamLoopPickLoop: key => { loops.push(key); },
};
const picker = new CamGeometryPicker(() => state, () => {
  samples++;
  return {
    viewport: { left: 0, top: 0, width: 100, height: 100 },
    clip: p => ({ ...p, w: 1 }), occluded: () => blocked,
    face: () => face, edge: () => edge, hole: () => hole,
  };
}, {
  referenceKey: candidate => candidate.key!,
  completeReference: candidate => { references.push(candidate); state.camPointPick = null; },
  hoverChain: key => { chainHover = key; },
  selectChain: (key, individual) => { chains.push([key, individual]); },
});
const request = (candidates: CamPointPickCandidate[]) => { state.camPointPick = { candidates, prompt: 'Pick', hoverKey: null }; };
const chain = (closed = false) => {
  state.camChainPick = { entities: [{ key: 'path', source: 'sketch', kind: 'line', closed,
    modelPoints: [point(-0.8, -0.8), point(0.8, -0.8), point(0.8, 0.8)] }], selectedKeys: ['existing'], hoverKey: null };
};

assert.deepEqual(picker.hover(pointer()), { handled: false, hit: false });
assert.equal(picker.select(pointer()), false);
assert.equal(samples, 0, 'No geometry queries without an active session');

chain();
request([{ key: 'bottom', label: 'Bottom', point: point(0, 0) }]);
assert.deepEqual(picker.hover(pointer()), { handled: true, hit: true });
assert.equal(state.camPointPick!.hoverKey, 'bottom');
picker.select(pointer());
assert.equal(references.at(-1)!.key, 'bottom');
assert.deepEqual(state.camChainPick!.selectedKeys, ['existing'], 'A temporary height pick preserves the path draft');
assert.equal(picker.select(pointer()), true, 'An empty-space click belongs to the active path picker');
assert.equal(chains.length, 0, 'Open chains never acquire an invented closing segment');
chain(true);
picker.select(pointer());
assert.deepEqual(chains.at(-1), ['path', false]);
state.camChainPick = null;

const faceCandidate: CamPointPickCandidate = { key: 'face', label: 'Face', point: point(0,0), target: { kind: 'face', bodyId: 1, id: 10 } };
const edgeCandidate: CamPointPickCandidate = { key: 'edge', label: 'Edge', point: point(0,0), target: { kind: 'edge', bodyId: 1, id: 20 } };
face = { bodyId: 1, faceId: 10 }; edge = { bodyId: 1, edgeId: 20 };
request([faceCandidate, edgeCandidate, { key: 'vertex', label: 'Vertex', point: point(0,0), occlude: true }]);
picker.hover(pointer());
assert.equal(state.camPointPick!.hoverKey, 'vertex');
blocked = true;
picker.hover(pointer());
assert.equal(state.camPointPick!.hoverKey, 'edge', 'An occluded point does not steal an edge');
assert.equal(hoveredEdge, 20); assert.equal(hoveredFace, null);
edge = null;
picker.hover(pointer());
assert.equal(hoveredFace, 10); assert.equal(hoveredEdge, null);
const before = references.length;
state.camPointPick!.candidates = [];
picker.select(pointer());
assert.equal(references.length, before, 'Click reacquires instead of committing a stale face hover');
state.camPointPick = null;

state.camLoopPick = { loops: [{ key: 'pocket', label: 'Pocket', modelPoints: [point(-0.8,-0.8), point(0.8,-0.8), point(0.8,0.8), point(-0.8,0.8)] }], selectedKey: null, hoverKey: null };
picker.hover(pointer()); picker.select(pointer());
assert.equal(state.camLoopPick.hoverKey, 'pocket'); assert.equal(loops.at(-1), 'pocket', 'Pocket interiors use the shared curve picker');
state.camLoopPick = null;

state.camChainPick = { mode: 'closed', selectedKeys: [], hoverKey: null, entities: [
  { key: 'rim', source: 'model', kind: 'line', planarInSetup: true, modelPoints: [point(-1,0), point(1,0)] },
  { key: 'seam', source: 'model', kind: 'line', planarInSetup: false, modelPoints: [point(-1,0), point(1,0)] },
] };
picker.hover(pointer()); assert.equal(chainHover, 'rim');
picker.select(pointer(50,50,true)); assert.deepEqual(chains.at(-1), ['seam', true], 'Option/manual picking removes the planar-rim preference');
state.camChainPick.entities = [{ key: 'clipped', source: 'sketch', kind: 'line', modelPoints: [point(-0.8,0,-2),point(0.8,0,0)] }];
picker.select(pointer(75,50)); assert.equal(chains.at(-1)![0], 'clipped', 'A visible portion remains pickable across the near plane');
state.camChainPick = null;

state.camHolePick = { holes: [], hoverKey: null };
assert.deepEqual(picker.hover(pointer()), { handled: true, hit: false });
hole = { key: '1:10', bodyId: 1, faceId: 10, radius: 2, modelPoint: point(0,0), point: { x:0, y:0 }, axis: [0,0,1], topZ:0, bottomZ:-5 };
picker.hover(pointer()); picker.select(pointer());
assert.equal(state.camHolePick.hoverKey, hole.key); assert.equal(holes.at(-1), hole);
picker.clearHover();
assert.equal(state.camHolePick.hoverKey, null); assert.equal(hoveredFace, null); assert.equal(hoveredEdge, null);
console.log('PASS: shared CAM picker ownership, reference priority, occlusion, current clicks, open/closed paths, loop interiors, chain bias, clipping and holes');
