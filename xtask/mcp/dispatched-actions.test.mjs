import assert from 'node:assert/strict';
import test from 'node:test';
import {dispatchedActions} from './dispatched-actions.mjs';

test('dispatcher discovery parses TypeScript and ignores comments, strings and other switches', async () => {
  const source = `export function dispatch(action: 'save' | 'extrude', other: string) {
    const example = "case 'string-only':";
    // case 'comment-only':
    switch (action) {
      case 'save': switch (other) { case 'other-switch': break; } break;
      case 'extrude': break;
      default: break;
    }
  }`;
  assert.deepEqual([...await dispatchedActions(source)], ['save', 'extrude']);
});

test('malformed dispatch source fails instead of silently dropping action coverage', async () => {
  await assert.rejects(dispatchedActions('function broken( { switch (action) {'));
});
