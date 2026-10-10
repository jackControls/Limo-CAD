# Session knowledge preservation and source reuse audit

Reviewed 2026-10-06; challenged and corrected 2026-10-07 for the CAD application
and the INJS2065 design research. The second pass was a direct adversarial review;
no specialized Bugbot or security runner is claimed.
Decisions below concern specific artifacts, not every item hosted by an author,
repository or government agency. **Copy** means reproduce the original content;
it is distinct from a link or an independently written factual explanation.

## Finding and preservation work

The research was substantially saved, but was not adequately discoverable from
CAD Help. The public [research index](RESEARCH-INDEX.md), simulation plan, original
Rust programs/results and historical reference experiment preserve methods,
failures and provenance. The private repository preserves the part mechanics,
slicer studies, CAD sections and evidence manifests at the revision linked by
that index. Detailed part geometry and results remain private.

`cad_help` searches Concept pages embedded from `knowledge/**`; saving an analysis
under `docs/` or `experiments/` does not make its body searchable there. This audit
adds [simulation verification](../../knowledge/machine-design/concepts/simulation-verification.md)
and extends the existing snap-fit, thin-wall and materials Concepts. The source
registry now links the primary references and marks their reuse scope. A rebuilt
Help catalog includes these pages; this does not claim the installed CAD/MCP
binary has already been rebuilt or deployed.

The session additions are project explanations and derivations; no vendor
passages, paper figures or material tables were added by these changes. No CAD
geometry, solver model or slicing settings changed during this audit.

## Most valuable knowledge, in priority order

1. **Reproducible solver counterexample.** Same-mesh Fenris/Rapier agreement of
   about 0.001114% coexists with a Tet4 refinement result still about 6.23% below
   the refined Hex27 reference. This is a particularly useful original fixture:
   it demonstrates why implementation agreement is not continuum accuracy.
   Preserve source, exact inputs, both successful and failed cases, report hashes
   and the original load-integration distinctions. Copy the original project
   explanation into Help and keep executable evidence in the experiment.
2. **Installed detent reasoning.** Contact normal, insertion direction and the
   intended flex direction must agree. Exact overlap does not establish elastic
   preload. The scalar tolerance/permanent-set travel window and the S-curve
   energy/force reasoning are reusable project derivations; neither is measured
   contact force or a service-life qualification. These belong on the existing
   snap-fit page, with original references and explicit assumptions.
3. **Print geometry versus appearance.** Bending thickness is not necessarily
   the narrow plan-view wall width; a circular cutter does not ensure tangent
   junctions; in-plane nub detail must survive actual wall paths, seams and gap
   fill rather than a layer-height rule. General lessons belong in Help. Private
   screenshots/toolpaths remain with the part and its source revision.
4. **Material evidence and imposed conditions.** Grade/formulation and datasheet
   revision matter. Prescribed-displacement and prescribed-force E scaling have
   different implications; printed anisotropy and relaxation require additional
   evidence. Keep the revision ledger and original calculations; reference the
   manufacturer's sheet instead of silently replacing historical values.
5. **Numerical acceptance and OCCT boundaries.** Preserve mesh Jacobian gates,
   reaction/moment/energy checks, stationarity checks, compliance-matrix methods,
   typed empty/error section outcomes and snapshot/job boundaries. Separate a
   bounded successful run from engineering qualification. Generic methods are
   valuable project knowledge; the architecture and detailed review belong in
   the existing plan/issues rather than duplicate Help chapters.

These are uniquely useful *project evidence and implementation lessons*, not
claims that the underlying mechanics theories were invented in this session.
The numerical values are fixture-specific and must not become clip load ratings.

## Copying evidence and candidates

**Confirmed license evidence** means a grant was inspected at the stated scope.
**Candidate** means copying a particular item still needs its coverage/notices
checked. Neither means third-party figures or every file on the same site have
been cleared. [Inspection receipts](SOURCE-REUSE-EVIDENCE.json) preserve source
URLs and hashes; the source bodies are not bundled into the repository.

- **Original public project research and diagrams:** copy into the knowledge
  base under the project's [LGPL-2.1-or-later license](https://github.com/limo-cad/Limo-CAD/blob/0152268695e069c86020bc3694c3d79b4856bb37/LICENSE),
  keeping producer, revision, inputs and evidence links. This covers our authored
  scripts, explanations and synthetic-fixture outputs; it does not sublicense
  externally authored inputs or private part files. Prefer original diagrams
  generated from project measurements over copying a vendor's figure.
- **OCCT release documentation and examples — release-level evidence, candidate:** the
  [7.9.3 documentation license section](https://github.com/Open-Cascade-SAS/OCCT/blob/a016080bf6738d6aeae020badee4e888ad1540a5/dox/introduction/introduction.md)
  identifies documentation copyright and the release's LGPL-2.1 terms with the
  [OCCT exception](https://github.com/Open-Cascade-SAS/OCCT/blob/a016080bf6738d6aeae020badee4e888ad1540a5/OCCT_LGPL_EXCEPTION.txt).
  This is evidence for licensed release reuse, not an explicit license
  determination for every standalone document or figure. High-value candidates
  are selected section/clipping, geometric continuity and
  shape-analysis explanations or examples. Preserve copyright, both license
  texts, modification identification and applicable source obligations. Check
  the selected file's coverage and separately credited assets before copying.
  No permission for unrelated commercial website pages is inferred.
- **Rapier repository documentation/code — repository-level evidence, candidate:** the pinned
  [soft-body documentation source](https://github.com/dimforge/rapier/blob/3406750a38286a0d3c7d0f4be2f9d53153e81d90/website/docs/user_guides/templates/soft_bodies.mdx)
  is in the current main repository with the root
  [Apache-2.0 license](https://github.com/dimforge/rapier/blob/3406750a38286a0d3c7d0f4be2f9d53153e81d90/LICENSE).
  No separate guide-specific license was found; coverage is inferred
  from the repository context. The guide imports other files, so a copied guide
  also needs those included components and media checked. Selected API examples
  are useful for the Rust experiment. Keep the license,
  relevant notices and change markers; check separately credited media. The old
  `rapier.rs` repository was archived after the website moved into `rapier`.
  A separately hosted Dimforge blog post is not automatically covered by this
  repository license.
- **Fenris code and associated documentation — explicit license scope confirmed:**
  [MIT license at the inspected revision](https://github.com/InteractiveComputerGraphics/fenris/blob/7181b15684ad1eb90d95cc75f1566a8259a0241d/LICENSE-MIT),
  with Apache-2.0 also offered. MIT copying requires preserving its copyright and
  permission notice. Element/load assembly examples have concrete value; bulk
  API duplication adds less value than pinned links plus our own verified fixture.
  The experiment's exact package versions remain pinned in its Cargo files;
  repository inspection revisions are not substitutes for those package pins.
- **faer 0.24.4 — declaration confirmed, copying candidate pending notice:**
  the [published package](https://crates.io/crates/faer/0.24.4) declares MIT and
  points to Codeberg. The cached registry archive matches its Cargo.lock checksum,
  but contains no LICENSE file; the upstream notice URL could not be verified.
  Obtain the selected source's actual copyright/permission notice before copying.
  Package metadata alone does not complete that attribution record. No right to
  unrelated website media is inferred.
- **Existing CC BY teaching sources — page/book license confirmed, exceptions remain:**
  [Bryan Guns' NWTC DFM chapter](https://eng.libretexts.org/Courses/Northeast_Wisconsin_Technical_College/Design_for_Various_Manufacturing_Methods/01%3A_Design_for_Manufacturing_%28DFM%29)
  declares CC BY 4.0 in its footer;
  [Gagnon/Bearman's PALNI DFMA book](https://pressbooks.palni.org/designmanufactureassembly/)
  declares CC BY 4.0 except where noted. Selected paragraphs/figures can be copied
  commercially with author/title/source/license attribution and identified
  changes, after checking item-level exceptions. Keep the CC BY attribution
  rather than representing the source as newly LGPL-authored. These are useful
  foundations but add less unique value than the new project evidence.

**Recommended actual copying order:** project-derived lessons and diagrams first;
small OCCT examples tied to a real feature second; selected Rust solver examples
third. There is no present need to import entire textbooks or manuals. The Help
promotion in this change completes the first prose step; upstream examples and
figures listed above are candidates, not claims that they were already copied.

## Valuable references to keep as links

- **Yoshida and Wada, Mechanics of a snap-fit, v2 (2020):**
  [stable paper version](https://arxiv.org/abs/2003.13566v2) and
  [journal DOI](https://doi.org/10.1103/PhysRevLett.125.194301).
  Especially useful for geometry/friction/elasticity and insertion/removal
  asymmetry. Its selected
  [arXiv license](https://arxiv.org/licenses/nonexclusive-distrib/1.0/license.html)
  gives arXiv nonexclusive distribution rights; it is not CC BY or a general
  downstream copying grant. Link to the paper. No paper figure or substantial
  passage is bundled, and its shell/cylinder model is not our printed assembly.
- **Covestro snap-fit design guide:** the
  [guide URL recorded in the original material assumptions](https://solutions.covestro.com/-/media/covestro/solution-center/brands/downloads/imported/1557218421.pdf)
  is retained separately from later guide revisions. It is valuable for root
  compliance, assembly strain, recovery and friction. Covestro's
  [conditions of use](https://www.covestro.com/en/legal/conditions-of-use)
  limit the site's copying permission to personal, noncommercial, unmodified
  use with notices; that is not permission to bundle the guide in commercial
  application Help. No broader guide-specific grant was verified here.
- **BASF design tools:** [official engineering services](https://pmtools-na.basf.com/services.php)
  are useful to challenge simplistic short-cantilever assumptions. No specific
  copying grant for their documents, figures or implementations was verified.
  Reference the tool/method; do not import its tables or code on that basis.
- **Bambu PETG Basic datasheets:** keep the
  [manufacturer-hosted sheet](https://store.bblcdn.com/s1/default/cb94589bf7994fdcbfa833badefae9cd/Bambu_PETG_Basic_Technical_Data_Sheet.pdf)
  and historical version distinctions. No open reproduction license was
  identified in the sheet, so this audit approves referencing, not copying the
  complete PDF, figures or tables into Help. Individually cited factual values
  in original calculations are different from copying a protected compilation;
  do not infer dataset redistribution rights from that distinction.
- **CATIA/SIMULIA, NX/Simcenter and SolidWorks:** keep official
  [SIMULIA](https://www.3ds.com/products/simulia/structural-simulation),
  [Simcenter 3D](https://www.siemens.com/en-gb/products/simcenter/mechanical-simulation/simcenter-3d/)
  and [SolidWorks Simulation](https://www.solidworks.com/product/solidworks-simulation)
  references for the capability/project plan. No open grant was identified for
  their product-page text, brochures or media. Our own feature comparison can
  be retained with citations; vendor illustrations and substantial prose stay
  outside the bundle.
- **DOE Build4Scale Module 3D:**
  [the actual deck](https://www.energy.gov/sites/default/files/2021-07/Module_3D.pdf)
  credits Wikipedia and a third-party casting source on numbered slides 9–12.
  [Government-work rules](https://www.usa.gov/government-works) do not make those
  materials public domain. Corrected the registry and existing DOE attribution
  labels to mixed rights/link-only. Copy only an individually verified item;
  government hosting alone is insufficient.

## Copyleft candidates and already copied data

[conspire.rs](https://github.com/mrbuche/conspire.rs/blob/4a3232fe378d3a3f3cd03e41698597a54edef197/LICENSE)
offers GPL-3.0 copying rights with its obligations. It is not a prohibited
experiment; retain GPL terms and choose a compatible destination/distribution
before copying code into a shipped component. Linking to research is useful now.
GPL/AGPL/ShareAlike obligations may extend to an adapted or combined work;
keeping a notice on one file does not by itself settle distribution compliance.

The [Gmsh manual](https://gmsh.info/doc/texinfo/gmsh.html) describes the software's
GPL-2.0-or-later copying conditions and exceptions. This review does not establish
a separate artifact-specific grant for a selected manual passage or figure.
Keep the manual as a reference until that scope is verified; neither a GNU FDL
documentation license nor software-license coverage of every figure is inferred.
CalculiX's solver code license likewise does not, by itself, establish rights
for every separately published manual or figure.

The application already preserves FreeCAD material-card and OrcaSlicer/Bambu
Studio profile provenance in its catalog, with CC BY and AGPL attribution
described by [third-party notices](../../THIRD_PARTY_NOTICES.md). This is existing
copied/adapted data, not a new session ingestion gap. Keep the per-artifact author,
commit/path/hash/license records and applicable distribution obligations. An
open-source slicer profile license is not a license for Bambu's separate PDF
datasheets. This audit did not requalify every existing catalog card/license.

## Remaining work and source-record requirements

- If an upstream passage/figure/code file is actually copied, save its exact
  source revision and SHA-256, author/copyright, artifact license and exceptions,
  acquisition date, local destination, license/NOTICE files and identified edits.
  A citation is necessary provenance, not a replacement for license compliance.
- Existing NIST/NASA references remain useful, but this session-focused audit
  does not certify every older source or third-party item in the whole library.
- Preserve private-source visibility restrictions independently of copyright.
  General methods can be explained publicly; ownership of project prose does
  not automatically authorize publishing private design artifacts.
- Source websites and terms may change. The pinned repository links above are
  inspection evidence; vendor/no-license findings mean **no grant verified**,
  not a declaration that copying can never be licensed.

Full methods and evidence remain in the [research index](RESEARCH-INDEX.md),
and source IDs/rights summaries in the
[Help source registry](../../knowledge/machine-design/SOURCES.md).

## Adversarial findings resolved

- **Engineering scope:** positive installed travel was too easy to read as
  adequate retention. The Help screen now requires separate minimum holding-force
  and peak insertion/removal travel/strain checks, and explicitly assumes a
  frictionless constant-stiffness spring for the S-curve formula. Ambiguous
  bend-versus-layer wording was replaced with tensile stress and layer-plane
  guidance.
- **License evidence:** repository/declaration-level findings were presented
  too close to clearance of individual copied artifacts. Scope and pending
  checks now distinguish these cases. The unverified faer notice is explicit;
  Gmsh's software terms are no longer treated as a manual/figure determination.
- **Traceability and policy:** benchmark percentages now link directly to the
  preserved result files with units, load, grid and denominators. Source receipts
  preserve inspected-byte hashes. The policy no longer simultaneously forbids
  pasting and permits licensed copying, or places the mixed-rights DOE deck in
  the copying-source list. Its stale branch-specific maintenance note is removed.

The common-mesh difference was recomputed as 0.0011136923%; the finest-tested
Tet4 displacement is 6.2347375% below the finest-tested Hex27 value. Hex27 is a
refined comparison, not an exact continuum solution. Neither percentage is a
general accuracy bound for either solver.

## Validation

The repository knowledge structure/source/link check and regenerated search
index freshness check passed. All 67 existing Help tests passed. Fresh embedded
Help searches ranked the intended page first for each of: FEM mesh convergence/
solver agreement, detent installed preload/S-curve, datasheet revision/prescribed
displacement, and relief cutter/tangent/wall paths. These checks establish corpus
discoverability; they do not independently certify the legal conclusions or
physical model.
