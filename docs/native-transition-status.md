# Native transition status

Checkpoint: 2026-10-02, follow-up audit implementation source `7e817c1a`. The native
desktop is integrated in
[PR #124](https://github.com/jackControls/noBS-CAD/pull/124). The default desktop
is Bevy `=0.20.0-rc.2`, using application version `0.2.2`. There is one native
host, one shared CAD/CAM engine and one document command path. Tauri, embedded
WebViews, desktop React assets and the `dev-bevy-host` switch are removed.
The independent browser/WASM application remains supported. The PR is ready for
review. The remaining native code identified in this takeover is complete;
the integration PR has not merged into `main` because required checks remain
unsatisfied.

## Implemented native desktop

The native host owns modeling and sketching, feature forms and history,
assemblies and joint motion, drawing authoring/reference repair/output, CAM,
Scripts source editing and lessons, preferences/localization, file and window
lifecycle, printing, accessibility, IME and 6DoF input. Windows, macOS and Linux
package the ordinary native executable. Document units retain the existing
engine's read-only contract.

The integration preserves incoming parent commits and the current `main`
baseline. Bevy child PRs #178, #179, #181–#186, #188 and #189 are integrated.
Only #187's isolated timing commit was taken: measurements are opt-in,
bounded and stored per process. Its unrelated unfinished rewrite is preserved
on its branch. #190's disconnected accessibility tree is superseded by the
production AccessKit adapter, including guarded Windows UI Automation text
editing; no disconnected substitute is installed.

Final reliability work uses real installed monochrome font outlines for Unicode
DXF labels, standard filled HATCH entities and one hidden original TEXT for
editing. It removes fabricated glyphs and the nonstandard embedded-font section.
Native printing uses the same grapheme font resolution, avoiding usvg's mixed
script fallback omission. Unsupported glyphs fail explicitly before output;
saved drawing DTOs, placement and paper size remain authoritative.

IME caret placement stays within the field's visible bounds and recomputes
across scale changes. Provisional composition retains the original editor
checkpoint and committed text. Drawing view captions reserve dimension
clearance. Assistive actions use the current control binding, document and
modal guards; writable fields expose SetValue and read-only fields do not.

Linux AppImages retain the Ubuntu 22.04/glibc 2.35 build baseline and use actual
linuxdeploy exclusions for host Wayland client libraries. DEB checks resolve
the native executable instead of treating the package archive as a URL handler.
Native fixtures use current localized control labels, an actual undoable solid
edit for profile-export history, and bounded read-only settlement for busy
previews without replaying input or mutations.

The current-main reconciliation (#206) includes `fc560af5`. The quick-win
review fixes (#197–#200, #202–#205, #207–#209) are merged into Bevy: smaller
document reads, retained viewport/input/accessibility state, borrowed and moved
scene data, warm sheet projections and CPU rasters, exact completion revisions,
and document-specific mesh cache incarnations. The standalone main PR #201 is
ready for review; its equivalent is already merged here through #202. Runtime
latency improvement has not been measured.

## Completed remaining native conversion

- **Native interface size (#211, #213, #215).** Settings offers 90%, 100%, 110%,
  125%, 150%, and 175% sizes with shared persistence and cross-window refresh.
  Layout, viewport bounds, pointer input, pixel scrolling, text selection,
  accessibility bounds, and lesson preview pixels use the same scale.
  Scale publishes with the matching full layout, so a model worker cannot
  combine a new scale with cached old viewport bounds. Active pointer gestures
  cancel across that change; text edits and composition remain retained.
  Input queued under a retired scale cannot address the replacement layout.
- **New independent CAM height references (#212).** Native height controls
  create planar-face, level-edge, vertex, sketch-point, and level-sketch-line
  associations through the existing bounded picker worker and draft/Apply path.
  The shared resolver supplies canonical levels and stable geometry identities;
  pointer handlers do not scan geometry or store click coordinates as intent.
  Picks survive operation-geometry form rebuilds. Changed drafts and source
  receipts invalidate picking; removed or nonplanar references fail explicitly
  and require repair. Overlapping height handles prefer the nearest camera depth.

These close both confirmed implementation gaps after the current-main
reconciliation. Current-head platform/device qualification and the main merge
gate remain separate release requirements.

## Follow-up audit corrections

The next code audit found omissions in the first interface-size implementation.
Sketch forms/menus, the sketch-origin dialog, the Drawing menu backdrop and the
CAM report still used unscaled Window dimensions. At 175% size, that could place
origin/Create controls or report paging/Close below the visible client area.
#215 replaces those reads with one adapter for monitor DPI and application UI
size, retaining the last usable client bounds while minimized. Sketch menus use
a second column when needed; the Drawing menu moves upward to keep its rows
visible. Both a scale change and rejected old-scale input now cancel every
pointer owner through the same path, preserving text and document drafts.

Locked Clippy across all native targets passed with the existing warning
baseline. Two new production-widget regressions compile and cover sketch-menu
and origin-dialog control bounds at the minimum logical window, 175% interface
size and 200% monitor DPI. They have not been executed. The command-route review
also checked new CAM operation creation and the saved-reference/draft/Apply
path; reserved browser ribbon controls were not counted as missing implemented
features. This audit found no additional confirmed conversion code gap; it is
not an exhaustive runtime or physical-device qualification.

## Evidence and release scope

The audit fixes at `7e817c1aedad9275cb7a09d12b4aee33deeb1b74` passed a
dedicated locked Windows x64 release build and portable packaging with 56
runtime DLLs. The packaged executable passed `--help` loader/CLI startup
(exit 0), opening no CAD window. Its exact clean source, compiler, lockfiles,
binary/ZIP checksums and validation scope are recorded at
`D:/noBS-CAD-builds/finish-bevy-rc2/20261002-7e817c1a-bevy-audit/build.json`.
ZIP SHA-256:
`5187a3c7487840cf14a48dbb4ffdb51ee3befaaeadfac0a9290e90165192107a`.

The earlier code at `93aeb81d5f59e73805c3877a4e81dfe633e3bbc2` passed a
dedicated locked Windows x64 release build and portable packaging with 56
runtime DLLs. Its packaged executable passed a `--help` loader/CLI startup
check (exit 0); that path starts no CAD window. The clean source, compiler,
lockfile, binary and ZIP checksums, and exact validation scope are retained at
`D:/noBS-CAD-builds/finish-bevy-rc2/20261002-93aeb81d-bevy-completion/build.json`.
ZIP SHA-256:
`9b09ff5621aa277732880973caf26a52a4de3a1cf0d0b9a50ba553c32ace5541`.
This package includes native interface size, new CAM height picking, and atomic
scale/layout publication. Earlier source-specific packages remain preserved.

The Windows x64 release at `cfcda995` compiled and packaged successfully with
56 runtime DLLs. Its clean source, compiler, lockfile and checksum receipt is
retained at
`D:/noBS-CAD-builds/finish-bevy-rc2/20261002-cfcda995-review-release/build.json`.
This earlier packaged binary has not been launched. Native production library
checks, a locked WASM rebuild and the TypeScript/Vite browser build passed during
the takeover. Completion code passed locked Cargo checks and Clippy across all
native targets, including compilation of the quick-win and new height-picker
regressions. The new preference regression covers persistence and retained edits,
including deferred scale application. A focused debug test-runner build was
stopped when it began rebuilding Bevy dependencies; no test runner executed.

Before the owner's instruction to stop suites, the integrated Windows native
library passed 781 tests, with eight ignored and no failures at `944202b1`.
The 65 drawing-export checks, 34 native-field checks, five focused accessibility
checks and both installed-font shaping checks passed during this takeover.
These are source-specific results; the final print/font-resolution and fixture
cleanup has not been rerun through a suite, as requested.

Dedicated SDK-free Windows builds passed headless MCP and owned desktop
lifecycle checks. The production Windows UI Automation fixture invoked File
and Rename, wrote `UIA Café 零件` through ValuePattern, then cancelled while
preserving the exact document. Owned-window fixtures passed lifecycle, drawing
holes, mechanisms, preferences, lessons and all drawing annotation authoring.
Independent DXF audits found zero errors/fixes in the three millimetre, inch
and mixed Unicode outputs, and their rendered captures were reviewed.
The dedicated build/evidence root is
`D:\noBS-CAD-builds\finish-bevy-rc2`; generated binaries and captures are kept
outside product source.

The initial profile-export fixture assumed Rename was undoable, and the joint
fixture stopped on an explicitly unapplied busy response after four joint kinds.
The profile fixture now uses an actual undoable solid edit. Joint fixtures wait
for read-only inspection and retain failing request context; their updated run
has not been performed. Document history and input ownership remain guarded.
The local OS-keyboard fixture correctly refused to
send keys when Windows kept the user's other CAD window in the foreground.

No additional suites are started after the owner's instruction. Current-head
runtime and supported-platform qualification remains open; historical passes
below are source-specific. Physical printing, screen-reader speech, 6DoF hardware,
monitor/DPI transitions and switching-latency attribution are not established by
the latest build, package and CLI startup check. Main's required checks are
unsatisfied, so #124 is review-ready but blocked from merging. The identified
native conversion work
is committed, pushed, and merged into Bevy. Remaining release work is applicable
current-head platform/device qualification and satisfying the main merge gate;
the large validation sweep remains stopped as requested.

## Historical September 28 checkpoint

Everything below records the earlier checkpoint and its source-specific
results, failures and then-open tasks. It is retained for traceability;
the October 2 implementation status above supersedes its migration flags,
RC version, implementation backlog and local-input restrictions.

### Historical decision

**The transition is unfinished; this draft is not approved for release.** Bevy
is pinned to `=0.20.0-rc.1`. Keep the PR draft; do not merge or force-push.
The requested implementation order is feature closure, removal of legacy desktop
dependencies, then integrated validation. The branch is replacing the desktop
React/Tauri build with the default native Cargo build. The independent browser
WASM target remains separate. Earlier feature-gated test results below describe
older source; they do not validate the new default build or native packages.

## Current implementation phase

Source now includes Scripts presentation/fast execution, pacing, chapter source
navigation and catalog previews; native printing; all shared drawing annotation
families, their exports and center grips; and annotation/derived-view reference
repair. The default native Windows library now passes its complete suite:
768 passed, eight ignored, zero failures or exclusions (138.60 seconds), including
the selected-text IME fixes and both explicit/default Scripts status polling.
Live validation remains open. Settings intentionally does
not invent a document-unit setter.

Three Bevy widget integration tests also pass. All five optional windowless GPU
preview tests pass (real-solid previews/orbit, concave strokes, grid visual and
zoom continuity, sketch boundaries; 51.70 seconds). The two optional Windows font
shaping checks pass for CJK/emoji and technical drawing symbols. Only the
operator-supplied private CAM profile test remains unrun from the ignored set. The executable
built at `718e1562`, staged with OCCT DLLs, passes all ten headless MCP checks
(27 recipe steps, real solid, 3MF output and clean EOF) with SDK environment
removed and no desktop session created. This debug staging check is not a
release-package signoff. The raw build without adjacent runtime DLLs failed
the same SDK-free verifier, as expected.

Tauri command adapters and embedded WebView surfaces are removed. Native package
scripts no longer build React assets. Cargo/OS URL handling and browser desktop adapters now use the native-only
architecture. The default native executable compiles on Windows; fresh package
and platform-input CI is running. Browser type checking, production Vite build
with freshly compiled Rust WASM, the full frontend suite, retained headless
Chromium contracts, and sketch regressions pass through `a04dc478`. Final
obsolete desktop harness removal at `27cb4be7` also passes browser input,
6DoF and responsive-ribbon checks. Workflow contracts pass on Windows with
Git Bash, and workflow syntax passes actionlint with the current runner labels.
Deleted WebView mock harnesses are not replacement native test evidence.

All 60 shared drawing export tests pass, including all annotation routes in
millimetres and inches and exact-reference rejection. Independent SVG/DXF
audits and renders verified continuous curved dashes after the DXF fix. These
synthetic fixtures do not replace live real-solid sheet checks. DXF viewers
using Arial can lack technical Unicode glyphs; long existing weld labels and
some saved baseline placements can crowd adjacent text. Headless native print
preparation passes physical-paper-size and invalid-page checks. Windows run
`36378349144` passes actual owned Print-dialog cancellation twice, with exact
exported model bytes preserved and reviewed second-cancel capture. Physical
output and macOS/Linux OS print dialogs remain unverified.

Source `9ae0d276` passes default native-host CI on Windows, macOS and
Linux, plus Linux engine, frontend and version CI. At native source `526560ef`,
owned-input run `36374312328` passes Windows/macOS keyboard and clipboard,
Linux keyboard/clipboard and real IBus at 100%/200%, chamfer/revision-cloud
placement and dragging, and drawing output. Real-solid dimension captures were
reviewed; a 6 mm offset can crowd the existing view caption, so passing offset
checks do not establish collision-free layout.

Mac Japanese IME run `36374431026` passes with reviewed same-job stock provenance,
real preedit/commit/cancel events, legible native captures, Cmd+A selection and
unchanged exact CAD model. This closes the earlier first-preedit failure for
that fixture at backing scale 1; OS candidate-popup placement and monitor DPI
transitions are not established. Paper navigation and note/linear-dimension OS gestures now pass at both scales
in `36374312328`: all 24 saved annotation variants survive, the 20-view dense
sheet has 1,160 visible segments, inverse panning restores exact pixels and
the model is unchanged. Reviewed captures retain the known fixture cloud/table
overlaps; they do not establish collision-free layout.

CAM run `36374312328` now passes both scales: setup/tool/operation editing,
libraries, posting, exact history, real OS row reordering, geometry selection,
linking and generation against the existing document. Reviewed representative
captures show the real solids and generated paths. Scripts now passes both
scales in `36384216181` at `86615673`: pause/step/resume, blank-only lesson
execution, source editing and validation, guarded retained new design, catalog
previews and exact saved/original model bytes. Reports and representative
captures were reviewed. This closes status-poll starvation. Windows IME
run `36384227678` retained an empty Commit after cancellation; its raw event-count
assertion was too strict. Independent selected-text regressions exposed real
cancellation and replacement-Undo defects. The native field now checkpoints its
existing editor during composition and treats empty insertion as composition
cleanup; nonempty commits are unchanged. All 31 field tests pass, including
selection, history, focus-loss, rebinding and external updates. The live fixture
now checks exact text, selection, model and owned nonempty commits, including
selected-text cancellation and a legitimate identical replacement. A fresh
Windows run remains required. The integrated xtask
suite passes 80 unit and two replay tests. Center picking/dragging passes at
both scales with reviewed exact frontmost associations and captures. Hole-note run `36378446585` passes native
authoring, real-solid references, exports, saved files and exact history at
both scales; reviewed captures show the modeled-hole leader and edited note.
That fixture uses interface controls, not physical hole-note mouse authoring.

Package run `36385261504` at `9ae0d276` passes macOS DMG and Linux DEB/AppImage
checks. Linux now passes actual X11 Unicode input, both Wayland and X11 desktop
lifecycles, exact retained model, dirty guard, self-close response and clean
process/stdout exit. Both lifecycle journals reach completion; the actual child
URI profiles resolve the packaged executable with `%u`. Retained logs were
reviewed without truncation. The native entry point now invokes the existing
bounded response drain after its event loop exits. Both shared transport
shutdown regressions and ten package-verifier tests pass.
Windows x64 also passes its current packaged lifecycle and Unicode-input checks. Windows ARM passes headless and lifecycle checks but
refuses input because a separate Microsoft-account WWAHost window covers the
owned target. The console-free helper now identifies this exact obstruction;
its ownership guard correctly sends no input. A narrowly scoped hosted-ARM preflight now closes only that exact system
account window, retains its identity/outcome and leaves input ownership guards
unchanged. Local parser/compilation and guard-refusal checks pass; live ARM
confirmation remains required. This is not evidence of a CAD focus defect or a
passing ARM input check. macOS URI declaration is audited, not actual OS GetURL
delivery; the DMG is ad-hoc signed, not notarized.
MCP core CI also exposed a stale native source-contract boundary and formatting;
the corrected focused test and both formatting checks pass locally. The current
full MCP library also passes: 223 passed, one ignored, zero failures
(151.75 seconds at `2b1c6e79`), and again with deterministic serialization
(148.43 seconds, product source through `14fa424e`). Preserved MCP run `36379705853` at `2b1c6e79` is fully green, including
all six Windows/Linux core, vise and turbine shards and final aggregates.

Switching-performance observations remain unattributed. Run `36374321979` built
both release hosts, but measurement was incomplete because it treated immediate
focus/sheet acknowledgments as settled state. The correction must observe actual
owned OS focus and exact selected-sheet publication without replaying mutations
or relaxing geometry/history checks. Run `36378351803` then passed all eight
matched document-tab cases (480 measured clicks) and all four native sheet
cases (240 clicks). Its four React sheet cases stopped because the verifier
expected the native textbox role instead of React's text-input role, despite
correct field and model values. The narrow role correction passes ten switching
tests. Final run `36382024049` again passes all eight matched tab cases and
four native sheet cases, then exposes pinned React's stale model publication
on Dense sheet: UI reports sheet 2 while the exact model retains sheet 1.
No further baseline repair is planned. Native acknowledgment medians are higher
in this final software-rendered Linux run; the presentation contracts and
inspection overhead differ. [The measurement note](native-switching-measurement.md)
records exact values and limits. This neither proves a universal speedup nor
attributes the owner's Windows irregularities.
No overall parity signoff is implied.

## Earlier findings

The most serious reproduced blocker was CAM Undo deleting the final solid
feature after an attached read-only planning query. `db159a16` fixes the shared
read/mutation receipt distinction and preserves native Undo/Redo; its real-solid
regression fails against the old bridge and passes against the fix. `e2ee94e5`
corrects rear-edge selection for coincident drawing circles. Its fresh live
center check and live CAM rerun now pass at both fixed scales.

The branch is large: at `75cb8439`, its diff against `origin/main` spans 637
files and roughly 156,000 added lines. This consolidation audit checks missing
work, test evidence, and status claims; it is not a complete independent review
of that implementation. Passing tests do not establish release parity.

## Original requested workflows

- Scripts runs the four catalog lessons only on a blank document, using the
  existing runner. Imported scripts can be inspected, then explicitly run in
  a new retained design; the complete live fixture now passes at both scales.
  The source editor now supports validation and Save As while protecting
  unsaved drafts. Five editor regressions pass in the integrated native suite.
  Earlier launch and presentation-claim failures are corrected. Run
  `36374312328` completed lessons, import, editing, saving, recipe browsing and
  preview captures, then reused a retired preview handle after capture. The
  fixture now inspects a fresh handle. Rerun `36379706855` then reproduced
  nondeterministic saved sketch-map ordering: identical models produced
  unequal JSON strings after reloading. Stable map serialization fixes this
  without changing values or schema; the regression fails before and passes
  after, and all 139 sketch-library tests pass. Exact Scripts equality remains
  required, with raw before/after evidence retained. Run `36381542476` exposed
  status-poll starvation at 200%. The scheduler correction passes an actual-inbox
  regression and the complete live rerun `36384216181` at both scales. Source
  open/save chooser gestures and physical multiline-editor IME are not proved
  by that fixture. Reviewed playback controls exposed a clipped Show/Hide label;
  its width now accommodates the existing text with right alignment.
  Catalog and recipe URL delivery load editable source without running it.
  Dirty/uncommitted source and ongoing saves retain exit guards.
- Document units remain read-only because the shared engine has no setter.
  No second unit system was introduced. Other shared preferences are editable.
- CAM edits the existing setup/tool/operation document. Exact mutation,
  history, archive and real-input row workflows pass. The reproduced read/history
  cause is fixed; fresh Linux geometry/linking and generation checks now pass
  both scales. This is not proof of every gesture on every operating system.
- Native drawing dimensions have real-solid and live-sheet pixel evidence.
  This does not establish every annotation's authoring or output parity.
  HoleNote shared SVG/DXF export and explicit through-hole extent are now
  implemented. Unmatched circles no longer claim `THRU`; legacy absent fields
  retain their saved intent. Shared/export and real blind/through OCCT tests
  pass. The earlier dispatch-guard failure is fixed and run `36378446585`
  now passes native live placement, exports, exact history and saved files at
  both scales with reviewed pixels. This is not physical mouse-authoring proof.
- Mechanism run `36378448883` delivered the real first drag, then queried its
  assembly before the modeling worker settled. It now shares CAM's bounded,
  read-only post-gesture settlement helper; gestures are never replayed, and
  exact ownership/geometry/Undo/Redo checks remain. Rerun `36379709170` passes
  the first drag and exact Undo/Redo, then exposes a fixture using the retired
  session publisher after history restoration. It now follows only acknowledged
  history-session receipts and still verifies the launch PID and active/attached
  IDs before input. Fresh run `36381545601` at `b1cf12f7` now passes both
  scales: consecutive OS drags, joint limits, grounded rejection, unchanged
  geometry and exact single-step Undo/Redo. Final captures were reviewed at
  both scales. This proves the slider fixture, not every joint type or physical
  hardware/focus-loss/monitor transition. The local xtask unit suite passed
  77 tests with one ignored before the later switching-role regression.
- Feature reorder and rollback dragging are implemented with earlier live and
  history checks. This does not validate every gesture on every platform.
- Keyboard/clipboard and fixed-scale Linux input pass in the current owned-input
  run. Actual macOS Japanese composition/commit/cancel passes in the separately
  reviewed current run described above. Windows IME, physical monitor DPI
  transitions, and platform-specific popup/gesture limits remain explicit.
  Validation uses local windowless checks and disposable CI input.

Windows IME run `36366130195` passed its same-runner stock prerequisite, then
failed hashing its provenance file before launching Bevy. It is not evidence of
a Bevy text-field failure. The Windows harness now hashes canonical file bytes
in-process and resolves equivalent canonical filesystem paths consistently,
preserving provenance checks; eleven platform guard/unit tests pass. Fresh run
`36375226068` passed stock provenance, native preedit and first commit, then
failed second-composition cancellation. Its final failed event snapshot was
written after the assertion and therefore lost. The fixture now retains it
before asserting. A native control test proves ordinary empty-preedit
cancellation preserves accepted text and a legitimate same-valued later commit
is accepted; no speculative duplicate-commit filter was added. Fresh OS
evidence is required to identify the failing tail.

## Historical evidence and resolved failures

- The attached-read fix passes seven native history tests and seven playback
  tests after its final feature-enabled build. Separate bridge checks pass 32;
  MCP session checks pass 43 plus the tool-map check; shared operation metadata
  checks pass eight. TypeScript and browser contracts pass. Five genuine CAM
  read operations over a real solid and generated setup preserve the exact
  document and Undo/Redo. Reads avoid false revision/dirty changes and geometry
  preparation while retaining playback progress and mutation/session fences.
- Native tab lifecycle checks reproduce a closed document retaining seven
  renderer entities and strong mesh/material handles. Successful Close now
  retires that exact owner; cancelled/rejected closes preserve open tabs.
  Separate checks restore each document's Drawing workspace and active sheet.
  Sixteen File tests and four Workbench tests pass at equivalent isolated
  source `c63a4829`. [The switching note](native-switching-measurement.md)
  records the limits and prepared matched baseline/branch inputs. These fixes
  do not establish the cause of the reported performance irregularities.
- [Native CI 36364329977](https://github.com/jackControls/noBS-CAD/actions/runs/36364329977)
  at `c0e01295` passes Windows and macOS. Linux fails during the **default**
  test compile because the new native regression imports its feature-gated
  controller. `64b62ebb` gates that regression with its host feature; the local
  default test compile now passes separately from the native suite.
- The integrated Windows feature test suite at `3024e2cd` passes **749 tests,
  zero failures, eight ignored**, including the source-editor, imported-script,
  CAM history, and lifecycle fixes. The separate default test compile passes.
  An earlier build failed linking with `LNK1180` (insufficient disk space);
  inactive native build output was moved reversibly to D: before the successful
  retry. Source and retained evidence remain intact.
- After the HoleNote and MCP effect fixes through `6a263390`, the integrated
  Windows native suite passes **750 tests, zero failures, eight ignored**.
  Catalog follow-up `17fb2d07` passes its feature-enabled binary check and five
  focused tests. Exit guard `52a0289b` passes four native regressions. These are
  headless source-specific checks, not replacements for live workflows.
- The MCP provenance follow-up reproduces successful CAM reads marking authored
  scripts as modified, appending trace edits, and clearing authored source on
  attached refresh. Shared effect metadata now distinguishes those reads while
  preserving their owning-engine inbox route. Three focused regressions and
  **222 MCP library tests pass, one ignored**, at isolated `7d2b516a`, integrated
  through `6a263390`. No read is rerouted to a stale snapshot engine.
- [Fresh center input 36364350404](https://github.com/jackControls/noBS-CAD/actions/runs/36364350404)
  at `c0e01295` passes native center authoring and real XTEST gestures at both
  100% and 200%. Four original captures were reviewed: centerlines align with
  the selected circles, and center-mark/line handles remain aligned after
  dragging. The unchanged strict association checks pass the previously
  failing frontmost-edge case. The existing fixture's revision cloud crosses
  the title block; this is no automatic drawing-layout proof.
- The recovered mechanism source through `7b3318cd` passes the full Windows
  feature-enabled native suite: **730 passed, zero failed, eight ignored**.
  Its six focused native checks and new MCP preview/atomic-commit regression
  also pass. The test executable was built from equivalent isolated source
  `47ffec84`; production Git blobs were checked against the integrated branch.
  Workspace/scoped formatting, diff checks, and the fixture's `xtask` compile
  pass. The original failed fixture and test-initializer compile logs remain
  retained. These results do not establish live mechanism dragging.
- At `75cb8439`, the local feature-enabled native suite passed **724 tests,
  with eight ignored**, and the native host build completed. The separate
  default React check passed. Shared drawing tests passed 58; driver tests
  passed 60 unit plus two CLI tests, with one ignored.
- [Native CI 36351693951](https://github.com/jackControls/noBS-CAD/actions/runs/36351693951)
  passed its Windows, Linux, and macOS native-host jobs at that head. Those jobs
  do not cover every opt-in physical-input family.
- [MCP acceptance 36351693905](https://github.com/jackControls/noBS-CAD/actions/runs/36351693905)
  failed all six Windows/Linux shards at that head. The common observed
  failures expect unit-less SVG labels after the shared formatter began
  including units. `29699eb1` corrects the exact expectations without weakening
  the measured-geometry checks. The local core rerun passes 217 library and 12
  nonflagship recipe tests (three ignored across those stages); the local vise
  rerun passes, while the turbine rerun has no retained completion result.
  Later Windows core CI failed formatting in the recovered mechanism test;
  `708fe4c7` corrects that and passes the MCP formatter. Fresh remote checks for
  the integrated fixes remain pending; do not call that head CI-green yet.
- [Focused Linux annotations 36351696956](https://github.com/jackControls/noBS-CAD/actions/runs/36351696956)
  passed chamfer and cloud fixtures at both fixed scales at `75cb8439`.
  Four fresh originals were reviewed: `Place note` fits and multiline Chinese
  cloud captions clear the scallops. The earlier 36-image review is separate.
  The prescribed quad still crosses the title-block border after dragging;
  no automatic-layout claim is made.
- [Expanded Linux 36348817288](https://github.com/jackControls/noBS-CAD/actions/runs/36348817288)
  uses the older `c78d1c5b` head and completed with failures. Keyboard/IBus,
  annotations, drawing-output, and paper jobs passed. Center input and CAM
  geometry/linking failed. These outcomes
  must remain visible even though narrower native/unit checks pass.
  The center failure selects a rear circular edge where the fixture requires
  the frontmost boss edge: center deduplication drops the depth ordering already
  used by radial picking. CAM fails an exact Undo-preservation comparison after
  drill-hole generation: the retained model loses the final solid extrusion
  while CAM generation remains unchanged. Source inspection points to a
  read-only `cam_plan_setup` advancing the engine revision without a matching
  edit-history entry, invalidating the saved Undo receipt. The dedicated
  regression now reproduces that data loss, and `db159a16` passes the corrected
  history behavior. Fresh OS-input evidence is still required. Both old failures
  occur at 100%, so their 200% cases did not execute. CAM row/WCS checks pass
  at both scales separately.
- [macOS Japanese IME 36349702501](https://github.com/jackControls/noBS-CAD/actions/runs/36349702501)
  at `09860f92` passed actual preedit, exactly one commit, second-composition
  cancellation, project preservation, and input-source restoration. Three
  original IME captures were reviewed. It does not validate physical keyboards,
  candidate-popup pixels, or monitor transitions.
- [Fresh macOS IME 36366040955](https://github.com/jackControls/noBS-CAD/actions/runs/36366040955)
  at `7510829e` passes the stock prerequisite but fails Bevy preedit: Japanese
  source and AppKit focus are confirmed, yet the field receives literal `haru`
  with no accepted IME events. The old pass does not establish current
  reliability. Narrow Winit/AppKit diagnostics are being added without changing
  the input sequence or weakening the assertions.
- [Provisioned Windows diagnosis 36350094873](https://github.com/jackControls/noBS-CAD/actions/runs/36350094873)
  at `42392539` installed Japanese capabilities and activated the modern
  Japanese profile after message pumping, then restored US input. It sent
  **zero keys**. This is prerequisite diagnosis, not Bevy IME validation.
- [Windows stock IME 36362996743](https://github.com/jackControls/noBS-CAD/actions/runs/36362996743)
  receives real Japanese preedit and one explicit commit, then **fails** its
  cancellation phase: one Escape leaves the second composition active.
  A bounded second Escape is now permitted only after fresh composition
  evidence, with strict exactly-one-result checks. The new explicit
  [Windows Bevy workflow](../.github/workflows/windows-native-ime.yml) requires
  that stock prerequisite in the same disposable job before native input.
  Eight driver ownership/prerequisite tests pass. The same-job stock prerequisite
  now passes in `36366130195`; actual Windows Bevy IME remains in progress.
- Sixteen synthetic cloud export images and eight clean DXF audits establish
  the tested geometry/caption behavior. Explicit Microsoft YaHei renders the
  tested Chinese text; default Arial DXFs lack those glyphs in the independent
  viewer. DXF records a font family, not embedded fonts or CSS fallback.

## Dogfooding observations awaiting reproduction

On September 27, the user reported performance irregularities when switching
between different drawings, different parts, and different instances of the
CAD application. The precise symptom, duration, frequency, and build used have
not yet been captured. This is a user-observed issue, not a reproduced test
failure or a measured regression.

The user noted that the behavior may already exist in the original code.
Attribution remains open: do not assume the Bevy transition introduced it.
Compare the same documents and switching sequence on a known baseline and this
branch, recording the exact commits, shell/build profile, concurrent workload,
and application-instance count. Measure switching latency and CPU/GPU/memory
activity, including single-instance versus multiple-instance behavior, before
assigning a cause. No local desktop reproduction was performed for this report.

## Historical release blockers

Resolve the current acceptance failures and failed live CAM, Scripts, HoleNote,
and fresh macOS IME checks. Fresh center input now passes at both fixed scales.
Finish and validate remaining annotation authoring/output, general script and
recipe editing, profile DXF and printing, native accessibility, localization,
6DoF hardware/driver parity, and the platform/input gaps above. New mechanism
dragging recovered during consolidation needs its own clearly scoped validation;
its presence in the branch cannot count as a live-input pass.
This audit establishes no new large-model performance or interaction-latency
benchmark; compilation and small-fixture correctness are not performance proof.

Keep retained failures and source-specific evidence. Do not convert a missing,
ignored, stale-head, synthetic, or unreviewed check into a pass. Record dogfooding
observations and their reproductions separately from automated validation.

## Consolidation and cleanup

The worktree audit found unpublished mechanism dragging in the older Bevy
checkout. `3155b8c3` recovers it through the shared assembly solver and atomic
motion command. Final guard/test corrections and their results are recorded in
[the mechanism note](native-mechanism-drag.md); OS dragging remains unvalidated.
The original files and patch are preserved. Twenty redundant Bevy task worktrees
were retired only after checking semantic integration and archiving branch
history, the superseded cloud patch, and isolated validation evidence. Their
branch refs and a verified Git bundle are retained. Unrelated worktrees and the
original mechanism checkout are untouched.

Generated executables, dependency caches, and raw local captures remain ignored;
they are not product source. Published CI links above identify remote evidence.
The local audit, hashes, patches, and retained evidence are indexed under
`.codex/handoff-audit/`. No local GUI was launched for this consolidation.
