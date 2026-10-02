import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import { parseLdd, verifyNeeded } from '../linux-xkb-runtime.mjs';

test('resolve dlopened runtime closure and retain non-glibc transitive dependencies', () => {
  const dependencies = parseLdd(`linux-vdso.so.1 (0x00007fff)
    libxkbcommon.so.0 => /lib/x86_64-linux-gnu/libxkbcommon.so.0 (0x00100000)
    libxcb-xkb.so.1 => /lib/x86_64-linux-gnu/libxcb-xkb.so.1 (0x00200000)
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00300000)
    /lib64/ld-linux-x86-64.so.2 (0x00400000)`);
  assert.equal(dependencies.size, 3);
  assert.equal(dependencies.get('libxcb-xkb.so.1'), '/lib/x86_64-linux-gnu/libxcb-xkb.so.1');
  verifyNeeded('libxkbcommon-x11.so.0', ['libxkbcommon.so.0', 'libxcb-xkb.so.1', 'libc.so.6'], new Set(dependencies.keys()));
  assert.throws(() => verifyNeeded('libxcb-xkb.so.1', ['libxcb.so.1'], new Set(dependencies.keys())), /missing bundled.*libxcb/);
});

test('unresolved or unrecognized dependency reports fail closed', () => {
  assert.throws(() => parseLdd('libxcb-xkb.so.1 => not found'), /Cannot resolve/);
  assert.throws(() => parseLdd('not a dynamic executable'), /Cannot resolve/);
});
