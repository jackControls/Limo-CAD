//! Binary STL writer (geometry only; millimetres assumed).

use crate::{ExportError, TriangleMesh};

pub fn write_stl(meshes: &[TriangleMesh]) -> Result<Vec<u8>, ExportError> {
    if meshes.is_empty() {
        return Err(ExportError("There are no active bodies to export.".into()));
    }
    let triangle_count: u32 = meshes.iter().map(|mesh| mesh.triangle_count() as u32).sum();
    let mut out = Vec::with_capacity(84 + triangle_count as usize * 50);
    let mut header = [0u8; 80];
    let label = b"Limo CAD binary STL (millimetres)";
    header[..label.len()].copy_from_slice(label);
    out.extend_from_slice(&header);
    out.extend_from_slice(&triangle_count.to_le_bytes());

    for mesh in meshes {
        if mesh.positions.len() % 3 != 0 {
            return Err(ExportError(format!(
                "body {} has a malformed position buffer",
                mesh.body_id.0
            )));
        }
        if mesh.indices.len() % 3 != 0 {
            return Err(ExportError(format!(
                "body {} has a malformed index buffer",
                mesh.body_id.0
            )));
        }
        for (triangle_index, tri) in mesh.indices.as_chunks::<3>().0.iter().enumerate() {
            let (a, b, c) = (
                vertex(&mesh.positions, tri[0])?,
                vertex(&mesh.positions, tri[1])?,
                vertex(&mesh.positions, tri[2])?,
            );
            let normal = triangle_normal(a, b, c).map_err(|error| {
                ExportError(format!(
                    "body {} triangle {}: {}",
                    mesh.body_id.0, triangle_index, error.0
                ))
            })?;
            out.extend_from_slice(&normal[0].to_le_bytes());
            out.extend_from_slice(&normal[1].to_le_bytes());
            out.extend_from_slice(&normal[2].to_le_bytes());
            for point in [a, b, c] {
                out.extend_from_slice(&point[0].to_le_bytes());
                out.extend_from_slice(&point[1].to_le_bytes());
                out.extend_from_slice(&point[2].to_le_bytes());
            }
            out.extend_from_slice(&0u16.to_le_bytes());
        }
    }
    Ok(out)
}

fn vertex(positions: &[f32], index: u32) -> Result<[f32; 3], ExportError> {
    let start = index as usize * 3;
    if start + 2 >= positions.len() {
        return Err(ExportError("triangle index out of range".into()));
    }
    Ok([positions[start], positions[start + 1], positions[start + 2]])
}

fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Result<[f32; 3], ExportError> {
    // Compute from the actual serialized vertices in double precision. An
    // absolute f32 epsilon cutoff incorrectly rejects small valid facets.
    let a = a.map(f64::from);
    let b = b.map(f64::from);
    let c = c.map(f64::from);
    if a.iter().chain(&b).chain(&c).any(|value| !value.is_finite()) {
        return Err(ExportError("STL vertex coordinates are non-finite".into()));
    }
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if !len.is_finite() || len == 0.0 {
        return Err(ExportError(
            "STL vertices form a zero-area or non-finite triangle".into(),
        ));
    }
    Ok([
        (n[0] / len) as f32,
        (n[1] / len) as f32,
        (n[2] / len) as f32,
    ])
}
