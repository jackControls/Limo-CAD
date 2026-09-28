// Native Cargo -> Debian/AppImage packaging. linuxdeploy only copies ELF
// dependencies; it neither supplies a host nor embeds frontend assets.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmodSync, copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { desktopRoot, projectRoot, stageProjectNotices, targetRoot, version } from './desktop-package.mjs';

export async function buildNativeLinux(licenseRoot) {
  if (process.arch !== 'x64') throw new Error('The Ubuntu package target is x86_64');
  const run = (command, args, options = {}) => execFileSync(command, args, { cwd: projectRoot, stdio: 'inherit', ...options });
  run('cargo', ['build', '--manifest-path', 'src-tauri/Cargo.toml', '--locked', '--release', '--bin', 'nbcad']);
  const bundle = join(targetRoot, 'release/bundle');
  const staging = join(bundle, 'native-linux');
  rmSync(staging, { recursive: true, force: true });
  const debRoot = join(staging, 'deb');
  const appDir = join(staging, 'noBS-CAD.AppDir');
  const entry = `[Desktop Entry]
Type=Application
Name=noBS CAD
Comment=Local-first mechanical CAD
Exec=nbcad %u
Icon=nbcad
Terminal=false
Categories=Graphics;Engineering;
MimeType=x-scheme-handler/nbcad;
StartupWMClass=nbcad
`;
  for (const root of [debRoot, appDir]) {
    for (const directory of ['usr/bin', 'usr/share/applications', 'usr/share/icons/hicolor/256x256/apps']) {
      mkdirSync(join(root, directory), { recursive: true });
    }
    copyFileSync(join(targetRoot, 'release/nbcad'), join(root, 'usr/bin/nbcad'));
    chmodSync(join(root, 'usr/bin/nbcad'), 0o755);
    writeFileSync(join(root, 'usr/share/applications/nbcad.desktop'), entry);
    copyFileSync(join(desktopRoot, 'icons/128x128@2x.png'), join(root, 'usr/share/icons/hicolor/256x256/apps/nbcad.png'));
    const notices = join(root, 'usr/share/nbcad/licenses');
    stageProjectNotices(notices);
    copyFileSync(join(licenseRoot, 'LGPL-2.1.txt'), join(notices, 'OCCT-LGPL-2.1.txt'));
    copyFileSync(join(licenseRoot, 'OCCT-copyright.txt'), join(notices, 'OCCT-copyright.txt'));
    cpSync(join(licenseRoot, 'xkb/licenses'), join(notices, 'xkb'), { recursive: true });
  }
  mkdirSync(join(debRoot, 'DEBIAN'));
  writeFileSync(join(debRoot, 'DEBIAN/control'), `Package: nbcad
Version: ${version}
Architecture: amd64
Maintainer: noBS CAD contributors <nbcad@users.noreply.github.com>
Section: graphics
Priority: optional
Depends: desktop-file-utils, libocct-data-exchange-7.9, libudev1, libvulkan1, libxkbcommon-x11-0, xdg-utils, xdg-desktop-portal, xdg-desktop-portal-gtk
Description: Local-first mechanical CAD with a native Bevy interface
`);
  mkdirSync(join(bundle, 'deb'), { recursive: true });
  mkdirSync(join(bundle, 'appimage'), { recursive: true });
  run('dpkg-deb', ['--build', '--root-owner-group', debRoot, join(bundle, 'deb', `noBS.CAD_${version}_amd64.deb`)]);
  cpSync(join(licenseRoot, 'xkb/lib'), join(appDir, 'usr/lib'), { recursive: true });

  // Immutable upstream release and published digest; fail closed on mismatch.
  const digest = 'c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d';
  const tool = join(bundle, 'linuxdeploy-x86_64.AppImage');
  if (!existsSync(tool) || createHash('sha256').update(readFileSync(tool)).digest('hex') !== digest) {
    const response = await fetch('https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage');
    if (!response.ok) throw new Error(`linuxdeploy download failed: ${response.status}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    if (createHash('sha256').update(bytes).digest('hex') !== digest) throw new Error('linuxdeploy checksum mismatch');
    writeFileSync(tool, bytes);
  }
  chmodSync(tool, 0o755);
  run(tool, ['--appimage-extract-and-run', '--appdir', appDir,
    '--desktop-file', join(appDir, 'usr/share/applications/nbcad.desktop'),
    '--icon-file', join(appDir, 'usr/share/icons/hicolor/256x256/apps/nbcad.png'),
    '--output', 'appimage'], {
    cwd: join(bundle, 'appimage'),
    env: { ...process.env, ARCH: 'x86_64', VERSION: version,
      OUTPUT: `noBS.CAD_${version}_amd64.AppImage`, APPIMAGE_EXTRACT_AND_RUN: '1',
      // Preserve diagnostic symbols according to the Cargo release profile.
      NO_STRIP: '1' },
  });
}
