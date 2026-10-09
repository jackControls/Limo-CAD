# Historical reference-solver experiment

Exact archival evidence from 2026-10-06, retained alongside the Rust experiment
rather than discarded when the preferred implementation changed. These scripts
are research fixtures, not application dependencies or a proposed Python backend.
`archive-manifest.json` records SHA-256 for each original file.

`benchmark.py` generated a synthetic 40 x 6 x 2 mm OCCT box with Gmsh 4.15.2,
exported/reimported STEP, identified root/tip physical groups, produced Tet10
volume meshes, explicitly mapped midside-node ordering to C3D10, and ran upstream
CalculiX 2.23. `results.json`, solver logs, input decks (`.inp`) and node/reaction
tables (`.dat`) are the original input/output, retained to audit the load and parser.
`beam.step` is the synthetic input, with its recorded hash.

The fixture is Windows-specific. Install Gmsh 4.15.2 in an isolated Python
environment; extract the [upstream CalculiX archive](https://www.dhondt.de/calculix_2.23_4win.zip)
to `calculix/calculix_2.23_4win/ccx_static.exe` beside the script. Run the script
from a **copy** of this directory: it writes STEP, meshes, solver inputs/logs and
the result report. The recorded solver SHA-256 is in `results.json`; no executable
or third-party package archive is redistributed here. The original run used
Python 3.14.5, one solver thread and a 120-second subprocess timeout.

Synthetic isotropic E=2000 MPa, nu=0.35; mm/N/MPa; root translations fixed;
total transverse tip force 0.1 N. **The load was divided equally among tip nodes**,
and the reported displacement is their arithmetic mean. That differs from the
consistent uniform traction/area mean in the Rust benchmark, so this is useful
independent reference evidence, not an exact apples-to-apples validation.

At mesh sizes 1.5/1.0/0.7 mm the reported mean tips were
0.2622949735/0.2626679740/0.2630425555 mm. The final change was about 0.143%; root
reaction 0.09999998445 N; minimum signed inverse condition number 0.29568 on the
finest beam. The one-dimensional beam screen is 0.2666666667 mm. No stress
convergence, energy balance, snap contact or manufactured material qualification
was established. Old external-solver recommendations are historical; the current
decision remains [the Rust simulation plan](../../docs/SIMULATION-EXPERIMENT.md).

Source interpretation: [Gmsh manual](https://gmsh.info/doc/texinfo/),
[CalculiX official site](https://www.dhondt.de/).
