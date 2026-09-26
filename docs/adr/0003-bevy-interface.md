# ADR 0003 — One Bevy application interface

- Status: Accepted direction; implementation and parity validation in progress
- Date: 2026-09-13
- Tracking: [#38](https://github.com/jackControls/noBS-CAD/issues/38)
- Supersedes: the React-shell and native-child-composition ownership decisions in [ADR 0002](0002-bevy-viewport.md)

## Decision

Move the existing application's complete interface to Rust and Bevy. This is
one product and one command system: a person, an MCP client, and a replayed
recipe operate the same document services. OCCT remains the solid-modeling
authority. This decision does not replace B-rep geometry with rendering meshes.

The conversion includes the application shell, editable command forms, sketch
interaction, assemblies, drawings, CAM, scripts, presentation controls,
settings, file handling, and window lifecycle. Moving only the viewport HUD or
putting native buttons over the old web shell does not complete it.

Use Bevy/Winit for the eventual native window, input and accessibility host.
Keep the existing binary and the ordinary desktop versus `--headless` choice;
do not introduce a second application or a permanent alternate-interface flag.
The current embedded viewport is a development integration point while the
replacement is incomplete. It is not a reason to maintain bespoke IME and
accessibility bridges for three window systems.

## Shared behavior

`interface/catalog.json` remains the product grouping authority. Native widgets
and MCP consume it; neither adds a competing operation taxonomy. Rendered
widgets register their actual entity identity, accessible name, computed
bounds, enabled state, input contract and modal ownership. Inspecting controls
must describe what a person can operate in the displayed interface.

Human input and MCP resolve through the same control validation and native
reducer. A queued control is bound to its document incarnation and semantic
binding, not merely a tab name or recycled widget position. Revalidate ownership
under the document's mutation lock. Reuse the existing engine dispatcher and
project-replacement path; do not translate native actions into hidden DOM clicks
or duplicate the modeling command switch.

The host owns presentation and file-dialog effects. Shared Rust services own
model changes, history, file encoding, validation and replay. Interactive
operations retain their draft, selection, preview, Apply/Cancel, error and undo
semantics. A generic JSON editor for API arguments is not a substitute for an
intuitive modeling form.

## Rendering and interaction

Use one composited Bevy surface for shell, floating controls and viewports.
Native layout must clip and hit-test menus, scroll areas and modal backdrops
consistently. Preserve accurate model/paper coordinate transforms in drawings
and viewport coordinates under DPI changes and panel resizing.

Retain widgets and geometry; update changed data rather than rebuild the entire
scene on pointer motion. Keep idle work event-driven. Long geometry and CAM
operations need cancellation, visible progress and owner-checked completion
without blocking navigation or assigning results to another document.

Distinguish accepted commands, completed model changes, published snapshots,
laid-out controls and submitted rendered frames. An action acknowledgement or
an inspected control tree is not proof that its pixels have been presented.
Background/headless operation must not wait indefinitely for a visible frame.

Native text entry must include selection, clipboard, shortcuts, composition
and IME. Focus traversal, modal trapping, keyboard activation and OS screen
reader access are part of the interface, not optional metadata. Winit and Bevy
integration still require real platform validation; their presence in the
dependency graph is not evidence that these behaviors work.

## Parity and retirement

The local baseline inventory and full-window captures establish the working
reference. They are temporary migration material, kept outside the repository
and removed after parity validation. Capture failures are marked as defects to
improve, rather than treated as successful parity. The original user document
and application window remain untouched during capture.

For each migrated area, validate its normal route, invalid-input route,
Cancel/Undo, keyboard and MCP operation, persistence, and actual rendering.
Use existing behavioral contracts and deterministic examples where useful.
Do not add tests that only repeat the catalog or bless a blank screenshot.

Do not remove an implemented old surface until its replacement passes those
checks. Remove inert placeholders deliberately and document them. Keep the
conversion PR in draft until the full interface beats the baseline and the
supported platform checks pass. Dependency/build simplification follows the
retirement of actual consumers; do not remove a dependency merely to improve
the dependency count while its behavior is still needed.

For lossless ribbon comparisons, set `NBCAD_RIBBON_LAB=1` and run
`cargo run --manifest-path src-tauri/Cargo.toml --features dev-ui-lab --bin bevy-ui-lab -- <output.png>`.
This renders the production widgets in normal, selected and disabled states
through Bevy's GPU pipeline, without opening another CAD window. Compare with
the original ribbon at the same display scale; keep the captures outside the
repository. Shared SVG sources prevent geometry drift but do not prove visual
parity: inspect typography, antialiasing, layout and interaction states too.

This ADR records the implementation direction. It is not a statement that the
conversion, screenshot inventory, platform coverage or performance validation
has finished.

The repeatable native sketch check is `cargo xtask test-mcp native-sketch
--server <rebuilt-CAD-binary> --session <blank-native-document-UUID> --out
<absolute-evidence-directory>`. It uses Rust and MCP to operate the rendered
controls and canvas, checks nine modification forms and Undo, driving/reference
dimensions, constraint deletion, Escape cancellation, capture and Save. It
requires an explicitly selected blank document and fresh output filenames;
it does not launch or close a desktop window, discard work or upload evidence.

The corresponding `native-build` suite uses the same arguments and safety
checks. It drives native Revolve, Sweep, Loft and Rib reference selection, creation,
history editing, close/Cancel, Undo/Redo, rendered capture and Save. Each case
is saved before the next blank tab is created. Loft's datum and section geometry
are seeded through the shared MCP contract; the datum is selected through native
Create Sketch and the browser. Kernel-backed form tests also exercise invalid values,
reference ownership, connected paths, ordered sections and coplanar axes.

`native-support` uses the same arguments to check Create Sketch, all three origin
planes, face selection and both coordinate-zero choices, datum selection in the
browser and canvas, Escape/close cancellation, capture and Save. It uses engine
MCP to seed precise support geometry so this suite does not depend on viewport
zoom or duplicate the native drawing-gesture checks.

Native document tabs retain their own cameras. New and newly opened files fit
their model; changing tabs or closing a tab restores that document's view rather
than inheriting the previous model's zoom. AccessKit publishes native radio and
checkbox states and text field values from the same retained control registry.

The shared native solid-form transaction serves both Build and Refine; control
surfaces come from each operation's existing catalog group. Fillet and Chamfer
use the shared Rust tangent-chain rule, also consumed by the current desktop
and MCP. An edge-feature editor prepares its pre-feature scene in an isolated
kernel on the modeling worker. Opening and Cancel preserve the live document,
revision and history cursor. Apply transfers the successfully recomputed kernel
under the original document receipt and records one Undo boundary. Separate
renderer cache incarnations prevent the input preview from reusing final meshes.
The `native-refine` Rust MCP suite checks rendered picking, units, invalid sizes,
context-menu/double-click editing, Cancel, Undo/Redo, captures and saved parts.

Shell uses this same form and isolated topology editor, with removable-face
selection, face hover/highlighting, typed wall thickness and inward/outward
offset. The Refine fixture exercises multi-face toggling and the history routes.
The shared OCCT command validates the resulting B-rep and volume before replacing
the body; an impossible inward thickness cannot silently commit an inverted or
oversized result. Native engine tests verify both offset directions and recovery
after a rejected edit.
Ordinary model edges have their own small depth bias: coincident cavity seams
remain visible with Bevy's strict reverse-Z depth test, while rear edges remain
occluded and the reference grid retains its original depth behavior.

Combine uses the shared solid form for its distinct target and tool selectors,
Add/Cut/Intersect and Keep tool bodies. Its history editor shows the pre-boolean
bodies in the isolated kernel, including tools consumed by the result. Native
engine tests check exact boolean volumes and restoration; the `native-body`
Rust MCP fixture checks rendered selection, choices, exact bounds, editing,
Cancel, Undo/Redo and saving each case before opening another blank document.

Offset Plane, Midplane and Plane at Angle use the same native transaction and
the existing `solid/reference` catalog group. Browser/canvas references, straight
axis picking, typed units and formulas, offset dragging and its on-canvas field
all change a draft until Apply. The shared sketch engine supplies the plane
calculation to both the preview and saved history. Angled display patches are
centered on their selected edge without changing the saved plane coordinates.
The isolated editor restores dependent sketches and solids in one Undo step.
`cargo xtask test-mcp native-planes` checks these controls, invalid input, exact
plane placement, history editing, Cancel, Undo/Redo, dependent sketches, captures
and Save. Kernel tests also verify dependent-solid rebuilds and exact restoration.

Mirror and Split Body reuse these persistent plane references and the shared
body selector. Their isolated history editor preserves consumed source bodies;
invalid split edits leave the live document untouched and can be corrected in
the same form. The `native-body` suite also checks body toggling, origin and datum
selection, exact mirrored/split extents, both history entry points, Cancel,
Undo/Redo, capture and Save. Fixture controls are scoped to their actual surface
when a form reference, browser item and history entry share a name.

Rectangular and Circular Pattern use compact native XYZ rows, body selection,
straight-edge direction/axis picking, units/formulas, second grid direction,
negative spacing and partial/full-circle angles. Long feature forms expose their
scroll buttons to the same control registry as human input. Original-input edits
and Cancel use the shared isolated transaction. Straight-edge validation is shared
with the native picker at f64 precision. Pattern expansion is bounded in the engine
before allocating IDs, including both grid counts and all selected source bodies.
`cargo xtask test-mcp native-pattern` checks these rendered controls, exact placements,
invalid inputs, scrolling, editing, Cancel, Undo/Redo, capture and Save. Kernel tests
also verify volumes, reference ownership and exact document restoration.

Native camera controls and MCP view requests share timed Bevy transitions. Fit,
orientation, full/partial orbit and body/component/active-sketch framing use the
rendered geometry and solved occurrence poses. Framing borrows meshes; animation
samples only the camera. Requests retain their document/revision and wait for the
final frame. Manual camera changes, replaced commands, model/input changes and
window close cancel the old motion without overwriting newer input. Full turns
sample the complete path even though their endpoints coincide. The `native-view`
MCP fixture checks all six orthographic directions, ISO fit, timed orbit receipts,
background/foreground, posed components and active-sketch focus, capture and Save.

External Thread uses the original shared glyph and compact native controls for
ISO/Unified sizes, custom shaft dimensions, rounded printable profiles, handedness,
modeled/cosmetic representation, full/partial length and opposite-end starts.
The current desktop and native forms read one `interface/thread-sizes.json` catalog.
Cylindrical reference picking rejects planar faces and internal hole walls; shaft
bounds are computed once per accepted reference. Length and start previews use
bounded analytic guides. Apply uses the existing kernel fit/profile validation;
history editing recovers the original cylinder in the shared isolated transaction.
The `native-thread` Rust MCP fixture checks actual controls, selection, dropdowns,
scrolling, invalid sizes/depth, both thread profiles, edit/cancel, Undo/Redo and Save.
Native kernel checks include exact history restoration and 3MF at two mesh qualities.

Rounded-thread runouts exposed a mesher handoff defect: independently refined
boundary chords could leave a missing face or grow into thousands of samples and
stall fine-quality export. Curved chords and circular boundaries now refine
in coupled, bounded passes before OCCT's general healer. Affected face/wire flags
are reset before re-healing. Original validity and missing-face checks remain in
force; model dimensions and requested export accuracy are unchanged.

## Continuation checkpoint: 2026-09-26

The native host remains behind `dev-bevy-host` on Bevy `=0.20.0-rc.1`.
The React shell remains the release build, and the conversion PR remains a
draft. A default `cargo check` does not compile the native controller.

The native Scripts card runs only the four bundled lessons from the shared
catalog. It requires a blank document, including no active sketch, drawing,
assembly or CAM work, and uses the existing script interpreter and owned inbox.
Its thread stays separate from the modeling worker that applies those commands.
The native presentation adapter uses the existing runner protocol for captions,
Pause, Step, Resume and Stop. Playback state stays with its document incarnation.
Presentation and camera requests queued during worker completion wait for the
current operation, then run before later inbox work.
Settings displays the document's existing units; there is no engine setter to
expose, so it does not introduce another unit system.

Native CAM edits the existing CAM document. Setup, tool and toolpath fields,
duplication, deletion and generation use the shared commands and validation.
Creation forms add a setup from an explicitly chosen model body, a flat end mill
with entered geometry and cutting data, and a face toolpath from an explicitly
chosen setup and tool. Face heights retain the shared associative expressions.
Unchanged numeric fields retain their original values; displayed length/feed
units convert through the CAM document's existing units. Undo/Redo restores the
model while retaining the current workspace. Text commits on the first Apply
click, including when that click moves focus out of the field.

CAM creation currently covers one-body box stock with model XYZ axes and a
stock-corner origin, project flat end mills, and face toolpaths. Other cutter and
operation creation, advanced/rest stock, custom WCS and multi-body setup editing,
machine/post configuration, central tool-library actions, picked chains/holes,
linking dialogs, simulation, NC export and persistent toolpath visualization
remain native parity work. Existing unexposed properties are preserved in the
shared document.

Drawing labels measure model millimetres independently of view scale; dimension
offsets remain paper millimetres. Annotation changes repaint without requiring a
solid geometry revision. Native and inbox drawing edits share the bounded model
snapshot history, including sheet selection, so Undo cannot delete the solid
instead of undoing the drawing edit. Failed edits leave history intact. The
paper fits inside the content area, clips its graphics and covers the 3D scene.

History supports dragging features and the rollback marker, with timed edge
scrolling and cancellation when the document or modal ownership changes.
Keyboard and pointer input retain Winit's modifier ordering across focus changes;
the text shortcuts recognize logical Command/Control chords. These changes have
automated coverage but still require real platform and IME checks.

The new `native-drawing`, `native-cam` and `native-lessons` Rust MCP suites use
the same `--server`, `--session` and `--out` arguments as the other native suites.
They require a chosen blank native document and fresh output paths. Captures use
`cad_interface` action `capture` of the live Bevy window; saved `.nbcad` model data
is compared with the authoritative engine export. The reports deliberately mark
pixel review as required rather than inferring rendering success from receipts.

Validation status for this checkpoint:

- Windows drawing state, dimension offsets, solid edits, Save and capture passed
  the live suite. Initial pixel review found a missing paper contrast, visible 3D
  content and an overlapping Create Sketch control. Rebuilt live capture passed
  containment, contrast and values at both view scales, including the 6 mm to
  8 mm solid change. Filled arrowheads, narrow-span placement, extension gaps and
  overrun follow the existing drawing renderer's geometry and sheet styles.
  Final pixel review passed all extension lines, arrows and dimension labels
  in the locally retained 6 mm and 8 mm captures.
  Paper layout disables rounding, and displayed strokes have a one-physical-pixel
  minimum accounting for window DPI and UiScale; shared paper-millimetre widths
  remain unchanged. The previously missing top-right extension has visible ink
  in every sampled row in both final captures.
- Windows `native-cam` passed creation of the first setup, tool and face toolpath
  entirely through native controls, then edits, invalid-input rejection,
  Generate, exact Undo/Redo, Save and capture. Reviewed screenshots show readable
  controls without the previous ribbon overlap.
- Windows `native-lessons` passed: Scripts exposes only the four lessons; the
  fillet lesson stayed unchanged while paused, Step applied exactly one modeling
  operation, and Resume completed 27 steps. The editable result was saved, and
  the nonblank guard rejected another run. Reviewed lesson screenshots show no
  caption/control overlap.
- The final feature-enabled native library suite passed 335 tests with 6 ignored,
  including the horizontal history-drop guard, modifier-key form routing and
  real subpixel paper-layout and DPI pixel-coverage regressions. The MCP suite
  passed 215 tests with 1 ignored. The default React library check also passed; it is a separate
  compatibility check and does not compile the native controller.
- Windows physical input passed at 100% scaling in a 1360 x 860 window:
  Control+A selected all 11 characters of `Face finish` without inserting `a`;
  Control+C, Unicode replacement and Control+V restored the exact text. Rebuilt
  visual verification passes visible caret/selection, and Control+A in Rename
  no longer routes a stray modifier key into an empty form submission.
- Windows physical history dragging reordered
  `[Sketch1, Extrude1, Sketch2]` to `[Sketch2, Sketch1, Extrude1]`. Dragging the
  rollback marker from 3 to 2 hid the solid and restoring 3 recovered it. Moving
  Extrude before its source Sketch1 was rejected. The checked document was saved
  as `native-history-windows.nbcad`. Final horizontal drop-boundary tests pass
  for both feature and rollback-marker drags outside the strip.
- IME, additional display scales, Linux and macOS checks, including macOS
  Command+A, remain outstanding. The release build remains React until the
  remaining workflow and platform checks pass.
