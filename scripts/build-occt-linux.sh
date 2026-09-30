#!/usr/bin/env bash
# Builds the OCCT 7.9 subset noBS CAD links into a prefix, for Linux builds
# that cannot use a distribution package (the AppImage is built on an older
# Ubuntu so it runs on older glibc, and that Ubuntu does not ship OCCT 7.9).
#
# The version matches the OCCT 7.9.3 pinned for the macOS and Windows
# packages (docs/OCCT_PACKAGING.md).
#
# usage: scripts/build-occt-linux.sh <install-prefix>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <install-prefix>" >&2
  exit 2
fi

version=7_9_3
sha256=5ecf094ec6b12d5413dfb851d8c3590c354058aee556e32e408bdfbf8c357d57
prefix="$(realpath -m "$1")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl --proto '=https' --tlsv1.2 -sSfL -o "$work/occt.tar.gz" \
  "https://github.com/Open-Cascade-SAS/OCCT/archive/refs/tags/V${version}.tar.gz"
echo "$sha256  $work/occt.tar.gz" | sha256sum -c -
tar -xzf "$work/occt.tar.gz" -C "$work"
source_dir="$work/OCCT-${version}"

# STEP translation (TKDESTEP) pulls in the XCAF document toolkits and, through
# them, TKV3d/TKService, so Visualization and ApplicationFramework stay on.
# Draw, Tcl/Tk, OpenGL and the samples are not linked by noBS CAD.
cmake -S "$source_dir" -B "$work/build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DINSTALL_DIR="$prefix" \
  -DINSTALL_DIR_LAYOUT=Unix \
  -DBUILD_LIBRARY_TYPE=Shared \
  -DBUILD_MODULE_FoundationClasses=ON \
  -DBUILD_MODULE_ModelingData=ON \
  -DBUILD_MODULE_ModelingAlgorithms=ON \
  -DBUILD_MODULE_Visualization=ON \
  -DBUILD_MODULE_ApplicationFramework=ON \
  -DBUILD_MODULE_DataExchange=ON \
  -DBUILD_MODULE_DETools=OFF \
  -DBUILD_MODULE_Draw=OFF \
  -DBUILD_DOC_Overview=OFF \
  -DUSE_TCL=OFF \
  -DUSE_TK=OFF \
  -DUSE_OPENGL=OFF \
  -DUSE_GLES2=OFF \
  -DUSE_FREETYPE=ON \
  -DUSE_FREEIMAGE=OFF \
  -DUSE_RAPIDJSON=OFF \
  -DUSE_DRACO=OFF \
  -DUSE_TBB=OFF \
  -DUSE_VTK=OFF
cmake --build "$work/build" --parallel "${CMAKE_BUILD_PARALLEL_LEVEL:-$(nproc)}"
cmake --install "$work/build"

# The packages ship the OCCT license and its LGPL exception next to the app.
doc="$prefix/share/doc/opencascade"
mkdir -p "$doc"
{
  echo "Open CASCADE Technology ${version//_/.}"
  echo "https://github.com/Open-Cascade-SAS/OCCT"
  echo "Copyright (c) Open CASCADE SAS"
  echo
  echo "OCCT is distributed under the GNU Lesser General Public License version 2.1"
  echo "with the following additional exception."
  echo
  cat "$source_dir/OCCT_LGPL_EXCEPTION.txt"
} > "$doc/copyright"
cp "$source_dir/LICENSE_LGPL_21.txt" "$doc/LGPL-2.1.txt"
echo "Installed OCCT ${version//_/.} into $prefix"
