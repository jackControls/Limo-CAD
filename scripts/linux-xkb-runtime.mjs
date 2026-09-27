// Winit dlopens this SONAME; ordinary DT_NEEDED discovery cannot find it.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { copyFileSync, existsSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from 'node:fs';
import { basename, join, relative, sep } from 'node:path';

export const xkbSoname = 'libxkbcommon-x11.so.0';
// Keep the platform loader/glibc on the supported Ubuntu baseline. Every other
// dependency of the dlopened library must be in the AppImage, including XCB.
const platformLibrary = /^(?:libc\.so\.6|libm\.so\.6|libpthread\.so\.0|libdl\.so\.2|librt\.so\.1|ld-linux[^/]*|linux-vdso[^/]*)$/;

export function parseLdd(output) {
  const found = new Map();
  for (const line of output.trim().split('\n')) {
    if (!line.trim()) continue;
    const linked = line.trim().match(/^(\S+)\s+=>\s+(\/\S+)\s+\(0x[\da-f]+\)$/i);
    if (linked) {
      if (found.has(linked[1]) && found.get(linked[1]) !== linked[2]) {
        throw new Error(`Conflicting XKB dependency paths for ${linked[1]}`);
      }
      found.set(linked[1], linked[2]);
      continue;
    }
    const direct = line.trim().match(/^(\/\S+|linux-vdso\S+)\s+\(0x[\da-f]+\)$/i);
    if (direct && platformLibrary.test(basename(direct[1]))) continue;
    throw new Error(`Cannot resolve XKB dependency: ${line.trim()}`);
  }
  return found;
}

export function verifyNeeded(soname, needed, names) {
  for (const dependency of needed) {
    if (!platformLibrary.test(dependency) && !names.has(dependency)) {
      throw new Error(`${soname} needs missing bundled XKB dependency ${dependency}`);
    }
  }
}

function run(command, args, options = {}) {
  return execFileSync(command, args, { encoding: 'utf8', ...options }).trim();
}
function needed(path) {
  return run('patchelf', ['--print-needed', path]).split('\n').filter(Boolean);
}
function owner(path) {
  for (const candidate of [path, path.replace(/^\/usr\/lib\//, '/lib/')]) {
    try {
      const line = run('dpkg-query', ['--search', candidate]).split('\n')[0];
      const split = line.indexOf(': /');
      if (split > 0) return line.slice(0, split);
    } catch { /* Try Ubuntu's pre-usrmerge spelling. */ }
  }
  throw new Error(`No Ubuntu package owns XKB runtime ${path}`);
}

export function stageXkbRuntime(licenseRoot) {
  const libdir = run('pkg-config', ['--variable=libdir', 'xkbcommon-x11']);
  const seed = realpathSync(join(libdir, xkbSoname));
  const closure = parseLdd(run('ldd', [seed], {
    env: { ...process.env, LD_LIBRARY_PATH: '', LD_PRELOAD: '', LD_AUDIT: '' },
  }));
  closure.set(xkbSoname, seed);
  for (const soname of closure.keys()) if (platformLibrary.test(soname)) closure.delete(soname);
  const names = new Set(closure.keys());
  for (const [soname, path] of closure) verifyNeeded(soname, needed(path), names);

  const stage = join(realpathSync(licenseRoot), 'xkb');
  // This generated child belongs to the existing ignored license staging root.
  rmSync(stage, { recursive: true, force: true });
  mkdirSync(join(stage, 'lib'), { recursive: true });
  mkdirSync(join(stage, 'licenses'), { recursive: true });
  const manifest = [];
  for (const [soname, path] of [...closure].sort(([a], [b]) => a.localeCompare(b))) {
    const source = realpathSync(path);
    const packageName = owner(source);
    const plainName = packageName.split(':')[0];
    const copyright = join('/usr/share/doc', plainName, 'copyright');
    if (!existsSync(copyright)) throw new Error(`Missing copyright notice for ${packageName}`);
    copyFileSync(source, join(stage, 'lib', soname));
    copyFileSync(copyright, join(stage, 'licenses', `${plainName}-copyright.txt`));
    const commonLicenses = [...new Set([...readFileSync(copyright, 'utf8')
      .matchAll(/\/usr\/share\/common-licenses\/([A-Za-z0-9.+-]+)/g)]
      .map((match) => match[1].replace(/[.,]+$/, '')))];
    for (const name of commonLicenses) {
      copyFileSync(join('/usr/share/common-licenses', name), join(stage, 'licenses', name));
    }
    manifest.push({
      soname,
      package: packageName,
      version: run('dpkg-query', ['--show', '--showformat=${Version}', packageName]),
      sha256: createHash('sha256').update(readFileSync(source)).digest('hex'),
      copyright: `${plainName}-copyright.txt`,
      commonLicenses,
    });
  }
  writeFileSync(join(stage, 'licenses', 'runtime.json'), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

export function verifyXkbRuntime(appDir, manifest) {
  const libraryRoot = realpathSync(join(appDir, 'usr/lib'));
  const names = new Set(manifest.map(({ soname }) => soname));
  for (const { soname } of manifest) {
    const path = join(libraryRoot, soname);
    if (!existsSync(path)) throw new Error(`AppImage is missing dlopened XKB runtime ${soname}`);
    const rel = relative(libraryRoot, realpathSync(path));
    if (rel === '..' || rel.startsWith(`..${sep}`)) throw new Error(`XKB runtime escapes AppImage: ${soname}`);
    verifyNeeded(soname, needed(path), names);
  }
  const resolved = parseLdd(run('ldd', [join(libraryRoot, xkbSoname)], {
    env: { ...process.env, LD_LIBRARY_PATH: libraryRoot, LD_PRELOAD: '', LD_AUDIT: '' },
  }));
  for (const [soname, path] of resolved) {
    if (platformLibrary.test(soname)) continue;
    const rel = relative(libraryRoot, realpathSync(path));
    if (rel === '..' || rel.startsWith(`..${sep}`)) {
      throw new Error(`AppImage XKB runtime falls back to host dependency ${soname}: ${path}`);
    }
  }
}
