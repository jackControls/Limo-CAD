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
Ordinary model edges retain their bounded world-space lift without an additional
gizmo depth bias. Coincident cavity seams remain readable while thin faces still
occlude edges behind them. The reference grid keeps its existing depth behavior.

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

Setup editing now includes named model-body membership, the existing stock kinds,
allowances, placement and WCS origin/orientation choices. Project tool fields
cover the shared cutter kinds. Operation sections expose parameters, associative
heights and linking; untouched canonical values and geometry remain unchanged.
Creation uses that same editor for Face, Contour, Pocket, Chamfer, Holemaking,
Thread milling and Adaptive. Named chains, modeled chamfers and hole references
resolve through the existing sketch/CAM adapters; they are not a second geometry
model. Manual collections remain explicit user input.

The native CAM view runs the shared planner and stock simulator on cancellable
background workers. Model, Stock and Compare views, retained paths and cutter,
verification reports and playback never mutate machining intent. One bounded
worker owns the shared playback kernel. A completed stock frame advances its
cutter/path clock; seeking coalesces requests and rejects obsolete completions.
Static paths are retained across clock ticks. Document/selection changes retire
the worker, and publication checks the exact document receipt and presentation
ownership. Previous/Next seek to physical move boundaries within the selected
operation, and the existing quarter-speed through 10x playback rates are retained.

NC and post-event output use the existing post commands and verification. The
review shows the setup's saved machine configuration, shared warnings and output
preview. Prepared bytes and the eventual atomic write belong to the same exact
document receipt. Native machine/library authoring and the remaining library
and preset workflows must pass their live checks before release retirement.

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
- Subsequent feature-enabled native validation passed 372 tests with 7 ignored.
  The installed-font check was also run explicitly on Windows and macOS: Latin
  metrics remain unchanged and CJK/emoji shape without missing glyphs. Actual
  window captures show `Café 零件 Ω 🦀`, a visible caret and text selection.
- The expanded Windows `native-cam` live suite passed setup/tool/operation edits,
  associative heights and linking, invalid input, generation, simulation views,
  verification report, playback/seek, shared NC/events review, stale-output
  invalidation, exact history and Save. These captures use a real two-body part.
- `native-exchange` passed All/Selected STEP, STL and 3MF export, native mesh
  scope selection/Cancel, embedded STEP import, exact Undo/Redo, and saving to
  the original project destination after history traversal. Exported geometry,
  millimetres and body counts are checked, and its live captures were reviewed.
- macOS actual CoreGraphics Command+A/copy/paste checks passed, without a literal
  `a` insertion. Its Metal boundary and thin-wall occlusion checks passed;
  occluded-edge captures are byte-identical to the corresponding solid-only
  references at all three tested zooms.
- Linux real input/IME and further platform checks remain in progress. The
  repeatable `native-platform --desktop-input` fixture uses only its owned
  window. Its optional IBus run uses a private Xvfb/D-Bus session; it does not
  change the user's desktop IME. The release build remains React until the
  remaining workflow and platform checks pass.
- The next feature-enabled library suite passed 379 tests with 7 ignored,
  and all 26 shared CAM regressions passed. A real cut-body contour exposed
  transient sketch Undo/Redo availability in the geometry fingerprint. Those
  two flags are now excluded; exact geometry and references remain tracked.
  Reload/attachment retains current toolpaths, actual sketch edits still make
  them stale, and affected old stamps require explicit regeneration.
- Native machine selection, post settings, invalid-input rejection and exact
  Undo/Redo passed the expanded live CAM suite. Playback checks include physical
  move stepping and quarter/half speed. NC review retains the existing machine
  review, output preview, Save and Back-to-settings flow. A real Windows Save
  dialog with typed filename wrote 3,458 bytes identical to the shared post
  command's output; the test output stays in the local QA directory.
- The live `native-cam-geometry` suite passes Contour, Pocket, Chamfer,
  Holemaking, Thread milling and Adaptive creation/editing through the actual
  fields on a solid with a through-hole. Generation remains current after MCP
  reconstruction; exact Undo/Redo and per-operation project archives pass.
  Captures show the real solid, named geometry fields and generated paths.
  An adaptive load exceeding the shared planner's memory budget is rejected
  without changing the project; a supported load entered through the same form
  generates successfully. This checks generation and persistence, not machine
  execution or collision verification of every example.
- Linux actual X11 input now passes at both 100% and 200%, including a distinct
  clipboard sentinel that proves copy completion before restoration. CJK/emoji,
  caret and selection captures are retained. IBus composition remains pending;
  a setup failure in the isolated fixture is not an IME behavior pass.
- The subsequent native platform run
  [36283940462](https://github.com/jackControls/noBS-CAD/actions/runs/36283940462)
  at `61d38dfa13bc5c6d8ef2ddd6848ae7d44cb8519b` passed all four jobs:
  Windows SendInput, macOS CoreGraphics shortcuts/clipboard, Linux XTEST at
  fixed 100%/200%, and Metal boundary/occlusion checks. Actual IBus/libpinyin
  composition also passed at both Linux scales in the private Xvfb session.
  Reviewed native captures show uncommitted Chinese preedit and committed
  `你好`; the second composition cancels without changing the committed text,
  and Home then moves the caret to the start. The PID-owned IBus popup shows
  `ni hao` and its Chinese candidates. Its origin is `(466,405)` beside the
  100% field `(466,388,428,32)` and `(932,810)` beside the 200% field
  `(932,776,856,64)`, preserving the expected physical-coordinate transform.
  The 200% native capture clearly shows the preedit underline. This proves
  X11/XIM behavior with this actual IME engine; it does not prove other OS IMEs,
  Wayland, physical keyboards, or moving a window between monitors.
- The next Windows annotation fixture passed exact existing-project intent,
  solid preservation and saved-archive checks for all 24 shared annotation
  variants on six sheets backed by four real OCCT solids. Both reviewers
  inspected `annotations-1.png` through `annotations-6.png` from the isolated
  `live-drawing-annotation-anchor` run. Linear, radial, angular, chain, ordinate
  and arc-length values, callouts, center geometry, GD&T, surface/edge/weld
  marks, datum, BOM balloon, revision cloud and multiline CJK notes are visible.
  Technical symbols use monochrome glyphs; actual text is anchored inside
  intrinsic layout containers, and view names are centered below projected
  bounds with their scale. This validates the rendering unit; annotation
  creation/editing and drawing output remain separate unfinished workflows.
- Windows `live-cam-presets-settings-fixed` passed native cutting-preset
  Add/Copy/Remove and explicit copying of a selected profile into existing
  operation cutting data. Presets remain in the shared CAM schema, and changing
  a tool's presets does not silently rewrite other operations. Simulation detail
  and tolerance use the existing shared settings in physical millimetres;
  applying those preferences leaves document geometry unchanged. NC review now
  uses its existing word-wrap setting.
- Windows `live-cam-linking-fixed` passed the expanded six-operation geometry
  fixture with editable linking-point arrays, exact history and saved-project
  checks. The native library suite passed 400 tests with 7 ignored before the
  annotation font/layout follow-ups. `NBCAD_CONFIG_DIR` isolates QA preferences
  from the user's application configuration during owned live-host runs.
- The next Windows native unit passed 453 library tests with 8 ignored using
  `--features dev-bevy-host`; the separate default-library check also passed.
  The native central tool library uses the same on-disk collection and explicit
  project import/publish snapshots. Storage Copy checks the opened source path
  and revision under the existing writer locks. Private post management retains
  shared catalog diagnostics and never executes reference-only catalog scripts.
  Setup/operation ordering uses complete validated permutations of the existing
  CAM document. Its live library, post and reorder workflows passed exact
  history and saved-project checks.
- Native sheet forms now select every sheet/view, edit shared sheet/title and
  tolerance settings, and preserve projected group alignment through scale,
  position and first/third-angle auto-layout changes. Bulk form commits call
  the existing `drawing_set_document` engine command inside the native owner,
  revision and history transaction; the MCP bulk-write surface is unchanged.
  A live eight-sheet fixture passes dirty-navigation rejection, exact Undo/Redo,
  archive preservation and solid preservation. Reviewed captures show the sheet
  border, title block, revision and BOM tables at saved paper coordinates.
  Fitted ANSI B text still needs paper zoom for comfortable reading; table and
  annotation authoring remain separate work.
- Native paper now caches complete projected curves and associations in one
  physical-resolution image, replacing the 800-segment cutoff. Raster work is
  limited to the visible paper region plus a stroke guard; explicit geometry,
  pixel and work budgets reject oversized output instead of silently dropping
  content. Fit and zoom use the same paper transform as picking; middle pan,
  wheel and pinch retain their owner and avoid engine locks. Section and
  removed-section edge-layer selection follows the existing React renderer.
  The Windows `live-drawing-navigation-mcp-flat` fixture passed exact model and
  archive preservation for all 24 annotations and a 20-view sheet with 1,160
  real projected segments. Reviewed Fit, 150% and 140% captures show the final
  views, aligned labels and clipped viewport. The corrected fixture checks
  the actual flat published canvas rectangle when restoring Fit. Its explicit
  `--mcp-only` mode proves no OS gesture behavior. A preceding guarded OS-input
  attempt was refused because an unrelated Windows Security `PickerHost.exe`
  (PID 68440) covered the owned window; that dialog was not touched.
  The combined native feature suite passed 488 tests with eight ignored;
  all 47 drawing tests passed, including ordered gesture ownership, inverse
  picking, raster reuse and DPI, section pixels, and exact released-sheet
  Undo/Redo. Native sheet/view content edits now return the edited released
  sheet to Draft while retaining release metadata; no-op edits retain release.
- Native imported-NC simulation now uses the existing shared interpreter and
  retained playback kernel, including setups with zero generated operations.
  The source editor supports bounded multiline paste, IME and file input; an
  oversized replacement cannot accidentally run an older accepted buffer.
  Parser failures remain readable in Report, and automatic empty-setup preview
  shows a neutral message instead of an error. The Windows `live-nc-complete`
  fixture passed invalid dialect/tool and byte-limit checks, physical stock
  playback/seek/rewind, exact Undo/Redo and saved-archive/real-solid preservation.
  Eleven live window captures cover source editing, both report pages, errors
  and stock stages. NC block labels use the engine's N sequence number when
  present, or physical source line otherwise. The locked native executable
  build and separate default React compatibility check passed.
- Native Settings now shares the existing theme, language and 6DoF speed
  preferences with the React desktop host. Explicit WebView preferences migrate
  without persisting detected defaults; field-level writes preserve unrelated
  settings and compatible metadata. External changes refresh idle windows.
  Failed saves retain the live choice and an explicit Retry action. Settings,
  File and ribbon labels use the existing four-language catalog. Theme changes
  repaint retained controls, glyphs, sketch previews and dimension labels without
  replacing unfinished text, command bindings or document history. The Windows
  `live-preferences-retry-final` fixture passed persistence, external refresh,
  corruption/retry, exact model/history and archive checks. Retry now remains
  visible above the error footer, with a shell pointer regression and a reviewed
  live failed-save capture confirming the fix.
  The history baseline includes the engine's existing monotonic
  allocation counters rather than dropping fields from equality checks.
  The final feature-enabled suite passed 531 tests with eight ignored and the
  locked native executable build passed. The React frontend regression suite,
  desktop build and separate default-library compatibility check also passed.
  Native 6DoF device/driver integration and remaining translated controls are
  still separate unfinished work; the speed control currently preserves the
  shared preference used by the release host.
- Native paper now creates, edits and deletes custom multiline notes and linear
  dimensions through the existing drawing commands and shared document. Exact
  projected topology anchors retain frontmost coincident geometry and occurrence
  identity; paper offsets and drag previews use the same cached projection and
  transform. Uncommitted forms survive same-document workspace switches.
  Read-only worker requests preserve a pending anchor pair, and semantic paper
  targets no longer occlude their own pointer hits. Nine focused regressions
  cover these behaviors, stale ownership, input cancellation and released-sheet
  history. Windows `live-drawing-authoring-verified` passed create/edit/delete,
  exact Undo/Redo, archive and real-solid preservation for all 24 saved annotation
  variants. All ten captures were reviewed, including Unicode notes, projected
  anchor markers and edited dimension graphics. This run uses published native
  controls; physical placement and dragging remain a separate validation gate.
  The disposable Linux XTEST fixture now exercises paper wheel/pan, note and
  anchor clicks, and annotation dragging at fixed 100%/200% scale. Its harness
  builds, coordinate tests, shell parsing and workflow lint pass, but its actual
  OS execution is still pending CI.
- Body appearance now edits the shared material catalog and canonical metadata,
  preserves untouched manufacturing fields, and commits through exact native
  history. Live checks pass all five shared 3MF slicer targets, exact archive
  contents and the unselected solid's material color. Pending drafts/errors
  survive unrelated document revisions while captured old actions are rejected.
  Open windows refresh the persisted slicer target before interactions and at
  a bounded cadence. Multiline support is opt-in on the existing text adapter;
  five regressions cover Enter, IME, Unicode/CRLF, history, caret and scrolling.
- Linux runtime packaging now declares the XKB X11 runtime and stages its
  dynamically loaded AppImage dependency closure with copyright notices.
  The shared SDK is used by package and native-test jobs. Commit `e7110705`
  adds opt-in diagnostic Bevy DEB/AppImage checks at fixed 100%/200% scale;
  [run 36288225761](https://github.com/jackControls/noBS-CAD/actions/runs/36288225761)
  passed package construction, runtime/license audit and real XTEST input on
  extracted DEB/AppImage launches at both scales. Reviewed captures show the
  Unicode text, selection and scaled layout without missing glyphs. The same
  run's ordinary macOS/Linux jobs passed, including IBus composition. Windows
  native tests, build, font checks and real SendInput keyboard/clipboard checks
  also passed, and all five Windows captures were reviewed at 1360 x 860, 100%
  scale. The Windows job was cancelled only during post-job Rust cache upload
  at its two-hour limit; this is not an overall successful job result. Its
  timeout is now 150 minutes to accommodate a cold OCCT build and cache upload.
  Windows 200% scaling, IME and monitor transitions remain unproven. This is
  not a release-host switch.
- The pinned Rust toolchain now requests its required rustfmt component. The
  Windows MCP check previously installed the formatter only for a different
  stable toolchain and failed before formatting could run. Both workspace and
  MCP formatting checks pass locally with the repository's pinned toolchain.

Remaining release-retirement checklist (React remains the release shell):

- [x] Render all 24 existing shared drawing annotation variants on native paper,
  with exact preservation checks and reviewed real-solid sheet captures.
- [x] Remove the previous 800 projected-segment cutoff and validate complex
  sheets without silent projection truncation or missed view associations.
- [x] Port sheet setup/selection beyond six sheets, auto-layout and view
  placement/scale/editing over the same drawing document.
- [x] Create/edit/delete custom note text and linear dimensions through the
  shared drawing document, with exact history/archive and live pixel checks.
- [ ] Finish placement/editing for the other annotation variants and table
  authoring. Validate actual note placement and annotation dragging through OS
  input. Custom frame dash patterns remain open; the current frame renderer
  uses solid strokes.
- [ ] Validate actual paper wheel, pinch and middle-pan input on supported
  platforms, including DPI transitions. Fit and button zoom have live Windows
  pixel checks; ordered gesture/DPI tests are not proof of real OS input.
- [ ] Complete derived-view presentation: section hatching, detail clipping,
  broken-view masks and source-view graphics still require native parity and
  real-solid pixel checks. Complete cached projection data alone is not proof
  that these decorations are rendered.
- [ ] Expose drawing DXF/profile exports and print through the shared export
  paths. Native STEP/STL/3MF exchange checks do not cover drawing output. The
  current Rust sheet exporter explicitly rejects annotation variants beyond
  note, linear, radial and angular dimensions; it cannot silently omit them.
- [x] Expose the existing body appearance/material metadata editor and shared
  3MF slicer-target preference, with exact history and exported metadata checks.
- [x] Share and migrate the existing persisted theme, language and navigation
  speed preferences, with native Settings editing, external refresh and Retry.
  Document-unit editing stays read-only until a shared engine setter exists.
- [ ] Complete native 6DoF device/driver integration and translation coverage
  beyond Settings, File and the ribbon. Persisting the speed value does not
  establish native device behavior or calibrated Windows driver parity.
- [x] Complete genuine Linux input at fixed 100%/200% scale and IBus preedit,
  commit, cancel and candidate placement checks with reviewed pixel evidence.
- [ ] Validate remaining platform behaviors: macOS/Windows real shortcut
  tests do not establish their IME behavior, Wayland support or transitions
  between monitors with different DPI.
- [x] Pass diagnostic Linux packaged-host checks with reviewed native input
  captures. Both extracted DEB/AppImage launches passed at fixed 100%/200% scale.
