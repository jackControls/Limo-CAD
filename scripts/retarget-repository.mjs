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
const skipped = [/^docs\/release-notes\//, /(^|\/)package-lock\.json$/, /(^|\/)Cargo\.lock$/, /\.(png|jpe?g|gif|webp|ico|icns|svg|mp4|zip|nbcad|woff2?|wasm|dmg|deb|pdf)$/i];
const slug = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;
const pagesOf = value => { const [owner, name] = value.split('/'); return `${owner.toLowerCase()}.github.io/${name}`; };

// Pure: returns the rewritten text. Case matters for the slug, not the Pages host.
export function retarget(text, { from, to, pagesFrom = pagesOf(from), pagesTo = pagesOf(to) }) {
  const tail = String.raw`(?![A-Za-z0-9_.-])`;
  const hosts = String.raw`(https://(?:github\.com|raw\.githubusercontent\.com|api\.github\.com/repos)/)`;
  return text
    .replace(new RegExp(hosts + escapeRegExp(from) + tail, 'g'), (_, host) => host + to)
    .replace(new RegExp(String.raw`(https://)${escapeRegExp(pagesFrom)}${tail}`, 'gi'), (_, scheme) => scheme + pagesTo);
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

function main(argv) {
  const value = flag => { const i = argv.indexOf(flag); return i < 0 ? undefined : argv[i + 1]; };
  const to = value('--to');
  if (!to || !slug.test(to)) throw new Error('Usage: retarget-repository.mjs --to owner/repo [--pages-url host/path] [--write]');
  const options = { from: repository, to, ...(value('--pages-url') && { pagesTo: value('--pages-url') }) };
  const files = execFileSync('git', ['ls-files', '-z'], { cwd: root, encoding: 'utf8' }).split('\0').filter(Boolean);
  const read = file => { try { return readFileSync(path.join(root, file), 'utf8'); } catch { return null; } };
  const changes = plan(files, read, options);
  for (const { file, before, after } of changes) {
    const count = after.split(options.to).length - before.split(options.to).length;
    console.log(`${argv.includes('--write') ? 'updated' : 'would update'} ${file}${count > 0 ? ` (${count})` : ''}`);
    if (argv.includes('--write')) writeFileSync(path.join(root, file), after);
  }
  console.log(`${changes.length} file(s) ${argv.includes('--write') ? 'rewritten' : 'to rewrite'}: ${options.from} -> ${options.to}`);
  if (!argv.includes('--write')) console.log('Dry run. Re-run with --write to apply.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main(process.argv.slice(2));
