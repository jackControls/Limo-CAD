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

The container deliberately extracts only the Ubuntu STEP development headers
from `libocct-data-exchange-dev`; installing that package normally also pulls
the unrelated VTK/IVTK development stack. Its matching OCCT runtime and the
lower-level OCCT development packages are installed normally.

## Native Ubuntu build dependencies

The authoritative dependency list is in
`scripts/docker/ubuntu-26.04.Dockerfile` and the shared
`.github/actions/setup-linux-desktop/action.yml` used by package and native-host
checks. It includes:

- GTK 3, WebKitGTK 4.1, Ayatana AppIndicator and librsvg;
- Vulkan, Wayland, X11/XKB (including `libxkbcommon-x11-dev`) and udev development files;
- OCCT 7.9 foundation, modeling and data-exchange libraries/headers;
- Rust stable, Node 22 and npm; and
- Tauri packaging utilities including `patchelf`, `file`, and FUSE 2.

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
missing from either package.

Winit loads `libxkbcommon-x11.so.0` dynamically. The DEB therefore explicitly
depends on `libxkbcommon-x11-0`. The bundle script stages that SONAME and its
non-glibc dependency closure in the AppImage's `usr/lib`, together with the
Ubuntu package copyright notices and referenced common-license texts. After
extraction it checks ELF dependencies and resolves them against the bundled
libraries; falling back to an unstaged host dependency fails the audit.

<details>
<summary>Underlying builder for packaging maintenance</summary>

The Rust entry point delegates to `scripts/bundle-linux.mjs`. The existing
`npm run bundle:linux` alias invokes that same builder; it remains available
to CI and packaging diagnostics.

</details>

## Diagnostic Bevy packages

The release build remains the React shell. An explicit diagnostic build uses
the same locked dependencies and package audit with `dev-bevy-host` enabled:

```sh
node scripts/bundle-linux.mjs --native-host
bash scripts/verify-linux-native-package.sh path/to/noBS-CAD.deb /tmp/native-deb-evidence
bash scripts/verify-linux-native-package.sh path/to/noBS-CAD.AppImage /tmp/native-appimage-evidence
```

Choose fresh evidence directories. The smoke runner extracts each package into
an owned temporary directory, starts a private Xvfb/D-Bus desktop at 100% and
200% scale, and runs the existing `native-platform` keyboard, clipboard, and
window-capture fixture. Configuration, session data, and caches are isolated;
it does not reuse an open design or the user's display. The GitHub workflow
`native-host-tests.yml` exposes this check as the opt-in `native-packages`
dispatch/call input. Its artifacts are diagnostic builds, not published releases.

## Native viewport verification

The release workflow launches the final AppImage in Xvfb and the executable
from the final Debian package in a headless Weston/XWayland session. GTK 3
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
