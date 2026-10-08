# Native transition status

Release qualification checkpoint: **2026-10-06 UTC**, with the local UI walkthrough
updated **2026-10-08 UTC**. The default desktop on the Bevy integration branch
uses **Bevy `=0.20.0-rc.2`**, application version **0.2.2**, one native host and
one shared CAD/CAM command path. The integration is tracked by
[PR #124](https://github.com/jackControls/Limo-CAD/pull/124) and has not merged
into `main`. Passing required checks and an external approval remain merge gates.

The public [Bevy preview](https://github.com/jackControls/Limo-CAD/releases/tag/bevy-preview-0.2.2-20261004.1)
contains **Windows x64 ZIP and Ubuntu 26.04 x64 DEB from `9b082687`**.
Windows passed SDK-free headless/desktop MCP
checks on Thunder; Ubuntu passed its hosted MCP, X11 and Wayland-desktop checks.
The independently built hosted Windows x64 package also passed native-input
checks on this clean source. The AppImage build and Ubuntu 26.04 qualification
passed in the [tagged package run](https://github.com/jackControls/Limo-CAD/actions/runs/37232261112),
but that artifact has not been added to the public preview. Windows ARM64 failed;
macOS built and signed but remains blocked by Apple's team-agreement HTTP 403.
The superseded October 2 preview release was removed; its source tag remains.
Application version alone does not identify which source was built. Published
packages retain the former noBS-CAD name while the repository and public project
name are Limo CAD.

Thunder uses one canonical **Limo CAD** executable at
`%LOCALAPPDATA%/limo-cad/bevy/Limo-CAD.exe` for the GUI and `--headless` MCP.
Managed local commands verify the source checkout, SDK, feature mode and installed
payload before reuse. The adjacent `runtime-manifest.json` records the deployed
revision and SHA-256; inspect `build_pair.status` and require `matched` before
qualifying a live GUI/MCP pair. Application version alone is insufficient.

The October 7 manual reconnect now has an explicit `cad-call --installed
--interactive` path. It verifies the recorded clean deployed identity, enabled
native control and all installed payload hashes without rebuilding, promoting or
closing CAD when the source checkout advances. Live reconnect to the unchanged
`0db4c009` payload returned a matched GUI/MCP pair and rejected an inactive bench
session after a second tab became active. Rendered bench capture succeeded. A
foreground request was denied and a fresh observation confirmed CAD was still
in the background; physical input qualification after this reconnect remains
pending foreground ownership.

The Windows focus source now synchronizes with the window queue for at most one
second after an accepted activation request, then rechecks process/document/window
ownership and actual foreground state. Its native-control build check and full
native desktop build pass. The fixes were deployed from clean `2e98c2f5`; the
reopened bench reports a matched GUI/MCP pair. Genuine Windows foreground denials
remain correctly rejected. On October 8, an accepted guarded activation on clean
`75a04561` was followed by a fresh foreground=true observation and a physical
click selecting the bench tab. This qualifies that accepted activation and
pointer sequence; it does not establish universal focus or gesture timing.

The user-reported `Medix_KW22_v4.STEP` import reached OCCT transfer, then failed
with "could not discretize tangential face boundaries without crossing chords".
The complete 42 MB SolidWorks AP214 file contains 24 solid definitions and 8,478
faces. The custom mesher now recognizes crossings inside a real shared vertex's
OCCT vertex/edge tolerance instead of refining those tolerated junctions until
failure. Crossings outside that tolerance retain refinement and rejection;
remaining errors identify the face, wire, edges, curve types and tolerance.
The STEP source is unchanged. The native desktop build passed and the correction
is installed in `2e98c2f5`; physical UI import qualification still requires
foreground ownership. This does not yet establish successful Medix import or
broader geometry qualification. Before deployment, the prior bench's published
model was compared semantically with its saved archive and matched exactly; its
second tab contained no features after the failed import. Recovery and session
data were preserved.

On October 8, foreground ownership was confirmed and guarded physical input
opened a new tab, selected File > Import STEP, entered the Downloads path in the
owned native dialog and accepted it. The original crossing-chord error no longer
occurred, but import still failed at body 1 face 2225 (area 0.14921575060521969,
aggregate mesh status 6); the tab remained empty at generation 1. The mesher now
runs OCCT's standard healer before its custom circular-boundary sampling, then
checks affected boundaries before clearing intersection-specific failure flags.
Generic failures and strict nonzero-face triangulation checks remain. Additional
diagnostics report the failed face's own status, surface and bounded wire/sample
counts. Clean `75a04561` built and was retried through guarded physical input;
the same face still failed. Its own status was 6, its surface was B-spline, and
its three boundary edges had 22, 11 and 18 samples. The next correction refines
only edges reported by OCCT's boundary-intersection checker, evaluates added
points on the exact curves, preserves healed UV endpoints and rechecks adjacent
faces. It is bounded to eight passes, 4,096 points per edge and 65,536 added
points per model. Strict rejection of unresolved nonzero faces remains. This
candidate built and was physically retried from clean `ef4c3422`. The same face
still failed with status 70 and edge sample counts 2,689, 11 and 2,177. Refinement
alone did not resolve it; successful Medix import is not yet established.
Further investigation measures the actual UV crossing and its physical distance
from the shared vertex before changing boundary connection behavior.

The October 7 [human-operated bench checkpoint](../examples/checkpoints/garden-bench-human-ui.limo)
was built through real OS mouse/keyboard input on matched clean GUI/MCP builds
through `0db4c009`. It contains eighteen named features, four separate posts,
two shared aprons, two shared upper side rails, eight post pilots, six shared apron clearance positions,
four shared upper-rail clearance positions, a separate 28 × 415 × 90 mm lower-rail
stock and five named views. The lower stock is fully constrained but still has no
bores, component definition, appearance or assembly placement. UI checks cover fully
constrained stock, translated/rotated shared editing, precise multi-position Hole
editing, instance removal and Undo/Redo. The [walkthrough](../examples/scripts/README.md)
states its unfinished geometry, joints, drawings and print layouts. The vise and
turbine have not yet been rebuilt in this human-operated pass. No test suites or
recipe replay were run for this pass; read-only MCP inspection verifies the result.

Live sketch checks on `f055cc02` confirmed that Dimension can pick an edge through
its constraint glyph without hiding annotations. Select still opens the glyph
inspector and the stored dimension editor; cancellation retains zero degrees of
freedom. Wheel zoom worked over a glyph after ordinary Select interactions. The
first wheel after Fit did not move the camera; its cause remains unattributed,
so this does not qualify every move/wheel timing sequence or physical pinch input.

Native computer control is opt-in through `native-computer-control`, with empty
default features. The Windows backend reuses Enigo for input, Windows Capture for
owned native dialogs and that crate's in-memory PNG encoder. The generic Windows
qualification driver uses Rust and this same control path behind xtask's opt-in
`native-control-harness` feature; specialized IME/print probes and Linux/macOS
drivers still include platform scripting. These source changes do not establish
cross-platform input qualification.

The historical `7137887f` payload passed ten SDK-free MCP checks and 27 steps,
including Save, disconnect survival and guarded close. That evidence remains
specific to that revision and does not qualify the newer walkthrough or replace
the public Windows/Linux package qualification.

Cursor and Codex now register **`limo-cad`**. Their retired CAD entries were
removed; the retired Grok entry was also removed. The Start menu, project/recipe
associations and previous launch paths route to the new payload. The three retired
physical payloads were deleted after the user's manual cleanup. Documents,
recovery saves and the session registry remain intact. See
[runtime identities and migration](limo-cad-runtime.md).

The October 5 cleanup removed 24 local branches only after confirming their
commits were reachable from Bevy and they had no open PR or active worktree.
Unique retired work remains in the verified 104-head Git bundle at
`%LOCALAPPDATA%/limo-cad/archives/Limo-CAD-retired-branches-20261003-051405.bundle`.
Three inactive runtime trees moved to
`D:/limo-cad-maintenance/retired-runtime-payloads`; all 229 files retained their
hashes, recovering about 1.6 GiB on C:. The subsequent manual deletion removed
all 229 runtime files and recovered about 2.4 GiB on D:. Their separate hash
manifests remain. Active CAD windows,
documents, recovery saves, SDKs and session data were preserved.

The complete identity migration moves native configuration to
`org.limocad.desktop` and new leases/inboxes to `limo-cad-sessions`.
The real previous profile moved intact; every existing file retained its SHA-256.
Fresh MCP attach, rendered inspection and read-only assembly execution passed
against the normal installed desktop without advancing its model generation.
The installed executable SHA-256 is
`8EDC25D99BACF40EC7B3887805AE4F39E36DB8D6B7A8F8F37738E155B36AFD02`.

Ordinary Rust/C++/JSONC comments and auxiliary workflow/probe comments were
removed while preserving Rust documentation and interpreter directives.
Stale TypeScript/React descriptions were corrected. Focused migration, archive,
CAM-header/replay and workflow checks passed; strict scoped tooling Clippy passed.
The comment-removal spacing issue in CAM documentation is corrected. Strict
all-target, all-feature Windows desktop Clippy and scoped native-engine/sketch
Clippy pass on the current integration source. The October 6 follow-up scopes
printer-only SVG resources to Windows and fixes macOS 6DoF lints; its hosted
Windows and Ubuntu native-host jobs pass. This does not establish a globally
warning-free build or qualify every platform. No large validation sweep was run.

## Implemented desktop

Bevy owns modeling/sketching, feature forms and history, assemblies and joint
motion, drawing authoring/reference repair/output, CAM, Scripts and lessons,
preferences/localization, file/window lifecycle, printing, accessibility, IME
and 6DoF input. Document units retain the shared engine's read-only contract.
The native document/OCCT host lives in `crates/native-engine`; the desktop uses
a small adapter. Its optional `native-occt` feature owns transactions, exports,
geometry revisions and tab retention without a Bevy/window dependency.
Ordinary engine builds leave that feature disabled and require no native SDK.

The conversion includes:

- Persisted interface sizes from 90% to 175%, independent cross-window refresh,
  matching layout/scale publication, and guarded pointer/text/composition state.
  The follow-up audit fixes large-size sketch menus, origin controls, drawing
  menus and CAM report bounds. Current main's normalization and Ctrl/Cmd
  plus/minus/zero shortcuts are preserved (#211, #213, #215, #251).
- Independent CAM height references for planar faces, level edges, vertices,
  sketch points and level sketch lines. The shared resolver supplies stable
  identities through the existing bounded picker and draft/Apply path (#212).
- Associative drawing exports, placed views and reference repair; installed-font
  outlines for Unicode DXF labels and native printing. Unsupported glyphs fail
  before output. Saved drawing DTOs, placement and paper size remain authoritative.
- Production AccessKit bindings with current control/document/modal guards,
  Windows UI Automation text editing, and read-only/writable field distinctions.
  IME caret placement follows visible field bounds and scale changes; provisional
  composition retains the existing editor checkpoint and committed text.
- Restored inactive-tab eviction, including finished-sketch Undo/Redo (#222, #249).
- Cached authored viewport metadata per native document, independently of
  assembly placement. Immutable engine queries share this data with UI and MCP
  and skip mutation evidence observation. Sketch/solid edits refresh metadata;
  drawing edits, placement changes and revisiting retained tabs reuse it.
  Eviction releases the cache. Browser/history actions, units/name reads and
  synchronous exports borrow their source data under the existing engine guard.
  Drawing panels and hit testing borrow retained annotation marks. Paper/cache
  keys and saved drag receipts share immutable view/style metadata; revision
  stamps remain independent so sheet selection cannot retag a saved receipt.
  The title-block cache borrows only rendered metadata during lookup and retains
  an owned snapshot after successful rendering. Annotation edits reuse frame
  artwork; rejected frames move the existing sheet snapshot into the error receipt.
  Pose-vector copies, per-body replay invalidation and presentation resource
  granularity remain tracked in [#333](https://github.com/jackControls/Limo-CAD/issues/333).
- An explicit Winit window-icon binding (#259). The deployed Windows small-icon
  handle and native chrome capture confirm the title-bar fix. The packaged MCP
  passed schema-7 attach, rendered inspect and a read-only assembly query.
- In-place sketch editing of a selected shared occurrence (#94). The native UI
  and `sketch_edit` MCP operation use the same validated occurrence frame for
  rendering, picking and dimensions. Surrounding occurrences fade without
  replacing shared meshes. Saved sketches retain definition coordinates;
  recompute updates shared bodies and joint references. Focused tests cover an
  offset component coordinate system, translated/rotated repeats, driving edits,
  save/reopen and rejection of unrelated targets. Linked external files remain
  a separate design decision; physical bench qualification remains outstanding.
- Stateless multi-document MCP routing (#12) through `cad_route`, preserving
  the default attachment and never loading routed models. Per-document inboxes
  and control queues enforce captured owners and generation conflicts. Tickets
  retain completion receipts across tab changes, replacement and close.
- Typed shared drawing commands (#93) add/update/delete specialized annotations
  and views, preserve aligned groups, manage templates and append revisions.
  Release commands validate current topology and occurrence selection. Accepted
  model/assembly edits return affected issued sheets to Draft while retaining
  revision history; unchanged recompute and save/reopen preserve issued metadata.
  Native view and annotation editing use these shared operations. Idle annotation
  preview borrows its sheet, and point/radial pick targets reuse projection stamps.
  Drawing topology capture borrows the solid scene instead of copying its meshes.
- The component-edit recovery lesson (#16) uses the shared in-place operation,
  demonstrates a wrong driving value and Undo, updates rotated repeats and
  saves/reopens. Native lesson and Help catalogs expose the same authored source.
  Teaching and physical-input review remain open.
- The bench recipe creates 22 editable review sheets, including part dimensions,
  machining coordinates, an assembly sheet and cut list. The focused native check
  reproduces every SVG/DXF after reopen and updates the picket-height dimension.
  Its notes retain authored machining inputs; manufacturing review and physical
  timber/hardware qualification remain required.
- System appearance following (#272). Bevy 0.20 moved Winit windows out of the
  World; querying the obsolete resource always selected Light. The main-thread
  capture now reads initial OS appearance and primary-window theme-change events.
  A focused regression covers Dark/Light changes and ignores other windows;
  explicit appearance preferences still take precedence.

Quick-win fixes retain viewport/input/accessibility state, warm drawing-sheet
projections and CPU rasters, reduce document/scene copies, and use exact completion
revisions and document-specific mesh-cache incarnations. Incoming parent commits,
main's naming/translations/drawing changes and the recovered mechanism-dragging
implementation are preserved. No runtime latency improvement is claimed without
measurement.

## Inactive-tab retention

The former desktop's low-memory eviction was lost when its memory-status caller
was retired. The native watcher now probes physical memory every 30 seconds.
The ordered worker makes eligible inactive tabs cold after 60 minutes; constrained
memory evicts the oldest eligible tab and critical memory evicts all eligible
inactive tabs. Active tabs, unfinished sketches and saves in progress are protected.

Cold tabs retain their parametric model, geometry revision, replay baseline,
file/archive ownership, saved receipts and history while releasing the OCCT
engine, Bevy model meshes and drawing caches. Activation rebuilds transactionally
and verifies body identities and feature errors. Failure preserves the snapshot
and previous active tab. The 128-tab bound includes cold tabs.

The October 3 audit also found that serialized reconstruction discarded finished
sketch command stacks. Finished sessions now move into a separate in-memory
retention record, preserving Undo/Redo, runtime editing state and entity identity
high-water marks without retaining an OCCT kernel or solid scene. Rebuilt sketch
states must match before ownership moves; mismatch leaves the snapshot available
for retry. Normal project serialization/schema and file-reopen history policy
are unchanged.

Ten focused Windows tests passed, including real-OCCT reconstruction, repeated
eviction, actual sketch Undo/Redo, rejected restoration and retry, mismatched
sketch-state rejection, file/archive history, protected states, pressure/idle/LRU
policy and drawing-cache isolation. They were isolated in-process checks and did
not change a live document. **The public preview includes both eviction
restoration and the finished-sketch history fix.**

## Dependencies and build tooling

Rust `1.99.0`, rustfmt and Clippy are pinned together. Bevy stays at rc.2; OCCT
stays on the 7.9 ABI. AccessKit remains on Bevy's `0.24` types, Windows bindings
on wgpu/gpu-allocator's shared `0.62.0` types, and usvg/resvg on `0.45.1` because
svg2pdf `0.13` consumes those trees. These are compatibility constraints.

Repository maintenance uses Rust `cargo xtask`: scoped checks/Clippy, dependency
inventory, deterministic archives, version/tag/repository/icon/knowledge guards,
OCCT SDK orchestration, native fixtures, WASM build/smoke, packaged MCP setup and
Windows ZIP/Linux DEB/AppImage/macOS app/DMG packaging. Builders retain runtime
library and license staging, audits, checksums and platform signing/notarization.

Tauri, embedded WebViews, the `dev-bevy-host` switch, React/Three.js source, npm
manifests/lockfiles, Vite/Tailwind configuration, Node drivers, legacy bundlers
and obsolete browser/desktop IPC harnesses are removed. Native SDK containers
and packaging workflows do not provision Node/npm. Embedded vectors and locale
dictionaries remain in `assets/`; viewport colors live in Rust.

Remaining native OS qualification helpers use shell, PowerShell, Python, C# or
Swift for platform APIs and input. The repository is not entirely Rust.
Retired harness ownership is recorded in [scripts/README.md](../scripts/README.md);
that reassignment does not establish equivalent coverage or passing native cases.
Unused Feathers/scene support, redundant widget declarations, unused icon
variants and GTK/Rsvg AppImage development inputs were removed. Winit's actual
X11/XCB/cursor/input runtime libraries and Linux desktop portals remain required.
`sysinfo` is required again for the portable physical-memory probe.

The incoming standalone SDK-cache repair in main #245 is also ported to Bevy
(#302). Reused extracted sources are verified against the pinned archive before
reuse and receipt publication. Install-prefix locks exclude concurrent installers
using different cache directories; receipts reject external/dangling SDK links
and track internal targets. Eight focused Windows cache/SDK checks and strict
all-target xtask Clippy passed. The two Unix symlink fixtures remain for hosted
Linux qualification; no real SDK was downloaded, rebuilt or modified locally.

Focused checks cover Windows desktop/MCP compilation, Rust/wasm32 compilation,
Clippy, repository/version/icon/knowledge guards, package staging/deletion guards,
archive determinism and a fresh engine-facade build. Rust task-runner compilation
also passed for Linux x64 and macOS ARM64; compile checks do not qualify native
packages or signing. No broad validation sweep is being run.

The October 6 drawing follow-up passed strict Desktop/MCP all-target, all-feature
release Clippy and all three workspace format checks. Eleven focused native checks
cover exact view/release Undo/Redo, borrowed idle preview, retained paper navigation,
lesson catalogs, guarded hole authoring and profile export. Native MCP checks cover
specialized annotations, stale-edit rejection, template/revision commands and
release preservation/revocation. The bench check reproduces 22 SVG/DXF sheets after
reopen and updates an associative dimension after a height edit. Its Rust author is
idempotent and preserves all 883 original construction steps and original checks.
The opt-in turbine recipe reproduction binds both placed assembly views to the
production Bevy paper image and reuses the document/image on idle repaint. Its
linework pixels were independently inspected; this CPU image proof does not qualify
GPU scanout or OS-window capture. These additions postdate the historical
`7137887f` payload and public `9b082687` packages above. Consult the current local
runtime manifest for installed source identity; public package qualification is
still recorded separately.

## CI and security review

The October 4 cleanup landed full native-workspace formatting, narrow Clippy
fixes and compiler API updates, plus persistent fmt/Clippy jobs in the existing
required workflows (#284, #286-#288). Local native all-target/all-feature release
Clippy, root tooling Clippy and all three workspace fmt checks passed. The next
pass cleared the 20 xtask warnings and added a strict all-target xtask gate
without replacing workspace-wide Clippy (#298). Four core default warnings are
also cleared with unchanged defaults and serialization (#300; main #299).
Strict all-target checks pass for those packages; other workspace warnings remain
visible. Focused existing regressions and workflow contracts passed. Actionlint
uses an explicit inventory of the verified Ubuntu 26.04 and Windows 11 VS2026 ARM
runner labels; unknown-label errors are not ignored.

The `b2e5e241` and `985392f2` integrations passed Windows, Ubuntu and macOS native
host CI. Later integration source requires fresh checks. The `985392f2` package
and MCP runs exposed three separate failures now repaired below.
No broad local validation sweep or new live-model test was run.

CodeQL 2.27.1 now scans Rust, C++, Actions, JavaScript and Python (#289; standalone
main counterpart #285). Matching Rust compiler, sources and proc-macro server
bindings restore usable extraction for current Bevy dependencies; scan checkout
uses LF to avoid the extractor's escaped-newline CRLF parsing defect. The local
Rust scan extracted 775 files, with one platform-only macro warning and no
extraction-error query results. Its 146 logging alerts were source-audited:
138 are test diagnostics and eight are production diagnostics carrying public
CAD routing UUIDs. The IDs are discoverable through session tools and do not
authenticate callers. The matching GitHub alerts were dismissed as false
positives and three bot threads resolved; the logging query remains enabled.
The corrected Actions scan reports zero findings. This historical scan does not
qualify the current head. Main #285's Rust and C++ jobs hit the 90-minute job
deadline: Rust spent about 69 minutes preparing the cold SDK, and C++ was still
building it at cancellation. The neutral SARIF check came from unsuccessful-run
diagnostics, not a completed Rust scan. Actions and JavaScript uploads succeeded
on that exact merge checkout; native coverage remains pending on the corrected
workflow. SDK caches were already published after successful setup, so cache
publication and security coverage were retained. #301 applies the same bounded
repair to Bevy: a 240-minute job, 150-minute SDK setup and 60-minute native analysis
phase. Fresh completed scans must still qualify the new heads. Main's earlier
Tauri macro warnings also need review after its scan completes; the clean Bevy
extraction does not establish main coverage.

The audit also confirmed Unix session snapshots could be readable by other local
users under permissive default filesystem modes. The shared Rust storage policy
is merged into Bevy (#292), with a separate main PR (#291). Unix uses a per-user
registry with private directories and snapshots. Reads and writes validate
ownership and ancestry, and reject symbolic links or non-regular payload files.
Existing registries must already be private and are never made acceptable by
changing their permissions. Windows keeps its existing default discovery
location. No live registry was moved or modified during this work, and the
installed package was not replaced.

Both privacy branches passed their focused hosted Windows and Ubuntu checks:
fmt, strict all-target storage Clippy, four Windows or 15 Unix storage tests and
ten actual MCP inbox tests. Ubuntu also passed all 15 storage tests under `sudo`
on disposable fixtures. The Bevy results qualify `bac76ce3`, merged at
`34cf84e7`; the main results qualify `03a15973`. They do not replace the remaining
required native/package checks or external main review.

The turbine acceptance reader now expands actual 3MF component/build transforms
and repeated occurrences into world coordinates (#293). It retains unit, finite
vertex, index, positive-volume, closed-edge orientation and solved-placement
checks. Four pure regressions and recipe-target compilation passed on both the
Bevy child and the corresponding main feature fix (#257). The preceding main
head `f2a9f396` passed hosted Windows/Ubuntu acceptance and desktop packaging.
The October 4 follow-up integrates hierarchical 3MF and named print layouts
into the Bevy native host and controls, using the existing CAD hierarchy and
one saved-view model. It includes printer selection, diagnostics, whole-group
corrections, deliberate export, repeated instances, and source-edit guards.
The actual Windows Bevy walkthrough and Bambu Studio 2.8.2.61/OrcaSlicer 2.4.1
native export round trips passed locally. Fresh pinned printer fetching matches
the embedded catalog. The material catalog remains the existing unified surface.
See [Bevy print-layout integration](bevy-print-layout-integration.md) for usage,
qualification and limits. This branch implementation does not change previously
published packages; current hosted/package checks and independent review remain
required.

MCP aggregate CI jobs now run on shard failures and skip whole-run cancellation
(#295; standalone main PR #294). Their required names, success-only gates and
artifact provenance remain intact. Focused workflow contracts and actionlint
passed. Superseded owned runs were canceled after source-identity checks; checks
for the latest open PR heads were preserved. A separate aggregate failure came
from cached empty generated-demo directories. Registry-only aggregate caches now
exclude build targets (#296); fresh-directory reservation, artifact identity and
fail-closed validation remain unchanged. Four focused workflow contracts passed.

The material PR (#263) merged normally at `59195010`, retaining both dependencies
in its sole lockfile conflict and all feature work. It also repaired the new
Linux package privacy failure: live profiles and session fixtures use an owned
private `/tmp` directory, with diagnostics copied to `RUNNER_TEMP` on exit.
Unsafe shared-runner ancestry is not accepted. Focused Linux success/failure
fixtures accompany the change; current hosted package qualification is pending.

The Windows ARM package input guard correctly refused hosted Start/Search
occluders. The preflight now dismisses only identity-verified foreground or
observed shell windows on disposable hosted ARM runners (#297). The native input
guard remains unchanged. Eight account-window and 22 shell-window managed checks
passed; actual hosted ARM input qualification remains pending.

Jack's draft GPU-stock PR #268 has separate fixes for finite-flute eligibility,
deferred grid visibility during paused playback, and missing tool-change timing.
Multi-tool or ambiguous timelines now retain CPU stock; a default-false producer
flag and unanimous matching-layer proof limit GPU removal to known single-tool
timelines. The existing CPU simulation remains authoritative. Native compilation,
frontend typechecking, the focused path producer check and actual Rust/ECS
regressions passed. The new commits do not have fresh interactive GPU evidence.
This main draft still targets the earlier viewport and is not integrated into
the Bevy rc.2 application.

CodeQL alert [#147](https://github.com/jackControls/Limo-CAD/security/code-scanning/147)
identified unsafe matrix-copy arithmetic in the packaged OpenCASCADE 7.9.3 header;
that SDK remains unpatched. Its size calculation can overflow or narrow before
`memmove`. A large-matrix application trigger has not been established. The
upstream 8.0.1 repair changes class layout and cannot be copied into the 7.9 SDK.
This finding is retained for an ABI-compatible SDK repair or a separately
reviewed SDK migration; it was not suppressed to clear CI.

## Deployment and preserved data

Codex/Cursor MCP settings use the installed Windows runtime above with
`--headless` and `LIMO_CAD_DESKTOP_BIN`. The Rust installer supports in-place
packaged runtimes and preserves Codex TOML comments (#258; main PR #262).
Start-menu, recipe URL, `.limo` file association, PATH and App Paths entries
select Bevy. Projects,
session inboxes, heartbeats and recovery snapshots survive runtime replacement.

The packaged MCP repair (#303) preserves absolute Windows UNC paths in every
client serializer. All 21 installer checks, formatting and strict all-target
xtask Clippy passed; the regression does not require a network share.

The October 4 Windows rebuild uses clean source `9b082687`. Candidate and
installed packages passed SDK-free headless and desktop MCP verification: ten
checks and 27 command steps, live-document binding, real geometry/export, Save,
retained unsaved work, disconnect survival and guarded shutdown. The checks used
private fixture documents. All five installed launch aliases have the same
executable checksum; older compatibility directories are junctions to that
runtime. A missed `.limo` association to a removed 0.1.0 download was repaired.
Three live designs were saved through MCP before the previous runtime closed.
Their recovery documents, session snapshots and deployment receipts are outside
Git under `D:/noBS-CAD-builds/bevy-prerelease-20261004`; earlier maintenance
receipts remain under `%LOCALAPPDATA%/nbcad/maintenance`.

Two audited purges remain outstanding: the 57 retired runtime binaries/DLLs in
`Roller-300/.local/cad-runtime-retired-20261003`, and
`C:/Users/jeffg/dev/noBS-CAD/target/debug/incremental`. Automatic approval review
rejected deletion with "blocked by policy", including the incremental-cache
request after explicit operator approval. No files were deleted in either purge.
The inactive cache was compressed on October 4; current build and temporary
outputs use D:. A subsequent purge of the October 4 retired installation and
inactive development executables was also rejected before execution. Those
copies remain; normal launch routes use the new installed runtime. Source
worktrees, CAD documents and live-session data are preserved.

Main PRs #308 and #309 were consolidated into #262 through merge `80e6bc3`,
preserving their original commits. They are closed as consolidated work; #262
still requires Jack's approval before main. No main merge was performed.

The [Rust agent board](agent-message-board.md) provides deployment notices through
NATS JetStream. Publishing a notice does not prove that every agent acknowledged
it, and the board does not replace the MCP document/session bridge.

## Release qualification still open

The public Windows x64 ZIP passed SDK-free headless and desktop MCP checks on
Thunder, both before and after installation. The Ubuntu DEB passed headless and
desktop MCP, X11 input/rendering and Wayland-desktop lifecycle/URI checks in the
[tagged package run](https://github.com/jackControls/Limo-CAD/actions/runs/37232261112/job/111525874975).
The restored-window and Unicode-field X11 captures were reviewed. Checksums and
embedded metadata identify clean `9b082687` source; a machine-readable build
receipt accompanies the packages. The hosted Windows x64 build also passed
MCP and native input/render checks on this source; its restored-window capture
was reviewed. The public Windows ZIP remains the verified local rebuild.

Other preview targets remain withheld:

- **macOS:** compiled and Developer ID signed; Apple notarization returned
  HTTP 403 for a missing/expired team agreement. The account owner must resolve
  that agreement before notarized distribution. No Intel Mac package is qualified.
- **Windows ARM64:** compiled and passed headless checks; the owned input fixture
  refused a click through hosted-runner Start/Search windows. #297 repairs the
  hosted preflight; fresh ARM native-input qualification remains open.
- **AppImage:** preceding-source build/glibc/headless checks passed and X11 startup
  was reached; input stopped because the host lacked `xclip`/`xdotool`. #223 restores
  those prerequisites. The later shared-runner session-fixture privacy failure
  is repaired in #263. This does not establish current-source package success.

Historical source-specific checks also cover Windows UI Automation, drawings and
Unicode output, CAM, Scripts, mechanisms, preferences and lessons. They do not
establish current-head package/device qualification. Outstanding limits include:

- Remaining platform packages,
  required PR checks and external review. Stable `v0.2.2` is a separate legacy
  release; its presence cannot qualify the Bevy branch.
- Fresh Windows/macOS Japanese IME evidence for the latest field implementation,
  candidate-popup placement and physical monitor/DPI transitions.
- Physical printing, macOS/Linux OS print dialogs, screen-reader speech,
  actual 6DoF hardware/driver behavior and macOS OS GetURL delivery.
- Broader real-input annotation/joint/gesture workflows. The joint fixture's
  read-only settlement correction has not been rerun; Scripts chooser gestures
  and physical multiline-editor IME are not established by source-level checks.
- Switching sputter attribution. The observed Windows tab/sheet irregularity
  has no matched current-source reproduction or latency benchmark. The optional
  comparison uses native Bevy builds; see [measurement scope](native-switching-measurement.md).

A build, ignored check, stale-source pass or synthetic geometry assertion is not
a current-device runtime pass. Evidence and generated captures are retained
outside product source under `D:/noBS-CAD-builds/finish-bevy-rc2`.

## Browser work still open

The replacement must reuse the desktop Bevy UI. The current Rust WASM engine
facade builds and has focused binding checks; it is not a browser CAD app.
The complete Bevy WASM host, file/storage/dialog services and geometry-service
transport remain unfinished. The planned first browser host offloads geometry
to native Rust/OCCT. The extracted native-engine host is its service-side
foundation. An optional in-browser OCCT WASM backend is separate work; the
native-service approach does not require that port. See [web/README.md](../web/README.md).

## Audited deletions and history

The snapshot [`6394fb44`](https://github.com/jackControls/Limo-CAD/commit/6394fb449f12e17dededd76dc702081ff7c277eb)
was previously mislabeled as an unfinished UI rewrite. Independently formatting
all 81 changed Rust files and their parent versions produced identical output:
it was formatting, not an upcoming feature. Its explicit revert removed no
functional implementation; the source remains reachable from
`feat/bevy-switch-timing`. The experimental accessibility tree at `8986fd77`
remains preserved; the production adapter supersedes its disconnected tree.
Recovered mechanism work remains implemented and documented in
[native-mechanism-drag.md](native-mechanism-drag.md).

Redundant integrated branches/worktrees and backup refs were retired only after
checking source representation and archiving unique history. Active work,
projects/session data and verified Git archives remain protected. Obsolete
September checkpoint prose and duplicated old release/validation narratives
are removed from this active status document; Git history retains them.

The October 4 comment cleanup at `ee7b07ce` changed 356 Rust files. Tokenizing
each file and its parent with Rust's `proc_macro2` produced identical token streams,
including documentation attributes. The source audit and all three workspace
formatting checks passed; this evidence does not qualify a new runtime package.
