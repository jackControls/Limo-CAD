//! Source/effective-setting summaries extend the existing layout preflight.
use crate::{slicer_verification::sha256, ExportError, TriangleMesh};
use nbcad_assembly::{AssemblySolutionDto, ComponentStructureDto};
use nbcad_core::{
    BodyAppearance, BodyId, PrintIntentDocumentDto, PrintIntentEffectiveReportDto,
    PrintSettingSourcesDto, PrintSettingsDto,
};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct ManufacturingObjectReport {
    pub source_document_id: Option<String>,
    pub body_id: BodyId,
    pub occurrence_id: u64,
    pub group_occurrence_id: u64,
    pub name: String,
    pub geometry_sha256: String,
    pub resolved_export_sha256: String,
    pub appearance: BodyAppearance,
    pub appearance_source: &'static str,
    pub modifiers: Vec<nbcad_core::PrintModifierEffectiveDto>,
    pub translation: [f64; 3],
    pub rotation: [f64; 4],
    pub requested: PrintSettingsDto,
    pub inherited_defaults: PrintSettingsDto,
    pub effective: PrintSettingsDto,
    pub sources: PrintSettingSourcesDto,
    pub unsupported: Vec<nbcad_core::PrintSettingFieldDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManufacturingPreflightReport {
    pub export_mode: &'static str,
    pub objects: Vec<ManufacturingObjectReport>,
    pub selected_process: Option<nbcad_core::ProcessProfileSnapshotDto>,
    pub project_defaults: PrintSettingsDto,
    pub warnings: Vec<String>,
    pub evidence: serde_json::Value,
}

pub fn manufacturing_preflight_report(
    meshes: &[TriangleMesh],
    appearances: &[BodyAppearance],
    structure: &ComponentStructureDto,
    solution: &AssemblySolutionDto,
    intent: &PrintIntentDocumentDto,
    effective: &PrintIntentEffectiveReportDto,
) -> Result<ManufacturingPreflightReport, ExportError> {
    structure.validate().map_err(ExportError)?;
    if !solution.solved {
        return Err(ExportError(
            "Resolve the layout before generating its manufacturing summary".into(),
        ));
    }
    let parents: BTreeMap<_, _> = structure
        .occurrences
        .iter()
        .map(|o| (o.id.0, o.parent_occurrence_id.map(|id| id.0)))
        .collect();
    let source: BTreeMap<_, _> = meshes.iter().map(|mesh| (mesh.body_id, mesh)).collect();
    let mut objects = Vec::new();
    let inherited_defaults =
        nbcad_core::resolve_print_settings(intent, &PrintSettingsDto::default()).0;
    for pose in solution
        .instance_body_poses
        .iter()
        .filter(|pose| pose.visible)
    {
        let Some(mesh) = source.get(&pose.body_id) else {
            continue;
        };
        let settings = effective
            .parts
            .iter()
            .find(|part| part.body_id == pose.body_id)
            .ok_or_else(|| {
                ExportError(format!(
                    "Missing effective print summary for body {}",
                    pose.body_id.0
                ))
            })?;
        let mut group = pose.occurrence_id.0;
        while let Some(parent) = parents.get(&group).copied().flatten() {
            group = parent;
        }
        let authored_appearance = appearances.iter().find(|a| a.body_id == pose.body_id);
        let appearance = authored_appearance
            .cloned()
            .unwrap_or_else(|| BodyAppearance::default_for(pose.body_id));
        let mut geometry = Vec::with_capacity(mesh.positions.len() * 4 + mesh.indices.len() * 4);
        for position in &mesh.positions {
            geometry.extend_from_slice(&position.to_le_bytes());
        }
        for index in &mesh.indices {
            geometry.extend_from_slice(&index.to_le_bytes());
        }
        let geometry_sha256 = sha256(&geometry);
        let placement = serde_json::to_vec(&(pose.translation, pose.rotation))
            .map_err(|e| ExportError(e.to_string()))?;
        geometry.extend_from_slice(&placement);
        objects.push(ManufacturingObjectReport {
            source_document_id: intent.source_document_id.clone(),
            body_id: pose.body_id,
            occurrence_id: pose.occurrence_id.0,
            group_occurrence_id: group,
            name: mesh.name.clone(),
            geometry_sha256,
            resolved_export_sha256: sha256(&geometry),
            appearance,
            appearance_source: if authored_appearance.is_some() { "cad_authored" } else { "portable_default" },
            modifiers: effective.modifiers.iter().filter(|zone| zone.modifier.body_id == pose.body_id).cloned().collect(),
            translation: pose.translation,
            rotation: pose.rotation,
            requested: settings.requested.clone(),
            inherited_defaults: inherited_defaults.clone(),
            effective: settings.settings.clone(),
            sources: settings.sources.clone(),
            unsupported: settings.unsupported.clone(),
        });
    }
    objects.sort_by_key(|part| (part.group_occurrence_id, part.occurrence_id, part.body_id));
    Ok(ManufacturingPreflightReport {
        export_mode: "portable_model",
        objects,
        selected_process: effective.selected_process.clone(),
        project_defaults: effective.project_defaults.clone(),
        warnings: effective.warnings.clone(),
        evidence: serde_json::json!({
            "cad_layout_checked":true,"project_metadata_written_and_read_back":false,
            "installed_slicer_imported":false,"toolpaths_generated":false,"physical_fit_load_checks":"not_run",
            "note":"This preflight reports requested settings. Portable 3MF does not serialize process overrides; realized walls, fit and strength remain unverified."
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbcad_assembly::OccurrenceId;
    #[test]
    fn repeated_sources_keep_alignment_quantity_and_unsupported_intent() {
        let body = BodyId(1);
        let mesh = TriangleMesh {
            body_id: body,
            name: "source".into(),
            positions: vec![0., 0., 0., 10., 0., 0., 0., 10., 0.],
            indices: vec![0, 1, 2],
        };
        // The report accepts a resolver snapshot; the kernel owns exact geometry.
        let structure = ComponentStructureDto::default();
        let solution = AssemblySolutionDto {
            solved: true,
            instance_body_poses: vec![
                nbcad_assembly::InstanceBodyPoseDto {
                    occurrence_id: OccurrenceId(1),
                    component_id: nbcad_assembly::ComponentId(1),
                    body_id: body,
                    translation: [0.; 3],
                    rotation: [0., 0., 0., 1.],
                    visible: true,
                },
                nbcad_assembly::InstanceBodyPoseDto {
                    occurrence_id: OccurrenceId(2),
                    component_id: nbcad_assembly::ComponentId(1),
                    body_id: body,
                    translation: [40., 0., 0.],
                    rotation: [0., 0., 0., 1.],
                    visible: true,
                },
            ],
            ..Default::default()
        };
        let intent = PrintIntentDocumentDto {
            defaults: PrintSettingsDto {
                wall_count: Some(6),
                ..Default::default()
            },
            ..Default::default()
        };
        let (settings, sources) = nbcad_core::resolve_print_settings(&intent, &Default::default());
        let effective = PrintIntentEffectiveReportDto {
            selected_process: None,
            project_defaults: intent.defaults.clone(),
            parts: vec![nbcad_core::PartPrintIntentEffectiveDto {
                body_id: body,
                binding: nbcad_core::PrintPartBindingDto::Live,
                requested: Default::default(),
                settings,
                sources,
                unsupported: vec![nbcad_core::PrintSettingFieldDto::WallCount],
            }],
            orphan_body_ids: vec![],
            profile_status: None,
            warnings: vec![],
            capabilities: vec![],
            modifiers: vec![],
        };
        let report = manufacturing_preflight_report(
            &[mesh],
            &[],
            &structure,
            &solution,
            &intent,
            &effective,
        )
        .unwrap();
        assert_eq!(report.objects.len(), 2);
        assert_eq!(report.objects[0].appearance_source, "portable_default");
        assert_eq!(
            report.objects[0].geometry_sha256,
            report.objects[1].geometry_sha256
        );
        assert_ne!(
            report.objects[0].resolved_export_sha256,
            report.objects[1].resolved_export_sha256
        );
        assert_eq!(report.objects[1].translation, [40., 0., 0.]);
        assert_eq!(
            report.objects[0].unsupported,
            [nbcad_core::PrintSettingFieldDto::WallCount]
        );
        assert_eq!(report.evidence["toolpaths_generated"], false);
    }
}
