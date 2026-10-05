use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use nbcad_export::{
    bambu_project::BambuProjectReport,
    slicer_verification::{LocalSlicerStartRequest, VerificationIdentity, local_slicer_service},
};
use serde_json::Value;

impl AppState {
    pub fn start_local_slicer_verification(&self, payload: &str) -> String {
        let result = (|| -> Result<_, String> {
            if payload.len() > 192 * 1024 * 1024 {
                return Err("Local verification request exceeds the project payload limit".into());
            }
            let request: LocalSlicerStartRequest =
                serde_json::from_str(payload).map_err(|e| e.to_string())?;
            let written: Value = serde_json::from_str(&self.bambu_project(
                &serde_json::to_string(&request.project).map_err(|e| e.to_string())?,
                false,
            ))
            .map_err(|e| e.to_string())?;
            if written["ok"] != true {
                return Err(written["error"]
                    .as_str()
                    .unwrap_or("Bambu project preparation failed")
                    .into());
            }
            let written = &written["value"];
            let bytes = BASE64
                .decode(
                    written["bytes_base64"]
                        .as_str()
                        .ok_or("Writer returned no project bytes")?,
                )
                .map_err(|e| e.to_string())?;
            let report: BambuProjectReport =
                serde_json::from_value(written["report"].clone()).map_err(|e| e.to_string())?;
            let workspace = self.inner.lock().map_err(|_| "Engine lock poisoned")?;
            let inner = workspace.active();
            let model = inner
                .manager
                .export_project_model()
                .map_err(|e| e.to_string())?;
            // Reject intervening edits between writing the reviewed copy and starting the worker.
            request
                .project
                .validate(&model)
                .map_err(|e| e.to_string())?;
            let layout = serde_json::to_value(
                inner
                    .manager
                    .named_view_solution(request.project.export.named_view.as_deref())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            if written["source_session_id"].as_str() != Some(workspace.active_session_id.as_str())
                || written["source_layout"] != layout
                || written["source_geometry_revision"].as_u64() != Some(inner.geometry_revision)
            {
                return Err("The owning tab or presentation layout changed while preparing validation; review the project again".into());
            }
            let exported_layout = nbcad_export::slicer_verification::resolved_bambu_layout(&report);
            let mut identity = VerificationIdentity::from_owned_export(
                &bytes,
                &model,
                &layout,
                &exported_layout,
                report.refresh_reference.profile_sha256,
                report.source_document_id,
                request.project.export.named_view.clone(),
            )?;
            identity.source_geometry_revision = Some(inner.geometry_revision);
            local_slicer_service().start(
                bytes,
                identity,
                report.template.plate_count as u32,
                request.options,
                workspace.verification_owner_key(),
            )
        })();
        match result {
            Ok(value) => ok_json(value),
            Err(error) => err_json(error),
        }
    }

    pub fn local_slicer_status(&self, payload: &str, cancel: bool) -> String {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Request {
            job_id: u64,
        }
        let result = (|| -> Result<_, String> {
            let request: Request = serde_json::from_str(payload).map_err(|e| e.to_string())?;
            let workspace = self.inner.lock().map_err(|_| "Engine lock poisoned")?;
            if cancel {
                return serde_json::to_value(
                    local_slicer_service()
                        .cancel_owned(request.job_id, &workspace.verification_owner_key())?,
                )
                .map_err(|error| error.to_string());
            }
            let manager = &workspace.active().manager;
            let source = manager
                .print_intent()
                .source_document_id
                .ok_or("Current CAD document has no persistent print identity")?;
            let model = manager.export_project_model().map_err(|e| e.to_string())?;
            let mut report = local_slicer_service().poll_owned(
                request.job_id,
                &source,
                &model,
                &workspace.verification_owner_key(),
                cancel,
            )?;
            report.check_current_geometry_revision(workspace.active().geometry_revision);
            report.check_current_layout(
                manager
                    .named_view_solution(report.identity.named_view.as_deref())
                    .map_err(|e| e.to_string())
                    .and_then(|layout| serde_json::to_value(layout).map_err(|e| e.to_string())),
            );
            serde_json::to_value(report).map_err(|error| error.to_string())
        })();
        match result {
            Ok(value) => ok_json(value),
            Err(error) => err_json(error),
        }
    }
}

impl AppState {
    /// Serve attached MCP preflight from its owning native workspace, retaining
    /// transient recalled placement and checking the same snapshot as export.
    pub fn export_preflight(&self, payload: &str) -> String {
        let request: nbcad_export::MeshExportRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(error) => return err_json(format!("bad request payload: {error}")),
        };
        let workspace = self.inner.lock().expect("engine lock poisoned");
        let inner = workspace.active();
        let result = (|| -> Result<_, String> {
            check_layout_request(inner, &request)?;
            let scene = inner.manager.solid_scene_ref();
            let errors: Vec<_> = scene
                .errors
                .iter()
                .map(|e| format!("feature {}: {}", e.feature_id.0, e.message))
                .collect();
            let body_ids: Vec<_> = scene.bodies.iter().map(|b| b.id.0).collect();
            let appearing: Vec<_> = inner
                .manager
                .body_appearances()
                .iter()
                .map(|a| a.body_id.0)
                .collect();
            let missing: Vec<_> = body_ids
                .iter()
                .copied()
                .filter(|id| !appearing.contains(id))
                .collect();
            let ok = errors.is_empty() && !body_ids.is_empty();
            let mut result = serde_json::json!({
                "ok":ok,"body_count":body_ids.len(),"body_ids":body_ids,"timeline_errors":errors,
                "appearances_assigned":appearing.len(),"bodies_missing_appearance":missing,
                "hints": if ok { vec!["Ready for solid_export_3mf (preferred) or solid_export_stl / solid_export_step."] }
                    else { vec!["Fix timeline_errors before export.","Empty documents cannot export meshes.",
                        "Optional: set_body_appearance / material_catalog for colored 3MF."] }
            });
            result["print_intent"] = serde_json::to_value(
                inner
                    .manager
                    .effective_print_intent(
                        request.body_ids.clone(),
                        Some(nbcad_core::PrintIntentTargetDto::Portable),
                    )
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            if ok {
                let (meshes, solution, bed) = native_layout_inputs(inner, &request, None)?;
                let layout = nbcad_export::analyze_print_layout(
                    &meshes,
                    &inner.manager.assembly_document().component_structure,
                    &solution,
                    &bed,
                )
                .map_err(|e| e.to_string())?;
                let effective = inner
                    .manager
                    .effective_print_intent(
                        request.body_ids.clone(),
                        Some(nbcad_core::PrintIntentTargetDto::Portable),
                    )
                    .map_err(|e| e.to_string())?;
                result["manufacturing"] = serde_json::to_value(
                    nbcad_export::manufacturing_report::manufacturing_preflight_report(
                        &meshes,
                        &inner.manager.body_appearances(),
                        &inner.manager.assembly_document().component_structure,
                        &solution,
                        &inner.manager.print_intent(),
                        &effective,
                    )
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                if layout.printable_instances == 0 {
                    result["ok"] = serde_json::json!(false);
                    result["hints"] =
                        serde_json::json!(["No visible occurrences are included in this export."]);
                }
                result["layout"] = serde_json::to_value(layout).map_err(|e| e.to_string())?;
            }
            Ok(result)
        })();
        match result {
            Ok(result) => ok_json(result),
            Err(error) => err_json(error),
        }
    }
}

fn check_layout_request(
    inner: &NativeEngine,
    request: &nbcad_export::MeshExportRequest,
) -> Result<(), String> {
    if request.scope != nbcad_export::MeshExportScope::Assembly {
        return Err("Print layout checks require assembly scope.".into());
    }
    if request.expected_model_json.is_some() {
        request
            .check_model_snapshot(
                &inner
                    .manager
                    .export_project_model()
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn native_layout_inputs(
    inner: &NativeEngine,
    request: &nbcad_export::MeshExportRequest,
    draft: Option<&nbcad_sketch::NamedViewConfigurationDto>,
) -> Result<
    (
        Vec<nbcad_export::TriangleMesh>,
        nbcad_sketch::AssemblySolutionDto,
        nbcad_core::PrintBedDto,
    ),
    String,
> {
    if !inner.manager.solid_scene_ref().errors.is_empty() {
        return Err("Resolve timeline errors before checking the print layout.".into());
    }
    let solution = match draft {
        Some(view) => inner.manager.resolve_named_view(view),
        None => inner
            .manager
            .named_view_solution(request.named_view.as_deref()),
    }
    .map_err(|e| e.to_string())?;
    let view_bed = request
        .named_view
        .as_ref()
        .and_then(|name| {
            inner
                .manager
                .named_views()
                .views
                .into_iter()
                .find(|view| &view.name == name)
        })
        .map(|view| view.print_bed);
    let bed = request
        .print_bed
        .clone()
        .or_else(|| draft.map(|view| view.print_bed.clone()))
        .or(view_bed)
        .unwrap_or_default();
    let mut meshes = inner
        .kernel
        .tessellate_bodies(request)
        .map_err(|e| e.to_string())?;
    let scene = inner.manager.solid_scene_ref();
    for mesh in &mut meshes {
        if let Some(body) = scene.bodies.iter().find(|body| body.id == mesh.body_id) {
            mesh.name = body.name.clone();
        }
    }
    Ok((meshes, solution, bed))
}

#[cfg(test)]
mod verification_ownership_tests {
    use super::*;
    #[test]
    fn separate_hosts_with_same_tab_ids_cannot_cancel_or_read_another_owned_job() {
        let first = AppState::new();
        let second = AppState::new();
        for host in [&first, &second] {
            let response: Value =
                serde_json::from_str(&host.bind_project_session("same-verification-tab")).unwrap();
            assert_eq!(response["ok"], true);
        }
        let first_owner = first.inner.lock().unwrap().verification_owner_key();
        let second_owner = second.inner.lock().unwrap().verification_owner_key();
        assert_ne!(first_owner, second_owner);
        let source = "01234567-89ab-4cde-8123-456789abcdef";
        let model = serde_json::json!({"print_intent":{"source_document_id":source}}).to_string();
        let bytes = b"owned cancellation lifecycle fixture".to_vec();
        let identity = VerificationIdentity::from_owned_export(
            &bytes,
            &model,
            &serde_json::json!({}),
            &serde_json::json!({}),
            "a".repeat(64),
            source.into(),
            None,
        )
        .unwrap();
        let service = local_slicer_service();
        let started = service
            .start(
                bytes,
                identity,
                1,
                nbcad_export::slicer_verification::LocalSlicerOptions {
                    executable: std::env::temp_dir().join("missing-bambu-private-owner-test.exe"),
                    timeout_seconds_per_plate: 1,
                },
                first_owner.clone(),
            )
            .unwrap();
        let payload = serde_json::json!({"job_id":started.job_id}).to_string();
        let denied: Value =
            serde_json::from_str(&second.local_slicer_status(&payload, true)).unwrap();
        assert_eq!(denied["ok"], false);
        assert!(denied["error"].as_str().unwrap().contains("owning engine"));
        assert!(
            service
                .poll_owned(started.job_id, source, &model, &second_owner, false)
                .is_err()
        );
        // The owning host is now blank with no original document UUID. Cancellation remains
        // authorized by its private host+tab ownership and returns no old document details.
        let receipt: Value =
            serde_json::from_str(&first.local_slicer_status(&payload, true)).unwrap();
        assert_eq!(receipt["ok"], true);
        assert_eq!(
            receipt["value"],
            serde_json::json!({"job_id":started.job_id,"cancel_requested":true})
        );
        assert_eq!(
            service
                .poll(started.job_id, None)
                .unwrap()
                .identity
                .source_document_id,
            source
        );
    }
}
