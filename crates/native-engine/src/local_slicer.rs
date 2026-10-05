use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use limo_cad_export::{
    bambu_project::BambuProjectReport,
    slicer_verification::{LocalSlicerStartRequest, VerificationIdentity, local_slicer_service},
};
use serde_json::Value;

impl NativeEngineHost {
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
                    .export_view_solution(request.project.export.named_view.as_deref())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
            let exported_layout =
                limo_cad_export::slicer_verification::resolved_bambu_layout(&report);
            let identity = VerificationIdentity::from_owned_export(
                &bytes,
                &model,
                &layout,
                &exported_layout,
                report.refresh_reference.profile_sha256,
                report.source_document_id,
                request.project.export.named_view.clone(),
            )?;
            local_slicer_service().start(
                bytes,
                identity,
                report.template.plate_count as u32,
                request.options,
                workspace.active_session_id.clone(),
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
                &workspace.active_session_id,
                cancel,
            )?;
            report.check_current_layout(
                manager
                    .export_view_solution(report.identity.named_view.as_deref())
                    .map_err(|e| e.to_string())
                    .and_then(|layout| serde_json::to_value(layout).map_err(|e| e.to_string())),
            );
            Ok(report)
        })();
        match result {
            Ok(value) => ok_json(value),
            Err(error) => err_json(error),
        }
    }
}
