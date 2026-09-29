/**
 * Reproducible Linux package entry point.
 *
 *   node scripts/bundle-linux.mjs [deb|appimage]
 *
 * With no argument it builds both. The Debian package intentionally consumes
 * Ubuntu 26.04's OCCT 7.9 runtime. The AppImage is self-contained by Tauri's
 * linuxdeploy pass; release CI builds it on Ubuntu 22.04 against OCCT built
 * from source (scripts/build-occt-linux.sh with OCCT_ROOT pointing at it) so
 * it also runs on distributions with an older glibc. Both packages carry the
 * project and third-party license notices.
 */
import { execFileSync } from 'node:child_process';
import {
  chmodSync,
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
import { homedir } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';

if (process.platform !== 'linux') {
  throw new Error('The Linux desktop packages must be built on Linux');
}

const allBundles = ['deb', 'appimage'];
const requested = process.argv.slice(2);
if (requested.length > 1 || requested.some((name) => !allBundles.includes(name))) {
  throw new Error('usage: node scripts/bundle-linux.mjs [deb|appimage]');
}
const bundles = requested.length ? requested : allBundles;

const projectRoot = realpathSync(join(import.meta.dirname, '..'));
const tauriRoot = join(projectRoot, 'src-tauri');
const licenseRoot = join(tauriRoot, 'linux-licenses');
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

// Tauri writes the AppRun it downloads for linuxdeploy with mode 0770 and
// linuxdeploy ships it as AppRun.wrapped. A mounted AppImage keeps the build
// user's uid, so any other user (a sandbox such as firejail, another account)
// cannot execute it and the application never starts. Tauri downloads the
// file only when it is not cached, so seed the cache with a world-executable
// copy. The permission audit below fails the build if this ever stops working.
function seedAppRun() {
  const arch = { x64: 'x86_64', arm64: 'aarch64' }[process.arch];
  if (!arch) throw new Error(`No AppImage AppRun for ${process.arch}`);
  const cache = join(process.env.XDG_CACHE_HOME || join(homedir(), '.cache'), 'tauri');
  const appRun = join(cache, `AppRun-${arch}`);
  if (!existsSync(appRun)) {
    mkdirSync(cache, { recursive: true });
    execFileSync('curl', [
      '--proto', '=https', '--tlsv1.2', '-sSfL', '-o', appRun,
      `https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-${arch}`,
    ]);
  }
  chmodSync(appRun, 0o755);
}
if (bundles.includes('appimage')) seedAppRun();

// The host's GPU drivers (Mesa's Vulkan and EGL drivers) load into the
// application and link the host's libwayland. Newer drivers need newer
// libwayland symbols, so an older bundled copy, which the dynamic loader finds
// first, keeps every driver from loading and the viewport finds no GPU. Every
// Linux desktop provides libwayland, so the AppImage uses the host's.
const hostLibraries = [
  'libwayland-client.so*',
  'libwayland-cursor.so*',
  'libwayland-egl.so*',
  'libwayland-server.so*',
];

execFileSync(
  'npx',
  [
    'tauri',
    'build',
    '--bundles',
    bundles.join(','),
    '--config',
    'src-tauri/tauri.linux.conf.json',
  ],
  {
    cwd: projectRoot,
    stdio: 'inherit',
    env: { ...process.env, LINUXDEPLOY_EXCLUDED_LIBRARIES: hostLibraries.join(';') },
  },
);

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

const targetRoot = process.env.CARGO_TARGET_DIR
  ? resolve(projectRoot, process.env.CARGO_TARGET_DIR)
  : join(tauriRoot, 'target');
const bundleRoot = join(targetRoot, 'release', 'bundle');
const deb = bundles.includes('deb') ? latestArtifact(join(bundleRoot, 'deb'), '.deb') : null;
const appImage = bundles.includes('appimage')
  ? latestArtifact(join(bundleRoot, 'appimage'), '.AppImage')
  : null;
const requiredNotices = [
  'noBS-CAD-LICENSE.txt',
  'THIRD_PARTY_NOTICES.md',
  'OPENCASCADE_JS_LICENSE.txt',
  'OCCT-LGPL-2.1.txt',
  'OCCT-copyright.txt',
];

function auditDeb(deb) {
  const debListing = execFileSync('dpkg-deb', ['--contents', deb], {
    encoding: 'utf8',
  });
  for (const notice of requiredNotices) {
    if (!debListing.includes(`/licenses/${notice}`)) {
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
  const entries = listing
    .split('\n')
    .map((line) => line.match(/^([-dl])([-rwxsStT]{9}) \S+ +\d+ \S+ \S+ squashfs-root\/(.+)$/))
    .filter(Boolean)
    .map(([, type, permissions, path]) => ({ type, permissions, path }));
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
