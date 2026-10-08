# Native transition status

Release qualification checkpoint: **2026-10-06 UTC**, with the local UI walkthrough
updated **2026-10-08 UTC**. The default desktop on the Bevy integration branch
uses **Bevy `=0.20.0-rc.2`**, application version **0.2.2**, one native host and
one shared CAD/CAM command path. The integration is tracked by
[PR #124](https://github.com/limo-cad/Limo-CAD/pull/124) and has not merged
into `main`. Passing required checks and an external approval remain merge gates.

The public [Bevy preview](https://github.com/limo-cad/Limo-CAD/releases/tag/bevy-preview-0.2.2-20261004.1)
contains **Windows x64 ZIP and Ubuntu 26.04 x64 DEB from `9b082687`**.
Windows passed SDK-free headless/desktop MCP
checks on Thunder; Ubuntu passed its hosted MCP, X11 and Wayland-desktop checks.
The independently built hosted Windows x64 package also passed native-input
checks on this clean source. The AppImage build and Ubuntu 26.04 qualification
passed in the [tagged package run](https://github.com/limo-cad/Limo-CAD/actions/runs/37232261112),
but that artifact has not been added to the public preview. Windows ARM64 failed;
macOS built and signed but remains blocked by Apple's team-agreement HTTP 403.
The superseded October 2 preview release was removed; its source tag remains.
Application version alone does not identify which source was built. Published
packages retain the former noBS-CAD name while the repository and public project
name are Limo CAD.

Thunder uses one canonical **Limo CAD** executable at
`%LOCALAPPDATA%/limo-cad/bevy/Limo-CAD.exe` for the GUI and `--headless` MCP.
Managed local commands verify the source checkout, SDK, feature mode and installed
payload before reuse. The adjacent `runtime-manifest.json` records the deployed
revision and SHA-256; inspect `build_pair.status` and require `matched` before
qualifying a live GUI/MCP pair. Application version alone is insufficient.

Current October 8 Medix qualification: clean matched `64472748` opens the complete
saved import with zero scene errors and zero display warnings. Its live model
matches the saved archive: one compound body, 8,908 face identities and dimensions
536 × 327.47 × 48 mm. Actual GUI 3MF export remains refused: the `cb464867`
trial reduced invalid boundary links to 49, while the latest split-strategy
trial regressed to 72, both with no multiply used links. Connector caps and
independent pole proposals now qualify. A bounded fallback to the earlier
split strategy now retains its primary trajectory and defers alternate trials;
remaining source-angle failures and curved trim crossings are under investigation.
All 70 installed payload hashes match the clean manifest. No successful 3MF or
native-precision-qualified STL is claimed. The Medix notes below record earlier
trials; they do not supersede this current qualification.

The subsequent clean matched `35722a50` GUI export still reports 86 unmatched
links. Three continuous source intersections are now qualified, but the thin
three-edge strips are refused as nonsimple or unoriented. The next candidate
replaces the scale-dependent UV segment classifier with conservative certified
separation; ambiguous contacts and every existing source/domain/closure guard
still reject. It is not yet qualified through the GUI.

The subsequent clean matched `8ee3a909` GUI export gets past that simplicity
check but still reports 86 unmatched links. Its leading thin spheres now stop
after meshing at **connector cap lacks a positive source fan**. A bounded
constrained cap triangulation now preserves every connector and tail station,
checks positive disjoint ears and seven source precision witnesses, and bounds
backtracking to 512 states. It awaits GUI qualification; no successful
native-precision export is claimed.

Actual guarded GUI export on clean matched `0226ec9f` qualifies those connector
caps and reduces invalid boundary links from 86 to 53 across the same 33 native
shells, with no multiply used links. Remaining groups include a source-angle
refinement failure on face 2449, curved trim crossings on 8033/8014, and two-pole
faces 7236/7146. A bounded two-pole export extension now certifies each independent
native vertex and original wedge against the final composed quotient, with at
most four representative choices; it awaits GUI qualification. Export is still
refused, and the saved Medix model remains unchanged.

On clean matched `cb464867`, actual GUI export reduces invalid links from 53 to
49, still with no multiply used links across 33 native shells. The independent
pole proposals now reach refinement; face 7236 fails its source-angle check
instead of the earlier one-pole eligibility gate. Medix reopens with zero scene
errors and zero display warnings and still matches its saved archive exactly.
The next bounded refinement proposal evaluates all four children of both owners
at seven source witnesses before selecting an interior split, and each child
inherits its own parent's depth. Original precision, work/depth/node limits,
native boundaries and final closure checks remain unchanged. GUI qualification
is pending; native-precision export remains refused.

The actual clean matched `64472748` lookahead trial regresses from 49 to 72
invalid links, with 35 repaired faces instead of the previous 37. The next
candidate restores the exact original metric and max-owner depth policy before
any alternate is attempted. Deferred lookahead begins from fully restored mesh,
wire/face status and boundary-index snapshots within the same cumulative caps.
Bounded stderr diagnostics include failed triangle coordinates and source
normals, plus exact trimmed source-PCurve intersection status/counts and quarter
chord errors; they change no precision or topology acceptance checks.

The actual guarded GUI export on clean matched `7c235990` still reports 86
unmatched links. Native curve diagnostics rule out zero-area incidence loss on
the leading sphere faces: all ten existing facets have positive area, but native
trim samples are missing from their mesh. Their crossing chords correspond to
nearly coincident source curves, with approximately 1.407e-7 mm native separation
and recorded tolerances of approximately 0.0483 mm. This observation alone does
not prove continuous curves are disjoint. `96b567a7` adds export-only continuous
intersection qualification and f64 strip witnesses. Its actual guarded GUI
qualification at `d2ba9017` exposed the source-normal failure described above;
the initial three proposed strip repairs were all rejected. Matching strip
boundary stations and facet connectivity are being investigated while retaining
the seven source witnesses. Export is still blocked; no mesh guard is waived.

The clean matched `7c235990` bench session now has a separate preserved
[lower-clearance checkpoint](../examples/checkpoints/garden-bench-lower-clearances-human-ui.limo)
with nineteen features. Four lower-rail bores were created through the physical
Hole dialog at U/V `(32.5, 40)`, `(32.5, 75)`, `(382.5, 40)`, `(382.5, 75)`, using
5.5 mm diameter, 28 mm distance, simple style, flat bottoms and no flip. The
feature is named; physical Undo/Redo and Ctrl+S restored the same feature and
positions, and the saved model matches the live document. The original bench
checkpoint remains unchanged. The lower rail remains unplaced source stock,
with no appearance or separate component definition yet; upper and lower rail
definitions must remain separate. This does not complete the bench walkthrough.

On clean matched `d2ba9017`, physical File/Open reopened the saved four-bore
bench. Its lower stock now has the upper rail's same deep-green timber appearance
and a separate named **Lower side rail** definition, with one named **Left lower
side rail** instance at the source origin. Ctrl+S and read-only archive comparison
qualified the saved [lower-component checkpoint](../examples/checkpoints/garden-bench-lower-component-human-ui.limo):
seven definitions and nine occurrences, with existing upper/apron shared
definitions and placements preserved. Lower-rail placements, its shared right
instance and the remaining bench geometry are unfinished.

On clean matched `19ca11c7`, the physical Assembly UI placed the left lower rail
at `(37, 30, 130)` mm, duplicated it, and named/placed the right lower rail at
`(1135, 30, 130)` mm. Both rotations are zero. Ctrl+S and read-only comparison
qualified the [lower-placement checkpoint](../examples/checkpoints/garden-bench-lower-placements-human-ui.limo):
nineteen features, seven definitions and ten occurrences. The two lower rails
share definition 14 and its four clearance bores, separately from upper-rail
definition 12. Remaining pilots, geometry, joints and drawing/layout work are
unfinished.

On clean matched `08efc3e7`, physical top-face selection and the Hole dialog
added the lower rail's two Ø3.5 mm, 32 mm deep, flat-bottom stretcher pilots at
stock points `(14, 195, 90)` and `(14, 235, 90)` mm. The named feature and Ctrl+S
produced the [lower-pilot checkpoint](../examples/checkpoints/garden-bench-lower-pilots-human-ui.limo):
twenty features, seven definitions and ten occurrences, with both lower rails
sharing the four side clearances and two top pilots. Read-only inspection of
the saved parameters and exact live/archive comparison passed. Upper-rail top
pilots and remaining post pilots, geometry, joints and drawings are unfinished.

On clean matched `35722a50`, the physical Hole dialog added the upper rail's five
Ø3.5 mm, 32 mm deep, flat-bottom seat-slat pilots at stock X 14 mm, Y
12.5/102.5/192.5/282.5/372.5 mm, Z 90 mm. The named feature, all-body Browser
visibility restoration and Ctrl+S produced the
[rail-pilot checkpoint](../examples/checkpoints/garden-bench-rail-pilots-human-ui.limo):
twenty-one features, seven definitions and ten occurrences. Upper and lower
rails retain separate definitions and their correct shared pilot patterns.
Saved parameters and exact live/archive comparison passed; the full frame was
captured. Remaining post pilots, geometry, joints and drawings are unfinished.

On clean matched `0226ec9f`, guarded physical Hole input added the left-front
post's six Ø3.5 mm, 39 mm deep, flat-bottom rail/arm pilots at stock
X 0 mm, Y 32.5 mm, Z 365/400/170/205/590/615 mm. The named feature, restored
all-body visibility, Isometric/Fit and Ctrl+S produced the
[left-front pilot checkpoint](../examples/checkpoints/garden-bench-left-front-pilots-human-ui.limo):
twenty-two features, seven definitions and ten occurrences. Saved parameters,
previous hole patterns and assembly preservation, and exact live/archive
comparison passed. Right-front apron and both rear-post pilot patterns,
remaining geometry, joints and drawings are unfinished.

On clean matched `cb464867`, physical Hole input added the right-front post's
two Ø3.5 mm, 39 mm deep, flat-bottom apron pilots at stock X 32.5 mm, Y 0 mm,
Z 350/390 mm. Naming, restoring all Browser eyes and Ctrl+S produced the
[front pilot checkpoint](../examples/checkpoints/garden-bench-front-pilots-human-ui.limo):
twenty-three features, seven definitions and ten occurrences. Both front posts
now retain their apron and six-position rail/arm pilots. Saved parameters and
exact live/archive comparison passed. Both rear-post patterns, remaining
geometry, joints and drawings are unfinished.

The October 7 manual reconnect now has an explicit `cad-call --installed
--interactive` path. It verifies the recorded clean deployed identity, enabled
native control and all installed payload hashes without rebuilding, promoting or
closing CAD when the source checkout advances. Live reconnect to the unchanged
`0db4c009` payload returned a matched GUI/MCP pair and rejected an inactive bench
session after a second tab became active. Rendered bench capture succeeded. A
foreground request was denied and a fresh observation confirmed CAD was still
in the background; physical input qualification after this reconnect remains
pending foreground ownership.

The Windows focus source now synchronizes with the window queue for at most one
second after an accepted activation request, then rechecks process/document/window
ownership and actual foreground state. Its native-control build check and full
native desktop build pass. The fixes were deployed from clean `2e98c2f5`; the
reopened bench reports a matched GUI/MCP pair. Genuine Windows foreground denials
remain correctly rejected. On October 8, an accepted guarded activation on clean
`75a04561` was followed by a fresh foreground=true observation and a physical
click selecting the bench tab. This qualifies that accepted activation and
pointer sequence; it does not establish universal focus or gesture timing.

The user-reported `Medix_KW22_v4.STEP` import reached OCCT transfer, then failed
with "could not discretize tangential face boundaries without crossing chords".
The complete 42 MB SolidWorks AP214 file contains 24 solid definitions and 8,478
faces. The custom mesher now recognizes crossings inside a real shared vertex's
OCCT vertex/edge tolerance instead of refining those tolerated junctions until
failure. Crossings outside that tolerance retain refinement and rejection;
remaining errors identify the face, wire, edges, curve types and tolerance.
The STEP source is unchanged. The native desktop build passed and the correction
is installed in `2e98c2f5`; physical UI import qualification still requires
foreground ownership. This does not yet establish successful Medix import or
broader geometry qualification. Before deployment, the prior bench's published
model was compared semantically with its saved archive and matched exactly; its
second tab contained no features after the failed import. Recovery and session
data were preserved.

On October 8, foreground ownership was confirmed and guarded physical input
opened a new tab, selected File > Import STEP, entered the Downloads path in the
owned native dialog and accepted it. The original crossing-chord error no longer
occurred, but import still failed at body 1 face 2225 (area 0.14921575060521969,
aggregate mesh status 6); the tab remained empty at generation 1. The mesher now
runs OCCT's standard healer before its custom circular-boundary sampling, then
checks affected boundaries before clearing intersection-specific failure flags.
Generic failures and strict nonzero-face triangulation checks remain. Additional
diagnostics report the failed face's own status, surface and bounded wire/sample
counts. Clean `75a04561` built and was retried through guarded physical input;
the same face still failed. Its own status was 6, its surface was B-spline, and
its three boundary edges had 22, 11 and 18 samples. The next correction refines
only edges reported by OCCT's boundary-intersection checker, evaluates added
points on the exact curves, preserves healed UV endpoints and rechecks adjacent
faces. It is bounded to eight passes, 4,096 points per edge and 65,536 added
points per model. Strict rejection of unresolved nonzero faces remains. This
candidate built and was physically retried from clean `ef4c3422`. The same face
still failed with status 70 and edge sample counts 2,689, 11 and 2,177. Refinement
alone did not resolve it; successful Medix import is not yet established.
Clean `15d127a0` built, deployed and was physically retried on a matched GUI/MCP
pair. The crossing is between internal samples of two edges sharing a vertex:
its surface point is 0.0347300908856 mm from that vertex, whose tolerance is
0.0346996401522 mm. One source pcurve differs from its native 3D curve by
0.0000319350385369 mm there. The standard healer shifted that edge's endpoint
by 0.0262846534818 mm on the surface. Dense sampling reached its per-edge limit
without removing this crossing. The next investigation targets a transactional,
tolerance-bounded repair of the sampled junction while retaining shared-edge
sample consistency, exact imported topology and every nonzero face.
Clean `a1e7f5da` built, deployed and was physically retried on a matched GUI/MCP
pair. Seven sampled junction repairs passed their adjacent-face boundary checks;
the first unresolved face moved from 2225 to 2281. Import still failed and the
new tab remained empty. The next crossing is 0.0314075013296 mm from its shared
vertex, whose tolerance is 0.0313735171429 mm; the measured source pcurve/3D
discrepancy there is 0.0000174693969219 mm. The candidate preserves imported
topology and rejects unresolved nonzero faces. Successful Medix import remains
unqualified while this next junction is investigated.
Clean `afe296af` built, deployed and was physically retried on a matched GUI/MCP
pair. The repair now includes measured original pcurve/3D discrepancies at the
shared endpoint, each bounded by its incident edge's recorded tolerance. All
eight sampled junction repairs passed. Import progressed to body 1 face 3779,
a spherical face with signed area -0.000015055593139196334 mm², status 4 and no
reported boundary intersections. That nonzero face was rejected rather than
omitted; the import tab stayed empty. Its meshing failure is the next unresolved
issue, and successful Medix import is still not established.
Clean `dee3319c` built, deployed and was physically retried with additional
read-only failure diagnostics. Face 3779 is a radius-1.2 mm spherical strip with
U span 1.560393444807 radians and V span 0.00001503703578442 radians. Its sampled
wire has positive UV area 0.00000278218692873, and the sphere range splitter is
valid. The default Watson triangulator produced no triangulation despite the
accepted boundary. Face tolerance is 0.0000001 mm; both circular edges have
approximately 0.04823 mm tolerance. This distinguishes the remaining failure
from the resolved boundary intersections. A bounded alternate triangulation of
the same shared samples is being investigated; complete import remains open.
Clean `6cb350e4` built, deployed and was physically retried on a matched GUI/MCP
pair. The bounded spherical Delabella retry was attempted on two freshly failed
faces, but neither yielded an accepted triangulation. Face 3779 still failed and
the import tab remained empty. No failed face was omitted. Retry progress-range
ownership and more precise trial diagnostics are the next investigation.
The diagnostic subclass in `000a546d` failed Windows linking because it exposed
unexported OCCT constraint helpers; deployment preserved the installed runtime.
The correction uses the exported factory API and bounded boundary diagnostics.

Clean `2ae55e47` built and was manually retried. Native curve measurements
confirmed an interior self-crossing in the thin spherical boundary, away from
either shared endpoint's tolerance envelope. Neither Watson nor the bounded
Delabella retry accepted the two affected faces. Their exact geometry was not
changed to invent a triangulation.

Clean `4d3cf633` built, deployed and passed the actual Windows UI import and
save/reopen walkthrough on a freshly observed matched GUI/MCP pair. Original
imported STEP bodies may now open with explicit display warnings for unavailable
face triangles. Exact STEP/BRep geometry, stable face slots and boundary edges
remain retained. The persistent amber banner and read-only MCP summary expose
these omissions; derived modeling, cutaway and mesh exports remain strict.
Imports without any usable triangles still fail.

The Medix import has one compound body, 8,908 face identities, 453,400 display
triangles, no feature errors and two warnings (`face:3779`, `face:3792`). It was
saved through the native picker as
`D:/limo-cad-maintenance/ui-models/Medix KW22 v4 - human UI import.limo`.
The archive's embedded STEP is byte-for-byte identical to the 42,065,951-byte
Downloads source (SHA-256
`17d7473c73db7bdf302aa118fe377bae10578f1c72fe03238f8448545d26d8d2`).
Closing and reopening through File > Open reproduced the same body metrics,
warning keys and saved model. A physical File > Export All Bodies as STL attempt
was explicitly refused for incomplete display tessellation; no STL was created.
Evidence is retained outside source under `D:/limo-cad-maintenance/ui-models/medix-4d3-*`.
This qualifies opening and document preservation, not complete triangulation or
printability of Medix. No test suites or recipe replay ran. The separate
Roller-300 scratch runtime was not restarted or operated. A foreground change
during the export check was recovered by guarded focus, fresh observation and
retry; universal Windows focus behavior remains unqualified.

The complete native-picker capture guard at `cbe74672` was manually exercised
on matched clean `0dbd0fdf`: a fresh Downloads STEP import and Save As used
full observed dialogs before each input. The saved archive retains identical
source bytes; the additional checkpoint is
`D:/limo-cad-maintenance/ui-models/Medix KW22 v4 - fresh UI import 0dbd.limo`.
The mesh-only spherical repair at `aeac1cd1` passed its geometric preparation
checks but refused nonanalytic neighboring charts. Clean `15c67f8c` retains
their healed UV trim polygons and bounds shared-point residuals instead of
performing ambiguous inverse projection. Both trials reach final adjacent
coverage/incidence validation, which rejects them and rolls back. A fresh
matched GUI/MCP pair still reports both original warnings;
complete triangulation and mesh export qualification remain unfinished.

Further matched runs through `6c6d5c5b` preserved and restored omitted native
trim corners only after coverage and precision checks, but still rolled back
both trials. The clean `feb57db3` run includes the repository transfer to
`limo-cad/Limo-CAD` and the `2252074f` diagnostics. Those identify native
degenerate edges on neighboring faces 5925 and 5497: their native endpoints
coincide, while the imported surface-coordinate endpoints differ by about
0.031 mm. The original model remains unchanged and both display warnings
remain visible. A tolerance-bounded treatment of those collapsed native edges
is still under development; successful complete mesh exports are not claimed.

Clean matched `367dfbfb` qualifies the native pole's localized chart crossing
and its sampled source-to-mesh precision without changing the STEP geometry.
Both reopening the prior checkpoint and a fresh physical File > Import STEP
from Downloads report zero scene errors and zero display warnings, with all
8,908 face identities and 453,491 triangles. Escape dismissed the import's
appearance panel; a physical Fit and native Save As created
`D:/limo-cad-maintenance/ui-models/Medix KW22 v4 - complete human UI.limo`.
Its embedded 42,065,951-byte STEP has the original SHA256 and the live model
matches its saved archive. This is complete display qualification, not complete
export qualification: actual STL export contains 27 triangles collapsed during
epsilon welding, and actual 3MF export refuses a degenerate welded triangle.
The initial STL is retained as diagnostic evidence; it is not a qualified mesh.
Source-aware export precision work remains open. The 536 mm imported model
also exceeds the default printer envelope; portable export does not qualify
physical printing or its unchanged source placement.
The matched `00603de3` read-only native tessellation inspection returns all
453,491 triangles without a float-precision failure. Its actual UI STL retry
is then refused by the strengthened writer at triangle 22,934, with no new
file created. The export placement layer was found to weld nearby distinct
vertices before the format writer. Preserving raw STL placement and trying
exact-coordinate indexing before 3MF tolerance welding are awaiting UI
qualification; both formats retain strict geometry checks.

Clean matched `f062b45a` now passes actual UI STL export. The new
`D:/limo-cad-maintenance/ui-models/Medix KW22 v4 - verified human UI.stl`
contains all 453,491 facets, with zero nonfinite coordinates, zero-area facets,
zero normals or opposing normals; its SHA256 is
`0edc2caa21296b927e18278c12b8084dc47828dfaab44aee48fbfd006fe76a49`.
Actual tab close and native File > Open reopen the complete checkpoint with
zero scene errors and display warnings, and the live model still matches the
saved archive. Actual UI 3MF still refuses a degenerate triangle after tolerance
welding. Read-only inspection of exact-coordinate STL topology finds 94 edges
with four incident facets and 113 single-use edges: coincident native solids
must retain separate identities, while small shared-boundary discrepancies need
native topology correspondence. Raising a global tolerance would collapse valid
small facets. Native topology-indexed export remains under development; no
successful 3MF or physical-print qualification is claimed.

The next clean matched `26a7ea22` uses native shell/vertex/edge identities for
export indexing and checks native winding before writing. Actual UI 3MF
preflight exposes a further precision defect at face 3794, triangle 1: ordinary
nearest-f32 rounding alone reverses this approximately 14-micrometre edge facet
by 1.649 radians. The same ordered rounded vertices are present at facet 160,565
in the preceding STL. Its earlier normal check verified stored normals against
serialized vertices, so that file is now retained only as diagnostic evidence;
it is not a native-winding-qualified export. A separate f64 export channel and
precision-preserving text STL path are under development. Display remains at
zero errors/warnings and the saved source geometry remains unchanged.

Clean matched `280e1f64` installs the separate double-precision native export
channel and retains every positive-area native facet. All 70 installed payload
hashes match the clean manifest. Reopening the complete checkpoint still reports
zero scene errors and display warnings; its live model matches the saved archive.
Actual UI 3MF preflight now reaches native closure validation but refuses 115
unmatched links across 33 native shells. Retaining positive tiny facets did not
change that count. Source-face diagnostics identify boundary shortcuts, including
faces 2449 and 1491, whose neighboring faces retain intervening native edge
samples. A native-topology-proven repair remains open; no precision-qualified
STL or successful 3MF export is claimed. Canceling preflight leaves the document
unchanged and the CAD window active.

Matched `b3d5d5fc` adds export-only, source-checked boundary recovery and corrects
its false candidates on ordinary reversed boundary links. Actual UI preflight
still refuses the same 115 links: five attempted face repairs were rejected,
with none installed. The ranked diagnostics identify genuine native degenerate
edges at the largest failing face groups, with coincident native endpoints but
distinct surface-coordinate representatives. Pole-aware recovery remains open;
the native precision, domain and closure guards have not been relaxed.

Matched `04cc231f` reduces the actual UI export refusal to 107 invalid links:
105 unmatched boundary links and two links used four times. Native edge ownership
is valid at the largest failing groups; the remaining gaps belong to the copied
export triangulation. Four positive boundary facets were restored with source
checks. The two multiply used links are internal diagonals on curved native
faces, whose incident UV regions classify inside their source trims.

Clean `175f571f` builds and installs without errors; all 70 payload hashes match
its manifest. Actual GUI 3MF preflight still refuses those 107 links. Both tested
surface-point diagonal refinements were rolled back after the complete face
winding guard rejected them. The live Medix document matches the saved complete
checkpoint and reports zero scene errors and display warnings, with 8,908 faces
and dimensions 536 × 327.47 × 48 mm. No native-precision-qualified STL or 3MF is
claimed. The foreground GUI/MCP pair is matched; unique GUI output/error logs
are retained for the previously unexplained process exit.

The export-only minimum-size trial at `3b61ad80` increases the actual refusal
to 136 links. Its diagnostics identify a genuinely inverted original native
facet, with a normalized source-normal angle of 3.009 radians; the derivatives
are finite and nonzero. `6d916088` reverts that minimum-size setting and trials
a larger source UV patch. Actual UI preflight returns to 107 links, rejecting
its replacement child at 0.810 radians against the unchanged 0.700 interior
angle budget. No export artifact is accepted by either trial. Source geometry
and the complete Medix checkpoint remain preserved.

Matched `d724917f` resolves both multiply used links through the certified larger
UV patch and centre search: actual GUI 3MF preflight now reports 105 unmatched
boundary links, with no multiply used links. One internal patch is accepted;
the boundary recovery count remains unchanged. Export is still refused and no
successful 3MF is claimed. The remaining boundary gaps are under investigation.

The bench Hole dialog was reopened through physical input on matched `3b61ad80`
without reproducing the earlier unexpected process exit. Enter in a position
field accepted the dialog; physical Undo removed that premature hole and restored
eighteen features. A separate local **Crown garden bench - lower stock isolated
human UI.limo** preserves the isolated lower-stock view. Its sketches, extrudes,
holes and assembly match the original bench archive, and its live model matched
the separate saved checkpoint before restart. This adds no lower-rail bores;
the original bench document and committed checkpoint remain intact.

The October 7 [human-operated bench checkpoint](../examples/checkpoints/garden-bench-human-ui.limo)
was built through real OS mouse/keyboard input on matched clean GUI/MCP builds
through `0db4c009`. It contains eighteen named features, four separate posts,
two shared aprons, two shared upper side rails, eight post pilots, six shared apron clearance positions,
four shared upper-rail clearance positions, a separate 28 × 415 × 90 mm lower-rail
stock and five named views. The lower stock is fully constrained but still has no
bores, component definition, appearance or assembly placement. UI checks cover fully
constrained stock, translated/rotated shared editing, precise multi-position Hole
editing, instance removal and Undo/Redo. The [walkthrough](../examples/scripts/README.md)
states its unfinished geometry, joints, drawings and print layouts. The vise and
turbine have not yet been rebuilt in this human-operated pass. No test suites or
recipe replay were run for this pass; read-only MCP inspection verifies the result.

Live sketch checks on `f055cc02` confirmed that Dimension can pick an edge through
its constraint glyph without hiding annotations. Select still opens the glyph
inspector and the stored dimension editor; cancellation retains zero degrees of
freedom. Wheel zoom worked over a glyph after ordinary Select interactions. The
first wheel after Fit did not move the camera; its cause remains unattributed,
so this does not qualify every move/wheel timing sequence or physical pinch input.

Native computer control is opt-in through `native-computer-control`, with empty
default features. The Windows backend reuses Enigo for input, Windows Capture for
owned native dialogs and that crate's in-memory PNG encoder. The generic Windows
qualification driver uses Rust and this same control path behind xtask's opt-in
`native-control-harness` feature; specialized IME/print probes and Linux/macOS
drivers still include platform scripting. These source changes do not establish
cross-platform input qualification.

The historical `7137887f` payload passed ten SDK-free MCP checks and 27 steps,
including Save, disconnect survival and guarded close. That evidence remains
specific to that revision and does not qualify the newer walkthrough or replace
the public Windows/Linux package qualification.

Cursor and Codex now register **`limo-cad`**. Their retired CAD entries were
removed; the retired Grok entry was also removed. The Start menu, project/recipe
associations and previous launch paths route to the new payload. The three retired
physical payloads were deleted after the user's manual cleanup. Documents,
recovery saves and the session registry remain intact. See
[runtime identities and migration](limo-cad-runtime.md).

The October 5 cleanup removed 24 local branches only after confirming their
commits were reachable from Bevy and they had no open PR or active worktree.
Unique retired work remains in the verified 104-head Git bundle at
`%LOCALAPPDATA%/limo-cad/archives/Limo-CAD-retired-branches-20261003-051405.bundle`.
Three inactive runtime trees moved to
`D:/limo-cad-maintenance/retired-runtime-payloads`; all 229 files retained their
hashes, recovering about 1.6 GiB on C:. The subsequent manual deletion removed
all 229 runtime files and recovered about 2.4 GiB on D:. Their separate hash
manifests remain. Active CAD windows,
documents, recovery saves, SDKs and session data were preserved.

The complete identity migration moves native configuration to
`org.limocad.desktop` and new leases/inboxes to `limo-cad-sessions`.
The real previous profile moved intact; every existing file retained its SHA-256.
Fresh MCP attach, rendered inspection and read-only assembly execution passed
against the normal installed desktop without advancing its model generation.
The installed executable SHA-256 is
`8EDC25D99BACF40EC7B3887805AE4F39E36DB8D6B7A8F8F37738E155B36AFD02`.

Ordinary Rust/C++/JSONC comments and auxiliary workflow/probe comments were
removed while preserving Rust documentation and interpreter directives.
Stale TypeScript/React descriptions were corrected. Focused migration, archive,
CAM-header/replay and workflow checks passed; strict scoped tooling Clippy passed.
The comment-removal spacing issue in CAM documentation is corrected. Strict
all-target, all-feature Windows desktop Clippy and scoped native-engine/sketch
Clippy pass on the current integration source. The October 6 follow-up scopes
printer-only SVG resources to Windows and fixes macOS 6DoF lints; its hosted
Windows and Ubuntu native-host jobs pass. This does not establish a globally
warning-free build or qualify every platform. No large validation sweep was run.

## Implemented desktop

Bevy owns modeling/sketching, feature forms and history, assemblies and joint
motion, drawing authoring/reference repair/output, CAM, Scripts and lessons,
preferences/localization, file/window lifecycle, printing, accessibility, IME
and 6DoF input. Document units retain the shared engine's read-only contract.
The native document/OCCT host lives in `crates/native-engine`; the desktop uses
a small adapter. Its optional `native-occt` feature owns transactions, exports,
geometry revisions and tab retention without a Bevy/window dependency.
Ordinary engine builds leave that feature disabled and require no native SDK.

The conversion includes:

- Persisted interface sizes from 90% to 175%, independent cross-window refresh,
  matching layout/scale publication, and guarded pointer/text/composition state.
  The follow-up audit fixes large-size sketch menus, origin controls, drawing
  menus and CAM report bounds. Current main's normalization and Ctrl/Cmd
  plus/minus/zero shortcuts are preserved (#211, #213, #215, #251).
- Independent CAM height references for planar faces, level edges, vertices,
  sketch points and level sketch lines. The shared resolver supplies stable
  identities through the existing bounded picker and draft/Apply path (#212).
- Associative drawing exports, placed views and reference repair; installed-font
  outlines for Unicode DXF labels and native printing. Unsupported glyphs fail
  before output. Saved drawing DTOs, placement and paper size remain authoritative.
- Production AccessKit bindings with current control/document/modal guards,
  Windows UI Automation text editing, and read-only/writable field distinctions.
  IME caret placement follows visible field bounds and scale changes; provisional
  composition retains the existing editor checkpoint and committed text.
- Restored inactive-tab eviction, including finished-sketch Undo/Redo (#222, #249).
- Cached authored viewport metadata per native document, independently of
  assembly placement. Immutable engine queries share this data with UI and MCP
  and skip mutation evidence observation. Sketch/solid edits refresh metadata;
  drawing edits, placement changes and revisiting retained tabs reuse it.
  Eviction releases the cache. Browser/history actions, units/name reads and
  synchronous exports borrow their source data under the existing engine guard.
  Drawing panels and hit testing borrow retained annotation marks. Paper/cache
  keys and saved drag receipts share immutable view/style metadata; revision
  stamps remain independent so sheet selection cannot retag a saved receipt.
  The title-block cache borrows only rendered metadata during lookup and retains
  an owned snapshot after successful rendering. Annotation edits reuse frame
  artwork; rejected frames move the existing sheet snapshot into the error receipt.
  Pose-vector copies, per-body replay invalidation and presentation resource
  granularity remain tracked in [#333](https://github.com/limo-cad/Limo-CAD/issues/333).
- An explicit Winit window-icon binding (#259). The deployed Windows small-icon
  handle and native chrome capture confirm the title-bar fix. The packaged MCP
  passed schema-7 attach, rendered inspect and a read-only assembly query.
- In-place sketch editing of a selected shared occurrence (#94). The native UI
  and `sketch_edit` MCP operation use the same validated occurrence frame for
  rendering, picking and dimensions. Surrounding occurrences fade without
  replacing shared meshes. Saved sketches retain definition coordinates;
  recompute updates shared bodies and joint references. Focused tests cover an
  offset component coordinate system, translated/rotated repeats, driving edits,
  save/reopen and rejection of unrelated targets. Linked external files remain
  a separate design decision; physical bench qualification remains outstanding.
- Stateless multi-document MCP routing (#12) through `cad_route`, preserving
  the default attachment and never loading routed models. Per-document inboxes
  and control queues enforce captured owners and generation conflicts. Tickets
  retain completion receipts across tab changes, replacement and close.
- Typed shared drawing commands (#93) add/update/delete specialized annotations
  and views, preserve aligned groups, manage templates and append revisions.
  Release commands validate current topology and occurrence selection. Accepted
  model/assembly edits return affected issued sheets to Draft while retaining
  revision history; unchanged recompute and save/reopen preserve issued metadata.
  Native view and annotation editing use these shared operations. Idle annotation
  preview borrows its sheet, and point/radial pick targets reuse projection stamps.
  Drawing topology capture borrows the solid scene instead of copying its meshes.
- The component-edit recovery lesson (#16) uses the shared in-place operation,
  demonstrates a wrong driving value and Undo, updates rotated repeats and
  saves/reopens. Native lesson and Help catalogs expose the same authored source.
  Teaching and physical-input review remain open.
- The bench recipe creates 22 editable review sheets, including part dimensions,
  machining coordinates, an assembly sheet and cut list. The focused native check
  reproduces every SVG/DXF after reopen and updates the picket-height dimension.
  Its notes retain authored machining inputs; manufacturing review and physical
  timber/hardware qualification remain required.
- System appearance following (#272). Bevy 0.20 moved Winit windows out of the
  World; querying the obsolete resource always selected Light. The main-thread
  capture now reads initial OS appearance and primary-window theme-change events.
  A focused regression covers Dark/Light changes and ignores other windows;
  explicit appearance preferences still take precedence.

Quick-win fixes retain viewport/input/accessibility state, warm drawing-sheet
projections and CPU rasters, reduce document/scene copies, and use exact completion
revisions and document-specific mesh-cache incarnations. Incoming parent commits,
main's naming/translations/drawing changes and the recovered mechanism-dragging
implementation are preserved. No runtime latency improvement is claimed without
measurement.

## Inactive-tab retention

The former desktop's low-memory eviction was lost when its memory-status caller
was retired. The native watcher now probes physical memory every 30 seconds.
The ordered worker makes eligible inactive tabs cold after 60 minutes; constrained
memory evicts the oldest eligible tab and critical memory evicts all eligible
inactive tabs. Active tabs, unfinished sketches and saves in progress are protected.

Cold tabs retain their parametric model, geometry revision, replay baseline,
file/archive ownership, saved receipts and history while releasing the OCCT
engine, Bevy model meshes and drawing caches. Activation rebuilds transactionally
and verifies body identities and feature errors. Failure preserves the snapshot
and previous active tab. The 128-tab bound includes cold tabs.

The October 3 audit also found that serialized reconstruction discarded finished
sketch command stacks. Finished sessions now move into a separate in-memory
retention record, preserving Undo/Redo, runtime editing state and entity identity
high-water marks without retaining an OCCT kernel or solid scene. Rebuilt sketch
states must match before ownership moves; mismatch leaves the snapshot available
for retry. Normal project serialization/schema and file-reopen history policy
are unchanged.

Ten focused Windows tests passed, including real-OCCT reconstruction, repeated
eviction, actual sketch Undo/Redo, rejected restoration and retry, mismatched
sketch-state rejection, file/archive history, protected states, pressure/idle/LRU
policy and drawing-cache isolation. They were isolated in-process checks and did
not change a live document. **The public preview includes both eviction
restoration and the finished-sketch history fix.**

## Dependencies and build tooling

Rust `1.99.0`, rustfmt and Clippy are pinned together. Bevy stays at rc.2; OCCT
stays on the 7.9 ABI. AccessKit remains on Bevy's `0.24` types, Windows bindings
on wgpu/gpu-allocator's shared `0.62.0` types, and usvg/resvg on `0.45.1` because
svg2pdf `0.13` consumes those trees. These are compatibility constraints.

Repository maintenance uses Rust `cargo xtask`: scoped checks/Clippy, dependency
inventory, deterministic archives, version/tag/repository/icon/knowledge guards,
OCCT SDK orchestration, native fixtures, WASM build/smoke, packaged MCP setup and
Windows ZIP/Linux DEB/AppImage/macOS app/DMG packaging. Builders retain runtime
library and license staging, audits, checksums and platform signing/notarization.

Tauri, embedded WebViews, the `dev-bevy-host` switch, React/Three.js source, npm
manifests/lockfiles, Vite/Tailwind configuration, Node drivers, legacy bundlers
and obsolete browser/desktop IPC harnesses are removed. Native SDK containers
and packaging workflows do not provision Node/npm. Embedded vectors and locale
dictionaries remain in `assets/`; viewport colors live in Rust.

Remaining native OS qualification helpers use shell, PowerShell, Python, C# or
Swift for platform APIs and input. The repository is not entirely Rust.
Retired harness ownership is recorded in [scripts/README.md](../scripts/README.md);
that reassignment does not establish equivalent coverage or passing native cases.
Unused Feathers/scene support, redundant widget declarations, unused icon
variants and GTK/Rsvg AppImage development inputs were removed. Winit's actual
X11/XCB/cursor/input runtime libraries and Linux desktop portals remain required.
`sysinfo` is required again for the portable physical-memory probe.

The incoming standalone SDK-cache repair in main #245 is also ported to Bevy
(#302). Reused extracted sources are verified against the pinned archive before
reuse and receipt publication. Install-prefix locks exclude concurrent installers
using different cache directories; receipts reject external/dangling SDK links
and track internal targets. Eight focused Windows cache/SDK checks and strict
all-target xtask Clippy passed. The two Unix symlink fixtures remain for hosted
Linux qualification; no real SDK was downloaded, rebuilt or modified locally.

Focused checks cover Windows desktop/MCP compilation, Rust/wasm32 compilation,
Clippy, repository/version/icon/knowledge guards, package staging/deletion guards,
archive determinism and a fresh engine-facade build. Rust task-runner compilation
also passed for Linux x64 and macOS ARM64; compile checks do not qualify native
packages or signing. No broad validation sweep is being run.

The October 6 drawing follow-up passed strict Desktop/MCP all-target, all-feature
release Clippy and all three workspace format checks. Eleven focused native checks
cover exact view/release Undo/Redo, borrowed idle preview, retained paper navigation,
lesson catalogs, guarded hole authoring and profile export. Native MCP checks cover
specialized annotations, stale-edit rejection, template/revision commands and
release preservation/revocation. The bench check reproduces 22 SVG/DXF sheets after
reopen and updates an associative dimension after a height edit. Its Rust author is
idempotent and preserves all 883 original construction steps and original checks.
The opt-in turbine recipe reproduction binds both placed assembly views to the
production Bevy paper image and reuses the document/image on idle repaint. Its
linework pixels were independently inspected; this CPU image proof does not qualify
GPU scanout or OS-window capture. These additions postdate the historical
`7137887f` payload and public `9b082687` packages above. Consult the current local
runtime manifest for installed source identity; public package qualification is
still recorded separately.

## CI and security review

The October 4 cleanup landed full native-workspace formatting, narrow Clippy
fixes and compiler API updates, plus persistent fmt/Clippy jobs in the existing
required workflows (#284, #286-#288). Local native all-target/all-feature release
Clippy, root tooling Clippy and all three workspace fmt checks passed. The next
pass cleared the 20 xtask warnings and added a strict all-target xtask gate
without replacing workspace-wide Clippy (#298). Four core default warnings are
also cleared with unchanged defaults and serialization (#300; main #299).
Strict all-target checks pass for those packages; other workspace warnings remain
visible. Focused existing regressions and workflow contracts passed. Actionlint
uses an explicit inventory of the verified Ubuntu 26.04 and Windows 11 VS2026 ARM
runner labels; unknown-label errors are not ignored.

The `b2e5e241` and `985392f2` integrations passed Windows, Ubuntu and macOS native
host CI. Later integration source requires fresh checks. The `985392f2` package
and MCP runs exposed three separate failures now repaired below.
No broad local validation sweep or new live-model test was run.

CodeQL 2.27.1 now scans Rust, C++, Actions, JavaScript and Python (#289; standalone
main counterpart #285). Matching Rust compiler, sources and proc-macro server
bindings restore usable extraction for current Bevy dependencies; scan checkout
uses LF to avoid the extractor's escaped-newline CRLF parsing defect. The local
Rust scan extracted 775 files, with one platform-only macro warning and no
extraction-error query results. Its 146 logging alerts were source-audited:
138 are test diagnostics and eight are production diagnostics carrying public
CAD routing UUIDs. The IDs are discoverable through session tools and do not
authenticate callers. The matching GitHub alerts were dismissed as false
positives and three bot threads resolved; the logging query remains enabled.
The corrected Actions scan reports zero findings. This historical scan does not
qualify the current head. Main #285's Rust and C++ jobs hit the 90-minute job
deadline: Rust spent about 69 minutes preparing the cold SDK, and C++ was still
building it at cancellation. The neutral SARIF check came from unsuccessful-run
diagnostics, not a completed Rust scan. Actions and JavaScript uploads succeeded
on that exact merge checkout; native coverage remains pending on the corrected
workflow. SDK caches were already published after successful setup, so cache
publication and security coverage were retained. #301 applies the same bounded
repair to Bevy: a 240-minute job, 150-minute SDK setup and 60-minute native analysis
phase. Fresh completed scans must still qualify the new heads. Main's earlier
Tauri macro warnings also need review after its scan completes; the clean Bevy
extraction does not establish main coverage.

The audit also confirmed Unix session snapshots could be readable by other local
users under permissive default filesystem modes. The shared Rust storage policy
is merged into Bevy (#292), with a separate main PR (#291). Unix uses a per-user
registry with private directories and snapshots. Reads and writes validate
ownership and ancestry, and reject symbolic links or non-regular payload files.
Existing registries must already be private and are never made acceptable by
changing their permissions. Windows keeps its existing default discovery
location. No live registry was moved or modified during this work, and the
installed package was not replaced.

Both privacy branches passed their focused hosted Windows and Ubuntu checks:
fmt, strict all-target storage Clippy, four Windows or 15 Unix storage tests and
ten actual MCP inbox tests. Ubuntu also passed all 15 storage tests under `sudo`
on disposable fixtures. The Bevy results qualify `bac76ce3`, merged at
`34cf84e7`; the main results qualify `03a15973`. They do not replace the remaining
required native/package checks or external main review.

The turbine acceptance reader now expands actual 3MF component/build transforms
and repeated occurrences into world coordinates (#293). It retains unit, finite
vertex, index, positive-volume, closed-edge orientation and solved-placement
checks. Four pure regressions and recipe-target compilation passed on both the
Bevy child and the corresponding main feature fix (#257). The preceding main
head `f2a9f396` passed hosted Windows/Ubuntu acceptance and desktop packaging.
The October 4 follow-up integrates hierarchical 3MF and named print layouts
into the Bevy native host and controls, using the existing CAD hierarchy and
one saved-view model. It includes printer selection, diagnostics, whole-group
corrections, deliberate export, repeated instances, and source-edit guards.
The actual Windows Bevy walkthrough and Bambu Studio 2.8.2.61/OrcaSlicer 2.4.1
native export round trips passed locally. Fresh pinned printer fetching matches
the embedded catalog. The material catalog remains the existing unified surface.
See [Bevy print-layout integration](bevy-print-layout-integration.md) for usage,
qualification and limits. This branch implementation does not change previously
published packages; current hosted/package checks and independent review remain
required.

MCP aggregate CI jobs now run on shard failures and skip whole-run cancellation
(#295; standalone main PR #294). Their required names, success-only gates and
artifact provenance remain intact. Focused workflow contracts and actionlint
passed. Superseded owned runs were canceled after source-identity checks; checks
for the latest open PR heads were preserved. A separate aggregate failure came
from cached empty generated-demo directories. Registry-only aggregate caches now
exclude build targets (#296); fresh-directory reservation, artifact identity and
fail-closed validation remain unchanged. Four focused workflow contracts passed.

The material PR (#263) merged normally at `59195010`, retaining both dependencies
in its sole lockfile conflict and all feature work. It also repaired the new
Linux package privacy failure: live profiles and session fixtures use an owned
private `/tmp` directory, with diagnostics copied to `RUNNER_TEMP` on exit.
Unsafe shared-runner ancestry is not accepted. Focused Linux success/failure
fixtures accompany the change; current hosted package qualification is pending.

The Windows ARM package input guard correctly refused hosted Start/Search
occluders. The preflight now dismisses only identity-verified foreground or
observed shell windows on disposable hosted ARM runners (#297). The native input
guard remains unchanged. Eight account-window and 22 shell-window managed checks
passed; actual hosted ARM input qualification remains pending.

Jack's draft GPU-stock PR #268 has separate fixes for finite-flute eligibility,
deferred grid visibility during paused playback, and missing tool-change timing.
Multi-tool or ambiguous timelines now retain CPU stock; a default-false producer
flag and unanimous matching-layer proof limit GPU removal to known single-tool
timelines. The existing CPU simulation remains authoritative. Native compilation,
frontend typechecking, the focused path producer check and actual Rust/ECS
regressions passed. The new commits do not have fresh interactive GPU evidence.
This main draft still targets the earlier viewport and is not integrated into
the Bevy rc.2 application.

CodeQL alert [#147](https://github.com/limo-cad/Limo-CAD/security/code-scanning/147)
identified unsafe matrix-copy arithmetic in the packaged OpenCASCADE 7.9.3 header;
that SDK remains unpatched. Its size calculation can overflow or narrow before
`memmove`. A large-matrix application trigger has not been established. The
upstream 8.0.1 repair changes class layout and cannot be copied into the 7.9 SDK.
This finding is retained for an ABI-compatible SDK repair or a separately
reviewed SDK migration; it was not suppressed to clear CI.

## Deployment and preserved data

Codex/Cursor MCP settings use the installed Windows runtime above with
`--headless` and `LIMO_CAD_DESKTOP_BIN`. The Rust installer supports in-place
packaged runtimes and preserves Codex TOML comments (#258; main PR #262).
Start-menu, recipe URL, `.limo` file association, PATH and App Paths entries
select Bevy. Projects,
session inboxes, heartbeats and recovery snapshots survive runtime replacement.

The packaged MCP repair (#303) preserves absolute Windows UNC paths in every
client serializer. All 21 installer checks, formatting and strict all-target
xtask Clippy passed; the regression does not require a network share.

The October 4 Windows rebuild uses clean source `9b082687`. Candidate and
installed packages passed SDK-free headless and desktop MCP verification: ten
checks and 27 command steps, live-document binding, real geometry/export, Save,
retained unsaved work, disconnect survival and guarded shutdown. The checks used
private fixture documents. All five installed launch aliases have the same
executable checksum; older compatibility directories are junctions to that
runtime. A missed `.limo` association to a removed 0.1.0 download was repaired.
Three live designs were saved through MCP before the previous runtime closed.
Their recovery documents, session snapshots and deployment receipts are outside
Git under `D:/noBS-CAD-builds/bevy-prerelease-20261004`; earlier maintenance
receipts remain under `%LOCALAPPDATA%/nbcad/maintenance`.

Two audited purges remain outstanding: the 57 retired runtime binaries/DLLs in
`Roller-300/.local/cad-runtime-retired-20261003`, and
`C:/Users/jeffg/dev/noBS-CAD/target/debug/incremental`. Automatic approval review
rejected deletion with "blocked by policy", including the incremental-cache
request after explicit operator approval. No files were deleted in either purge.
The inactive cache was compressed on October 4; current build and temporary
outputs use D:. A subsequent purge of the October 4 retired installation and
inactive development executables was also rejected before execution. Those
copies remain; normal launch routes use the new installed runtime. Source
worktrees, CAD documents and live-session data are preserved.

Main PRs #308 and #309 were consolidated into #262 through merge `80e6bc3`,
preserving their original commits. They are closed as consolidated work; #262
still requires Jack's approval before main. No main merge was performed.

The [Rust agent board](agent-message-board.md) provides deployment notices through
NATS JetStream. Publishing a notice does not prove that every agent acknowledged
it, and the board does not replace the MCP document/session bridge.

## Release qualification still open

The public Windows x64 ZIP passed SDK-free headless and desktop MCP checks on
Thunder, both before and after installation. The Ubuntu DEB passed headless and
desktop MCP, X11 input/rendering and Wayland-desktop lifecycle/URI checks in the
[tagged package run](https://github.com/limo-cad/Limo-CAD/actions/runs/37232261112/job/111525874975).
The restored-window and Unicode-field X11 captures were reviewed. Checksums and
embedded metadata identify clean `9b082687` source; a machine-readable build
receipt accompanies the packages. The hosted Windows x64 build also passed
MCP and native input/render checks on this source; its restored-window capture
was reviewed. The public Windows ZIP remains the verified local rebuild.

Other preview targets remain withheld:

- **macOS:** compiled and Developer ID signed; Apple notarization returned
  HTTP 403 for a missing/expired team agreement. The account owner must resolve
  that agreement before notarized distribution. No Intel Mac package is qualified.
- **Windows ARM64:** compiled and passed headless checks; the owned input fixture
  refused a click through hosted-runner Start/Search windows. #297 repairs the
  hosted preflight; fresh ARM native-input qualification remains open.
- **AppImage:** preceding-source build/glibc/headless checks passed and X11 startup
  was reached; input stopped because the host lacked `xclip`/`xdotool`. #223 restores
  those prerequisites. The later shared-runner session-fixture privacy failure
  is repaired in #263. This does not establish current-source package success.

Historical source-specific checks also cover Windows UI Automation, drawings and
Unicode output, CAM, Scripts, mechanisms, preferences and lessons. They do not
establish current-head package/device qualification. Outstanding limits include:

- Remaining platform packages,
  required PR checks and external review. Stable `v0.2.2` is a separate legacy
  release; its presence cannot qualify the Bevy branch.
- Fresh Windows/macOS Japanese IME evidence for the latest field implementation,
  candidate-popup placement and physical monitor/DPI transitions.
- Physical printing, macOS/Linux OS print dialogs, screen-reader speech,
  actual 6DoF hardware/driver behavior and macOS OS GetURL delivery.
- Broader real-input annotation/joint/gesture workflows. The joint fixture's
  read-only settlement correction has not been rerun; Scripts chooser gestures
  and physical multiline-editor IME are not established by source-level checks.
- Switching sputter attribution. The observed Windows tab/sheet irregularity
  has no matched current-source reproduction or latency benchmark. The optional
  comparison uses native Bevy builds; see [measurement scope](native-switching-measurement.md).

A build, ignored check, stale-source pass or synthetic geometry assertion is not
a current-device runtime pass. Evidence and generated captures are retained
outside product source under `D:/noBS-CAD-builds/finish-bevy-rc2`.

## Browser work still open

The replacement must reuse the desktop Bevy UI. The current Rust WASM engine
facade builds and has focused binding checks; it is not a browser CAD app.
The complete Bevy WASM host, file/storage/dialog services and geometry-service
transport remain unfinished. The planned first browser host offloads geometry
to native Rust/OCCT. The extracted native-engine host is its service-side
foundation. An optional in-browser OCCT WASM backend is separate work; the
native-service approach does not require that port. See [web/README.md](../web/README.md).

## Audited deletions and history

The snapshot [`6394fb44`](https://github.com/limo-cad/Limo-CAD/commit/6394fb449f12e17dededd76dc702081ff7c277eb)
was previously mislabeled as an unfinished UI rewrite. Independently formatting
all 81 changed Rust files and their parent versions produced identical output:
it was formatting, not an upcoming feature. Its explicit revert removed no
functional implementation; the source remains reachable from
`feat/bevy-switch-timing`. The experimental accessibility tree at `8986fd77`
remains preserved; the production adapter supersedes its disconnected tree.
Recovered mechanism work remains implemented and documented in
[native-mechanism-drag.md](native-mechanism-drag.md).

Redundant integrated branches/worktrees and backup refs were retired only after
checking source representation and archiving unique history. Active work,
projects/session data and verified Git archives remain protected. Obsolete
September checkpoint prose and duplicated old release/validation narratives
are removed from this active status document; Git history retains them.

The October 4 comment cleanup at `ee7b07ce` changed 356 Rust files. Tokenizing
each file and its parent with Rust's `proc_macro2` produced identical token streams,
including documentation attributes. The source audit and all three workspace
formatting checks passed; this evidence does not qualify a new runtime package.
