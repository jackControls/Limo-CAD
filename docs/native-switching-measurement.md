# Bounded switching measurement

`cargo xtask test-mcp switching-measurement` measures the existing published
document-tab click from request to application acknowledgment. It preserves raw
samples and exact model checks. It launches only after the existing private
Xvfb process-tree guard succeeds; do not run it on the user's desktop.

Prepare two `.nbcad` archives with distinct document names and real solids.
Keep these exact archive bytes for every run. Both tabs start in Solid. The
fixture copies them into a new evidence directory, launches one or two owned
applications with an isolated session registry/config directory, opens the
archives through File, and alternates the visible tab controls. Two warm-up
cycles precede the measured cycles. Lookup, attachment, model verification,
and screenshots are excluded from the timed click. Each request has a
45-second deadline; the measurement loop has a 15-minute budget. Failure stops
the run and preserves partial output.

On a disposable Linux runner with the repository's usual desktop dependencies,
window manager, and private Xvfb launch, invoke the current xtask binary against
each separately built host:

```sh
cargo xtask test-mcp switching-measurement \
  --server /absolute/build/nbcad \
  --out /absolute/empty/evidence \
  --model-a /absolute/fixtures/part-a.nbcad \
  --model-b /absolute/fixtures/part-b.nbcad \
  --shell native --commit FULL_BUILD_SOURCE_SHA --profile release \
  --cycles 20 --instances 1
```

Use `--shell react` for the pinned legacy baseline and `--shell native` for the
current desktop host. The same driver controls both without adding it to the
old source. Build the baseline's React frontend assets with `custom-protocol`;
the current native desktop no longer requires `--features dev-bevy-host`.
Record build logs alongside output: the supplied
commit/profile are declarations, while binary and input SHA-256 hashes are
computed. Use equivalent profiles and adapters and avoid concurrent builds.
Repeat single-instance runs before `--instances 2`. The latter brings each
owned window forward through the existing window control and records that
request separately; it does not synthesize or validate physical Alt+Tab.

The original acknowledgment timing is retained. Separate settled-navigation
timings include read-only inspection and exact-model verification overhead.
After one foreground request, bounded observation requires both actual X11
active/input window PIDs and application focus to match the owned process,
with the same document owner. Sheet observation requires the expected Sheet
name as well as the exact model. These observations have a five-second budget;
they never repeat the mutation or accept an unrelated model change.

`samples.jsonl` retains each timed action, original control identity, raw
receipt fields, owned-process CPU ticks/RSS pages/I/O, and errors. Summary
statistics exclude warmups. Loaded canonical models and initial UI snapshots
are saved separately; compare those across hosts before comparing timings.
An action may complete before the OS displays a new image: React reports UI
visibility, while the native host separately reports GPU submission. The
driver explicitly does not equate those receipts or establish a performance
acceptance threshold. It does not measure GPU time, actual GPU memory,
physical input, monitor transitions, or the cause of the user's observation.

Each tab sample also records the current owned process tree, including React's
WebKit children, before and after the timed request. Enumeration is capped at
4096 processes and 256 owned processes; each proc file is capped at 16 KiB.
Per-process PID/start ticks, CPU counters, RSS pages, and I/O counters accompany
the sums, observation duration, and any missing or truncated observations.
Sampling occurs outside the timed action. These snapshots are not atomic and
can miss exited/reparented children or work between observations. Compare
counters by PID and start ticks; subtracting whole-tree sums across changed
membership is misleading. Summed RSS can count shared pages repeatedly and
does not measure unique physical memory or GPU memory. The earlier top-level
process fields are retained, but exclude WebKit subprocess work.

The default `--scenario document-tabs` measures Solid project tabs and optional
process foreground requests. Use the separate `--scenario drawing-sheets` with
one `--model-a` archive and no `--model-b` to time the existing visible sheet
selectors inside a single Drawing workspace. That archive must contain a real
solid and exactly two named sheets, both with projected views. Workspace entry
occurs before timing. Each selection must preserve the complete loaded model
except for the requested `drawings.active_sheet_id`; no other fields are
ignored. This isolates sheet selection from per-document workspace restoration.
The checked-in `sheets.nbcad.jsonc` creates six blocks, one A4 sheet with one
front view, and one A3 sheet with twelve front/top/right views. Report sheet
samples separately from document tabs; neither measures physical input or
proves that projection pixels have reached the compositor.

## In-process switch durations

With `NBCAD_NATIVE_SWITCH_TIMING=1`, `DocumentWorkspace::activate_guarded` times each successful activation with
`std::time::Instant` and writes `duration_ms` to
`<NBCAD_SESSION_DIR>/_ui/switch-timings-<pid>.json` plus a `nbcad_switch_timing`
stderr line. File tab completion labels a restored Drawing workspace as kind
`drawing` and a Solid workspace as kind `part`. Normal application launches do
not write timing files or log samples. Each process retains at most 64 samples
in its own file. A second independent `SessionBridgeState` test fixture keeps
kind `document`; it is not a second-process measurement. The headless test
`drawing_part_and_second_instance_document_switches_record_non_negative_durations`
writes `switch-measurement.json` with `drawing_switch_ms`, `part_switch_ms`,
and `instance_document_switch_ms`, and checks that each field exists, is
finite, and is at least zero. It sets no speed budget. The clock stops when
activation returns its receipt. Rendering is unchanged. The record does not
identify a cause of the 2026-09-27 report. The test enables recording only for
its scope. Real multi-process and rendered latency checks use the disposable
switching-measurement command above.

```sh
cargo test --manifest-path src-tauri/Cargo.toml --lib \
  drawing_part_and_second_instance_document_switches_record_non_negative_durations \
  -- --test-threads=1
```

## Headless lifecycle evidence

The native File lifecycle regression uses real OCCT extrusions and the normal
renderer mesh rebuild system without a window or GPU. Before the successful-close
retirement fix, closing the second tab left its cache entry and seven renderer
entities, including strong mesh/material handles, after the engine had removed
that tab. The fixture also retains an unrelated renderer owner to catch overly
broad cache pruning. With the fix, three create/switch/close cycles each leave
exactly the live tab and unrelated owner: two cache/geometry sessions and zero
closed-tab entities. Cancelled and rejected closes preserve both open tabs.

This proves an ownership leak and its bounded retirement; it measures neither
GPU memory nor switching latency. It does not explain the user's performance
observation. Ordinary activation can invalidate the shared instance-layout
revision, a behavior also present in the baseline renderer; exact warm mesh
reuse needs a separate matched measurement.

At source `c63a4829`, the feature-enabled Windows build passed all 16 native File
tests and all four Workbench tests. These include restoring a document's Drawing
workspace and canonical active sheet after switching or successfully closing a
different tab, unchanged exact project models, cancelled/stale Close, history
epochs, and window ownership. New tabs start in Solid. Only the workspace choice
is retained: editor drafts and paper/pick receipts remain transient. Paper
pan/zoom refits on document/sheet changes in both native and React implementations.

Reproduce the headless checks on the current native desktop with the platform's
OCCT runtime available (the historical `c63a4829` build required the feature):

```sh
cargo test --manifest-path src-tauri/Cargo.toml --lib \
  session_bridge::native_interface::controller::files::tests:: -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml --lib \
  session_bridge::native_interface::controller::workbench::tests:: -- --test-threads=1
```

The existing `native-host-tests.yml` dispatcher now accepts
`desktop-input=true,linux-only=true,input-family=switching` to call the dedicated
disposable comparison workflow. It builds pinned React main
`f62248e1310fd511df20ee8bf6f2b8b268c39d50` and the selected branch's native host
in the same release profile before any measurement. The baseline embeds its
desktop assets with `custom-protocol`. Each of the document-tab and Drawing-sheet
scenarios has eight sequential invocations covering one/two instances and two
repetitions with reversed host order. Their samples stay separate. The comparison
rejects missing runs, different archive hashes, or different loaded models.
The initial dispatched run at `d4a69ade` samples only the top-level host; WebKit
children are excluded. The subsequent driver adds bounded owned process-tree
samples. Inspect each sample's completeness and process membership before
comparing counters; summed RSS is not unique physical memory.

The first dispatch (`36367020031`, source `d4a69ade`) failed before timing:
the pinned baseline's desktop lockfile omitted its already-declared local
`nbcad-print` dependency. The same `--locked` failure was reproduced in a clean
detached checkout. `baseline-cargo-lock.patch` records the nine-line repair:
one local dependency and its package entry, using already-locked `png` and
`serde_json` versions. No application source or registry version changes.
The comparison retains the original and effective locks, exact patch and hashes,
and builds with `--locked`. Any results must identify the baseline as the pinned
source **plus this lock repair**, not an untouched baseline build.

## Recorded live comparison

[Run 36382024049](https://github.com/jackControls/noBS-CAD/actions/runs/36382024049)
tested candidate `7e31bae4a7a56bdc5adb00cc4b5b869720331ad5` against the pinned
React source and lock repair above. Both locked release builds passed.
Measurements ran on 2026-09-28, 06:02:02–06:27:52 UTC. The retained
`matched-switching-evidence` artifact is `10954581962`.

**12 of 16 cases completed:** all eight document-tab cases and all four native
Drawing-sheet cases. This includes both instance counts and both host orders.
The tab comparison has 240 measured switches per host; native sheets have 240.
Each completed case preserved its exact expected full model, and two-instance
cases verified owned OS/application focus. All input archive hashes matched;
all loaded models matched across hosts. The complete two-scenario comparison
remains failed because the four baseline sheet cases did not publish the
selected sheet in their model snapshots.

The baseline failures occur on the first actual switch to Dense sheet (ID 2).
Repeated UI receipts show `Sheet name = Dense sheet`, while both
`cad_project_model` and on-disk `model.json` retain `active_sheet_id = 1`.
The active heartbeat reports generation 4 but model/published generation 2.
The five-second observation therefore correctly fails. At the pinned source,
`src/drawing/document.ts:966` queues the command and later changes
`drawingDocument`; the subscription at `src/sessionBridge.ts:376` does not
include `drawingDocument`. This is consistent with a missed publication after
an early click acknowledgment. It establishes a baseline publication limitation
in this fixture, not model corruption or a native-sheet failure. The strict
checks remain intact; no baseline repair or mutation retry was used.

Document-tab acknowledgment medians in milliseconds (40 samples per instance
and repeat, excluding four warmup switches):

- One instance, repeats 1/2: React **423.2/419.7**, native **631.3/600.3**.
  Their p95 values were React **528.4/473.9**, native **748.3/651.3**.
- Two instances, repeat 1: React **518.1/525.6**, native **633.9/623.7**;
  repeat 2: React **552.8/517.8**, native **634.7/614.6**.
- Verified-navigation medians, including observer overhead: one instance
  React **518.7/506.8**, native **874.8/855.3**; two instances across repeats
  React **600.0–646.9**, native **861.7–876.9**.
- Native sheet acknowledgment medians were **680.3–704.9**, with verified
  navigation **976.4–1009.4**. There is no completed baseline sheet comparator.

Native acknowledgment medians were higher in this run. These measurements use
GitHub-hosted Linux, private Xvfb and software Vulkan/llvmpipe, with different
React/native presentation receipts. The settled timings also include inspection,
attachment and model-read overhead; neither metric is physical input or
equivalent compositor/GPU completion. They establish **no Windows attribution,
universal performance improvement, acceptance threshold, or cause of the
user-reported irregularities**. Process-tree samples include three React
processes versus one native process; one of 480 React tab observations is marked
partial, while native tab/sheet observations are complete. RSS sums still do
not represent unique physical or GPU memory.

Earlier runs are retained without treating them as successful comparisons:
`36374321979` exposed premature focus/sheet acknowledgment checks;
`36378351803` completed all tab/native-sheet cases but rejected React's correct
`role="text"` input because the new observer expected native `role="textbox"`.
The final run above includes that fixture correction and passes the initial
sheet check before reaching the distinct baseline publication failure.
