# Native transition status

Checkpoint: 2026-09-27. This is the current status for draft
[PR #124](https://github.com/jackControls/noBS-CAD/pull/124), not a release signoff.
Older implementation notes in [ADR 0003](adr/0003-bevy-interface.md) are
historical evidence and may describe checks superseded below.

## Decision

**The transition is unfinished. React remains the release shell.** Bevy is
pinned to `=0.20.0-rc.1`. Keep the PR draft; do not merge or force-push.
A default Cargo check does not compile the native controller. Native proof
requires `--features dev-bevy-host`.

The most serious reproduced blocker was CAM Undo deleting the final solid
feature after an attached read-only planning query. `db159a16` fixes the shared
read/mutation receipt distinction and preserves native Undo/Redo; its real-solid
regression fails against the old bridge and passes against the fix. `e2ee94e5`
corrects rear-edge selection for coincident drawing circles. Fresh live CAM and
center checks remain required; source fixes are not a live-input signoff.

The branch is large: at `75cb8439`, its diff against `origin/main` spans 637
files and roughly 156,000 added lines. This consolidation audit checks missing
work, test evidence, and status claims; it is not a complete independent review
of that implementation. Passing tests do not establish release parity.

## Original requested workflows

- Scripts runs the four catalog lessons only on a blank document, using the
  existing runner. General script authoring and recipe URL editing remain
  unfinished; native startup explicitly rejects recipe URL editing.
- Document units remain read-only because the shared engine has no setter.
  No second unit system was introduced. Other shared preferences are editable.
- CAM edits the existing setup/tool/operation document. Exact mutation,
  history, archive, and several real-input workflows have passed. Fresh Linux
  geometry/linking checks failed. The reproduced history cause is fixed, while
  the fresh live rerun remains open; do not call all CAM gestures validated.
- Native drawing dimensions have real-solid and live-sheet pixel evidence.
  This does not establish every annotation's authoring or output parity.
  HoleNote authoring is implemented but its live fixture/export proof is open.
  An unmatched circular pick defaults to a manual `THRU` label; that is not a
  measurement proving the source solid has a through-hole.
- Feature reorder and rollback dragging are implemented with earlier live and
  history checks. This does not validate every gesture on every platform.
- Keyboard/clipboard and fixed-scale Linux input have live evidence. Actual
  macOS Japanese IME delivery now passes its bounded scenario. Windows Bevy
  IME delivery, candidate-popup placement, Wayland, and mixed-monitor DPI remain
  open. This consolidation used local headless checks and disposable CI input.

## Evidence and failures

- The attached-read fix passes seven native history tests and seven playback
  tests after its final feature-enabled build. Separate bridge checks pass 32;
  MCP session checks pass 43 plus the tool-map check; shared operation metadata
  checks pass eight. TypeScript and browser contracts pass. Five genuine CAM
  read operations over a real solid and generated setup preserve the exact
  document and Undo/Redo. Reads avoid false revision/dirty changes and geometry
  preparation while retaining playback progress and mutation/session fences.
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
- [Provisioned Windows diagnosis 36350094873](https://github.com/jackControls/noBS-CAD/actions/runs/36350094873)
  at `42392539` installed Japanese capabilities and activated the modern
  Japanese profile after message pumping, then restored US input. It sent
  **zero keys**. This is prerequisite diagnosis, not Bevy IME validation.
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

Resolve the current acceptance failures and failed live CAM/center checks.
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
