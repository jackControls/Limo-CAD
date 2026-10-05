# Bambu project handoff

The portable 3MF scene remains the geometry and placement source. The Bambu adapter refreshes a complete saved Bambu Studio **02.08.02.61** project with explicit CAD definition/occurrence bindings. It preserves existing object grouping, intentional repeats, printer/process/filament configuration, volume UUIDs, plate assignments, and unrelated settings. It never creates a guessed machine or filament profile.

## Choose and review a source

Inspect a saved project before handoff. Its summary identifies the source SHA-256, printer/nozzles, process defaults, filament chemistry/colors, support mapping, plates, objects and normal volumes. Bind each CAD occurrence explicitly to the intended object/instance/volume; display names are labels, not identity. Every selected occurrence and target normal-volume instance must be bound exactly once. CAD hierarchy groups must match the target grouping. A mismatch requires another template or an explicitly reviewed restructuring of that template.

`resolved_scene` placement uses the application's existing resolved assembly/named-view transforms. It currently requires a single-plate source because automatically assigning a chosen layout to several native plates is not qualified. Explicit `template` placement retains saved plate positions and centered volume transforms. Clear a selected named view when deliberately choosing template placement; the engine rejects conflicting placement instructions.

The adapter checks CAD appearance against each explicitly mapped template filament slot. A mismatch requires correcting the mapping or deliberately accepting the template's chemistry/colors. The report preserves the chosen mapping and states the actual exported world transform and plate for every occurrence.

## Settings and precedence

The qualified keys are wall count, infill density/pattern, and top/bottom shell layers. Requested project defaults override the selected complete process's defaults; native object and inherited volume overrides remain effective, and an explicit CAD part override applies last. The report lists inherited values, written part overrides, effective values and each value's origin. A selected process snapshot must match its actual sourced template defaults. Put deliberate differences in project/part intent.

Rectilinear maps to Bambu's `zig-zag`. Incompatible 100% infill patterns are rejected; the adapter does not silently replace one. Unsupported scoped controls are not implied by arbitrary metadata keys. The initial project stage rejects existing print-only modifiers or height-edit entries until their coordinated adapters can preserve their meaning.

## Refresh after a slicer save

Keep `report.refresh_reference` with the CAD handoff. It records the document namespace, original template/profile lineage, target normal-volume UUID, per-instance `identify_id`, and bounded original/written values for the five supported settings. Supply that reference when refreshing a project saved by the slicer. Bambu removes custom noBS metadata and can renumber resource IDs; matching therefore uses UUID plus instance identity, never a name or guessed instance order.

Unexpected native changes to managed settings block refresh until explicitly reviewed. Accepting those changes adopts only changed inherited fields, then applies current CAD overrides. Later clearing a CAD override restores the reviewed baseline. Changes to complete profile hashes are reported; preserving hardware/process/material identifiers does not prove that every changed native setting remains qualified.

Bambu Studio 2.8.2.61's plate map preserves only one loaded instance identity when the same object has several instances on one plate. Native save retains their quantity and grouping, but can replace later `identify_id` values. Missing or ambiguous identities fail with source and native candidate IDs. Inspect and explicitly rebind such instances. Clearing an old reference and choosing exact new bindings adopts a new inherited baseline; it cannot recover the previous override-reset lineage automatically. No automatic fallback is used.

## Output and evidence

The writer replaces validated welded meshes, updates geometry/source-offset metadata, clears old toolpaths, thumbnails, estimates and slice caches, and repairs package relationships/content types. It independently parses the resulting ZIP/XML and checks mesh coordinates/topology, transforms, target UUIDs and effective settings before returning bytes. The report identifies source/output hashes, actual placements/materials, setting origins, invalidated entries and limitations.

`metadata_readback_verified` is distinct from installed-slicer import and generated toolpaths. Export alone leaves the latter fields false. Successful local slicing provides separate version, exit status, input/output hashes, per-plate results and G-code evidence. Neither a metadata check nor a generated toolpath demonstrates physical strength or dimensional accuracy.

## Reproducing qualification

Deterministic export contracts require no installed slicer:

```text
cargo test --locked -j1 -p limo-cad-export --lib bambu_project::tests
```

Opt-in tests take an operator-provided complete saved template through `LIMO_BAMBU_TEMPLATE` and write synthetic fixtures into a fresh absolute directory selected by `LIMO_BAMBU_QUALIFICATION_DIR`. They do not commit, modify or copy private model geometry. The five-object/four-plate fixture deliberately recognizes the reference acceptance names to assign test settings; production binding never uses names.

```text
cargo test --locked -j1 -p limo-cad-export --lib write_five_part_four_plate_native_qualification_fixtures -- --ignored --nocapture
cargo test --locked -j1 -p limo-cad-export --lib write_resolved_repeated_and_thin_native_qualification_fixtures -- --ignored --nocapture
```

Slice each generated project with the qualified local binary and an owned output directory:

```text
bambu-studio.exe --arrange 0 --slice 0 --outputdir <owned-output> --export-3mf <owned-saved-name.3mf> <synthetic-input.3mf>
```

The Windows launcher can reopen stdout/stderr to `CONOUT$`; empty redirected logs do not establish success. Require exit 0, native `result.json`, all requested plate G-code files and the runtime version in their headers. After saving the resolved repeat/thin fixtures to the documented `<case>-validated/<case>-sliced.3mf` paths, run the independent native geometry/group/material readback test.

Fresh qualification on 2026-10-04 passed baseline/configured five-object/four-plate slices and a subsequent native-save → explicit-reference refresh → four-plate re-slice. Separate resolved-scene qualification preserved two rotated multipart instances, each with two normal volumes, within 0.001 mm in independent native saved-project readback. Thick rectangular sections showed six wall loops at Z=5 mm versus two in the paired baseline. A 1.2 mm section still requested six walls but produced an outer perimeter and gap infill, without inner-wall paths at that layer. The unmodified sleeve's parsed linear-toolpath fingerprint was unchanged. These checks establish local import/toolpath behavior for these fixtures, not a strength claim or GUI Objects-panel inspection.
