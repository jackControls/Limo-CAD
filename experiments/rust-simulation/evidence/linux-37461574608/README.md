# Actual hosted Linux evidence

Downloaded from the `rust-simulation-results` artifact of
[run 37461574608](https://github.com/limo-cad/Limo-CAD/actions/runs/37461574608),
completed successfully 2026-10-06 12:19 UTC at PR head
`e8b733cfe5b8029fb2549256ba78d5c7feefd03a`.

These are actual Ubuntu runner outputs, not copies of the committed Windows
baseline. `archive-manifest.json` identifies the exact downloaded bytes.
Formatting, strict Clippy, the structural qualification and world diagnostic
executables passed. The world executable's success means its execution checks
passed; it does not gate FEM compliance accuracy or all cases' stationarity.

The finest Fenris displacement agrees with the Windows baseline to the stored
digits, and every recorded Rapier motion/displacement diagnostic agrees to its
stored digits. Other Fenris values differ at roundoff level; elapsed times differ.
This is one observed Windows/Linux comparison on this pinned dependency graph,
not a general cross-platform determinism guarantee. The fresh `same_mesh` probe
was added after this run and is **not** included in this historical artifact.
