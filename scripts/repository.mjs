// Single source of truth for the GitHub repository slug: package.json.
// Scripts that match or build GitHub URLs read it here, so a repository rename
// is one `scripts/retarget-repository.mjs` run instead of a hunt through regexes.
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const manifest = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'package.json');

export const escapeRegExp = text => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function slugFrom(url) {
  const match = /^https:\/\/github\.com\/([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?(?:[#/].*)?$/.exec(url ?? '');
  if (!match) throw new Error(`package.json repository.url is not a github.com URL: ${url}`);
  return match[1];
}

export const repository = slugFrom(JSON.parse(readFileSync(manifest, 'utf8')).repository?.url);
