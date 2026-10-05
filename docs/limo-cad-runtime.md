# Limo CAD runtime identity

The Bevy desktop and MCP share the same Rust engine. The desktop workspace is
`desktop/`; all engine crates use `limo-cad-*` package names and `limo_cad_*`
Rust imports. `cargo xtask` owns builds, packages, WASM bindings and MCP setup.

The Windows deployment is `%LOCALAPPDATA%/limo-cad/bevy/Limo-CAD.exe`.
`cargo xtask install-mcp` registers `limo-cad`, removes retired CAD server entries
and preserves unrelated servers. Reload the client configuration after setup.

```text
cargo xtask install-mcp --clients cursor,codex --no-build --binary ABSOLUTE_LIMO_CAD_PATH --in-place --server-arg --headless --desktop ABSOLUTE_LIMO_CAD_PATH
```

New projects use `.limo` ZIP archives and `limo-cad-project` model/manifest
identifiers. Readers accept existing `.nbcad` and `.tfcad` projects; saving an old
project retains its selected path and migrates its model and manifest without
losing ancillary archive entries. New command scripts use `.limo.jsonc`.
Script parsing remains content based, so existing command files remain readable.
Project-file launch arguments use the same guarded File workflow as UI and MCP
opens. Windows registers `.limo` and `.nbcad` projects with the current executable.

Recipe and knowledge links use `limo-cad://`; old `nbcad://` links remain accepted
at the same restricted parsing boundaries. MCP build metadata is `limo-cad/build`.
Build, SDK, desktop and diagnostic overrides use the `LIMO_CAD_*` prefix.
Old `NBCAD_*` variables are no longer consumed; rerun MCP setup for fresh settings.

The profile is `org.limocad.desktop`. On first launch the complete previous
`org.nbcad.desktop` folder moves atomically, preserving preferences, private
libraries and recovery data. A `LIMO_CAD_CONFIG_DIR` override bypasses migration.
If both profiles exist, startup reports the conflict without overwriting either.

New desktop leases and inboxes use the private `limo-cad-sessions` registry
(with the effective user ID appended on Unix). During deployment, save and close
old desktop windows before restarting all MCP workers. Existing recovery files
and snapshots are preserved; new windows publish only to the new registry.

Packages use `Limo-CAD-<version>-windows-<arch>.zip`,
`Limo.CAD_<version>_amd64.deb`, `Limo.CAD_<version>_amd64.AppImage` and
`Limo.CAD_<version>_<arch>.dmg`. The macOS bundle is `Limo CAD.app`;
Linux uses `limo-cad.desktop` and the `limo-cad` package. Debian replaces and
conflicts with the previous `nbcad` package to prevent parallel installations.

Recorded diagnostic inputs and previously published previews retain the names
and hashes of the actual artifacts. They also exercise project read compatibility.
