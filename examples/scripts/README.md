# Bundled script examples

Recipes are bundled construction scripts in the same Scripts feature as your
own `.limo.jsonc` files. Open **Scripts → Browse examples** in CAD, select an
example to inspect its source, then choose **Run in new design** in the source
editor. Edit and **Validate source** before running if you want to change it.
Start with **Sketch, extrude, ease the edges**: it builds a small part,
changes its extrusion from 12 to 18 mm while retaining the fillet, and restores
the 12 mm reference. Follow [the first-part guide](../../docs/INSTALL.md#make-your-first-part)
to edit, save and reopen the result yourself.

Maximum rate builds the same model without presentation waits. Pause and Step
let you inspect the sequence; paced presentation adds chapter notes and camera
moves. All modes use the same Rust interpreter and construction source.

- [Sketch, extrude, ease the edges](fillet-basics.limo.jsonc): a first-part lesson
  for Solid → Refine → Fillet. Builds a fully located 60 × 30 mm sketch, extrudes
  12 mm of stock and rounds the four top edges by 2 mm. Demonstrates an 18 mm
  extrusion edit and restoration, with captioned preview frames and final checks.
- [Four-hole mounting plate](mounting-plate.limo.jsonc): fully located 60 × 40 × 5 mm
  stock, four 5 mm through bores and face-basis projection after each cut.
- [Component edit and recovery](component-edit-recovery.limo.jsonc): edit a
  translated, rotated repeat in place, undo an incorrect driving value and
  update both shared occurrences to 30 × 10 × 3 mm. Includes paced chapters,
  camera focus, captioned previews, analytic checks and editable save/reopen.
- [Revolved annular spacer](revolved-spacer.limo.jsonc): a located radial section,
  20 mm outside diameter, 10 mm bore and 12 mm length, revolved about Y.
- [Dimensioned angle bracket](angle-bracket.limo.jsonc): a closed, fully constrained
  L section with 40 × 30 mm envelope, 5 mm walls and 20 mm extrusion width.
- [Repeated bracket assembly](repeated-bracket-assembly.limo.jsonc): three native
  part definitions, four occurrences and explicit mating frames. Editing one
  bracket extrusion from 20 to 25 mm updates both of its occurrences. This is a
  component/placement lesson, not a fastened or manufacturing-qualified assembly.
- [Crown garden bench](garden-bench.limo.jsonc): a complete timber assembly from
  dimensioned sketches, native features and physical mating references. Includes
  authored captions, camera framing and final manufacturing contracts.
- [Vertical-axis turbine](vertical-axis-turbine.limo.jsonc): two reused Savonius
  stages, constrained 72:18 generator gearing, native part and assembly drawings,
  printable definition exports and explicit physical qualification inputs.
- [D-screw vise](d-screw-vise.limo.jsonc): six printed parts, 100 mm gripping
  faces and 90 mm captured-jaw travel. Its custom rounded Ø24 × 4 mm screw has a
  shallow print flat and detachable thrust fitting. Includes seven drawing
  sheets, a print layout for each part and simplified M5/M6 hardware envelopes;
  30 bodies including optional mounting hardware.
- [D-screw fit coupon](d-screw-vise-fit.limo.jsonc): four specimens for the actual
  print process: the custom rounded screw, matching female thread and a
  male/female captured-guide pair. Qualify their fit before making a full vise.
- [Turbine fit coupons](turbine-fit-coupons.limo.jsonc): four native specimens
  reuse the shaft clamp, bearing seat, motor cradle and pinion geometry, with
  driving fit dimensions, print orientations and associative drawings.
- [Version 1 editor schema](limo-cad-script.schema.json): step and expression guidance
  for JSONC-aware editors. The Rust interpreter validates references, and the
  shared interface owns each modeling operation’s argument schema.

For hands-on UI and typed MCP work, open the editable
[garden bench checkpoint](../checkpoints/garden-bench.limo) with **File → Open**.
It was rebuilt through published UI controls and typed MCP operations on matching
Bevy GUI and MCP builds at `abf4667d`. This checkpoint has 53 named features,
25 fully constrained sketches, 18 body definitions and 32 placed occurrences.
It reaches the continuous arm supports; it is not the complete bench recipe.

The separate [human-operated checkpoint](../checkpoints/garden-bench-human-ui.limo)
records the Windows UI walkthrough through `69e66631` on 7 October 2026.
It contains twelve named features, four separately defined and placed posts,
two shared long-apron occurrences, two left-front apron pilots,
six right-front rail/arm pilots and four
presentation views. The three-model walkthrough is still in progress.
Every modeling change used real mouse/keyboard input through
the optional Rust `native-computer-control` feature, with no recipe replay or
direct MCP modeling commands. Read-only MCP inspection checked the resulting
geometry and references.

1. Create an XY sketch and anchor a 65 × 65 mm rectangle at the origin, with
   driving width/height dimensions. Finish the sketch and extrude 630 mm.
2. Rename the sketch **Front leg and arm post / stock from A-B-C** and the
   extrusion **Front leg and arm post / stock from A-B-C / Stock 630 mm**.
   Undo/Redo the sketch rename; its extrusion reference and feature IDs survive.
3. Apply **Deep green** (`#41544D`) and **Painted timber (visual designation)**.
   These are visual metadata, not structural material qualification.
4. Make the reusable **Front leg and arm post** component, name its occurrence,
   place it at `[65, 30, 0]` with identity rotation and ground it.
5. Save through the native picker, close and reopen the document. Hide the
   finished sketch and save **01 / Front post stock** as a presentation view.
   Undo removes the view, Redo restores it, and Recall restores its camera.
   A temporary occurrence offset previews separately from mechanical placement;
   Escape removes the offset and restores the prior presentation.
6. Create the separate **Right front leg and arm post / stock from A-B-C** XY
   sketch. Constrain its rectangle to 65 × 65 mm at the origin, then finish and
   create **Right front leg and arm post / Stock 630 mm**, with zero taper.
7. Make and name the reusable **Right front leg and arm post** component and
   its occurrence. Place it at `[1070, 30, 0]` with identity rotation. Apply the
   same painted-timber visual metadata and deep-green color to both posts.
8. On the translated left post's front face, create **Front post / Apron pilot
   Ø3.5 x 39 / Z350** at source-local U = 32.5 mm and V = 350 mm. Use a simple
   3.5 mm hole, 39 mm blind depth and flat bottom. Undo/Redo removes and restores
   the cut without changing component placement or grounding.
9. Save **02 / Front posts and first apron pilot** with both bodies visible,
   then save the document. Recall the first view to isolate the original post;
   the second view shows both placed stocks.
10. Repeat the fully constrained 65 × 65 mm XY stock for the separate left/right
    rear posts, extruding 790 mm. Name both sketches, extrusions, definitions
    and occurrences. Place them at `[65, 380, 0]` and `[1070, 380, 0]`, with
    identity rotations and the same deep-green visual metadata.
11. Duplicate and rotate the right-front occurrence temporarily. Create a
    source-local pilot through the placed occurrence and verify that both
    shared instances update. Remove the temporary instance; Undo restores its
    exact ID, visibility and pose, and Redo removes it without deleting the
    definition or its geometry. Restore the retained post to identity rotation.
12. Edit the left-front apron feature to add U = 32.5 mm, V = 390 mm while
    retaining V = 350 mm. A blank added position blocks Apply; removing that
    blank preserves both completed rows. Apply and Undo/Redo preserve the ten
    feature IDs and four component placements.
13. Correct the right-front pilot support to its outward +X face, with source
    origin `[65, 0, 0]`, U along +Y and V along +Z. Set U = 32.5 mm and V =
    365, 400, 170, 205, 590 and 615 mm in one named rail/arm pilot feature.
    Keep the simple 3.5 mm diameter, 39 mm blind depth and flat bottom.
14. Save **03 / Four post stocks and handed pilots** with all four bodies visible,
    then save the project through the native picker. Named views retain camera
    and visibility, so recalling an earlier chapter shows its bodies' current
    geometry; it does not roll back the feature history.
15. Create a fully constrained 1070 × 28 mm XY rectangle and extrude 155 mm
    as **Long apron — face lap / stock from A-B-C / Stock 155 mm**. Apply the
    same painted-timber metadata, make one reusable definition and place its
    front/rear occurrences at `[65, 2, 260]` and `[65, 445, 260]`, with identity
    rotations. Their tops are at Z = 415 mm. Save **04 / Front and rear face-lap
    aprons** with all five bodies visible, then save the document. Other post
    pilots, apron/rail clearance bores, side rails, joints, drawing sheets and
    print layouts remain unfinished.

Use the saved **Named Views** for a demonstration:

1. Recall **01 / Front frame assembly**, **03 / Seat support frame** and
   **06 / Seat deck before fastening** to explain the frame and post clearances.
2. Compare **07 / Back rails before pickets** with **08 / Shared back rails lifted
   for joinery review**. Both rails use one part definition.
3. Recall **09 / Crowned back with nine shared pickets**, then **11 / Center picket
   lifted for slot review**. The 18 mm slot is centered on a real mid-thickness
   datum; the exploded presentation leaves mechanical placement unchanged.
4. Return to **10 / Continuous arm supports before armrests** to continue with
   the handed armrests, fastening, joints and drawing package in the design plan.

Build the next steps through UI controls or literal typed MCP operations.
[`cad_route` with `action: "batch"`](../../docs/mcp-harness.md#choose-the-document-owner)
groups up to 16 ordered calls; inspect each receipt and return to the agent loop
for new IDs. Follow `active_session_id` after
Undo, Redo or document replacement, and inspect `build_pair.status` before
qualifying a live GUI/MCP pair.

For command-line replay, use the [developer guide](../../docs/DEVELOPMENT.md#replay-a-recipe).
It covers packaged CAD (`--server-arg --headless`), standalone servers and AppImage arguments.
MCP lists the same collection with `cad_interface {"action":"recipes"}` and runs
one with `{"action":"script","recipe":"mounting-plate","mode":"fast"}`.
The app's example list is supplied by that same Rust catalog in `crates/recipes`.
Catalog discovery does not build anything. A caller must choose a file or recipe;
there is no implicit bench run.
Use `--repeat 2` for independent headless comparison. After preserving the current
document, use `--session UUID --new --present --speed 2` to create a blank design tab
and watch the same sequence in that existing window. Omit `--new` if the named tab
is already blank. The script refuses to construct over an existing model.

See [the native script format](../../docs/native-scripts.md). Each `.limo.jsonc` source
replays construction; the generated `.limo` project retains the editable result.

The bench, turbine and vise are all runnable manufacturing candidates. The two
new sources include their editable drawing packages and teaching notes. Their
native tests check independent replay, model reload, intended dimension edits,
solved motion, interference and printable mesh integrity. See
[flagship status](../../docs/flagship-examples.md) and the individual design docs
for evidence and remaining physical qualification. The bench's full drawing
package remains open, and no flagship has a physical load or durability rating.

The migrated small fixtures preserve #89's analytic bounds/volume, independent
replay, fresh-process native restore, STEP round trip and STL/3MF checks in
`mcp-server/tests/recipes.rs`. The repeated bracket check restores both the 20 mm
baseline and 25 mm edited project, checks solved occurrence poses and verifies
all three profiles retain zero degrees of freedom. Export is a tested derivative;
native sketches and features construct the source model. The old Node recipe
runner and custom argument substitutions are not part of this library.

To add a feature lesson, commit its `.limo.jsonc`, add one catalog entry, and add
the focused geometry/edit check that establishes the feature's intent. Titles,
chapters, step counts and the actual operation list come from the source. Declare
the operation the lesson teaches; do not advertise every incidental setup command
as a hover lesson. Use existing native CI and replay tests, without an all-tools
percentage gate or an additional runner.

- [Compose box from collection](compose-box.limo.jsonc): tiny root script that
  `includes` [collections/box-stock.collection.jsonc](collections/box-stock.collection.jsonc).
  Demonstrates part-file isolation; run with an absolute `path` and `mode: "fast"`.

