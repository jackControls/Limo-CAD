# Persistent print intent

`PrintIntentDocumentDto` stores requested process settings separately from body
appearance, mechanical features, joints, and named layout poses. Version 2 owns
optional wall counts, infill density/pattern, and top/bottom shell layer counts.
These values describe a requested handoff. They do not measure strength or say
how many wall loops a slicer can realize on a thin region.

## Inheritance and identity

Effective settings resolve a selected process snapshot, project defaults, then
the part's explicit overrides. Each field reports its source. An absent or null
field inherits; zero explicitly requests zero. Unresolved profiles contribute
no invented defaults and produce a diagnostic. Counts accept 0..1000 and infill
percentages accept finite values in 0..100. Infill patterns are a closed enum:
grid, gyroid, rectilinear, concentric, cubic, honeycomb, and lightning.

Overrides bind to stable source `BodyId`, so all intentional occurrences of a
definition inherit the same values. Rename, reordering, recomputation, and
ordinary body moves retain that identity. CAD copies, mirrors, and patterns
create separate identities and initially inherit project defaults; users can
deliberately copy explicit settings through `print_intent_copy_part`.

Bodies retained outside the current rollback stage, or consumed by later
features, keep their authored settings. Deleted creator outputs become explicit
orphans: their requested values remain recoverable, their effective settings
are empty, and they cannot apply to exported geometry. Reload reserves those
body IDs before allocating new geometry, preventing an unrelated replacement
from acquiring an orphan's settings. Copying an orphan's explicit settings to a
chosen live part is a deliberate recovery operation.

The document gets a UUID `source_document_id` on its first successful metadata
edit. Reads and old projects do not generate an identity or settings. Later
edits preserve the identity; ordinary editing cannot replace it. Project load
restores that identity from the saved project. Undo/Redo changes settings while
retaining an already assigned identity in the same owning document. A target
project refresh must match this namespace and source IDs as well as its exact
input/project preconditions.

## Shared engine and ownership

UI and MCP use the same Rust operations:

- `print_intent_get`: read the versioned document.
- `print_intent_effective`: read optional `body_ids` and a target (portable by
  default), returning live/retained/orphan bindings, field sources, unsupported
  fields, profile status, and diagnostics.
- `print_intent_set_part`: replace one part's `settings`.
- `print_intent_reset_part`: remove one override, restoring inheritance.
- `print_intent_copy_part`: copy explicit overrides from `source_body_id` to
  `target_body_ids` atomically.
- `print_intent_set_document`: replace the complete `document`, including
  defaults and the selected snapshot.
- `print_intent_upsert_preset` and `print_intent_remove_preset`: manage named
  reusable settings. Applying a preset stores a settings snapshot.

Every mutation requires `expected_model_json` from the complete current project.
The owning engine checks it before changing any metadata under its document
lock. Desktop controls additionally fence document, session, revision, and
replacement epoch; inbox operations enter the same owner and history path.
The older desktop shell records bounded native before/after receipts in its
existing application history. Restore requires the same owning document and
session, the current full model, and a recorded transition; it changes metadata
without replaying geometry. An identity assigned by the first edit survives
Undo. Cold tabs retain serialized metadata. A stale precondition rejects the
complete mutation. Controller/inbox history qualification remains part of #312;
shared model tests alone do not establish desktop Undo/Redo acceptance.

Native tab eviction archives bounded print-history receipts and the exact
owning model. A private SHA-256 attestation permits cold restoration only into
a fresh context for the same tab, from unchanged archived bytes and model.
Verified receipts acquire the new live session identity; the retired session
is never revived. Ordinary Open, replacement, close, and window teardown discard
that authority. Missing or invalid receipts disable matching Undo/Redo and
report the problem instead of removing a CAD feature. The production browser
fixture covers tab eviction and cold Undo/Redo. Native bridge regressions also
verify fresh ownership, exact archived models, tamper rejection, session
retirement, and ordinary Open invalidation.

Payloads reject unknown fields, occurrence/layout overrides, unknown patterns,
and invalid values. The print-specific raw JSON limit is 32 MiB including the
optimistic project snapshot, with up to 4096 part records and 128 named presets.
Body IDs in manufacturing records must be nonzero safe integers; oversized IDs
cannot poison the reserved-body allocator.

## Profiles and target capabilities

A selected process snapshot records typed defaults and provenance. A resolved
snapshot needs either pinned repository provenance or the SHA-256 and label of
a saved template. This status means requested defaults are available; complete
printer/filament compatibility and slicer execution remain separate evidence.
Bed constraints belong to `PrintBedDto`, and filament chemistry/color belong to
body appearance and the material catalog.

The typed capability registry distinguishes target and scope. Initial Bambu
project/part representations are known. Portable 3MF, Orca, and Prusa process
representations are reported unsupported until their adapters are qualified.
Occurrence and named-layout process overrides are deferred and rejected rather
than silently stored. Target adapters must also reject incompatible combinations
or profile mappings; representing a setting never proves it was applied.

Schema 12 protects this metadata from older readers that would discard it on
save. Schema 11 migrates existing version-1 settings and source identity without
introducing target references. Schema 10 and earlier migrate with empty intent
and unchanged geometry; named layouts and existing metadata remain intact. Main's foundation
is based on the existing multipart/layout PR, while Bevy uses the same schema
and engine DTOs. New controls belong to Bevy; the legacy UI has no second editor.

## Persistent target handoffs

`target_handoffs` retains up to 16 named target references in the same metadata
and history surface. `print_intent_upsert_handoff` accepts a typed `handoff` and
`print_intent_remove_handoff` accepts its `name`; both require the complete
`expected_model_json`. The initial `bambu_studio` variant stores a source label,
source-document namespace, exact template/profile hashes, reviewed project and
part setting baselines, and each intentional source body/occurrence binding.
These records describe refresh identity and requested settings, not slicing
success or physical strength. No private template file is embedded in the CAD
project.

Target volume UUIDs may repeat for intentional instances; their UUID plus native
instance identity must be unique, as must each source body/occurrence pair.
Resource indices can change during a slicer save. A new reference must bind to
an existing source occurrence. Previously authored deleted bindings remain
recoverable through load, settings history, and explicit removal. New made-up
orphans are rejected during metadata edits. Reload reserves retained body and
occurrence identities so unrelated new geometry cannot inherit old bindings.
Allocation and persisted counters stay within exact JavaScript integer bounds;
exhaustion returns an error without emitting an unsafe or reused identity.

Future outer or print-intent schema versions are rejected rather than silently
dropping target data. Orca process handoffs remain unsupported pending separate
adapter qualification.

Target-specific projects and combined handoff/slicer evidence extend this
foundation independently. Portable exports remain useful without process
metadata. Evidence must distinguish layout checked, metadata written/read back,
imported, toolpaths generated, and physical testing pending.
