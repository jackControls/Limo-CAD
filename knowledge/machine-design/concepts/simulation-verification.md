---
type: Concept
title: Structural simulation evidence and verification
description: Separate CAD contact geometry, solver agreement, mesh convergence and physical qualification; preserve reproducible evidence before trusting a result.
status: draft
updated: 2026-10-07
topics: simulation, structural-analysis, validation, materials
keywords: FEM, FEA, Fenris, Rapier, Tet4, Hex27, mesh convergence, solver agreement, stationarity, contact, preload, equilibrium, reactions, strain energy
related_recipes: []
sources: session-simulation, fenris, rapier, occt-docs
---

# Structural simulation evidence and verification

**Solver agreement, mesh convergence and physical validation are different
checks.** A completed run or attractive deformation image establishes none of
them by itself. These are lessons from the project's reproducible Rust
experiments; structural analysis remains experimental, with no certified
material allowables or assembly load rating.

## Geometry establishes the problem

Use retained OCCT solids for sections, gaps, overlap and contact-surface
identification. A viewport clipping plane is a visual operation; a kernel
section provides geometric edges. Neither determines elastic contact force.
Record the contact normal, insertion direction, intended flex direction and
support faces before defining loads. See
[assembly interference](../../concepts/assembly-interference.md) and
[AM snap-fits](am-snap-fit.md).

OCCT's [release documentation](https://github.com/Open-Cascade-SAS/OCCT/blob/a016080bf6738d6aeae020badee4e888ad1540a5/dox/introduction/introduction.md)
is the primary geometry reference. It does not supply a structural solver for
these experiments.

## Compare like problems

Before comparing solvers, match units, material law, geometry, mesh topology,
element order, supports, load integration and the displacement statistic.
Uniform surface traction and equal force per node can produce different load
distributions. An area-weighted mean displacement is not generally an arithmetic
nodal mean.

The project's settled low-load beam compared Fenris static elasticity with
Rapier FEM on the **same coarse Tet4 mesh**: the area-mean tip displacements
differed by about **0.001114%**. Yet the independent Tet4 refinement sweep remained
about **6.23%** below the refined Hex27 reference. Agreement on a coarse mesh can
confirm implementation consistency while sharing discretization error.
For the preserved Windows case, the beam is 40 x 6 x 2 mm with E = 2000 MPa,
nu = 0.35 and a 0.1 N tip load. The common coarse grid is 21 x 4 x 3 nodes.
The agreement percentage uses Fenris's area-mean displacement as denominator;
the 6.23% difference uses the finest tested Hex27 result, not an exact solution.
These are selected benchmark observations, not general solver error bounds.
The [Tet4 report](https://github.com/limo-cad/Limo-CAD/blob/0152268695e069c86020bc3694c3d79b4856bb37/experiments/rust-simulation/same-mesh-results.json)
and [Hex27 report](https://github.com/limo-cad/Limo-CAD/blob/0152268695e069c86020bc3694c3d79b4856bb37/experiments/rust-simulation/structural-results.json)
preserve the numbers. Methods, inputs, failures and results are preserved in the
[project research record](https://github.com/limo-cad/Limo-CAD/blob/0152268695e069c86020bc3694c3d79b4856bb37/docs/analysis/RESEARCH-INDEX.md).
These beam results are not validation of a snap-fit assembly.

## Required evidence

1. **Reproducibility:** CAD revision/hash, solver and dependency versions,
   coordinate system, units, material source/revision, supports and load patches.
2. **Mesh validity:** Jacobians/element orientation and quality, including after
   curved or higher-order elements are constructed. A surface preview is
   insufficient.
3. **Numerical checks:** free-DOF residual, root reaction force and moment,
   energy/work consistency, and convergence of the actual quantity being used.
   A configured tolerance is not proof that an iteration cap achieved it.
4. **Transient checks:** independently measure velocity and displacement over
   a settling window before comparing a transient result with a static solve.
   Preserve failed timestep/iteration cases alongside successful ones.
5. **Physical checks:** realistic unilateral contact/friction, print-direction
   properties, tolerance/relaxation, and measured coupons or assembly tests.
   Bilateral floor constraints are a support assumption, not a bound on real
   mating contact.

For multiple flex regions, independent load cases and a work-conjugate
compliance matrix can test coupling, reciprocity and positivity. Do not assume
independent springs from their visual separation.

## Library references and reuse

[Fenris](https://github.com/InteractiveComputerGraphics/fenris) supplies the
static elasticity implementation used here;
[Rapier's soft-body guide](https://rapier.rs/docs/user_guides/rust/soft_bodies/)
describes the world/FEM API. Capabilities and package versions belong to the
experiment record, not a claim that they are integrated into CAD Help or the
production application.

This page is original project prose. Upstream code and documentation retain
their own licenses; copying requires the applicable notices. See
[source inventory](../SOURCES.md). Detailed numerical evidence lives in the
repository; the linked files are not themselves embedded Help resources.
