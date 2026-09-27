# Manufacturing export validation

## Automated (required before merge)

```sh
cargo test -p nbcad-core -p nbcad-export -p nbcad-sketch --lib
```

Expect:

- appearance serde defaults + round-trip
- STL header + triangle count
- 3MF `unit="millimeter"` + basematerials name (filament type, color name) and display color
- Standard, Bambu, and Orca packages have Application `noBS CAD` and no `project_settings.config`
- Prusa `Metadata/Slic3r_PE.config`
- Cura `Metadata/cura_materials.json` + basematerials
- catalog JSON parse + Bambu/Prusa/Sunlu/eSun/Anycubic presets (≥40 entries)
- project round-trip scrubbing orphan appearances

For native OCCT/MCP coverage, use the runtime setup and sequential native test
command in [DEVELOPMENT.md](../DEVELOPMENT.md#verify-changes).

## Manual slicer smoke (KR3.6)

Regenerate fixtures: `cargo test -p nbcad-export --lib tests::regen_manual_smoke_fixtures -- --ignored --exact`

Then open from `crates/export/fixtures/smoke/`:

1. **`print_in_place_latch_bambu.3mf`** → **Bambu Studio** (Import / drag onto plate, not Open Project) — black housing + red bolt; slice & print, then slide the bolt. The file is a model, so Studio keeps the printer you already have selected.
2. **`print_in_place_latch_prusa.3mf`** → **PrusaSlicer** — same mechanism with PE metadata.
3. Optional: `*_orca.3mf`, `*_cura.3mf`, or simple `cube_*.3mf` colour checks.
4. App path: Extrude box → Bambu PLA Basic Red → Export 3MF; Export STL (appearance warning); Export STEP (no color expectation).

Record date/app versions when checking off GitHub issue #13.
