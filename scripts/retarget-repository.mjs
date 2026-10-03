// Point repository links at a renamed or transferred GitHub repository.
//   node scripts/retarget-repository.mjs --to owner/repo            (dry run)
//   node scripts/retarget-repository.mjs --to owner/repo --write
// Rewrites the slug in github.com / raw.githubusercontent.com / api.github.com
// URLs and the project Pages URL in tracked text files. It does not touch
// release-note history, lockfiles, or release artifact file names, which follow
// their own release steps (see docs/limo-rename-checklist.md).
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { escapeRegExp, repository } from './repository.mjs';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const skipped = [/^docs\/release-notes\//, /^scripts\/retarget-repository\.test\.mjs$/, /(^|\/)package-lock\.json$/, /(^|\/)Cargo\.lock$/, /\.(png|jpe?g|gif|webp|ico|icns|svg|mp4|zip|nbcad|woff2?|wasm|dmg|deb|pdf)$/i];
const slug = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;
const pagesOf = value => { const [owner, name] = value.split('/'); return `${owner.toLowerCase()}.github.io/${name}`; };

// Pure: returns the rewritten text. Case matters for the slug, not the Pages host.
// The slug may end a URL, continue with a path, or carry a `.git` suffix; a
// longer repository name (`noBS-CAD-fork`) is a different repository.
export function retarget(text, { from, to, pagesFrom = pagesOf(from), pagesTo = pagesOf(to) }) {
  const tail = String.raw`(?=(?:\.git)?(?![A-Za-z0-9_.-]))`;
  const prefixes = String.raw`(https://(?:github\.com|raw\.githubusercontent\.com|api\.github\.com/repos)/|https://img\.shields\.io/github/(?:[A-Za-z0-9_.-]+/)*?|git@github\.com:|\x60)`;
  return text
    .replace(new RegExp(prefixes + escapeRegExp(from) + tail, 'g'), (_, prefix) => prefix + to)
    .replace(new RegExp(String.raw`(https://)${escapeRegExp(pagesFrom)}${tail}`, 'gi'), (_, scheme) => scheme + pagesTo);
}

// Old-slug mentions the rewrite left behind, for the operator to resolve by hand.
export function leftovers(text, from) {
  const pattern = new RegExp(escapeRegExp(from) + String.raw`(?![A-Za-z0-9_-])`, 'i');
  return text.split('\n').flatMap((line, index) => pattern.test(line) ? [index + 1] : []);
}

export function plan(files, read, options) {
  return files.flatMap(file => {
    if (skipped.some(pattern => pattern.test(file))) return [];
    const before = read(file);
    if (before === null || before.includes('\0')) return [];
    const after = retarget(before, options);
    return after === before ? [] : [{ file, before, after }];
  });
}

const usage = 'Usage: retarget-repository.mjs --to owner/repo [--pages-url host/path] [--write]';

export function parseArgs(argv) {
  const value = flag => {
    const i = argv.indexOf(flag);
    if (i < 0) return undefined;
    const next = argv[i + 1];
    if (!next || next.startsWith('--')) throw new Error(`${flag} needs a value. ${usage}`);
    return next;
  };
  const to = value('--to');
  if (!to || !slug.test(to)) throw new Error(usage);
  const pagesTo = value('--pages-url')?.replace(/^https?:\/\//i, '').replace(/\/+$/, '');
  return { to, pagesTo, write: argv.includes('--write') };
}

function main(argv) {
  const { to, pagesTo, write } = parseArgs(argv);
  const options = { from: repository, to, ...(pagesTo && { pagesTo }) };
  const files = execFileSync('git', ['ls-files', '-z'], { cwd: root, encoding: 'utf8' }).split('\0').filter(Boolean);
  const read = file => { try { return readFileSync(path.join(root, file), 'utf8'); } catch { return null; } };
  const changes = plan(files, read, options);
  for (const { file, after } of changes) {
    console.log(`${write ? 'updated' : 'would update'} ${file}`);
    if (write) writeFileSync(path.join(root, file), after);
  }
  console.log(`${changes.length} file(s) ${write ? 'rewritten' : 'to rewrite'}: ${options.from} -> ${options.to}`);
  const rewritten = new Map(changes.map(change => [change.file, change.after]));
  const remaining = files.flatMap(file => {
    if (skipped.some(pattern => pattern.test(file))) return [];
    const text = rewritten.get(file) ?? read(file);
    if (text === null || text.includes('\0')) return [];
    return leftovers(text, options.from).map(line => `${file}:${line}`);
  });
  if (remaining.length) console.log(`Still mentions ${options.from}; review by hand:\n  ${remaining.join('\n  ')}`);
  if (!write) console.log('Dry run. Re-run with --write to apply.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
