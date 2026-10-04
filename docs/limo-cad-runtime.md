# Limo CAD runtime identity

The Bevy desktop is **Limo CAD**. Source builds produce `limo-cad` (`limo-cad.exe`
on Windows); the portable Windows package contains `Limo-CAD.exe`. The standalone
Rust server builds as `limo-cad-mcp`. Its MCP initialization reports the name
`limo-cad` and title `Limo CAD`.

`cargo xtask install-mcp` registers **`limo-cad`** and removes retired `nobs-cad`,
`noBS-CAD` and `nbcad` entries from selected client configurations. Other servers
are preserved. Reload the client's MCP configuration after migration: restarting
a previously registered server does not change its registration name.

The Thunder deployment location is `%LOCALAPPDATA%/limo-cad/bevy/Limo-CAD.exe`.
Configure that executable in place with `--headless`, keeping the OCCT DLLs beside
it. The desktop and agent worker then use the same compiled engine and schema.

```text
cargo xtask install-mcp --clients cursor,codex --no-build --binary ABSOLUTE_LIMO_CAD_PATH --in-place --server-arg --headless --desktop ABSOLUTE_LIMO_CAD_PATH
```

New packages use `Limo-CAD-<version>-windows-<arch>.zip`,
`Limo.CAD_<version>_amd64.deb`, `Limo.CAD_<version>_amd64.AppImage` and
`Limo.CAD_<version>_<arch>.dmg`. The macOS bundle is `Limo CAD.app`;
Linux uses `limo-cad.desktop` and the `limo-cad` package. The Debian package
replaces and conflicts with the old `nbcad` package to prevent two installations.

Persisted project formats, `.nbcad` files, recipe/resource URIs, session registry
locations, environment variables and engine crate identifiers keep their existing
contracts. The profile remains `org.nbcad.desktop`, so existing preferences and
libraries remain visible. These storage and protocol identities are separate
from the application's displayed name and the client's MCP registration.

Previously published previews and recorded diagnostic fixtures retain the names
and hashes of the artifacts actually tested. Renaming source does not rename or
qualify those older downloadable packages.
