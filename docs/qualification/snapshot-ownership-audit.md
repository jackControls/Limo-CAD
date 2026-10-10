# Native snapshot ownership audit

Source review for [#333](https://github.com/limo-cad/Limo-CAD/issues/333), based on
`9cfa76a7043989d341ec1de49488379ab0e1c92a` plus the borrowed native print/history
reads described below, on 2026-10-10. This is a concrete inventory of the
production snapshot APIs and their desktop consumers, shared host/MCP read
contracts, and the document caches they feed. It is not a census of every clone
in the repository, a benchmark, or live UI qualification. Test fixtures and
JSON-schema construction are excluded from production copy counts.

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

5. **Undo/Redo decision reads ? repaired**: `native/history.rs`
   `apply_native_history_guarded` previously copied a whole DocumentDto to read
   rollback position, feature count and latest feature ID. One `with_document`
   callback now returns those three scalars coherently. Its engine guard ends
   before history dispatch. Existing owner/revision/control checks, empty
   history errors, DeleteLatest identity, owned Undo model tickets and
   transactional restore/history snapshots remain unchanged. Owned complete
   model strings/history tickets are required for recovery across mutations.

6. **Drawing paper and editors** ?
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

7. **Native print preparation ? repaired**: `native/controller/files/printing.rs`
   `prepare` formerly copied every drawing sheet/template only to find the
   active sheet ID/name. Its short `with_drawing` callback now returns that ID
   and one name String. No active-sheet fallback was added: missing selection
   still returns `Select a drawing sheet to print`. SVG export and the immutable
   `native_print::Page` remain owned output, sized to the exported page. They
   cross worker/native dialog boundaries, with the existing repeated document
   receipt checks. The drawing callback ends before export or title lookup
   reacquires the engine lock.

8. **CAM intent consumers**: `native/controller/workbench/cam.rs` owns
   CamDocumentDto on editor owner/revision refresh; `cam_view.rs` owns an Arc
   per owner/revision/selection key and clears it when inactive, cancelling
   pending work. `cam/central.rs` owns the project CAM intent when opening the
   library; Close clears the receipt/draft but retains project/library context
   until the next Open or resource destruction. `cam_export.rs` temporarily
   owns CAM intent to derive retained posting fields/summary. These copies
   scale with tools/setups/operations, not NativeViewportFrame geometry.
   `cam_view.rs` additionally discards the DTO returned by `cam_snapshot`
   when reading its warning; a warning-only borrowed accessor is a concrete
   remaining copy candidate. The host's actual background `cam_snapshot`
   boundary owns CAM intent so voxel work can run after the engine guard ends.
   Do not remove those required worker snapshots. CAM posting/editor/library
   reads remain candidates for narrower borrowed accessors; this patch does
   not introduce a new CAM API or change retained draft behavior.

9. **Borrowed menus and actions**: `native/controller/browser.rs::action_node`,
   history validation/rename/edit preparation, file export readiness, sketch
   edit validation, `feature/editing.rs`, and `workbench/ribbon_menu.rs` use
   `with_document`/`with_drawing` callbacks. They return only the necessary
   IDs, small names/choice lists, booleans or history indices before updating
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
   publisher fence while its callback runs; `with_document`/`with_drawing` hold
   only the inner engine guard for the synchronous borrowed read. Never call
   another engine method or publisher-locking bridge method inside them.
2. **Print/history changes here:** small owned callback outputs return and
   release the engine guard before export, native page preparation or history
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

Keep #333 open. Remaining work is the complete all-entry-point copy census,
justified reductions of the metadata/CAM candidates above, memory-pressure and
background-holder acceptance, and any consumer-specific presentation split
supported by an actual invalidation problem. Per-body render updates require
stable authoritative kernel body revisions or dirty-body identities; body IDs
alone cannot prove unchanged geometry. Do not implement that architecture from
this audit or recreate the already shared metadata/pose caches.

Existing identity/retention tests cover shared frames, independent metadata and
placement invalidation, drawing edits/tab reuse/eviction, mesh reuse and dirty
archive/history retention. The issue records historical focused passes; this
source audit did not rerun them. The two scalar/metadata-only fixes received
source review and scoped edition-2021 rustfmt/diff checks. No tests, builds,
benchmarks or live GUI/MCP qualification were performed here, and no FPS,
latency, universal cache-release or current deployed-build claim is made.
