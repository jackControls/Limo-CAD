# PR124 visual parity and sketch snapping audit

Reference: the supplied noBS CAD app from `cam-pr180-followups`, source commit
`391a025c2ec7047bc7c369e972cd7a4799f604b2`.
Stacked base: PR124 (`feat/bevy-interface`), commit
`b36fabeaee3c4a745ff3c8b64686e85217086814`.

## Requested fixes

| Area | Finding in PR124 | Result |
| --- | --- | --- |
| Workspace switcher and menu | Generic outline icons and different captions | Port the reference's original shaded cube and milling-machine artwork in both appearances; use its FileText icon for Drawing, compact/wide captions, current-workspace check, and sketch badge. Drawing and Manufacture are disabled during a sketch. |
| Sketch Palette / orientation dial | Both panels occupy the top-right viewport | Retire the dial and axis labels during sketch editing, matching the reference; restore them when sketch editing ends. Navigation controls remain available. |
| Drawing acquisition | Only Line presents a snap marker; other tools use fixed engine defaults | Share cursor acquisition across every creation tool, including polygon center placement. Origin and point capture use 14 logical pixels; grid capture uses 7. The grid interval follows the visible 1–2–5 lattice at the current zoom. |
| Sketch tool groups | Dimension and Select are combined into other groups; dividers are missing | Restore DRAW, EDIT, DIMENSION, REPEAT, CONSTRAIN and SELECT, with vertical dividers and centered inline captions/chevrons. Compact layouts retain group menus for secondary commands. |
| Additional chrome | Palette and browser typography, borders, rows and active-sketch presentation differ | Match palette width/spacing/title/footer, Flat View icon, browser heading/divider/units badge, plane labels, active sketch emphasis and automatic folder expansion, plus history caption treatment. |

## Interaction safeguards

Preview and click use the same acquisition query. The ordered creation command
carries its viewport snap distances so another caller cannot alter those distances
between preview and commit. The host scopes and restores this runtime context;
it does not persist viewport zoom in the project or invalidate geometry on a
preview. Calls without viewport context retain their existing engine defaults.

Typed circles and rectangles, and line inference, keep the raw direction hint
used by the engine preview. A nearby snap target therefore cannot make a typed
circle jump when clicked. Snap off preserves cursor coordinates. Ctrl suppresses
relational snapping while preserving the reference's grid behavior. Creation
remains one undo step.

## Validation

- Shared vector audit: all 99 declared/available icons pass, including provenance
  and external-reference/executable-content checks.
- Regression coverage exercises all 13 primitive creation variants against
  origin, grid and existing-point acquisition, preview immutability, committed
  geometry, Snap/Ctrl switches and undo. Polygon uses the same acquisition path.
- Additional coverage checks scoped-context restoration after success/error,
  midpoint-line grid spacing, typed-circle preview/commit agreement, dial
  retirement/restoration, compact/wide workspace layouts, shaped group captions
  and active-sketch folder expansion without overriding a later manual collapse.
- Live native-interface checks verify rectangle and circle origin/grid snapping,
  Snap off, preview markers, and undo using an isolated test document.

The reference uses WebKit text while the native interface uses Bevy text shaping;
minor font-metric differences remain. This audit covers the requested chrome
and sketch interactions rather than certifying pixel-identical rendering of
every native workspace.
