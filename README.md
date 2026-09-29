# noBS CAD

**Free, open-source parametric CAD that you and your AI agent can both drive.**
Design mechanical parts, assemblies and drawings locally, and keep every
sketch and feature editable, whether you model by hand or through MCP.

[![Latest release](https://img.shields.io/github/v/release/jackControls/noBS-CAD?label=release)](https://github.com/jackControls/noBS-CAD/releases/latest)
[![License: LGPL 2.1+](https://img.shields.io/badge/license-LGPL%202.1%2B-blue)](LICENSE)
[![Discussions](https://img.shields.io/github/discussions/jackControls/noBS-CAD?label=discussions)](https://github.com/jackControls/noBS-CAD/discussions)

**Pre-alpha · Release 0.2.1**
· [Release notes and checks](https://github.com/jackControls/noBS-CAD/releases/tag/v0.2.1)
· [Installation help](docs/INSTALL.md)

| Platform | Download |
|---|---|
| Windows 11 | [x64 ZIP](https://github.com/jackControls/noBS-CAD/releases/download/v0.2.1/noBS-CAD-0.2.1-windows-x64.zip) · [ARM64 ZIP](https://github.com/jackControls/noBS-CAD/releases/download/v0.2.1/noBS-CAD-0.2.1-windows-arm64.zip) |
| macOS (Apple silicon) | [DMG](https://github.com/jackControls/noBS-CAD/releases/download/v0.2.1/noBS.CAD_0.2.1_aarch64.dmg), signed and notarized |
| Linux | [Ubuntu 26.04 DEB](https://github.com/jackControls/noBS-CAD/releases/download/v0.2.1/noBS.CAD_0.2.1_amd64.deb) · [AppImage](https://github.com/jackControls/noBS-CAD/releases/download/v0.2.1/noBS.CAD_0.2.1_amd64.AppImage) |

Windows code signing is in progress. Until it lands, SmartScreen may warn on
first launch; choose **More info → Run anyway**. Keep backups of important
projects while the application is pre-alpha.

## Why noBS CAD

- **Local and yours.** No account, subscription or cloud service. A whole
  project (parts, assemblies and drawings) lives in one `.nbcad` file.
- **Real parametric history.** Constrained sketches drive solid features;
  change a dimension and everything downstream rebuilds.
- **Agent-ready.** A built-in MCP server lets any MCP-compatible agent build and
  edit models, and what it makes is the same editable history you would make by hand.
- **Open formats.** STEP, STL and 3MF export; drawings to DXF and print-to-PDF.

## Made in noBS CAD

Each design was built from a blank document through MCP, and its sketches,
features and assembly relationships remain editable. The loops are accelerated
excerpts; **Watch** opens the full recording.

<table>
<tr>
<td align="center" valign="top" width="33%">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#garden-bench"><img src="docs/assets/showcase/bench-loop.gif" alt="Accelerated construction excerpt of the garden bench"></a><br>
<b>Garden bench</b><br>
Change one picket dimension and the whole back updates. Frame, arms and joints stay editable.
</td>
<td align="center" valign="top" width="33%">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#d-screw-vise"><img src="docs/assets/showcase/vise-loop.gif" alt="Accelerated construction excerpt of the vise"></a><br>
<b>Vise</b><br>
Turn the screw and the jaw follows. 100 mm jaws, 90 mm travel, six printed parts plus hardware.
</td>
<td align="center" valign="top" width="33%">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#vertical-axis-turbine"><img src="docs/assets/showcase/turbine-loop.gif" alt="Accelerated construction excerpt of the vertical-axis turbine"></a><br>
<b>Vertical-axis turbine</b><br>
Two Savonius stages on a bearing-supported shaft, driving a generator through a 4:1 drive.
</td>
</tr>
<tr>
<td align="center">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#garden-bench"><b>Watch</b></a>
· <a href="https://jackcontrols.github.io/noBS-CAD/open.html#garden-bench">Open recipe</a><br>
<a href="examples/scripts/garden-bench.nbcad.jsonc">Source</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/bench.nbcad">.nbcad</a>
· <a href="docs/assets/showcase/bench.png">Image</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/bench-build-full.mp4">MP4</a>
</td>
<td align="center">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#d-screw-vise"><b>Watch</b></a>
· <a href="https://jackcontrols.github.io/noBS-CAD/open.html#d-screw-vise">Open recipe</a><br>
<a href="examples/scripts/d-screw-vise.nbcad.jsonc">Source</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/vise.nbcad">.nbcad</a>
· <a href="docs/assets/showcase/vise.png">Image</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/vise-build-full.mp4">MP4</a>
</td>
<td align="center">
<a href="https://jackcontrols.github.io/noBS-CAD/showcase.html#vertical-axis-turbine"><b>Watch</b></a>
· <a href="https://jackcontrols.github.io/noBS-CAD/open.html#vertical-axis-turbine">Open recipe</a><br>
<a href="examples/scripts/vertical-axis-turbine.nbcad.jsonc">Source</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/turbine.nbcad">.nbcad</a>
· <a href="docs/assets/showcase/turbine.png">Image</a>
· <a href="https://github.com/jackControls/noBS-CAD/releases/download/showcase-v0.2.0/turbine-build-full.mp4">MP4</a>
</td>
</tr>
</table>

<!-- Print photos: add docs/assets/showcase/<design>-printed.jpg when supplied. -->

Recipe links load source into **Scripts** for review before running. To inspect
a completed design immediately, download its `.nbcad` and use **File → Open**.
These are development examples; physical fit and load qualification remain open.
[Designs, drawings and validation](docs/flagship-examples.md) · [All recipes](examples/scripts/README.md)

## Make your first part

After [installing CAD](docs/INSTALL.md), open **Scripts**, choose
**Sketch, extrude, ease the edges**, and select **Run in new design**.
The lesson builds a 60 × 30 × 12 mm block with rounded top edges.

When it finishes, double-click the extrusion in the feature history and change
its **Distance** from **12 to 18 mm**. Save the result as `first-part.nbcad`,
then reopen it to continue editing. [Step-by-step instructions](docs/INSTALL.md#make-your-first-part)

## Design, assemble, draw

Constrained sketches and reference geometry drive editable solid features.
Reuse parts in assemblies, define joints and check their motion and interference.
Keep the parts, assemblies and drawings together in one `.nbcad` project.

Assign per-body materials and colors, then export **3MF** for your slicer.
Material labels and color metadata assist the handoff; choose the actual print
profile in the slicer. **STEP** carries exact geometry and **STL** supplies mesh
export. Drawing sheets export to **DXF** and print/PDF.
[Assemblies](docs/ASSEMBLIES.md) · [Drawings and export coverage](docs/2D_DRAWINGS.md)

## Work with an agent

Local stdio MCP is always available in the installed application. Bring your preferred
MCP-compatible agent and model to build a part, edit an existing feature, inspect
an assembly or replay a demonstration. CAD keeps the same editable project
whether you use the tools yourself or ask an agent to use them.

[Connect your agent](docs/INSTALL.md#connect-an-mcp-agent), then try:

> Use noBS CAD to run the fillet-basics lesson in a new design in the open CAD
> window. Preserve my existing documents. After the final checks pass, change
> the stock extrusion from 12 to 18 mm, inspect the result and keep it open.

An agent is optional. **Scripts** can build a bundled example, explain its
chapters and show the construction with captions, camera moves and playback
controls. You can inspect and edit the recipe before running it.
[MCP interface](mcp-server/README.md) · [Recipes and playback](docs/native-scripts.md)
· [Engineering knowledge](knowledge/index.md)

## Help build it

Contributions are welcome. Our priorities are **reliability, performance and
ease of use**, in that order. Bring a part, a reproducible bug or a focused
improvement. [Contributing](CONTRIBUTING.md) · [Developer setup](docs/DEVELOPMENT.md)
· [Documentation](docs/INDEX.md)

Questions, ideas, or something you made? Start a thread in
[Discussions](https://github.com/jackControls/noBS-CAD/discussions). If noBS CAD
is useful to you, a star helps other people find it.

We are working toward guided design lessons and conversational wizards, and
developing an early **3-axis CAM** foundation with toolpath generation, stock
simulation and machine-aware posts. It is not production-safe CAM; review the
[CAM guides and safety limits](docs/cam/README.md). Strength analysis remains a
future capability. [Project direction](docs/goals.md)

## Open-source foundations

- **[Open CASCADE Technology](https://github.com/Open-Cascade-SAS/OCCT)** — geometry and CAD interchange.
- **[Bevy](https://bevy.org/) and [wgpu](https://wgpu.rs/)** — native rendering.
- **[Rust](https://rust-lang.org/)** — modeling, assemblies and recipe execution.
- **[Tauri](https://tauri.app/) and [React](https://react.dev/)** — desktop shell and interface.
- **[OpenCascade.js](https://github.com/donalffons/opencascade.js)** — browser development builds.

Thanks also to [FreeCAD](https://www.freecad.org/) and the wider open-source CAD community.

## License

[GNU LGPL 2.1 or later](LICENSE). Free to use, inspect and improve.

<details>
<summary>Third-party notices and 3D mouse support</summary>

Dependency licenses and attribution live in [Third-party notices](THIRD_PARTY_NOTICES.md).
Icon sources are recorded in [Icon provenance](docs/ICON_PROVENANCE.md).
Peer CAD projects have their own licenses; see [contribution guidance](CONTRIBUTING.md#license--borrow).

noBS CAD supports 3Dconnexion SpaceMouse devices. The optional browser-development
driver bridge loads only after the user enables it.
noBS CAD is independent and is not affiliated with, endorsed by or certified by
3Dconnexion. 3Dconnexion and SpaceMouse are trademarks or registered trademarks
of 3Dconnexion. 3D input device development tools and related technology are
provided under license from 3Dconnexion. © 3Dconnexion 1992–2020. All rights reserved.

</details>
