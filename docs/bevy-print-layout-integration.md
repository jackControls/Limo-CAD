# Bevy print-layout integration

The requested multipart 3MF and named print-layout feature is not delivered in
Bevy. At `9b082687`, Bevy exports flat 3MF. Main PR
[#257](https://github.com/jackControls/Limo-CAD/pull/257) contains the shared layout
implementation and a legacy React editor. Port the implementation into the
existing Bevy application; do not develop another legacy interface.

## Product decisions

- Use the CAD component/assembly hierarchy for printable groups and preserve
  every intentional repeated occurrence.
- Use one named-view model for presentation and printing. Any view may export;
  marking a view as a print layout opts into print checks.
- Report overlaps, bed violations and missing occurrences. Record intentional
  exclusions explicitly, permit deliberate export, and propose whole-group
  corrections without modifying mechanical geometry or joints.
- Start with Bambu Studio and OrcaSlicer adapters and the Bambu X2D bed. Keep
  portable 3MF as the model contract.
- Fetch pinned printer sources through Cargo xtask during CI and embed the
  verified catalog. Retain committed data for offline builds, as the integrated
  unified material catalog already does.

## Integration still required

1. Port shared print-bed/profile data, named-view persistence and migration,
   occurrence layout resolution, portable scene export and layout diagnostics
   from #257. Reuse its core, assembly and export modules rather than duplicating
   them in UI code. Preserve frozen profile geometry and source provenance.
2. Integrate named-view state and resolved presentation into the native engine,
   document/history ownership and host/MCP/script APIs. The current
   `NativeEngineHost::export_3mf` prepares meshes for the flat writer; its
   viewport uses the mechanical assembly solution.
3. Add create/edit/recall and occurrence placement through existing Bevy controls.
   Populate the current Named Views tree node, extend export intent with view
   selection, and expose print designation, profile choice, diagnostics and
   proposed corrections through the same surface.
4. Qualify the complete Bevy path against the criteria below before claiming
   feature completion. Legacy-interface results establish the source behavior;
   they do not qualify the Bevy integration.

## Acceptance

- Save/reopen and schema migration retain views, exclusions, repeated instances
  and printer snapshots. Undo/Redo, tab eviction and asynchronous operations
  preserve document ownership and newer user actions.
- Viewport and exported world transforms agree for nested rotations and
  translated repeated parts. Entering modeling resolves picks against source
  geometry rather than stale presentation coordinates.
- Diagnostics and applied corrections preserve multipart alignment, hierarchy
  and quantity. Deliberate export remains available after warnings.
- `cargo xtask printer-profiles --fetch --check` verifies pinned upstream input
  and the embedded catalog on the Bevy branch; offline builds use committed data.
- Bambu Studio and OrcaSlicer import/re-export preserve group membership,
  repeated quantity and world vertices. Record tested versions and source head.
  Existing #257 evidence used Bambu Studio 2.8.2.61 and OrcaSlicer 2.4.1 with a
  0.001 mm world-vertex tolerance; rerun on Bevy-produced files.

Physical printing, per-face painting, filament-slot qualification, process
presets and toolpaths remain outside this first portable-model milestone.
