# Ubuntu 26.04 Linux packaging

Ubuntu 26.04 LTS x86_64 is the official Linux desktop baseline. The native
application keeps the same production boundary used on macOS and Windows:
React/CSS owns menus, dialogs, tabs, input and accessibility; Bevy/wgpu owns
the embedded CAD viewport; native OCCT owns exact geometry.

To use the application, follow [Install noBS CAD](INSTALL.md#ubuntu).
For development, `cargo xtask package` selects the Linux builder;
[the developer guide](DEVELOPMENT.md) is the shared build entry point.

## Supported desktop paths

- X11 through a child GTK `DrawingArea` and native Xlib window/display
  handles.
- Ubuntu's standard Wayland desktop through XWayland and the same child-window
  path. The Debian package declares `xwayland` as a runtime dependency.
- Vulkan rendering through wgpu. Mesa's lavapipe software Vulkan driver is
  used only by the headless CI probe; it is a compatibility fallback, not a
  performance target.
- A fully opaque GTK/Tauri top-level window. The input-transparent native X11
  child sits above WebKitGTK and is shaped around React's visible overlay
  islands, so DOM menus and dialogs remain intact without depending on
  accelerated transparent-WebKit compositing.

Other distributions may work when they provide compatible GTK, WebKitGTK,
Vulkan and OCCT 7.9 libraries, but Ubuntu 26.04 is the tested support contract.

The AppImage is the exception to Ubuntu 26.04 as a build system. An AppImage
bundles every library it links except the C library, so it runs only where
glibc is at least as new as the build system's. It is therefore built on
Ubuntu 22.04 (glibc 2.35) against OCCT 7.9.3 compiled from pinned source by
`scripts/build-occt-linux.sh`, because Ubuntu 22.04 does not package OCCT 7.9.
Release CI refuses an AppImage that needs a newer glibc, and launches it on both
Ubuntu 22.04 and 26.04. The Debian package stays on Ubuntu 26.04's OCCT.

Like glibc, the Wayland client libraries come from the host rather than the
AppImage. The host's Mesa Vulkan and EGL drivers load into the application and
link those libraries; Mesa 26 needs symbols that Ubuntu 22.04's Wayland 1.20
lacks, so bundled client copies stopped every GPU driver from loading on Ubuntu
26.04. The bundler excludes `libwayland-client`, `libwayland-cursor`, and
`libwayland-egl` through linuxdeploy's `LINUXDEPLOY_EXCLUDED_LIBRARIES`
(honoured by the linuxdeploy that Tauri CLI 2.12 and later downloads) and fails
if the AppImage contains them. It still bundles `libwayland-server`, which the
application links directly and which is not guaranteed on an X11-only or
minimal desktop. Cross-version verification explicitly installs the host EGL,
Vulkan, and Wayland client loaders because GitHub's Ubuntu runner is a minimal
server image rather than the Ubuntu desktop represented by that runtime
contract.

## Reproducible container build

```sh
docker build \
  -f scripts/docker/ubuntu-26.04.Dockerfile \
  -t nbcad-ubuntu-26.04 \
  .

docker run --rm \
  -v "$PWD:/workspace" \
  -w /workspace \
  nbcad-ubuntu-26.04 \
  sh -lc 'npm ci && cargo xtask package'
```

That container builds both packages. The published AppImage comes from the
Ubuntu 22.04 SDK instead, which compiles OCCT once while the image builds:

```sh
docker build \
  -f scripts/docker/appimage-ubuntu-22.04.Dockerfile \
  -t nbcad-appimage-ubuntu-22.04 \
  .

docker run --rm \
  -v "$PWD:/workspace" \
  -w /workspace \
  nbcad-appimage-ubuntu-22.04 \
  sh -lc 'npm ci && npm run bundle:linux -- appimage'
```

`npm run bundle:linux -- deb` builds only the Debian package.

The 26.04 container deliberately extracts only the Ubuntu STEP development headers
from `libocct-data-exchange-dev`; installing that package normally also pulls
the unrelated VTK/IVTK development stack. Its matching OCCT runtime and the
lower-level OCCT development packages are installed normally.

## Native Ubuntu build dependencies

The authoritative dependency list is in
`scripts/docker/ubuntu-26.04.Dockerfile` and the `build-linux-ubuntu` job in
`.github/workflows/desktop-packages.yml`. It includes:

- GTK 3, WebKitGTK 4.1, Ayatana AppIndicator and librsvg;
- Vulkan, Wayland, X11/XKB and udev development files;
- OCCT 7.9 foundation, modeling and data-exchange libraries/headers;
- Rust stable, Node 22 and npm; and
- Tauri packaging utilities including `patchelf`, `file`, FUSE 2 and
  `squashfs-tools` (the AppImage permission audit reads the image with
  `unsquashfs`).

After installing those dependencies:

```sh
npm ci
cargo xtask package
```

Artifacts are written under:

```text
src-tauri/target/release/bundle/deb/*.deb
src-tauri/target/release/bundle/appimage/*.AppImage
```

Each artifact has a neighboring `.sha256` file. The bundler fails if the
project, third-party, OpenCascade.js, OCCT copyright, or LGPL notices are
missing from either package. It also fails if any file in the AppImage cannot
be read, or executed where its owner can execute it, by other users: the
mounted image keeps the build user's uid, so a sandbox such as firejail or
another account runs it as "other". Tauri writes the `AppRun` it downloads
for linuxdeploy with mode 0770, so the bundler seeds Tauri's tool cache
(`~/.cache/tauri`) with a world-executable copy first.

<details>
<summary>Underlying builder for packaging maintenance</summary>

The Rust entry point delegates to `scripts/bundle-linux.mjs`. The existing
`npm run bundle:linux` alias invokes that same builder; it remains available
to CI and packaging diagnostics.

</details>

## Native viewport verification

The release workflow launches the final AppImage in Xvfb on Ubuntu 22.04 and
26.04, and the executable from the final Debian package in Xvfb and in a
headless Weston/XWayland session. GTK 3
does not expose an independent child `wl_surface` for the drawing widget; using
its top-level surface would let GTK and Vulkan attach competing buffers. The
application therefore selects the reliable X11 child-window backend on both
desktop types. The development-only readiness probe confirms the X11/XWayland
surface, Vulkan renderer, physical size, and rendered frame count. It records
no pointer or model data.

Manual verification on an Ubuntu SDK image uses:

```sh
scripts/verify-linux-viewport.sh path/to/noBS-CAD.AppImage x11 /tmp/nbcad-x11
scripts/verify-linux-viewport.sh path/to/noBS-CAD.deb xwayland /tmp/nbcad-xwayland
```

## 3D mouse permissions

Linux HID devices can require a distribution udev rule before an unprivileged
application may open their `hidraw` node. Install the vendor's Linux driver or
an administrator-provided least-privilege udev rule for the specific device,
then reconnect it. Do not run noBS CAD as root. Ordinary mouse, touchpad and
keyboard navigation do not require extra permissions.

## Scope

The CI release is x86_64. The source and Ubuntu SDK also compile on AArch64,
but an AArch64 release artifact is not part of the official matrix yet. Package
installation, signing/repository distribution, and automatic udev-rule setup
remain separate release-engineering work.
