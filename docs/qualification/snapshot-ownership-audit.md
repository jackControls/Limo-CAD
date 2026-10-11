# Native snapshot ownership audit

Source review for [#333](https://github.com/limo-cad/Limo-CAD/issues/333), based on
`278025a52247c7316c8553fd2a390f9179150733` plus the borrowed CAM posting and
drawing-export reads described below, on 2026-10-10. This is a concrete inventory
of production snapshot APIs and their desktop consumers, shared host/MCP read
contracts, and the document caches they feed. It is not a census of every clone
in the repository, a benchmark, or live UI qualification. Test fixtures and
JSON-schema construction are excluded from production copy counts.

The narrow active-sketch browser-name follow-up below is based on
`96b642b9fb481daa77351e7eedccd917ee112fc1`; its source review is separate from
runtime or performance qualification.

## Copy inventory

Each entry records the consumer, owned data and scale, owner/lifetime,
invalidation boundary, and remaining disposition. Paths under
`desktop/src/session_bridge/native_interface/` are abbreviated as `native/`.

1. **Coherent viewport frames**: `crates/native-engine/src/host.rs`,
   `NativeEngineHost::viewport_frame` and `solid_scene_snapshot`.
   `SolidSceneDto` is shared by `Arc`; ordinary reads do not duplicate its mesh
   or topology buffers. A metadata-cache miss owns sketch DTOs, datum planes,
   profiles and appearances, scaling with authored entities. Placement misses
   own body/occurrence pose vectors, scaling with placements. Each warm project
   owns its cache; frame consumers share those allocations. Metadata and
   placement invalidate separately (see below). This caching is implemented.

2. **Synchronous geometry consumers**: `native_editor/support.rs`, native
   menu body checks, `files/io.rs::capture`, `files/bambu.rs`, named views,
   print-intent controls, and `controller/section_review.rs`. Their
   `solid_scene_snapshot` calls retain an Arc for the local computation;
   collecting body IDs, choices, errors or summaries allocates those results,
   not tessellation copies. Selected export intent retains IDs/settings in an
   Arc across dialogs, not the scene/kernel. Native feature edit/cancel
   snapshots can retain a coherent viewport frame until the edit ends. These
   intentional holders can outlive one frame; a weak cache stamp alone cannot
   prove every reader has released its Arc.

3. **Retained browser/history panels**: `native/controller/browser.rs`
   `synchronize` and `controller/history/panel.rs::synchronize` each capture
   `document_snapshot` on owner/revision refresh and retain it in an Arc.
   `DocumentDto` owns names/settings, browser nodes and history feature metadata
   (`crates/core/src/dto.rs`), not evaluated solid geometry. Cost scales with
   nodes/features; unchanged refreshes share the retained Arc. Owner changes
   replace the panel snapshot. These are remaining metadata-copy candidates:
   document whether each panel needs the whole DTO or can share a smaller
   immutable projection. Do not report ordinary camera/menu reads as mesh
   copying merely because these revision-bound snapshots exist.

4. **History and feature edit boundaries**: `native/controller/history/drag.rs`
   captures a receipt-bound DocumentDto used across drag events for slot and
   operation decisions; changed receipts reject the drag. `native/feature.rs`
   `Snapshot::capture` owns document metadata, parameter/source-occurrence
   lists and decoded assembly intent/solution, while sharing its viewport.
   The edit snapshot belongs to the exact document receipt and survives
   Apply/Cancel preparation. Preserve this lifetime; replacing it with a lock
   held across user events would be incorrect. Copies scale with history,
   parameters and assembly structure, not shared scene buffers.

5. **Undo/Redo decision reads: repaired**: `native/history.rs`
   `apply_native_history_guarded` previously copied a whole DocumentDto to read
   rollback position, feature count and latest feature ID. One `with_document`
   callback now returns those three scalars coherently. Its engine guard ends
   before history dispatch. Existing owner/revision/control checks, empty
   history errors, DeleteLatest identity, owned Undo model tickets and
   transactional restore/history snapshots remain unchanged. Owned complete
   model strings/history tickets are required for recovery across mutations.

6. **Drawing paper and editors**:
   `native/controller/workbench/drawing_paper_view.rs::paint` owns one
   DrawingDocumentDto per owner/revision receipt; annotation `runtime.rs` and
   `drawing_editor.rs` share that paper-document Arc when its receipt matches,
   with an owned fallback when no matching retained publication exists.
   Copies scale with authored sheets/views/annotations/templates, not OCCT
   projected line buffers. Editor retirement releases the applicable Arc;
   dirty draft protection must run before transitions. Paper artwork retains
   one selected-sheet DTO after successful rendering, keyed by geometry revision,
   sheet content and units; a rejected frame moves that same snapshot into its
   error receipt. Cost scales with that sheet's authored data. A selected
   annotation or sheet/view draft owns the edited record/fields for Reset/Apply.
   `drawing_authoring/hole/submit.rs` creates a changed drawing document for the
   existing whole-document setter; this is an owned mutation result, not an
   inspection snapshot.

7. **Native print preparation: repaired**: `native/controller/files/printing.rs`
   `prepare` formerly copied every drawing sheet/template only to find the
   active sheet ID/name. Its short `with_drawing` callback now returns that ID
   and one name String. No active-sheet fallback was added: missing selection
   still returns `Select a drawing sheet to print`. SVG export and the immutable
   `native_print::Page` remain owned output, sized to the exported page. They
   cross worker/native dialog boundaries, with the existing repeated document
   receipt checks. The drawing callback ends before export or title lookup
   reacquires the engine lock.

   `files/drawing_output.rs::capture` formerly serialized and decoded the whole
   drawing document for SVG/DXF export selection. Its `with_drawing` callback
   now returns only the selected sheet ID/name and requested format under the
   existing owner/revision fence. Missing active-sheet selection still returns
   `Select a drawing sheet to export`. The owned output intent survives the
   picker; delayed export still revalidates the receipt and owns its SVG/DXF
   payload. Document validation remains at the existing setters/loaders.

8. **CAM intent consumers**: `native/controller/workbench/cam.rs` owns
   CamDocumentDto on editor owner/revision refresh; `cam_view.rs` owns an Arc
   per owner/revision/selection key and clears it when inactive, cancelling
   pending work. `cam/central.rs` owns the project CAM intent when opening the
   library; Close clears the receipt/draft but retains project/library context
   until the next Open or resource destruction. These copies scale with
   tools/setups/operations and regeneration fingerprint stamps. Derived toolpath
   motion and NativeViewportFrame geometry are not stored in this DTO.
   `cam_view.rs` previously discarded the DTO returned by `cam_snapshot`
   when reading its warning. Its warning-only accessor now reads the same
   setup validation under a short engine guard without copying CAM intent.
   The existing owner/revision fence still covers that read. The unchanged
   compatibility `cam_snapshot` accessor still returns owned CAM intent.
   The actual `cam_view.rs` worker retains the existing Arc document so voxel
   work can run after the engine guard ends. Preserve that worker snapshot.

   `cam_export.rs` previously copied the whole CAM document to derive its
   retained Post NC/event-stream draft. The new `with_cam` callback selects the
   same setup/operation/default and calls the unchanged `Draft::new`, returning
   only owned posting fields/configuration and summary Strings. It preserves
   `Choose a CAM setup`/`Choose a setup to post` errors, the exact receipt fence,
   and worker/export snapshots. Summary construction scans current setup/tool
   metadata under the engine guard; it is not constant-time. The callback does
   not reenter the host/bridge or retain a borrow across user events. Full CAM
   editor/library snapshots remain intentional lifetime boundaries; their
   projection/retirement acceptance is still open.

9. **Borrowed menus and actions**: `native/controller/browser.rs::action_node`,
   history validation/rename/edit preparation, file export readiness, sketch
   edit validation, `feature/editing.rs`, and `workbench/ribbon_menu.rs` use
   `with_document`/`with_drawing`/`with_cam` callbacks. They return only necessary
   IDs, names/choice lists, booleans or history indices before updating
   ECS or entering another engine method. The callbacks inspected here do not
   reenter the engine. Ribbon sheet tabs clone at most six names; datum/profile
   choices scale with the displayed metadata. These are owned UI outputs,
   not copies of the source drawing/model geometry.

10. **MCP and transport**: `crates/sketch/src/host.rs::handle_read_only`
    serializes `solid_scene_ref`, `drawing_document_ref` and
    `assembly_document_ref` directly. `mcp-server/src/lib.rs` summary,
    section/projection, preflight and mesh export paths borrow source scene
    and assembly structure; generated query/export payloads are owned
    transport results. `document` still derives an owned DocumentDto, and
    `cam_document` clones CAM intent before serialization: document those
    remaining metadata-copy boundaries or add a borrowed serializer where
    valuable. Owned tessellation/export meshes, JSON/base64 and saved model
    strings scale with the requested output and must remain valid after the
    call. The compatibility API `NativeEngineHost::viewport_snapshot`
    explicitly returns owned geometry/metadata. No ordinary desktop menu or
    navigation consumer was found using that compatibility method; current
    desktop matches are fixtures/tests. `native_viewport/ui_lab.rs` is an
    opt-in capture fixture and is excluded from normal UI copy claims.

## Native UI entrypoint census

The finite source scan at the exact base above covers the production desktop
calls to `document_snapshot`, `drawing_snapshot`, `cam_document_snapshot`,
`viewport_snapshot`, `solid_scene_snapshot`, `viewport_frame`, `cam_snapshot`,
borrowed document/drawing callbacks and the CAM warning accessor. It also covers
multiline literal `engine_call` reads for document/drawing/CAM/assembly intent,
assembly solution, active sketch, named views, print intent and project model
export. It found 73 matching source blocks in 34 files before the two new
reductions. Blocks can contain multiple calls; this is not an allocation count.
Tests, replay/capture fixtures and schema construction were excluded. Dynamic
dispatch, aliases and derived-output consumers were inspected by the groups
below, rather than assumed absent because they lack a literal API name.

The exact source excerpts remain outside the repository at
`D:/limo-cad-maintenance/backlog-autonomous-20261010/issue333-native-entrypoint-census-278-source.json`.
The covered native entrypoint groups and their lifetime dispositions are:

- **Host/publication/save/history bridge**: `native_interface.rs`,
  `native/workspace.rs`, `native/publication.rs` and `native/history.rs`.
  Published model text/active-sketch payloads and archive/history tickets are
  owned across broker/write/mutation/recovery boundaries. Publication uses the
  existing reservation/owner fence and releases engine reads before publishing.
  Published snapshot reads are not instantaneous live-engine snapshots. The
  model-frame alias shares `viewport_frame` Arcs. Active-sketch nullness and
  Undo/Redo-availability reads still serialize a whole sketch to obtain scalars;
  these are remaining narrow projection candidates.

- **Sketch and mechanism input**: `native_editor/mod.rs`, `support.rs` and
  `mechanism.rs`, plus the `active` helper consumers in `palette.rs`,
  `annotations.rs` and `interaction.rs`. Palette/annotation/hover snapshots own
  sketch DTOs keyed by editor stamp; they survive pointer/paint events and share
  retained geometry. A mechanism drag validates `can_drag_occurrence` from a
  decoded assembly under the receipt fence, then owns scalar pose/camera state.
  That predicate is another projection candidate; do not hold a host guard
  across the drag. Browser sketch-name lookup now borrows the manager's active
  session name under the existing engine guard and owns only that optional
  string. It no longer serializes/parses a complete SketchDto for the name;
  browser receipt/revision refresh and retained document lifetimes are unchanged.

- **Browser/history/feature ingress**: `native/feature.rs`,
  `native/feature/editing.rs`, `native/controller.rs`, `controller/browser.rs`,
  `controller/history.rs`, `controller/history/panel.rs` and
  `controller/history/drag.rs`. Borrowed menu/history validation returns owned
  metadata. Browser/panel snapshots, drag intent and feature edit/cancel
  snapshots retain exact receipts across events; shared frames carry geometry.
  Full DTO projections for clean browser/history panels remain candidates,
  while edit/recovery snapshots require their owned lifetime.

- **Assembly/named-view/section ingress**: `controller/assembly.rs`,
  `controller/named_views.rs`, `named_views/panel.rs` and `section_review.rs`.
  Assembly state retains an Arc per exact receipt; joints/studies/inspection
  borrow that retained intent. Named-view choices own IDs/names/placements and
  section/body selection shares scene Arcs. The assembly decode used only for
  occurrence names in named-view diagnostics remains a projection candidate.
  Section query/output buffers are derived owned results, not copied source
  scenes, and stale worker results remain owner/revision fenced.

- **File/export/print ingress**: `controller/files/io.rs`, `files/bambu.rs`,
  `files/drawing_output.rs`, `files/panel.rs`, `files/printing.rs`,
  `controller/print_intent.rs`, `print_intent/heights.rs` and
  `workbench/ribbon_menu.rs`. Menus borrow selection metadata or share scene
  Arcs. Print/SVG/DXF selection now retains only sheet identity/name/format.
  Archive, STEP, print, Bambu and drawing workers must own their selected intent
  and requested output across dialogs/writes. Print/Bambu compound summaries
  still decode model/print/assembly intent; their scalar/output consumers need
  individual projection review rather than blanket removal of export snapshots.

- **Drawing workbench ingress**: `workbench/drawing_paper_view.rs`,
  `drawing_editor.rs`, `drawing_authoring/runtime.rs` and
  `drawing_authoring/hole/submit.rs`. Paper, sheet artwork and editor drafts use
  the retained document/selected-sheet lifetimes described above. The hole
  submission creates owned changed intent for the setter. Shared paper Arcs
  avoid duplicate editor captures when their receipts match; fallback captures
  remain owned until their editor retires.

- **CAM workbench ingress**: `workbench/cam.rs`, `cam/central.rs`, `cam_view.rs`
  and `cam_export.rs`. Editor/library and background voxel work retain owned
  intent with receipt/generation fences. Posting now projects an owned draft;
  warning-only synchronization no longer copies CAM intent. Prepared program
  bytes/events/warnings are owned output. The compatibility `cam_snapshot`
  method has no ordinary desktop production caller in this census.

Direct feature/forms contexts borrow retained metadata; generated mesh/HUD/line
buffers and operation/transport outputs have their own owned lifetimes. The
opt-in script-preview service owns produced scenes and bounded frame/cache
payloads in a separate worker context; it does not read the live UI document
through these native entrypoints. No script/recipe was executed for this audit.
This closes the bounded native ingress inventory, not the full application
allocation graph, service/cache architecture or memory-pressure acceptance.

## Cache invalidation and lifetime matrix

- **Authoritative warm project / viewport cache:** native engine mutex protects
  each manager/kernel/cache. Ordinary `handle_read_only` calls return before
  invalidation. Drawing/CAM/print intent changes, document title and non-sketch
  feature rename retain viewport metadata; a renamed sketch invalidates it.
  Assembly, named-view and visibility commands invalidate placement only.
  Authored sketch/solid changes invalidate metadata; recompute publishes new
  geometry on commit. The execute path conservatively invalidates metadata
  before preparation; that does not prove a stale-data defect or justify a new
  caching architecture. Warm tab activation keeps the project cache. Cold
  thaw constructs and verifies a replacement before publishing it.

- **Installed frame / model resource:**
  `desktop/src/native_viewport/platform.rs::ModelResource` strongly retains the
  active shared document/pose vectors; replacing the frame releases the prior
  active holder. `ModelInstanceState` retains identity/visibility layout per
  document, not another full scene. Camera, HUD, annotations and preview mesh
  revisions are separate from geometry. Camera Fit uses installed geometry.

- **Body meshes / local edges:** the document-indexed `BodyMeshCache` owns mesh
  handles, and `ModelEdgeCache` owns local bounds/edge-side/face-boundary data.
  Both store weak scene identity stamps. Occurrence/visibility changes can
  reuse body mesh handles. Changed scene identity rebuilds the body map/uploads
  coarsely; transient isolated scenes are distinct. Close/eviction drops only
  the addressed document entities/cache/instance state. Weak stamps do not
  pin CPU scenes, but handles and derived boundaries intentionally retain
  assets until that retirement.

- **Drawing derived source/raster cache:** `workbench/drawing_edges.rs::EdgeCache`
  retains current and previous source/raster results. SourceKey separates
  document/geometry revisions from an Arc layout containing owner epoch,
  sheet, views and relevant styles. Refresh replaces layout only when those
  inputs differ. Only committed adjacent sheet-selection revisions can retag
  reusable source; edits/Undo/replacement miss the exact key. Document eviction
  clears matching current/previous/error entries and paper geometry. Retaining
  one previous result is deliberate rollback/error presentation, not a claim
  of unlimited sheet caching.

- **Editors / background work:** drawing/CAM clean inactive editor snapshots
  retire on the matching document policy; dirty drafts remain protected until
  Apply/Reset or explicit document close. Workers carry receipts and immutable
  payloads, validate owner/revision/control before application and reject stale
  completions. Pending native print SVG/page storage is local to the operation.
  Source correctness does not establish that every live dialog cancellation
  releases at the expected time. CAM process-service cache budgets and
  cancellation rules are documented in [CAM_SIM_PLAYBACK](../CAM_SIM_PLAYBACK.md);
  their complete global lock/lifetime graph remains outside this bounded audit.

- **Cold retention / saves:** workspace inactive-LRU/memory policy excludes the
  active tab, unfinished sketches and in-flight saves. Engine eviction replaces
  the warm manager/kernel/cache with model text, body IDs/errors, sketch recovery
  and named-view metadata, not a scene Arc. File/archive/history receipts remain
  in the workspace/publisher. Cold restoration validates IDs/errors/sketches
  before swapping. This structural policy plus weak cache stamps does not
  imply total-process memory containment or prove no temporary reader retains
  an old scene.

## Lock order and ownership

1. **UI/MCP/worker document operations:** publisher mutex -> owner/session/epoch
   validation -> engine mutex. `with_native_document_receipt` retains the outer
   publisher fence while its callback runs. `with_document`, `with_drawing` and
   `with_cam` hold only the inner engine guard for the synchronous borrowed read. Never call
   another engine method or publisher-locking bridge method inside them.
2. **Print/history/drawing-output/CAM-posting changes here:** owned callback
   outputs return and release the engine guard before export, page preparation or history
   mutation reenters the engine. The publisher fence, revision checks and
   history transaction boundaries remain in their existing positions. Print
   preparation/result storage is taken independently before completion enters
   another receipt fence; no slot guard is held across that reentry.
3. **Frame/cache synchronization:** capture a coherent Arc frame under the
   engine mutex, release it, then change ECS/cache state. Pointer/camera reads
   use retained frame/geometry and do not borrow an engine guard across events.
   Owned edit snapshots carry exact receipts instead of retaining mutex guards.
4. **Read-only routing:** `engine_call` checks immutable host dispatch before
   mutation invalidation; native inbox uses the registered read-only flag to
   preserve model revision and draft ownership. Read-only JSON allocation is
   not a model mutation. This is a reviewed path-specific order, not proof of
   the full application/service lock graph.

## Remaining acceptance and validation

Keep #333 open. The native ingress census above is source-complete for its
explicit API/literal-read scope. Remaining work is the wider service/transport
allocation and lock graph, justified reductions of the remaining metadata
candidates above, memory-pressure and background-holder acceptance, and any
consumer-specific presentation split
supported by an actual invalidation problem. Per-body render updates require
stable authoritative kernel body revisions or dirty-body identities; body IDs
alone cannot prove unchanged geometry. Do not implement that architecture from
this audit or recreate the already shared metadata/pose caches.

Existing identity/retention tests cover shared frames, independent metadata and
placement invalidation, drawing edits/tab reuse/eviction, mesh reuse and dirty
archive/history retention. The issue records historical focused passes; this
source audit did not rerun them. All five bounded metadata/selection reductions
received independent source review and scoped edition-2021 rustfmt/diff checks.
The two new CAM/drawing-output reductions have not been compiled or exercised
by this audit; the installed runtime and earlier focused passes predate them.
No tests, builds, benchmarks or live GUI/MCP qualification were performed here, and no FPS,
latency, universal cache-release or current deployed-build claim is made.
