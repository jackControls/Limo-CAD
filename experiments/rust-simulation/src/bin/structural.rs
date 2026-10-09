//! Isolated 3-D elasticity qualification spike: millimetres, newtons, MPa.
//! Material values are synthetic, not a calibrated printed-PETG constitutive law.
use anyhow::{ensure, Context, Result};
use faer::{linalg::solvers::Solve, sparse::SparseColMat, sparse::Triplet, Mat, Side};
use fenris::{
    assembly::{
        global::CsrAssembler,
        local::{ElementEllipticAssemblerBuilder, UniformQuadratureTable},
    },
    mesh::{procedural::create_rectangular_uniform_hex_mesh, Hex27Mesh},
    nalgebra::DVector,
    nalgebra_sparse::CsrMatrix,
    quadrature,
};
use fenris_solid::{
    materials::{LameParameters, LinearElasticMaterial, YoungPoisson},
    MaterialEllipticOperator,
};
use serde::Serialize;
use std::{path::PathBuf, time::Instant};

const LENGTH: f64 = 40.0;
const WIDTH: f64 = 6.0;
const HEIGHT: f64 = 2.0;
const YOUNG: f64 = 2000.0;
const POISSON: f64 = 0.35;
const FORCE: f64 = 0.1;

#[derive(Serialize)]
struct Sample {
    refinement: usize,
    elements: usize,
    nodes: usize,
    free_dofs: usize,
    nonzeros: usize,
    tip_area_mean_z_mm: f64,
    tip_node_mean_z_mm: f64,
    euler_bernoulli_relative_error: f64,
    free_equilibrium_relative_residual: f64,
    root_reaction_n: [f64; 3],
    root_reaction_moment_n_mm: [f64; 3],
    strain_energy_n_mm: f64,
    half_external_work_n_mm: f64,
    elapsed_seconds: f64,
}

#[derive(Serialize)]
struct MissingConstraintCheck {
    sparse_cholesky_rejected: bool,
    factorization_diagnostic: String,
    attempted_solution_relative_residual: Option<f64>,
    translation_nullspace_relative_stiffness: f64,
    net_unrestrained_load_n: f64,
    accepted_as_valid_analysis: bool,
}

fn sparse(
    matrix: &CsrMatrix<f64>,
    free: &[Option<usize>],
    dimension: usize,
) -> Result<SparseColMat<usize, f64>> {
    let entries = matrix
        .triplet_iter()
        .filter_map(|(row, col, &value)| Some(Triplet::new(free[row]?, free[col]?, value)))
        .collect::<Vec<_>>();
    SparseColMat::try_new_from_triplets(dimension, dimension, &entries)
        .context("convert Fenris CSR to faer CSC")
}

fn product(matrix: &CsrMatrix<f64>, vector: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; matrix.nrows()];
    for (row, col, value) in matrix.triplet_iter() {
        result[row] += value * vector[col];
    }
    result
}

fn norm(values: &[f64]) -> f64 {
    values.iter().map(|v| v * v).sum::<f64>().sqrt()
}

fn missing_constraint_check(
    matrix: &CsrMatrix<f64>,
    force: &[f64],
) -> Result<MissingConstraintCheck> {
    let identity = (0..matrix.nrows()).map(Some).collect::<Vec<_>>();
    let unconstrained = sparse(matrix, &identity, matrix.nrows())?;
    // A rigid Z translation is an exact continuum null mode. Floating-point
    // factorization alone can occasionally accept a numerically perturbed PSD
    // matrix; also check its compatibility with the nonzero resultant load.
    let translation = (0..matrix.nrows())
        .map(|i| if i % 3 == 2 { 1.0 } else { 0.0 })
        .collect::<Vec<_>>();
    let null_residual = product(matrix, &translation);
    let matrix_norm = matrix.values().iter().map(|v| v * v).sum::<f64>().sqrt();
    let relative_stiffness = norm(&null_residual) / (matrix_norm * norm(&translation));
    let net_load: f64 = force.iter().step_by(3).sum::<f64>();
    let net_load_z: f64 = force.iter().skip(2).step_by(3).sum();
    ensure!(net_load.abs() < 1e-12 && net_load_z > 0.0);
    ensure!(
        relative_stiffness < 1e-12,
        "rigid translation must remain a null mode"
    );
    let (rejected, diagnostic, residual) = match unconstrained.sp_cholesky(Side::Lower) {
        Err(error) => (true, error.to_string(), None),
        Ok(factor) => {
            let mut candidate = Mat::from_fn(force.len(), 1, |i, _| force[i]);
            factor.solve_in_place(&mut candidate);
            let u = (0..force.len())
                .map(|i| candidate[(i, 0)])
                .collect::<Vec<_>>();
            let r = product(matrix, &u)
                .iter()
                .zip(force)
                .map(|(a, b)| a - b)
                .collect::<Vec<_>>();
            let residual = norm(&r) / norm(force);
            (false, "Numerical Cholesky accepted a perturbed semidefinite matrix; nullspace/load compatibility rejects the unsupported model".into(), residual.is_finite().then_some(residual))
        }
    };
    let null_mode_load_incompatible = relative_stiffness < 1e-12 && net_load_z.abs() > FORCE * 1e-8;
    let accepted =
        !rejected && !null_mode_load_incompatible && residual.is_some_and(|value| value < 1e-7);
    ensure!(
        !accepted,
        "unsupported loaded model must fail qualification"
    );
    Ok(MissingConstraintCheck {
        sparse_cholesky_rejected: rejected,
        factorization_diagnostic: diagnostic,
        attempted_solution_relative_residual: residual,
        translation_nullspace_relative_stiffness: relative_stiffness,
        net_unrestrained_load_n: net_load_z,
        accepted_as_valid_analysis: accepted,
    })
}

fn run(
    refinement: usize,
    check_unsupported: bool,
) -> Result<(Sample, Option<MissingConstraintCheck>)> {
    let started = Instant::now();
    let mut linear = create_rectangular_uniform_hex_mesh(2.0_f64, 10, 3, 1, refinement);
    linear.transform_vertices(|point| point.x *= 2.0);
    let mesh = Hex27Mesh::from(&linear);
    let parameters = LameParameters::from(YoungPoisson {
        young: YOUNG,
        poisson: POISSON,
    });
    let rule = UniformQuadratureTable::from_quadrature_and_uniform_data(
        quadrature::tensor::hexahedron_gauss(3),
        parameters,
    );
    let material = LinearElasticMaterial;
    let operator = MaterialEllipticOperator::new(&material);
    let zero = DVector::zeros(3 * mesh.vertices().len());
    let local = ElementEllipticAssemblerBuilder::new()
        .with_finite_element_space(&mesh)
        .with_operator(&operator)
        .with_quadrature_table(&rule)
        .with_u(&zero)
        .build();
    let stiffness = CsrAssembler::default()
        .assemble(&local)
        .map_err(|error| anyhow::anyhow!("Fenris elasticity assembly: {error}"))?;
    ensure!(stiffness.values().iter().all(|v| v.is_finite()));
    let vertices = mesh.vertices();
    let mut force = vec![0.0; zero.len()];
    let mut tips = Vec::new();
    // Exact consistent nodal load for uniform traction on each quadratic
    // rectangular tip face: tensor-product composite Simpson weights.
    let simpson = |index: usize, end: usize| {
        if index == 0 || index == end {
            1.0
        } else if index % 2 == 1 {
            4.0
        } else {
            2.0
        }
    };
    for (node, point) in vertices.iter().enumerate() {
        if (point.x - LENGTH).abs() < 1e-10 {
            let iy = (point.y * refinement as f64).round() as usize;
            let iz = (point.z * refinement as f64).round() as usize;
            let area = simpson(iy, 6 * refinement) * simpson(iz, 2 * refinement)
                / (9.0 * (refinement * refinement) as f64);
            force[3 * node + 2] = FORCE * area / (WIDTH * HEIGHT);
            tips.push(node);
        }
    }
    ensure!(
        (force.iter().sum::<f64>() - FORCE).abs() < 1e-12,
        "tip traction must integrate to total force"
    );
    let unsupported = check_unsupported
        .then(|| missing_constraint_check(&stiffness, &force))
        .transpose()?;
    let mut count = 0;
    let free = vertices
        .iter()
        .flat_map(|p| {
            let fixed = p.x.abs() < 1e-10;
            (0..3).map(move |_| fixed)
        })
        .map(|fixed| {
            if fixed {
                None
            } else {
                let index = count;
                count += 1;
                Some(index)
            }
        })
        .collect::<Vec<_>>();
    let reduced = sparse(&stiffness, &free, count)?;
    let mut solution = Mat::zeros(count, 1);
    for (index, mapped) in free.iter().enumerate() {
        if let Some(mapped) = mapped {
            solution[(*mapped, 0)] = force[index];
        }
    }
    reduced
        .sp_cholesky(Side::Lower)
        .context("supported elastic model must be positive definite")?
        .solve_in_place(&mut solution);
    let displacement = free
        .iter()
        .map(|mapped| mapped.map_or(0.0, |i| solution[(i, 0)]))
        .collect::<Vec<_>>();
    ensure!(
        displacement.iter().all(|v| v.is_finite()),
        "non-finite elastic displacement"
    );
    let internal = product(&stiffness, &displacement);
    let residual = internal
        .iter()
        .zip(&force)
        .map(|(a, b)| a - b)
        .collect::<Vec<_>>();
    let free_residual = residual
        .iter()
        .zip(&free)
        .filter_map(|(r, i)| i.map(|_| *r))
        .collect::<Vec<_>>();
    let relative_residual = norm(&free_residual) / norm(&force);
    ensure!(
        relative_residual < 1e-7,
        "free equilibrium failed: {relative_residual}"
    );
    let mut reaction = [0.0; 3];
    let mut moment = [0.0; 3];
    for (node, point) in vertices
        .iter()
        .enumerate()
        .filter(|(_, p)| p.x.abs() < 1e-10)
    {
        let r = &residual[3 * node..3 * node + 3];
        for axis in 0..3 {
            reaction[axis] += r[axis];
        }
        moment[0] += point.y * r[2] - point.z * r[1];
        moment[1] += point.z * r[0] - point.x * r[2];
        moment[2] += point.x * r[1] - point.y * r[0];
    }
    ensure!(
        (reaction[2] + FORCE).abs() < 1e-8 && reaction[0].abs() < 1e-8 && reaction[1].abs() < 1e-8,
        "root force balance failed"
    );
    ensure!(
        (moment[1] - FORCE * LENGTH).abs() < 1e-7,
        "root bending moment balance failed"
    );
    ensure!(
        (moment[0] + FORCE * WIDTH / 2.0).abs() < 1e-7 && moment[2].abs() < 1e-7,
        "root transverse moment balance failed"
    );
    let energy = 0.5
        * displacement
            .iter()
            .zip(&internal)
            .map(|(u, f)| u * f)
            .sum::<f64>();
    let half_work = 0.5
        * displacement
            .iter()
            .zip(&force)
            .map(|(u, f)| u * f)
            .sum::<f64>();
    ensure!(
        energy.is_finite() && energy > 0.0 && (energy - half_work).abs() / energy < 1e-7,
        "elastic energy/work balance failed"
    );
    let area_mean = 2.0 * half_work / FORCE;
    let reference = FORCE * LENGTH.powi(3) / (3.0 * YOUNG * WIDTH * HEIGHT.powi(3) / 12.0);
    let sample = Sample {
        refinement,
        elements: mesh.connectivity().len(),
        nodes: vertices.len(),
        free_dofs: count,
        nonzeros: stiffness.nnz(),
        tip_area_mean_z_mm: area_mean,
        tip_node_mean_z_mm: tips.iter().map(|i| displacement[3 * i + 2]).sum::<f64>()
            / tips.len() as f64,
        euler_bernoulli_relative_error: area_mean / reference - 1.0,
        free_equilibrium_relative_residual: relative_residual,
        root_reaction_n: reaction,
        root_reaction_moment_n_mm: moment,
        strain_energy_n_mm: energy,
        half_external_work_n_mm: half_work,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    };
    Ok((sample, unsupported))
}

fn main() -> Result<()> {
    let mut samples = Vec::new();
    let mut unsupported = None;
    for refinement in [1, 2, 3] {
        let (sample, check) = run(refinement, refinement == 1)?;
        unsupported = unsupported.or(check);
        samples.push(sample);
    }
    let last_change = (samples[2].tip_area_mean_z_mm / samples[1].tip_area_mean_z_mm - 1.0).abs();
    ensure!(
        last_change < 0.005,
        "tip-displacement refinement change exceeds experimental 0.5% gate"
    );
    ensure!(
        samples[2].euler_bernoulli_relative_error.abs() < 0.03,
        "3D tip displacement exceeds experimental 3% beam-reference screen"
    );
    let report = serde_json::json!({
        "schema": 1, "experiment": "Fenris quadratic Hex27 + faer sparse Cholesky",
        "versions": {"fenris": "0.0.33", "fenris_solid": "0.0.33", "faer": "0.24.4"},
        "units": {"length": "mm", "force": "N", "modulus": "MPa", "energy": "N mm"},
        "material": {"young_mpa": YOUNG, "poisson": POISSON, "status": "synthetic isotropic benchmark, not printed PETG calibration"},
        "geometry_mm": [LENGTH, WIDTH, HEIGHT], "load_n": FORCE,
        "boundary_conditions": "all root-face translations fixed; uniform consistent quadratic tip traction in +Z",
        "euler_bernoulli_tip_mm": FORCE * LENGTH.powi(3) / (3.0 * YOUNG * WIDTH * HEIGHT.powi(3) / 12.0),
        "samples": samples, "last_refinement_relative_change": last_change,
        "qualification_gates": {"last_refinement_relative_change_limit": 0.005, "beam_reference_relative_error_screen": 0.03},
        "timing_scope": "elapsed_seconds includes mesh generation, stiffness assembly, consistency checks, sparse conversion/factorization and solve",
        "missing_constraint_check": unsupported,
        "scope": "3D linear elasticity library/solver qualification. Beam theory comparison includes differing 3D clamped-end and shear behavior. No snap/contact, anisotropy, plasticity, creep, or actual clip certification."
    });
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("structural-results.json"));
    std::fs::write(&output, serde_json::to_string_pretty(&report)? + "\n")
        .with_context(|| format!("write {}", output.display()))?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
