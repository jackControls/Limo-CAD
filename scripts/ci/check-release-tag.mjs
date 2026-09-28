#!/usr/bin/env node
// A `v*` tag builds and publishes a release from its own commit with
// `contents: write`, so the tag itself has to be legitimate: it names the
// VERSION recorded on that commit, and the commit already landed on main, so a
// tag cannot ship code that never passed review. `version_preflight` runs this
// before the package matrix starts; `publish_release` repeats it before it
// writes the release.
//
//   node scripts/ci/check-release-tag.mjs <tag> <sha>
//
// The checkout may be shallow: main is fetched, and the history completed,
// before the ancestry test.

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

function git(cwd, ...args) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  if (result.error) throw result.error;
  return result;
}

function gitOutput(cwd, ...args) {
  const result = git(cwd, ...args);
  if (result.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed: ${result.stderr.trim()}`);
  }
  return result.stdout;
}

/** Bring `<remote>/<branch>` into the checkout, completing a shallow history so ancestry is decidable. */
export function fetchMain(cwd, { remote = 'origin', branch = 'main' } = {}) {
  const shallow = gitOutput(cwd, 'rev-parse', '--is-shallow-repository').trim() === 'true';
  const ref = `refs/remotes/${remote}/${branch}`;
  gitOutput(
    cwd,
    'fetch', '--quiet', '--no-tags',
    ...(shallow ? ['--unshallow'] : []),
    remote, `+refs/heads/${branch}:${ref}`,
  );
  return ref;
}

/** Why `tag` at `sha` must not publish, as messages; empty when it may. */
export function releaseTagProblems({ tag, sha, mainRef, cwd = process.cwd() }) {
  const problems = [];
  if (!/^v\d/.test(tag)) {
    problems.push(`${tag} is not a release tag; release tags are v<VERSION>`);
  }
  const shown = git(cwd, 'show', `${sha}:VERSION`);
  if (shown.status !== 0) {
    problems.push(`${sha} carries no VERSION file: ${shown.stderr.trim()}`);
  } else {
    const version = shown.stdout.trim();
    if (tag !== `v${version}`) {
      problems.push(`${tag} does not name the VERSION on its commit (${version}); tag the commit whose VERSION is ${tag.replace(/^v/, '')}`);
    }
  }
  const ancestry = git(cwd, 'merge-base', '--is-ancestor', sha, mainRef);
  if (ancestry.status === 1) {
    problems.push(`${sha} is not on main (${mainRef}); tag the merge commit after the bump has landed`);
  } else if (ancestry.status !== 0) {
    problems.push(`could not decide whether ${sha} is on main: ${ancestry.stderr.trim()}`);
  }
  return problems;
}

function main() {
  const [tag, sha] = process.argv.slice(2);
  if (!tag || !sha) {
    console.error('usage: node scripts/ci/check-release-tag.mjs <tag> <sha>');
    process.exitCode = 2;
    return;
  }
  const mainRef = fetchMain(process.cwd());
  const problems = releaseTagProblems({ tag, sha, mainRef });
  for (const problem of problems) console.error(`::error::${problem}`);
  if (problems.length > 0) {
    process.exitCode = 1;
    return;
  }
  console.log(`${tag} names the VERSION at ${sha}, which is on main; the tag may publish.`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
