/** Build the native Cargo binary, portable .app, and signed DMG without a web host. */
import { execFileSync } from 'node:child_process';
import { chmodSync, copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, join } from 'node:path';
import { desktopRoot, projectRoot, stageProjectNotices, targetRoot, version } from './desktop-package.mjs';

if (process.platform !== 'darwin') throw new Error('The macOS app bundle must be built on macOS');
const identity = process.env.APPLE_SIGNING_IDENTITY?.trim() || '-';
const production = identity !== '-';
const run = (command, args, options = {}) => execFileSync(command, args, { cwd: projectRoot, stdio: 'inherit', ...options });
const stage = join(desktopRoot, 'occt-libs');
run(process.execPath, [join(projectRoot, 'scripts/stage-occt-macos.mjs')]);
run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--release', '--bin', 'nbcad'], {
  env: { ...process.env, NBCAD_OCCT_LIB_DIR: stage },
});
const bundle = join(targetRoot, 'release/bundle');
const app = join(bundle, 'macos/noBS CAD.app');
rmSync(app, { recursive: true, force: true });
const contents = join(app, 'Contents');
for (const directory of ['MacOS', 'Resources', 'Frameworks']) mkdirSync(join(contents, directory), { recursive: true });
const executable = join(contents, 'MacOS/nbcad');
copyFileSync(join(targetRoot, 'release/nbcad'), executable);
chmodSync(executable, 0o755);
copyFileSync(join(desktopRoot, 'icons/icon.icns'), join(contents, 'Resources/icon.icns'));
stageProjectNotices(join(contents, 'Resources/licenses'));
cpSync(join(stage, 'licenses'), join(contents, 'Resources/licenses'), { recursive: true });
const libraries = JSON.parse(readFileSync(join(stage, 'libraries.json'), 'utf8'));
for (const library of libraries) copyFileSync(join(stage, library), join(contents, 'Frameworks', library));
writeFileSync(join(contents, 'Info.plist'), `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>org.nbcad.desktop</string>
<key>CFBundleName</key><string>noBS CAD</string>
<key>CFBundleDisplayName</key><string>noBS CAD</string>
<key>CFBundleExecutable</key><string>nbcad</string>
<key>CFBundleIconFile</key><string>icon.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>${version}</string>
<key>CFBundleVersion</key><string>${version}</string>
<key>LSMinimumSystemVersion</key><string>12.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>CFBundleURLTypes</key><array><dict><key>CFBundleURLName</key><string>noBS CAD recipe</string><key>CFBundleURLSchemes</key><array><string>nbcad</string></array></dict></array>
</dict></plist>\n`);

// Link against staged @rpath names, then resolve them from the installed app.
const loadCommands = execFileSync('otool', ['-l', executable], { encoding: 'utf8' });
if (!loadCommands.includes('@executable_path/../Frameworks')) {
  run('install_name_tool', ['-add_rpath', '@executable_path/../Frameworks', executable]);
}
const dependencies = execFileSync('otool', ['-L', executable], { encoding: 'utf8' })
  .split('\n').slice(1).map(line => line.trim().split(' (compatibility version')[0]).filter(Boolean);
for (const dependency of dependencies) {
  if (dependency.startsWith('/usr/lib/') || dependency.startsWith('/System/Library/')) continue;
  if (!libraries.includes(basename(dependency))) throw new Error(`Unbundled executable dependency: ${dependency}`);
  if (!dependency.startsWith('@rpath/')) run('install_name_tool', ['-change', dependency, `@rpath/${basename(dependency)}`, executable]);
}
const signing = ['--force', '--sign', identity, ...(production ? ['--options', 'runtime', '--timestamp'] : [])];
for (const library of libraries) run('codesign', [...signing, join(contents, 'Frameworks', library)]);
run('codesign', [...signing, app]);
run('codesign', ['--verify', '--deep', '--strict', app]);
const notarize = (path) => {
  for (const key of ['APPLE_API_KEY_PATH', 'APPLE_API_KEY', 'APPLE_API_ISSUER']) {
    if (!process.env[key]) throw new Error(`Production notarization requires ${key}`);
  }
  run('xcrun', ['notarytool', 'submit', path, '--key', process.env.APPLE_API_KEY_PATH,
    '--key-id', process.env.APPLE_API_KEY, '--issuer', process.env.APPLE_API_ISSUER, '--wait']);
};
if (production) {
  const archive = join(bundle, 'macos/notarization.zip');
  rmSync(archive, { force: true });
  run('ditto', ['-c', '-k', '--keepParent', app, archive]);
  notarize(archive);
  run('xcrun', ['stapler', 'staple', app]);
  rmSync(archive, { force: true });
}

mkdirSync(join(bundle, 'dmg'), { recursive: true });
const architecture = process.arch === 'arm64' ? 'aarch64' : 'x64';
const dmg = join(bundle, 'dmg', `noBS.CAD_${version}_${architecture}.dmg`);
const imageRoot = mkdtempSync(join(tmpdir(), 'nbcad-dmg-'));
try {
  cpSync(app, join(imageRoot, 'noBS CAD.app'), { recursive: true });
  symlinkSync('/Applications', join(imageRoot, 'Applications'));
  run('hdiutil', ['create', '-volname', 'noBS CAD', '-srcfolder', imageRoot, '-ov', '-format', 'UDZO', dmg]);
} finally { rmSync(imageRoot, { recursive: true, force: true }); }
if (production) {
  run('codesign', [...signing, dmg]);
  notarize(dmg);
  run('xcrun', ['stapler', 'staple', dmg]);
}
run('hdiutil', ['verify', dmg]);
console.log(`Native app: ${app}\nDisk image: ${dmg}`);
