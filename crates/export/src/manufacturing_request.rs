use crate::{bambu_project::BambuProjectRequest, MeshExportRequest};
use serde::{Deserialize, Serialize};

/// Explicit target-project mode. Portable mesh export never implicitly reads a profile/template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BambuExportRequest {
    pub export: MeshExportRequest,
    pub project: BambuProjectRequest,
    /// A complete saved unsliced project, supplied by the caller without overwriting its source.
    pub template_base64: String,
}

impl BambuExportRequest {
    pub fn validate(&self, current_model_json: &str) -> Result<(), crate::ExportError> {
        if self.export.scope != crate::MeshExportScope::Assembly {
            return Err(crate::ExportError("Bambu project export requires resolved assembly/layout scope; choose portable export for source definitions.".into()));
        }
        if self
            .export
            .expected_model_json
            .as_deref()
            .is_none_or(str::is_empty)
        {
            return Err(crate::ExportError("Bambu project export requires expected_model_json from the reviewed completed document.".into()));
        }
        self.export.check_model_snapshot(current_model_json)?;
        if self.template_base64.len() > 180 * 1024 * 1024 {
            return Err(crate::ExportError(
                "Saved Bambu template exceeds the 128 MiB input limit.".into(),
            ));
        }
        if !self.export.include_appearance {
            return Err(crate::ExportError("Bambu projects require an explicit material/color mapping; include_appearance cannot be disabled.".into()));
        }
        if self.project.placement == crate::bambu_project::BambuPlacementMode::Template
            && self
                .export
                .named_view
                .as_deref()
                .is_some_and(|name| !name.is_empty())
        {
            return Err(crate::ExportError("Template placement preserves its saved plates and orientation. Clear named_view to choose it explicitly, or use resolved_scene with a compatible template to export the selected CAD layout.".into()));
        }
        Ok(())
    }
}
