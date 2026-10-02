/**
 * Reproducible Linux package entry point.
 *
 *   node scripts/bundle-linux.mjs [deb|appimage]
 *
 * With no argument it builds both. The Debian package intentionally consumes
 * Ubuntu 26.04's OCCT 7.9 runtime. The AppImage is self-contained through the native
 * linuxdeploy pass; release CI builds it on Ubuntu 22.04 against OCCT built
 * from source (scripts/build-occt-linux.sh with OCCT_ROOT pointing at it) so
 * it also runs on distributions with an older glibc. Both packages carry the
 * project and third-party license notices.
 */
import { execFileSync } from 'node:child_process';
import { parseAppImageListing } from './ci/appimage-listing.mjs';
import {
  chmodSync,
  mkdtempSync,
  rmSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { stageXkbRuntime, verifyXkbRuntime } from './linux-xkb-runtime.mjs';
import { buildNativeLinux } from './native-linux-package.mjs';

const options = new Set(process.argv.slice(2));
for (const option of options) {
  if (!['--stage-licenses', 'deb', 'appimage'].includes(option)) {
    throw new Error(`Unknown Linux bundle option: ${option}`);
  }
}

if (process.platform !== 'linux') {
  throw new Error('The Linux desktop packages must be built on Linux');
}

const allBundles = ['deb', 'appimage'];
const requested = process.argv.slice(2).filter((option) => option !== '--stage-licenses');
if (requested.length > 1 || requested.some((name) => !allBundles.includes(name))) {
  throw new Error('usage: node scripts/bundle-linux.mjs [deb|appimage]');
}
const bundles = requested.length ? requested : allBundles;

const projectRoot = realpathSync(join(import.meta.dirname, '..'));
const desktopRoot = join(projectRoot, 'src-tauri');
const licenseRoot = join(desktopRoot, 'linux-licenses');
mkdirSync(licenseRoot, { recursive: true });

function firstExisting(paths, label) {
  const found = paths.find((path) => path && existsSync(path));
  if (!found) {
    throw new Error(`${label} was not found; checked:\n${paths.join('\n')}`);
  }
  return found;
}

const occtCopyright = firstExisting(
  [
    process.env.OCCT_COPYRIGHT_FILE,
    process.env.OCCT_ROOT && join(process.env.OCCT_ROOT, 'share/doc/opencascade/copyright'),
    '/usr/share/doc/libocct-foundation-7.9/copyright',
    '/usr/share/doc/libocct-data-exchange-7.9/copyright',
  ],
  'OCCT copyright notice',
);
const lgpl21 = firstExisting(
  ['/usr/share/common-licenses/LGPL-2.1', '/usr/share/common-licenses/LGPL-2'],
  'system LGPL 2.1 text',
);
copyFileSync(occtCopyright, join(licenseRoot, 'OCCT-copyright.txt'));
copyFileSync(lgpl21, join(licenseRoot, 'LGPL-2.1.txt'));
const xkbRuntime = stageXkbRuntime(licenseRoot);

// License staging is independently usable by package audits.
if (options.has('--stage-licenses')) {
  process.exit(0);
}

await buildNativeLinux(licenseRoot, bundles);

function latestArtifact(directory, suffix) {
  const artifacts = readdirSync(directory)
    .filter((name) => name.endsWith(suffix))
    .map((name) => join(directory, name))
    .sort((left, right) => statSync(left).mtimeMs - statSync(right).mtimeMs);
  const artifact = artifacts.at(-1);
  if (!artifact) throw new Error(`No ${suffix} artifact was created under ${directory}`);
  // GitHub replaces spaces in release-asset names. Normalize before hashing so
  // downloaded filenames and their `sha256sum -c` sidecars continue to match.
  const publishedPath = join(dirname(artifact), basename(artifact).replaceAll(' ', '.'));
  if (publishedPath !== artifact) renameSync(artifact, publishedPath);
  return publishedPath;
}

const hostLibraries = ['libwayland-client.so*', 'libwayland-cursor.so*', 'libwayland-egl.so*'];

const targetRoot = process.env.CARGO_TARGET_DIR
  ? resolve(projectRoot, process.env.CARGO_TARGET_DIR)
  : join(desktopRoot, 'target');
const bundleRoot = join(targetRoot, 'release', 'bundle');
const deb = bundles.includes('deb') ? latestArtifact(join(bundleRoot, 'deb'), '.deb') : null;
const appImage = bundles.includes('appimage')
  ? latestArtifact(join(bundleRoot, 'appimage'), '.AppImage')
  : null;
const requiredNotices = [
  'noBS-CAD-LICENSE.txt',
  'THIRD_PARTY_NOTICES.md',
  'OCCT-LGPL-2.1.txt',
  'OCCT-copyright.txt',
  'runtime.json',
  ...new Set(xkbRuntime.flatMap((entry) => [entry.copyright, ...entry.commonLicenses])),
];

function auditDeb(deb) {
  const debListing = execFileSync('dpkg-deb', ['--contents', deb], {
    encoding: 'utf8',
  });
  for (const notice of requiredNotices) {
    if (!debListing.includes(`/licenses/${notice}`) && !debListing.includes(`/licenses/xkb/${notice}`)) {
      throw new Error(`Required license notice is missing from the Debian package: ${notice}`);
    }
  }
}

function auditAppImage(appImage) {
  chmodSync(appImage, 0o755);
  // Read the modes stored in the squashfs image, which is what a mounted
  // AppImage presents. `--appimage-extract` is not a faithful view: newer
  // AppImage runtimes extract every directory as 0700.
  const offset = execFileSync(appImage, ['--appimage-offset'], { encoding: 'utf8' }).trim();
  const listing = execFileSync('unsquashfs', ['-o', offset, '-lln', appImage], {
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
  const entries = parseAppImageListing(listing);
  if (!entries.some(({ path }) => path === 'AppRun')) {
    throw new Error(`Could not read the AppImage file listing:\n${listing.slice(0, 2000)}`);
  }
  // The mounted image keeps the build user's uid, so everyone else runs it as
  // "other": everything must be readable by others, and anything executable by
  // its owner (directories, AppRun, binaries) executable by others too.
  const unusable = entries
    .filter(({ type, permissions }) => {
      if (type === 'l') return false;
      const searchable = type === 'd' || 'xs'.includes(permissions[2]);
      return permissions[6] !== 'r' || (searchable && !'xt'.includes(permissions[8]));
    })
    .map(({ type, permissions, path }) => `${type}${permissions} ${path}`);
  if (unusable.length) {
    throw new Error(
      `AppImage contains files other users cannot read or execute:\n${unusable.join('\n')}`,
    );
  }
  const extractionRoot = mkdtempSync(join(tmpdir(), 'nbcad-appimage-audit-'));
  try {
    execFileSync('unsquashfs', ['-o', offset, '-d', join(extractionRoot, 'root'), appImage], { stdio: 'pipe' });
    verifyXkbRuntime(join(extractionRoot, 'root'), xkbRuntime);
  } finally {
    rmSync(extractionRoot, { recursive: true, force: true });
  }
  for (const notice of requiredNotices) {
    if (!entries.some(({ type, path }) => type === '-' && basename(path) === notice)) {
      throw new Error(`Required license notice is missing from the AppImage: ${notice}`);
    }
  }
  const hostOnly = entries
    .map(({ path }) => path)
    .filter((path) => hostLibraries.some((pattern) => basename(path).startsWith(pattern.slice(0, -1))));
  if (hostOnly.length) {
    throw new Error(`AppImage bundles libraries it must take from the host:\n${hostOnly.join('\n')}`);
  }
}

if (deb) auditDeb(deb);
if (appImage) auditAppImage(appImage);

function writeChecksum(path) {
  const hash = createHash('sha256');
  hash.update(readFileSync(path));
  const checksumPath = `${path}.sha256`;
  writeFileSync(checksumPath, `${hash.digest('hex')}  ${basename(path)}\n`);
  return checksumPath;
}

for (const [label, artifact] of [['Debian package', deb], ['AppImage', appImage]]) {
  if (!artifact) continue;
  console.log(`Verified ${label}: ${artifact}`);
  console.log(`Checksum: ${writeChecksum(artifact)}`);
}
