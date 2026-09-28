import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import test from 'node:test';
import { fetchMain, releaseTagProblems } from './check-release-tag.mjs';

const identity = {
  GIT_AUTHOR_NAME: 'ci', GIT_AUTHOR_EMAIL: 'ci@example.invalid',
  GIT_COMMITTER_NAME: 'ci', GIT_COMMITTER_EMAIL: 'ci@example.invalid',
};

function git(cwd, ...args) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8', env: { ...process.env, ...identity } });
  assert.equal(result.status, 0, `git ${args.join(' ')}: ${result.stderr}`);
  return result.stdout.trim();
}

async function commitVersion(repo, version, message) {
  await writeFile(path.join(repo, 'VERSION'), `${version}\n`);
  git(repo, 'add', 'VERSION');
  git(repo, 'commit', '-q', '-m', message);
  return git(repo, 'rev-parse', 'HEAD');
}

test('a release tag must name VERSION on a commit that main already contains', async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), 'nbcad-release-tag-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  const origin = path.join(root, 'origin');
  git(root, 'init', '-q', '-b', 'main', origin);
  const released = await commitVersion(origin, '0.9.0', 'release 0.9.0');
  git(origin, 'tag', '-a', 'v0.9.0', '-m', 'noBS CAD 0.9.0');
  // main moves on afterwards; the released commit stays one of its ancestors.
  await commitVersion(origin, '0.9.1', 'bump to 0.9.1');
  git(origin, 'checkout', '-q', '-b', 'feature');
  const stray = await commitVersion(origin, '0.9.2', 'work main never merged');
  git(origin, 'tag', '-a', 'v0.9.2', '-m', 'points off main');
  git(origin, 'checkout', '-q', 'main');
  git(origin, 'tag', '-a', 'v9.9.9', '-m', 'wrong number', released);

  // The tag build checks out the tag alone at depth 1, as actions/checkout does.
  for (const [tag, sha, expected] of [
    ['v0.9.0', released, []],
    ['v9.9.9', released, [/does not name the VERSION on its commit \(0\.9\.0\)/]],
    ['v0.9.2', stray, [/is not on main/]],
  ]) {
    const clone = path.join(root, `clone-${tag}`);
    git(root, 'clone', '-q', '--depth', '1', '--branch', tag, pathToFileURL(origin).href, clone);
    assert.equal(git(clone, 'rev-parse', '--is-shallow-repository'), 'true');
    const mainRef = fetchMain(clone);
    assert.equal(mainRef, 'refs/remotes/origin/main');
    const problems = releaseTagProblems({ tag, sha, mainRef, cwd: clone });
    assert.equal(problems.length, expected.length, `${tag}: ${problems.join('\n')}`);
    expected.forEach((pattern, index) => assert.match(problems[index], pattern));
  }

  const clone = path.join(root, 'clone-v0.9.0');
  const [problem] = releaseTagProblems({ tag: 'showcase-v0.9.0', sha: released, mainRef: 'refs/remotes/origin/main', cwd: clone });
  assert.match(problem, /not a release tag/);
});
