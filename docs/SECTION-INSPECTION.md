# Section inspection and diagram capability

Section Analysis is a modeling inspection tool in the native Bevy desktop.
It reuses the existing OCCT drawing-section pipeline without creating features
or drawing sheets and without changing source geometry.

## Capability audit and order of work (2026-10-06)

1. **Modeling section inspection (this change).** OCCT section intersections,
   drawing sections, hatch primitives, associative drawing dimensions, SVG/DXF
   export and Bevy screenshot capture already existed. The modeling ribbon's
   Section Analysis command was a disabled placeholder. Expose the shared
   engine through a disposable Bevy panel and MCP read, with sampled material
   intervals across the section. No second drawing engine or chart dependency.
2. **Interactive geometric measurements.** Drawing dimensions already exist;
   modeling's Measure placeholder still needs a general selected-point/edge/face
   tool. Reuse the topology references and measurement/unit forms. The section
   probe here is a directional material-span measurement, not automatic minimum
   wall thickness.
3. **3D inspection and comparison presentation.** Build clipping-plane/cap
   display and cross-section zoom/picking alongside the Bevy renderer. Exact
   assembly interference and source-solid comparison already exist; improve
   discoverability and visual evidence before adding another geometry API.
4. **Engineering report composition.** Named views, screenshot capture and
   drawing exports already supply most ingredients. Add a shared review layout
   with model-revision provenance, multiple views and diagrams. Do not recreate
   those primitives in bespoke Python as an application feature.
5. **Structural analysis ([issue #336](https://github.com/jackControls/Limo-CAD/issues/336)).** No production structural solver
   was found. Require material provenance, boundary conditions, mesh convergence
   and benchmark validation before presenting force/strain predictions.

## Use

Solid workspace → Check → Section Analysis. Choose one source body, an XY/XZ/YZ
plane and its offset; enter an optional probe's vertical coordinate; Inspect.
Fields accept the document's length units or an explicit `mm`, `cm` or `in`
suffix using the existing arithmetic input. The diagram and returned distances
are explicitly millimetres. These are source-definition coordinates, independent
of occurrence transforms, exploded presentation and print-bed placement.

`solid_section_review` takes `body_id`, `plane` (`xy`, `xz`, `yz`), `offset_mm`,
optional `probe_mm`, and optional `deflection_mm` (0.001–0.1; default 0.01).
It returns section-only bounds, sampled intersection curves, separate
`probe_spans` with start/end/length in mm, and an SVG document. The SVG can be
saved by a client without creating a drawing sheet. XY uses X/Y, XZ X/Z, YZ Y/Z
as horizontal/vertical axes. The offset refers to Z, Y, X respectively.
The Bevy panel's Copy SVG button places the current diagram on the native
clipboard for saving or pasting into another tool; Escape closes the panel.

Intersections come from the existing OCCT drawing-section pipeline; hatching
uses the existing bounded paper-graphics code. Probe distances use the sampled
section curves, including holes and disconnected material with even/odd
crossings. Open/ambiguous probe intersections reject rather than invent a
thickness. Tangent planes and invalid requests reject; an empty intersection
returns no bounds and no SVG. Curves are limited to 100,000 points and probe
results to 128 intervals. A probe on a boundary follows a half-open convention;
move it inside the material for an interior measurement.

The preview uses the ordered native query worker. Document owner, revision and
draft generation guard both dispatch and publication. Editing fields or changing
the model clears the image and measurements. Reads create no feature, drawing,
history entry or geometry revision. Closing the panel releases its image.

This tool supplies geometric evidence. It does not calculate snap force,
displacement, stress, strain, creep or slicer wall-loop behavior.
