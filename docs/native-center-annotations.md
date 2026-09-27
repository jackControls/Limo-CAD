# Native center annotations

The Drawing workspace's existing **More dimensions** menu offers **Center Mark** and **Centerline**. This native Centerline increment supports two circular centers in one view. The sidebar names that method explicitly. Centerline between straight edges, automatic symmetry axes, and bolt-circle authoring remain separate parity work; their saved records are preserved.

Center targets require complete projected circles and respect the view's hidden-line setting. Coincident centers follow the existing React visibility/radius preference. New references retain the exact body, occurrence, edge key, topology signature, and projected model diagnostics. A first center is disposable; switching owner, revision, view, or projection retires it. A pair of translated occurrences of the same source edge is valid.

Select a saved center stroke to edit **Extension (paper mm)**, reset, or delete it. Selected endpoint grips preview extension along the outward direction and commit through the existing history transaction on release. Cancel restores the saved document. Native painting and export use the same pure center extent helpers; broken references keep the existing selectable `!` marker. Form edits retain loaded associations and other document metadata.

## Focused verification

Set `NBCAD_NATIVE_CENTERS_ONLY=1` for the existing `xtask test-mcp native-drawing-authoring` command, retaining its normal disposable blank-session arguments. This uses the established real OCCT rectangle/three-boss fixture and its 24 saved annotation variants. It performs published-control creation in both centerline orders, duplicate/cancel/reset checks, extension edit/delete, exact Undo/Redo, and archive preservation. It retains nine new target/created/edited PNGs, exact before/after models, and the actual projection.

The focused path sends no OS mouse or keyboard input. The fixture's report explicitly leaves physical center picking, extension-grip dragging, and Linux/macOS center pixels unproven. Original PNGs still require visual review.

Native unit regressions live in `drawing_authoring/center/{tests,history_tests}.rs` and `drawing_annotations/center_tests.rs`. Compile and run them with `--features dev-bevy-host`; a default Cargo check does not cover this controller.
