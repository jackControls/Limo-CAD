import assert from 'node:assert/strict';
import { anglesFromWcs, wcsFromAngles, restStockToSetup, setupPointToModel, modelPointToSetup } from '../src/cam/geometry';
import type { CamSetupDto } from '../src/engine/types';
const origin={x:7,y:-3,z:11};
for (const angles of [[0,0,0],[180,0,37],[23,-41,79],[-61,90,12],[74,-90,-23]]) {
  const wcs=wcsFromAngles(origin,...angles as [number,number,number]);
  const restored=wcsFromAngles(origin,...anglesFromWcs(wcs));
  for (const axis of ['x_axis','y_axis','z_axis'] as const) for(let i=0;i<3;i++)
    assert.ok(Math.abs(wcs[axis][i]-restored[axis][i])<1e-8,`orientation round trip ${angles}`);
  const p={x:17,y:-9,z:3};
  const roundtrip=modelPointToSetup(setupPointToModel(p,wcs),wcs);
  for (const axis of ['x','y','z'] as const) assert.ok(Math.abs(p[axis]-roundtrip[axis])<1e-8);
}
const source={stock:{min:{x:-5,y:-3,z:-2},max:{x:5,y:3,z:2}},wcs:wcsFromAngles(origin,23,41,79)} as CamSetupDto;
// Same tilted frame must not inflate the original box through a world AABB.
const same=restStockToSetup(source,source.wcs);
for(const bound of ['min','max'] as const) for(const axis of ['x','y','z'] as const)
 assert.ok(Math.abs(same[bound][axis]-source.stock[bound][axis])<1e-8);
const to=wcsFromAngles({x:-3,y:8,z:1},-51,37,21);
const moved=restStockToSetup(source,to);
for(const x of [-5,5]) for(const y of [-3,3]) for(const z of [-2,2]) {
 const p=modelPointToSetup(setupPointToModel({x,y,z},source.wcs),to);
 for(const axis of ['x','y','z'] as const) assert.ok(p[axis]>=moved.min[axis]-1e-8 && p[axis]<=moved.max[axis]+1e-8);
}
console.log('PASS: arbitrary WCS round trips including gimbal lock, and direct source-to-setup stock bounds');
