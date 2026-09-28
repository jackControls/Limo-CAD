import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {buildManufacturingProfileDxf} from '../src/drawing/dxf';
import type {ProfileCatalogItemDto} from '../src/engine/types';

// The native writer consumes this same catalog contract. This independently
// checks analytic entities and nested-region selection in the release shell.
const catalog = JSON.parse(readFileSync(new URL('../crates/export/tests/fixtures/profile-dxf.json', import.meta.url), 'utf8')) as ProfileCatalogItemDto;
function entities(text: string): Array<Array<[string, string]>> {
  const lines = text.trimEnd().split(/\r?\n/);
  const pairs: Array<[string, string]> = [];
  for (let i = 0; i < lines.length; i += 2) pairs.push([lines[i], lines[i + 1]]);
  const start = pairs.findIndex(([code, value]) => code === '2' && value === 'ENTITIES');
  const rows: Array<Array<[string, string]>> = [];
  for (const pair of pairs.slice(start + 1)) {
    if (pair[0] === '0' && pair[1] === 'ENDSEC') break;
    if (pair[0] === '0') rows.push([]);
    rows.at(-1)!.push(pair);
  }
  return rows;
}
const value = (row: Array<[string, string]>, code: string) => row.find(p => p[0] === code)![1];
const outer = entities(buildManufacturingProfileDxf(catalog, 0));
assert.equal(outer.length, 2);
for (const [row, radius, layer] of [[outer[0], '10', 'PROFILE_OUTER'], [outer[1], '4', 'PROFILE_HOLES']] as const) {
  for (const [code, expected] of [['0', 'CIRCLE'], ['10', '-5'], ['20', '20'], ['40', radius], ['8', layer]]) assert.equal(value(row, code), expected);
}
assert.throws(() => buildManufacturingProfileDxf(catalog, 1));
const island = entities(buildManufacturingProfileDxf(catalog, 2));
assert.equal(island.length, 1); assert.equal(value(island[0], '40'), '2');
for (const [index, start, end] of [[3, 0, 180], [4, 180, 0]]) {
  const arc = entities(buildManufacturingProfileDxf(catalog, index))[0];
  assert.equal(value(arc, '0'), 'ARC');
  for (const [code, expected] of [['10', 40], ['20', 0], ['40', 5], ['50', start], ['51', end]] as const) assert.equal(Number(value(arc, code)), expected);
}
const polygon = entities(buildManufacturingProfileDxf(catalog, 5))[0];
assert.equal(value(polygon, '0'), 'LWPOLYLINE'); assert.equal(value(polygon, '70'), '1'); assert.equal(value(polygon, '90'), '3');
console.log('PASS shared native/React profile DXF contract: analytic curves, hole wires, nested islands, local millimetres');
