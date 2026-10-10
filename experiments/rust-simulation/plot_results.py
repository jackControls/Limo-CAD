"""Optional publication plot from tracked JSON; requires Matplotlib only."""
import json
from pathlib import Path
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

root = Path(__file__).resolve().parent
tet = json.loads((root / "same-mesh-results.json").read_text(encoding="utf-8"))
hex_report = json.loads((root / "structural-results.json").read_text(encoding="utf-8"))
rapier = tet["rapier_comparison"]["source_sample"]["mean_tip_displacement_mm"]
plt.rcParams.update({"font.size": 10, "svg.fonttype": "none"})
fig, axes = plt.subplots(1, 2, figsize=(11, 4.7), layout="constrained")
left, right = axes
left.bar(["Fenris static\nexact coarse Tet4", "Rapier settled\nsame coarse Tet4", "Fenris static\nrefined Hex27"],
         [tet["samples"][0]["tip_area_mean_z_mm"], rapier, hex_report["samples"][-1]["tip_area_mean_z_mm"]],
         color=["#245f87", "#245f87", "#868e96"])
left.axhline(tet["euler_bernoulli_tip_mm"], color="#444444", linestyle="--", label="Euler-Bernoulli screen")
left.set_ylabel("Area-mean tip displacement (mm)")
left.set_title("Solver agreement does not remove coarse-mesh stiffness")
left.set_ylim(0, .3)
left.legend(frameon=False, loc="upper left")
for i, value in enumerate([tet["samples"][0]["tip_area_mean_z_mm"], rapier, hex_report["samples"][-1]["tip_area_mean_z_mm"]]):
    left.text(i, value + .006, f"{value:.6f}", ha="center")
joint = [tet["samples"][i] for i in [0, 3, 4]]
right.plot([s["nodes"] for s in joint], [s["tip_area_mean_z_mm"] for s in joint], "o-", color="#245f87", label="Tet4 joint refinement")
right.plot([s["nodes"] for s in hex_report["samples"]], [s["tip_area_mean_z_mm"] for s in hex_report["samples"]], "s-", color="#868e96", label="Hex27 refinement")
right.axhline(tet["euler_bernoulli_tip_mm"], color="#444444", linestyle="--", label="Euler-Bernoulli screen")
right.set_xscale("log")
right.set_xlabel("Mesh nodes (log scale)")
right.set_ylabel("Area-mean tip displacement (mm)")
right.set_title("Full 3D Tet4 refinement remains incomplete")
right.set_ylim(.11, .3)
right.legend(frameon=False, loc="lower right")
for axis in axes:
    axis.spines[["top", "right"]].set_visible(False)
    axis.grid(axis="y", alpha=.2)
    axis.set_axisbelow(True)
fig.suptitle("40 x 6 x 2 mm clamped beam | E=2000 MPa, nu=0.35 | 0.1 N consistent tip traction")
fig.text(.02, -.045, "Source: tracked Windows Rust experiment JSON, 2026-10-06. Synthetic isotropic material; no clip/contact qualification.", fontsize=9)
fig.savefig(root / "mesh-comparison.svg", bbox_inches="tight")
svg = root / "mesh-comparison.svg"
svg.write_text("\n".join(line.rstrip() for line in svg.read_text(encoding="utf-8").splitlines()) + "\n", encoding="utf-8")
fig.savefig(root / "mesh-comparison.png", dpi=160, bbox_inches="tight")
plt.close(fig)
