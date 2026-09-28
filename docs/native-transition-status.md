# Native transition status

Checkpoint: 2026-09-27. This is the current status for draft
[PR #124](https://github.com/jackControls/noBS-CAD/pull/124), not a release signoff.
Older implementation notes in [ADR 0003](adr/0003-bevy-interface.md) are
historical evidence and may describe checks superseded below.

## Decision

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
762 passed, eight ignored, zero failures or exclusions (141.90 seconds, source
through `526560ef`). Live validation remains open. Settings intentionally does
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
preparation passes physical-paper-size and invalid-page checks; actual OS print
dialogs and physical output remain unverified.

Current source `15a8f7f6` passes default native-host CI on Windows, macOS and
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
transitions are not established. Windows IME and remaining paper/CAM/center/
Scripts/HoleNote/mechanism live checks are still pending.

Package run `36375228814` built and audited Linux and macOS packages. Linux X11
input passed; Wayland and macOS preserved an unsaved document across MCP EOF,
saved it, then hit an obsolete verifier assertion expecting a legacy close
response field. The verifier now requires the native clean-close acknowledgment
and matching document before its existing mandatory child-exit/stdout checks;
eight verifier tests pass, fresh packaged lifecycle completion remains pending.
Windows packages and the new owned Windows print-cancel check remain pending.
MCP core CI also exposed a stale native source-contract boundary and formatting;
the corrected focused test and both formatting checks pass locally.

Switching-performance observations remain unattributed. Run `36374321979` built
both release hosts, but measurement was incomplete because it treated immediate
focus/sheet acknowledgments as settled state. The correction must observe actual
owned OS focus and exact selected-sheet publication without replaying mutations
or relaxing geometry/history checks. No overall parity signoff is implied.

## Earlier findings

The most serious reproduced blocker was CAM Undo deleting the final solid
feature after an attached read-only planning query. `db159a16` fixes the shared
read/mutation receipt distinction and preserves native Undo/Redo; its real-solid
regression fails against the old bridge and passes against the fix. `e2ee94e5`
corrects rear-edge selection for coincident drawing circles. Its fresh live
center check now passes at both fixed scales; the live CAM rerun remains open.

The branch is large: at `75cb8439`, its diff against `origin/main` spans 637
files and roughly 156,000 added lines. This consolidation audit checks missing
work, test evidence, and status claims; it is not a complete independent review
of that implementation. Passing tests do not establish release parity.

## Original requested workflows

- Scripts runs the four catalog lessons only on a blank document, using the
  existing runner. Imported scripts can be inspected, then explicitly run in
  a new retained design; their live fixture still needs its fresh CI run.
  The source editor now supports validation and Save As while protecting
  unsaved drafts. Five editor regressions pass in the integrated native suite.
  The first live Scripts job failed before launch because its workflow omitted
  a required session; the owned-window wrapper is corrected. Its fresh run
  `36367025145` now reaches the host but fails with `native_busy`; this remains
  fixed by the presentation-claim routing change, with fresh live confirmation
  pending. Catalog browsing and recipe URL delivery now load
  editable source without running it; five feature-enabled catalog tests pass.
  Four native exit tests protect dirty/uncommitted source and ongoing saves.
  Preview integration is now implemented; fresh live validation remains open.
- Document units remain read-only because the shared engine has no setter.
  No second unit system was introduced. Other shared preferences are editable.
- CAM edits the existing setup/tool/operation document. Exact mutation,
  history, archive, and several real-input workflows have passed. Fresh Linux
  geometry/linking checks failed. The reproduced history cause is fixed, while
  the fresh live rerun remains open; do not call all CAM gestures validated.
- Native drawing dimensions have real-solid and live-sheet pixel evidence.
  This does not establish every annotation's authoring or output parity.
  HoleNote shared SVG/DXF export and explicit through-hole extent are now
  implemented. Unmatched circles no longer claim `THRU`; legacy absent fields
  retain their saved intent. Shared/export and real blind/through OCCT tests
  pass; the fresh native live fixture `36367022603` fails on a pending interface
  transition before note placement. Its real solid and four circular targets
  are present. The dispatch guard is fixed in source with focused regressions;
  fresh live placement and pixel verification remain open.
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
`36375226068` passed its stock prerequisite and is building the native host;
actual Bevy input remains pending.

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

## Remaining release blockers

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
