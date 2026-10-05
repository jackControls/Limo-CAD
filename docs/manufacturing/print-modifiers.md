# Local print modifiers

Print modifiers are manufacturing metadata attached to a stable CAD body definition. They never add a CAD solid, mechanical occurrence, feature, material volume or joint. A body's intentional repeated occurrences inherit its zones; the existing assembly and named-view resolver supplies placement before each local zone transform is composed.

## Shape, identity and editing

The first delivery supports centered boxes with full XYZ dimensions and cylinders centered on their local Z axis, with radius and height. Placement is a local translation in millimeters and a unit quaternion in x/y/z/w order. Dimensions must be positive and finite; coordinates and dimensions are bounded at 1,000,000 mm. A document holds at most 256 authored zones.

Each zone has an immutable UUID and parent BodyId, a name, enable flag, primitive, local pose and five optional settings: walls, infill density/pattern and top/bottom shell layers. Copy creates a fresh UUID on an explicitly chosen existing body. Reset clears overrides while retaining identity, name, shape and placement. Deleting a parent retains an identifiable orphan; exporting an enabled orphan blocks handoff. Explicit copy can recover its settings onto another body. Body allocation reserves retained identities, preventing accidental reattachment to a later unrelated part.

Shared engine/MCP operations are `print_modifier_create`, `print_modifier_update`, `print_modifier_remove`, `print_modifier_copy`, `print_modifier_reset` and `print_modifier_effective`. Every mutation requires the exact current `expected_model_json`; rejected or stale requests leave the document unchanged. Read/effective results use the existing print-intent surface and classify live, retained and orphan attachments. Snapshot history and project persistence preserve metadata without changing mechanical modeling history. Project schema 13 stores print-intent version 3; older documents migrate with an empty collection.

New interactive editing belongs to the Bevy workflow. This backend checkpoint supplies the shared operations and export behavior; it does not establish a passed Bevy overlay/editor walkthrough by itself.

## Target contract

Bambu Studio 02.08.02.61 receives `modifier_part` volumes inside the existing multipart object. No extra build item is created. The template-first writer preserves normal grouping, repeated quantities, mapped filament/process settings and actual placement. Cylinders use bounded tessellation with a maximum 0.025 mm chord deviation. Both template placement and resolved assembly/named-layout placement use the same normal-part bindings.

An enabled zone must intersect actual parent mesh material with positive volume; touching a surface or lying entirely in a cavity is insufficient. A conservative cross-sibling bounds check rejects zones that might affect another normal volume in the same native object, because Bambu applies a modifier to all intersecting siblings. Conflicting enabled zones with overlapping bounds are rejected. Equal requested maps do not add wall counts. Bounds checks can reject some disjoint rotated shapes; explicitly adjust the zones rather than relying on undocumented native overlap priority.

Bambu configuration inherits process, object, normal-volume and then modifier overrides. The report exposes each zone occurrence's actual world transform, plate, bounds, effective values and their sources. Local speed, ironing, support, material, layer-height and compensation controls are unsupported in this delivery. Height-range coordination is a separate extension; existing unsupported height/profile data blocks handoff until its adapter is available. Per-volume capability does not imply every setting is valid at every scope.

Portable 3MF and STL omit zones because only mechanical bodies are tessellated. Effective print-intent reporting for unsupported targets explicitly identifies omitted fields. Disabled, reset/no-effect and excluded zones are reported and omitted from Bambu geometry.

## Safe refresh

Save the generated output and its `report.refresh_reference` together. Modifier lineage records the authored zone, parent normal-volume UUID, generated volume UUID and source mesh center. After a native save, the adapter independently verifies supported settings and centered relative mesh geometry before replacing or removing its managed zones. Unexpected native edits and unmanaged print-only volumes require explicit review; they are never silently deleted. Existing normal-volume UUID plus instance identity rules still apply, including the documented repeated-object `identify_id` limitation in Bambu 2.8.2.61.

The writer invalidates stale toolpaths and previews, reparses the final package and verifies geometry/settings before returning bytes. `verify_bambu_modifier_reference` checks native saved metadata and geometry; it does not infer toolpaths. `read_bambu_volume_geometry` exposes actual native target identities, mesh coordinates, transforms, bounds and plate IDs for the existing diagnostics path. Those native IDs must be explicitly mapped to CAD bindings, and print-only volumes must be excluded from physical-body layout checks. Readback is capped at 4096 volume instances and 512 MiB of expanded mesh buffers.

## Qualification

Deterministic core, manager and export tests cover atomic CRUD, metadata snapshot Undo/Redo and save/reopen, orphan identity allocation, rotated repeated zones, nested named-layout offsets, positive intersections including hollow cavities, conservative conflicts, independent geometry readback and managed refresh removal.

Opt-in native fixtures require an operator-provided complete saved template via `LIMO_BAMBU_TEMPLATE` and a fresh existing absolute `LIMO_BAMBU_QUALIFICATION_DIR`. Private geometry is replaced by public synthetic boxes; private complete process configuration is not committed. Run:

```text
cargo test --locked -j1 -p nbcad-export --lib write_native_print_modifier_qualification_fixtures -- --ignored --nocapture
bambu-studio.exe --arrange 0 --slice 0 --outputdir <owned-case-output> --export-3mf <case-sliced.3mf> <synthetic-case.3mf>
cargo test --locked -j1 -p nbcad-export --lib verify_native_print_modifier_geometry_settings_and_automatic_refresh -- --ignored --nocapture
```

Use the fixture's baseline/configured and unique-baseline/unique-configured cases, each saved as `<case>-validated/<case>-sliced.3mf`. The readback gate writes `modifier-unique-refreshed.3mf`; slice it to `unique-refreshed-validated/unique-refreshed-sliced.3mf`, then run `verify_native_modifier_automatic_refresh_reslice` with `--ignored`.

On 2026-10-04, installed Bambu Studio 02.08.02.61 returned exit 0 for all four initial cases and the automatic unique-instance refresh reslice. Independent readback preserved all normal/modifier world triangles, UUID settings, quantity, plate placement, filament mapping and centered attachment. Normal object triangle counts excluded the print-only geometry.

Paired G-code analysis at Z=10 mm found local sparse-infill extrusion increased from 2.39 to 6.51 mm and 2.54 to 6.73 mm in the two rotated-box occurrences, and from 0.89 to 1.83 mm and 0.87 to 1.83 mm in their tilted-cylinder zones. Local inner-wall paths appeared. Layers at Z=3 and 17 mm, outside both zones, retained the baseline aggregate feature counts, extrusion and linear path lengths within floating precision. Unmodified sibling wall counts and lengths matched, while minor infill/bridge clipping differences remained; an identical full sibling path fingerprint is not claimed.

The owned evidence archive is `native-modifier-qualification-02` under the manufacturing-intent build archive: input/output hashes and exact commands in each `native-run.json`, native `result.json` and G-code headers, `native-modifier-readback-evidence.json`, `native-modifier-refresh-reslice-evidence.json` and `localized-toolpath-evidence.json`. These establish local behavior for these fixtures, not six realized walls at every boundary, a GUI Objects-panel inspection or physical strength.

The main-stack actual-kernel MCP fixture `print_modifier_mcp_guarded_roundtrip_keeps_physical_scene_appearance_and_script` passed on 2026-10-05 (1 passed). It verifies guarded creation/reset, exact normalized shape and local pose preservation, stale-request rejection, persistence, mechanical scene and appearance invariance, and explicit portable unsupported-field reporting. Its receipt is `modifiers-main-final-mcp.log` in the owned manufacturing-intent build archive.
