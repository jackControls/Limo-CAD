# Ubuntu 26.04 Linux packaging

Ubuntu 26.04 LTS x86_64 is the Linux package target. Bevy owns the full
native interface and CAD viewport; OCCT owns exact geometry. This draft branch
has removed the React/Tauri desktop host. Final native package qualification is
still required before publishing a release.

For development, `cargo xtask package` selects the Linux builder; see the
[developer guide](DEVELOPMENT.md). Published packages retain their own release
notes and requirements.

## Supported desktop paths

Winit creates the application window directly on X11 or Wayland. Rendering uses
wgpu/Vulkan. GTK 3 supplies native file dialogs; no WebKit runtime is needed.
Disposable CI uses Mesa lavapipe for correctness, not performance acceptance.

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
  sh -lc 'cargo xtask package'
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

- GTK 3 for native file dialogs;
- Vulkan, Wayland, X11/XKB (including `libxkbcommon-x11-dev`) and udev development files;
- OCCT 7.9 foundation, modeling and data-exchange libraries/headers;
- Rust stable and Node 22; and
- Native packaging utilities including `patchelf`, `file`, and FUSE 2.

After installing those dependencies:

```sh
cargo xtask package
```

Artifacts are written under:

```text
src-tauri/target/release/bundle/deb/*.deb
src-tauri/target/release/bundle/appimage/*.AppImage
```

Each artifact has a neighboring `.sha256` file. The bundler fails if the
project, third-party, OCCT copyright, or LGPL notices are
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

## Native package verification

The ordinary package build is the Bevy application; there is no migration flag.
After building, use fresh evidence directories:

```sh
bash scripts/verify-linux-native-package.sh path/to/noBS-CAD.deb /tmp/native-deb-evidence
bash scripts/verify-linux-native-package.sh path/to/noBS-CAD.AppImage /tmp/native-appimage-evidence
scripts/verify-linux-viewport.sh path/to/noBS-CAD.AppImage x11 /tmp/nbcad-x11
scripts/verify-linux-viewport.sh path/to/noBS-CAD.deb wayland /tmp/nbcad-wayland
```

The input checks own a private Xvfb/D-Bus desktop and exercise the existing native
keyboard, clipboard, and Bevy window-capture fixture. The Wayland check uses a
private headless Weston compositor and the desktop lifecycle/MCP fixture; it
does not claim physical Wayland keyboard or IME coverage. Package metadata and
recipe URI registration are also checked. These scripts never reuse a user's
open design or display.

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
