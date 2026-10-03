import assert from 'node:assert/strict';
import test from 'node:test';
import { leftovers, parseArgs, plan, retarget } from './retarget-repository.mjs';
import { repository, slugFrom } from './repository.mjs';

const options = { from: 'jackControls/noBS-CAD', to: 'limo-cad/limo-cad' };

test('rewrites repository, raw, API and Pages URLs', () => {
  const text = [
    '[repo](https://github.com/jackControls/noBS-CAD) [issues](https://github.com/jackControls/noBS-CAD/issues)',
    '<img src="https://raw.githubusercontent.com/jackControls/noBS-CAD/v0.2.0/docs/a.png">',
    'https://api.github.com/repos/jackControls/noBS-CAD/releases',
    'https://jackcontrols.github.io/noBS-CAD/showcase.html#garden-bench',
  ].join('\n');
  assert.equal(retarget(text, options), [
    '[repo](https://github.com/limo-cad/limo-cad) [issues](https://github.com/limo-cad/limo-cad/issues)',
    '<img src="https://raw.githubusercontent.com/limo-cad/limo-cad/v0.2.0/docs/a.png">',
    'https://api.github.com/repos/limo-cad/limo-cad/releases',
    'https://limo-cad.github.io/limo-cad/showcase.html#garden-bench',
  ].join('\n'));
});

test('rewrites .git clone URLs, SSH remotes, shields badges and code-quoted slugs', () => {
  const text = [
    '"url": "https://github.com/jackControls/noBS-CAD.git"',
    '"url": "git+https://github.com/jackControls/noBS-CAD.git"',
    'git clone git@github.com:jackControls/noBS-CAD.git',
    '![release](https://img.shields.io/github/v/release/jackControls/noBS-CAD?label=release)',
    '![ci](https://img.shields.io/github/actions/workflow/status/jackControls/noBS-CAD/ci.yml)',
    'branches directly on `jackControls/noBS-CAD` with',
  ].join('\n');
  assert.equal(retarget(text, options), text.replaceAll('jackControls/noBS-CAD', 'limo-cad/limo-cad'));
  assert.equal(retarget('https://github.com/jackControls/noBS-CAD.gitx', options), 'https://github.com/jackControls/noBS-CAD.gitx');
});

test('lists old-slug mentions the rewrite leaves behind', () => {
  assert.deepEqual(leftovers('ok\nsee jackcontrols/nobs-cad\njackControls/noBS-CAD-fork', 'jackControls/noBS-CAD'), [2]);
});

test('parses arguments and normalizes the Pages URL', () => {
  assert.deepEqual(parseArgs(['--to', 'o/r', '--pages-url', 'https://limo.example/cad/', '--write']), { to: 'o/r', pagesTo: 'limo.example/cad', write: true });
  assert.throws(() => parseArgs(['--to', 'o/r', '--pages-url', '--write']), /--pages-url needs a value/);
  assert.throws(() => parseArgs(['--to', 'not a slug']));
});

test('keeps artifact file names, other repositories and longer names intact', () => {
  const text = [
    'https://github.com/jackControls/noBS-CAD/releases/download/v0.2.2/noBS-CAD-0.2.2-windows-x64.zip',
    'https://github.com/jackControls/noBS-CAD-fork',
    'https://github.com/jackControls/other',
    'noBS-CAD-LICENSE.txt',
  ].join('\n');
  const result = retarget(text, options).split('\n');
  assert.equal(result[0], 'https://github.com/limo-cad/limo-cad/releases/download/v0.2.2/noBS-CAD-0.2.2-windows-x64.zip');
  assert.deepEqual(result.slice(1), text.split('\n').slice(1));
});

test('accepts a custom Pages URL', () => {
  assert.equal(
    retarget('https://jackcontrols.github.io/noBS-CAD/open.html', { ...options, pagesTo: 'limo.example/cad' }),
    'https://limo.example/cad/open.html',
  );
});

test('skips release history, lockfiles and binaries; reports only changed files', () => {
  const files = { 'README.md': 'https://github.com/jackControls/noBS-CAD', 'docs/release-notes/v0.2.2.md': 'https://github.com/jackControls/noBS-CAD', 'package-lock.json': 'https://github.com/jackControls/noBS-CAD', 'a.png': 'https://github.com/jackControls/noBS-CAD', 'clean.md': 'nothing' };
  const changes = plan(Object.keys(files), file => files[file], options);
  assert.deepEqual(changes.map(change => change.file), ['README.md']);
});

test('the repository slug comes from package.json', () => {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/);
  assert.equal(slugFrom('https://github.com/o/r.git'), 'o/r');
  assert.equal(slugFrom('https://github.com/o/r#readme'), 'o/r');
  assert.equal(slugFrom('git+https://github.com/o/r.git'), 'o/r');
  assert.throws(() => slugFrom('git@github.com:o/r.git'));
});
