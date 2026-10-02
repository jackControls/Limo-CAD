// TypeScript 7 ships no stable compiler API. Use Vite's existing Rust-backed
// transform/parser instead of adding a second, older TypeScript compiler.
import {parseAst, transformWithOxc} from 'vite';

export async function dispatchedActions(source) {
  const {code} = await transformWithOxc(source, 'dispatch.ts', {target: 'esnext'});
  const actions = new Set();
  function visit(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'SwitchStatement' && node.discriminant.type === 'Identifier'
      && node.discriminant.name === 'action') {
      for (const clause of node.cases) {
        if (clause.test?.type === 'Literal' && typeof clause.test.value === 'string') {
          actions.add(clause.test.value);
        }
      }
    }
    for (const child of Object.values(node)) {
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  }
  visit(parseAst(code));
  return actions;
}
