//! Local, opt-in Bambu CLI verification. The only child commands import and slice owned copies.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_PROJECT_BYTES: usize = 128 * 1024 * 1024;
const MAX_LOG_BYTES: usize = 64 * 1024;
const MAX_OUTPUT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_JOBS: usize = 16;
const QUALIFIED_VERSION: &str = "02.08.02.61";

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Hashes captured by the owning engine, not supplied as unverified labels by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationIdentity {
    pub source_document_id: String,
    pub project_sha256: String,
    pub source_model_sha256: String,
    pub print_intent_sha256: String,
    pub resolved_layout_sha256: String,
    pub named_view: Option<String>,
    pub profile_sha256: String,
}
impl VerificationIdentity {
    pub fn from_owned_export(
        project: &[u8],
        model_json: &str,
        layout: &serde_json::Value,
        profile_sha256: String,
        source_document_id: String,
        named_view: Option<String>,
    ) -> Result<Self, String> {
        let model: serde_json::Value =
            serde_json::from_str(model_json).map_err(|e| e.to_string())?;
        if model["print_intent"]["source_document_id"].as_str() != Some(source_document_id.as_str())
        {
            return Err(
                "Verification source identity does not match the owning CAD document".into(),
            );
        }
        Ok(Self {
            source_document_id,
            project_sha256: sha256(project),
            source_model_sha256: model_sha256(model_json)?,
            print_intent_sha256: sha256(
                &serde_json::to_vec(&model["print_intent"]).map_err(|e| e.to_string())?,
            ),
            resolved_layout_sha256: sha256(&serde_json::to_vec(layout).map_err(|e| e.to_string())?),
            profile_sha256,
            named_view,
        })
    }
}
pub fn model_sha256(model_json: &str) -> Result<String, String> {
    let model: serde_json::Value = serde_json::from_str(model_json).map_err(|e| e.to_string())?;
    Ok(sha256(
        &serde_json::to_vec(&model).map_err(|e| e.to_string())?,
    ))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalSlicerStartRequest {
    pub project: crate::BambuExportRequest,
    pub options: LocalSlicerOptions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalSlicerOptions {
    /// Explicit local executable. Arguments and output paths cannot be supplied by the caller.
    pub executable: PathBuf,
    #[serde(default = "default_timeout")]
    pub timeout_seconds_per_plate: u32,
}
fn default_timeout() -> u32 {
    120
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlateVerificationState {
    NotRun,
    ToolpathsGenerated,
    Failed,
    TimedOut,
    Cancelled,
    MissingSlicer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlateVerification {
    pub plate: u32,
    pub state: PlateVerificationState,
    pub exit_status: Option<i32>,
    pub elapsed_milliseconds: u64,
    pub slicer_version: Option<String>,
    pub toolpath_sha256: Option<String>,
    pub native_result: Option<serde_json::Value>,
    pub native_effective_settings: Option<serde_json::Value>,
    pub toolpaths_generated: bool,
    pub native_project_read_back: bool,
    pub compatibility_issues: Vec<String>,
    pub setting_changes: Vec<String>,
    pub stdout: String,
    pub stderr: String,
    pub message: Option<String>,
}
impl PlateVerification {
    fn not_run(plate: u32) -> Self {
        Self {
            plate,
            state: PlateVerificationState::NotRun,
            exit_status: None,
            elapsed_milliseconds: 0,
            slicer_version: None,
            toolpath_sha256: None,
            native_result: None,
            native_effective_settings: None,
            toolpaths_generated: false,
            native_project_read_back: false,
            compatibility_issues: Vec::new(),
            setting_changes: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
            message: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalSlicerReport {
    pub job_id: u64,
    pub state: VerificationState,
    pub identity: VerificationIdentity,
    pub executable: PathBuf,
    pub executable_sha256: Option<String>,
    pub plates: Vec<PlateVerification>,
    pub stale: bool,
    pub physical_qualification: String,
    pub warnings: Vec<String>,
}

impl LocalSlicerReport {
    /// Resolve the same saved/current presentation layout on the owning engine during polling.
    pub fn check_current_layout(&mut self, layout: Result<serde_json::Value, String>) {
        match layout.and_then(|value| serde_json::to_vec(&value).map_err(|e| e.to_string())) {
            Ok(bytes) => self.stale |= sha256(&bytes) != self.identity.resolved_layout_sha256,
            Err(error) => {
                self.stale = true;
                self.warnings
                    .push(format!("Current layout unavailable: {error}"));
            }
        }
    }
}

struct Job {
    owner_key: String,
    report: Mutex<LocalSlicerReport>,
    cancel: AtomicBool,
}
#[derive(Default)]
pub struct LocalSlicerService {
    jobs: Mutex<BTreeMap<u64, Arc<Job>>>,
    next: AtomicU64,
}
pub fn local_slicer_service() -> &'static LocalSlicerService {
    static SERVICE: OnceLock<LocalSlicerService> = OnceLock::new();
    SERVICE.get_or_init(LocalSlicerService::default)
}

pub fn new_verification_owner() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "headless-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

impl LocalSlicerService {
    /// Start validation of a validated unsliced project. This never opens the user's active file.
    pub fn start(
        &self,
        bytes: Vec<u8>,
        identity: VerificationIdentity,
        plate_count: u32,
        options: LocalSlicerOptions,
        owner_key: String,
    ) -> Result<LocalSlicerReport, String> {
        if bytes.is_empty() || bytes.len() > MAX_PROJECT_BYTES {
            return Err("Project must be 1–128 MiB".into());
        }
        if identity.project_sha256 != sha256(&bytes) {
            return Err("Project hash does not match the exact validation artifact".into());
        }
        if !(1..=64).contains(&plate_count) {
            return Err("Local verification supports 1–64 plates".into());
        }
        if !(1..=600).contains(&options.timeout_seconds_per_plate) {
            return Err("Per-plate timeout must be 1–600 seconds".into());
        }
        if !options.executable.is_absolute() {
            return Err("Choose an absolute local Bambu Studio executable path".into());
        }
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| "Verification registry lock poisoned")?;
        if jobs
            .values()
            .filter(|job| {
                job.report.lock().is_ok_and(|r| {
                    matches!(
                        r.state,
                        VerificationState::Queued | VerificationState::Running
                    )
                })
            })
            .count()
            >= 2
        {
            return Err("Two local validations are already active; cancel or await one".into());
        }
        while jobs.len() >= MAX_JOBS {
            let completed = jobs
                .iter()
                .find(|(_, job)| {
                    job.report.lock().is_ok_and(|r| {
                        !matches!(
                            r.state,
                            VerificationState::Queued | VerificationState::Running
                        )
                    })
                })
                .map(|(id, _)| *id);
            if let Some(id) = completed {
                jobs.remove(&id);
            } else {
                return Err("Verification registry is full".into());
            }
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let report = LocalSlicerReport { job_id: id, state: VerificationState::Queued, identity,
            executable: options.executable.clone(), executable_sha256: None,
            plates: (1..=plate_count).map(PlateVerification::not_run).collect(), stale: false,
            physical_qualification: "not_run".into(), warnings: vec![
                "Toolpaths do not qualify physical fit or strength. Requested walls are not guaranteed realized loops in thin sections.".into(),
                "Local slicing operates on a temporary copy; no print, printer or cloud command is issued.".into()] };
        let job = Arc::new(Job {
            owner_key,
            report: Mutex::new(report.clone()),
            cancel: AtomicBool::new(false),
        });
        jobs.insert(id, job.clone());
        thread::Builder::new()
            .name(format!("local-slicer-{id}"))
            .spawn(move || run_job(&job, bytes, options))
            .map_err(|e| {
                jobs.remove(&id);
                format!("Cannot start verification worker: {e}")
            })?;
        Ok(report)
    }

    /// Current engine hashes mark evidence stale without changing or reinterpreting prior results.
    pub fn poll(
        &self,
        id: u64,
        current: Option<&VerificationIdentity>,
    ) -> Result<LocalSlicerReport, String> {
        let job = self.job(id)?;
        let mut report = job
            .report
            .lock()
            .map_err(|_| "Verification report lock poisoned")?
            .clone();
        if let Some(current) = current {
            report.stale = report.identity != *current;
        }
        Ok(report)
    }
    pub fn cancel(&self, id: u64) -> Result<LocalSlicerReport, String> {
        let job = self.job(id)?;
        job.cancel.store(true, Ordering::Release);
        self.poll(id, None)
    }
    pub fn poll_owned(
        &self,
        id: u64,
        source_document_id: &str,
        current_model_json: &str,
        owner_key: &str,
        cancel: bool,
    ) -> Result<LocalSlicerReport, String> {
        let job = self.job(id)?;
        if job.owner_key != owner_key {
            return Err("Verification belongs to a different owning engine session".into());
        }
        let mut report = job
            .report
            .lock()
            .map_err(|_| "Verification report lock poisoned")?
            .clone();
        if report.identity.source_document_id != source_document_id {
            return Err("Verification belongs to a different CAD document".into());
        }
        report.stale = report.identity.source_model_sha256 != model_sha256(current_model_json)?;
        if cancel {
            job.cancel.store(true, Ordering::Release);
        }
        Ok(report)
    }
    fn job(&self, id: u64) -> Result<Arc<Job>, String> {
        self.jobs
            .lock()
            .map_err(|_| "Verification registry lock poisoned")?
            .get(&id)
            .cloned()
            .ok_or_else(|| "Unknown or expired local verification job".into())
    }
}

fn update(job: &Job, f: impl FnOnce(&mut LocalSlicerReport)) {
    if let Ok(mut report) = job.report.lock() {
        f(&mut report);
    }
}

fn run_job(job: &Job, bytes: Vec<u8>, options: LocalSlicerOptions) {
    update(job, |report| report.state = VerificationState::Running);
    let result = run_job_inner(job, bytes, &options);
    update(job, |report| {
        if let Err(error) = result {
            report.warnings.push(error);
            report.state = VerificationState::Failed;
        } else if job.cancel.load(Ordering::Acquire) {
            report.state = VerificationState::Cancelled;
        } else if report
            .plates
            .iter()
            .all(|p| p.state == PlateVerificationState::ToolpathsGenerated)
        {
            report.state = VerificationState::Completed;
        } else {
            report.state = VerificationState::Failed;
        }
    });
}

struct OwnedDirectory(PathBuf);
impl OwnedDirectory {
    fn create() -> Result<Self, String> {
        static SERIAL: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let root = std::env::temp_dir()
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let path = root.join(format!(
            "nbcad-slice-{}-{stamp}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path)
            .map_err(|e| format!("Cannot create temporary validation directory: {e}"))?;
        Ok(Self(path))
    }
}
impl Drop for OwnedDirectory {
    fn drop(&mut self) {
        // Only the directory created above is removed, after our child has exited and been reaped.
        if self
            .0
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("nbcad-slice-"))
        {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

fn cli_path(path: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        // Canonical Rust paths use the Win32 extended namespace, unsupported by the slicer's parser.
        let text = path.to_string_lossy();
        if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{unc}"));
        }
        if let Some(local) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(local);
        }
    }
    path.to_owned()
}

fn fingerprint_executable(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Cannot fingerprint selected slicer: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > MAX_OUTPUT_BYTES {
            return Err("Selected executable exceeds the 512 MiB fingerprint limit".into());
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn run_job_inner(job: &Job, bytes: Vec<u8>, options: &LocalSlicerOptions) -> Result<(), String> {
    if job.cancel.load(Ordering::Acquire) {
        update(job, |report| {
            for plate in &mut report.plates {
                plate.state = PlateVerificationState::Cancelled;
            }
        });
        return Ok(());
    }
    if !options.executable.is_file() {
        update(job, |report| {
            for plate in &mut report.plates {
                plate.state = PlateVerificationState::MissingSlicer;
                plate.message = Some(
                    "Bambu Studio was not found; export remains usable without local verification"
                        .into(),
                );
            }
        });
        return Ok(());
    }
    if std::fs::metadata(&options.executable)
        .map_err(|e| e.to_string())?
        .len()
        > MAX_OUTPUT_BYTES
    {
        return Err("Selected executable exceeds the 512 MiB fingerprint limit".into());
    }
    let executable_sha256 = fingerprint_executable(&options.executable)?;
    update(job, |report| {
        report.executable_sha256 = Some(executable_sha256)
    });
    let directory = OwnedDirectory::create()?;
    let input = directory.0.join("input.3mf");
    std::fs::write(&input, bytes).map_err(|e| e.to_string())?;
    let count = job
        .report
        .lock()
        .map_err(|_| "Verification lock poisoned")?
        .plates
        .len();
    for index in 0..count {
        if job.cancel.load(Ordering::Acquire) {
            update(job, |r| {
                for p in &mut r.plates[index..] {
                    p.state = PlateVerificationState::Cancelled;
                }
            });
            break;
        }
        let output = directory.0.join(format!("plate-{}", index + 1));
        std::fs::create_dir(&output).map_err(|e| e.to_string())?;
        let mut command = Command::new(&options.executable);
        command
            .args(["--arrange", "0", "--slice"])
            .arg((index + 1).to_string())
            .arg("--outputdir")
            .arg(cli_path(&output))
            .arg("--export-3mf")
            .arg("resliced.3mf")
            .arg(cli_path(&input))
            .current_dir(&output)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        let mut result = run_plate(
            command,
            &output,
            index as u32 + 1,
            Duration::from_secs(options.timeout_seconds_per_plate.into()),
            &job.cancel,
        );
        if result.state == PlateVerificationState::ToolpathsGenerated {
            match native_setting_readback(&input, &output.join("resliced.3mf"), index as u32 + 1) {
                Ok((values, changes, issues)) => {
                    result.native_project_read_back = true;
                    result.native_effective_settings = Some(values);
                    result.setting_changes = changes;
                    if !issues.is_empty() {
                        result.state = PlateVerificationState::Failed;
                        result.message = Some("Native project mappings or quantities changed; review the compatibility issues before using these toolpaths".into());
                        result.compatibility_issues = issues;
                    }
                }
                Err(error) => {
                    result.message =
                        Some(format!("Native effective settings unavailable: {error}"));
                }
            }
        }
        update(job, |r| r.plates[index] = result);
    }
    Ok(())
}

fn native_setting_readback(
    input: &Path,
    output: &Path,
    plate: u32,
) -> Result<(serde_json::Value, Vec<String>, Vec<String>), String> {
    let read = |path: &Path, selected_plate: Option<u32>| -> Result<_, String> {
        if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_PROJECT_BYTES as u64 {
            return Err("Native saved project exceeds 128 MiB readback limit".into());
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        match selected_plate {
            Some(plate) => crate::bambu_project::inspect_bambu_plate(&bytes, plate),
            None => crate::bambu_project::inspect_bambu_template(&bytes),
        }
        .map_err(|e| e.to_string())
    };
    let before = read(input, Some(plate))?;
    let after = read(output, None)?;
    let collect = |summary: &crate::bambu_project::BambuTemplateSummary| -> Result<BTreeMap<String,serde_json::Value>,String> {
        let defaults = serde_json::to_value(&summary.process_defaults).map_err(|e| e.to_string())?;
        let mut settings = BTreeMap::from([("process_defaults".into(), defaults),
            ("profile_mappings".into(),serde_json::json!({"printer_settings_id":summary.printer_settings_id,
            "printer_model":summary.printer_model,"printer_variant":summary.printer_variant,"process_settings_id":summary.process_settings_id,"nozzle_diameter_mm":summary.nozzle_diameter_mm,
            "filament_settings_ids":summary.filament_settings_ids,"filament_types":summary.filament_types,"filament_colors":summary.filament_colors,
            "support_filament":summary.support_filament,"support_interface_filament":summary.support_interface_filament,
            "filament_map":summary.filament_map,"filament_nozzle_map":summary.filament_nozzle_map}))]);
        for object in &summary.objects {
            for part in object.parts.iter().filter(|p| p.subtype == "normal_part") {
                let uuid = part.uuid.as_ref().ok_or("Native normal volume lacks a stable UUID")?;
                let mut values = crate::bambu_project::settings_map(&summary.process_defaults);
                for scoped in [&object.settings, &part.settings] {
                    for key in ["wall_loops","sparse_infill_density","sparse_infill_pattern","top_shell_layers","bottom_shell_layers"] {
                        if let Some(value) = scoped.get(key) { values.insert(key.to_owned(), value.clone()); }
                    }
                }
                let key = format!("volume:{uuid}");
                if settings.insert(key,serde_json::json!({"settings":values,"instance_count":object.instance_count})).is_some() {
                    return Err("Native volume identity is ambiguous; effective readback requires review".into());
                }
            }
        }
        Ok(settings)
    };
    let expected = collect(&before)?;
    let actual = collect(&after)?;
    let issues = mapping_compatibility_issues(&expected, &actual);
    let changes = expected
        .keys()
        .chain(actual.keys())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|key| expected.get(*key) != actual.get(*key))
        .map(|key| {
            format!(
                "{key}: writer {} → native {}",
                expected.get(key).unwrap_or(&serde_json::Value::Null),
                actual.get(key).unwrap_or(&serde_json::Value::Null)
            )
        })
        .collect();
    Ok((
        serde_json::to_value(actual).map_err(|e| e.to_string())?,
        changes,
        issues,
    ))
}

fn mapping_compatibility_issues(
    expected: &BTreeMap<String, serde_json::Value>,
    actual: &BTreeMap<String, serde_json::Value>,
) -> Vec<String> {
    let mut issues = Vec::new();
    if expected.get("profile_mappings") != actual.get("profile_mappings") {
        issues.push("Native printer, nozzle, process, filament or support mapping differs from the reviewed project".into());
    }
    for key in expected
        .keys()
        .chain(actual.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        if key.starts_with("volume:") {
            match (expected.get(key), actual.get(key)) {
                (Some(before), Some(after))
                    if before["instance_count"] == after["instance_count"] => {}
                _ => issues.push(format!("Native volume identity or quantity changed: {key}")),
            }
        }
    }
    issues
}

struct CapturedPipe {
    captured: Arc<Mutex<Vec<u8>>>,
    worker: thread::JoinHandle<()>,
}
fn bounded_pipe(mut pipe: impl Read + Send + 'static) -> CapturedPipe {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let copy = captured.clone();
    let worker = thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        while let Ok(count) = pipe.read(&mut buffer) {
            if count == 0 {
                break;
            }
            if let Ok(mut captured) = copy.lock() {
                let keep = count.min(MAX_LOG_BYTES.saturating_sub(captured.len()));
                captured.extend_from_slice(&buffer[..keep]);
            }
        }
    });
    CapturedPipe { captured, worker }
}
fn finish_pipe(pipe: Option<CapturedPipe>) -> String {
    let Some(pipe) = pipe else {
        return String::new();
    };
    // A descendant inheriting a pipe must never extend the validation time bound.
    let deadline = Instant::now() + Duration::from_millis(100);
    while !pipe.worker.is_finished() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    if pipe.worker.is_finished() {
        let _ = pipe.worker.join();
    }
    pipe.captured
        .lock()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

fn output_size(path: &Path) -> u64 {
    let mut pending = vec![path.to_owned()];
    let mut entries_seen = 0usize;
    let mut total = 0u64;
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return MAX_OUTPUT_BYTES + 1;
        };
        for entry in entries {
            entries_seen += 1;
            if entries_seen > 10_000 {
                return MAX_OUTPUT_BYTES + 1;
            }
            let Ok(entry) = entry else {
                return MAX_OUTPUT_BYTES + 1;
            };
            let Ok(kind) = entry.file_type() else {
                return MAX_OUTPUT_BYTES + 1;
            };
            if kind.is_symlink() {
                return MAX_OUTPUT_BYTES + 1;
            }
            if kind.is_dir() {
                pending.push(entry.path());
            } else {
                let Ok(metadata) = entry.metadata() else {
                    return MAX_OUTPUT_BYTES + 1;
                };
                total = total.saturating_add(metadata.len());
                if total > MAX_OUTPUT_BYTES {
                    return total;
                }
            }
        }
    }
    total
}

fn run_plate(
    mut command: Command,
    directory: &Path,
    plate: u32,
    timeout: Duration,
    cancel: &AtomicBool,
) -> PlateVerification {
    let mut result = PlateVerification::not_run(plate);
    let start = Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            result.state = PlateVerificationState::Failed;
            result.message = Some(format!("Cannot launch Bambu Studio: {error}"));
            return result;
        }
    };
    let stdout = child.stdout.take().map(bounded_pipe);
    let stderr = child.stderr.take().map(bounded_pipe);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                result.exit_status = status.code();
                result.state = if status.success() {
                    PlateVerificationState::ToolpathsGenerated
                } else {
                    PlateVerificationState::Failed
                };
                break;
            }
            Ok(None) => {}
            Err(error) => {
                result.state = PlateVerificationState::Failed;
                result.message = Some(format!("Cannot await slicer: {error}"));
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
        }
        let reason = if cancel.load(Ordering::Acquire) {
            Some((PlateVerificationState::Cancelled, "Cancelled by user"))
        } else if start.elapsed() >= timeout {
            Some((
                PlateVerificationState::TimedOut,
                "Local slicer exceeded its time limit",
            ))
        } else if output_size(directory) > MAX_OUTPUT_BYTES {
            Some((
                PlateVerificationState::Failed,
                "Local slicer exceeded the 512 MiB per-plate output limit",
            ))
        } else {
            None
        };
        if let Some((state, message)) = reason {
            let _ = child.kill();
            let _ = child.wait();
            result.state = state;
            result.message = Some(message.into());
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }
    result.elapsed_milliseconds = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
    result.stdout = finish_pipe(stdout);
    result.stderr = finish_pipe(stderr);
    if result.state == PlateVerificationState::Failed && result.message.is_none() {
        let native = directory.join("result.json");
        let reason = std::fs::metadata(&native)
            .ok()
            .filter(|m| m.len() <= 4 * 1024 * 1024)
            .and_then(|_| std::fs::read(native).ok())
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .and_then(|value| value["error_string"].as_str().map(str::to_owned));
        result.message = Some(reason.unwrap_or_else(|| {
            format!(
                "Local Bambu exited with status {:?}; no native error report was available",
                result.exit_status
            )
        }));
    }
    if result.state == PlateVerificationState::ToolpathsGenerated {
        if output_size(directory) > MAX_OUTPUT_BYTES {
            result.state = PlateVerificationState::Failed;
            result.message = Some("Local slicer exceeded the bounded output limit".into());
        } else {
            inspect_plate_output(directory, &mut result);
        }
    }
    result
}

fn inspect_plate_output(directory: &Path, result: &mut PlateVerification) {
    let validation = (|| -> Result<(), String> {
        let native_path = directory.join("result.json");
        if std::fs::metadata(&native_path)
            .map_err(|e| format!("Native slice result is missing: {e}"))?
            .len()
            > 4 * 1024 * 1024
        {
            return Err("Native result exceeds 4 MiB".into());
        }
        let native: serde_json::Value =
            serde_json::from_slice(&std::fs::read(native_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if native["return_code"].as_i64() != Some(0) {
            return Err(format!(
                "Native slicing rejected the project: {}",
                native["error_string"]
            ));
        }
        let plates = native["sliced_plates"]
            .as_array()
            .ok_or("Native output has no sliced plates")?;
        let selected = plates
            .iter()
            .find(|p| p["id"].as_u64() == Some(result.plate as u64))
            .ok_or("Requested plate is missing from native slice results")?;
        result.native_result = Some(selected.clone());
        let path = directory.join(format!("plate_{}.gcode", result.plate));
        if std::fs::metadata(&path)
            .map_err(|e| format!("Generated toolpath is missing: {e}"))?
            .len()
            > MAX_OUTPUT_BYTES
        {
            return Err("Generated toolpath exceeds the output limit".into());
        }
        let gcode = std::fs::read(path).map_err(|e| e.to_string())?;
        let header = String::from_utf8_lossy(&gcode[..gcode.len().min(4096)]);
        let version = header
            .lines()
            .find_map(|line| line.strip_prefix("; BambuStudio "))
            .ok_or("Native toolpath lacks a Bambu Studio version header")?
            .trim();
        result.slicer_version = Some(version.into());
        if version != QUALIFIED_VERSION {
            return Err(format!(
                "Bambu Studio {version} is not qualified by this adapter; supported version is {QUALIFIED_VERSION}"
            ));
        }
        if !gcode.windows(3).any(|window| window == b"G1 ") {
            return Err("Native output contains no generated moves".into());
        }
        result.toolpath_sha256 = Some(sha256(&gcode));
        result.toolpaths_generated = true;
        Ok(())
    })();
    if let Err(error) = validation {
        result.state = PlateVerificationState::Failed;
        result.message = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn identity(bytes: &[u8]) -> VerificationIdentity {
        VerificationIdentity {
            source_document_id: "01234567-89ab-4cde-8123-456789abcdef".into(),
            project_sha256: sha256(bytes),
            source_model_sha256: "a".repeat(64),
            print_intent_sha256: "b".repeat(64),
            resolved_layout_sha256: "c".repeat(64),
            named_view: None,
            profile_sha256: "d".repeat(64),
        }
    }
    #[test]
    fn missing_slicer_staleness_and_request_limits() {
        let service = LocalSlicerService::default();
        let bytes = b"project".to_vec();
        let options = LocalSlicerOptions {
            executable: std::env::temp_dir().join("no-such-bambu-tool.exe"),
            timeout_seconds_per_plate: 1,
        };
        assert!(
            service
                .start(
                    bytes.clone(),
                    identity(b"wrong"),
                    4,
                    options.clone(),
                    "test-owner".into()
                )
                .is_err()
        );
        assert!(
            service
                .start(
                    bytes.clone(),
                    identity(&bytes),
                    0,
                    options.clone(),
                    "test-owner".into()
                )
                .is_err()
        );
        let started = service
            .start(
                bytes.clone(),
                identity(&bytes),
                4,
                options,
                "test-owner".into(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let done = loop {
            let report = service.poll(started.job_id, None).unwrap();
            if report.state == VerificationState::Failed {
                break report;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        };
        assert!(
            done.plates
                .iter()
                .all(|p| p.state == PlateVerificationState::MissingSlicer)
        );
        assert!(
            service
                .poll_owned(
                    started.job_id,
                    &identity(&bytes).source_document_id,
                    "{}",
                    "other-owner",
                    true
                )
                .is_err()
        );
        let mut changed = identity(&bytes);
        changed.print_intent_sha256 = "e".repeat(64);
        assert!(service.poll(started.job_id, Some(&changed)).unwrap().stale);
        assert!(
            !service
                .poll(started.job_id, Some(&identity(&bytes)))
                .unwrap()
                .stale
        );
    }
    #[test]
    fn a_transient_layout_edit_invalidates_evidence_even_when_saved_model_is_unchanged() {
        let service = LocalSlicerService::default();
        let bytes = b"owned-project".to_vec();
        let layout = serde_json::json!({"rotation":[0.,0.,0.,1.],"translation":[0.,0.,0.]});
        let mut captured = identity(&bytes);
        captured.resolved_layout_sha256 = sha256(&serde_json::to_vec(&layout).unwrap());
        let mut report = service.start(bytes, captured, 1, LocalSlicerOptions {
            executable: std::env::temp_dir().join("absent-bambu-layout-fixture.exe"),
            timeout_seconds_per_plate: 1,
        }, "layout-owner".into()).unwrap();
        report.check_current_layout(Ok(layout));
        assert!(!report.stale);
        report.check_current_layout(Ok(serde_json::json!({"rotation":[0.,1.,0.,0.],"translation":[0.,0.,0.]})));
        assert!(report.stale);
    }

    #[test]
    fn one_failing_plate_does_not_invent_toolpath_evidence() {
        let directory = OwnedDirectory::create().unwrap();
        std::fs::write(
            directory.0.join("result.json"),
            br#"{"return_code":0,"sliced_plates":[{"id":1,"filaments":[]}]}"#,
        )
        .unwrap();
        std::fs::write(
            directory.0.join("plate_1.gcode"),
            "; BambuStudio 02.08.02.61\nG1 X10 E1\n",
        )
        .unwrap();
        let mut first = PlateVerification::not_run(1);
        first.state = PlateVerificationState::ToolpathsGenerated;
        inspect_plate_output(&directory.0, &mut first);
        assert_eq!(first.state, PlateVerificationState::ToolpathsGenerated);
        assert!(first.toolpath_sha256.is_some());
        let mut second = PlateVerification::not_run(2);
        second.state = PlateVerificationState::ToolpathsGenerated;
        inspect_plate_output(&directory.0, &mut second);
        assert_eq!(second.state, PlateVerificationState::Failed);
        assert!(second.toolpath_sha256.is_none());
    }
    #[test]
    fn native_setting_clamps_are_reported_but_mapping_changes_fail_qualification() {
        let before = BTreeMap::from([
            (
                "profile_mappings".into(),
                serde_json::json!({"printer_model":"Bambu X2D"}),
            ),
            (
                "volume:stable".into(),
                serde_json::json!({"settings":{"wall_loops":"6"},"instance_count":2}),
            ),
        ]);
        let mut after = before.clone();
        after.get_mut("volume:stable").unwrap()["settings"]["wall_loops"] = serde_json::json!("4");
        assert!(mapping_compatibility_issues(&before, &after).is_empty());
        after.get_mut("volume:stable").unwrap()["instance_count"] = serde_json::json!(1);
        assert_eq!(mapping_compatibility_issues(&before, &after).len(), 1);
        after.insert(
            "profile_mappings".into(),
            serde_json::json!({"printer_model":"unreviewed"}),
        );
        assert_eq!(mapping_compatibility_issues(&before, &after).len(), 2);
    }

    #[test]
    fn child_timeout_and_cancellation_are_bounded() {
        // Re-exec this test binary's ignored sleeper. No shell, active slicer or external file is used.
        let executable = std::env::current_exe().unwrap();
        for cancel_first in [false, true] {
            let directory = OwnedDirectory::create().unwrap();
            let mut command = Command::new(&executable);
            command
                .args([
                    "--ignored",
                    "--exact",
                    "slicer_verification::tests::verification_child_sleeper",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            let cancel = AtomicBool::new(cancel_first);
            let result = run_plate(
                command,
                &directory.0,
                1,
                Duration::from_millis(100),
                &cancel,
            );
            assert_eq!(
                result.state,
                if cancel_first {
                    PlateVerificationState::Cancelled
                } else {
                    PlateVerificationState::TimedOut
                }
            );
            assert!(result.elapsed_milliseconds < 2000);
            assert!(result.toolpath_sha256.is_none());
        }
    }
    #[test]
    #[ignore = "child process fixture used only by the bounded lifecycle test"]
    fn verification_child_sleeper() {
        thread::sleep(Duration::from_secs(30));
    }

    #[test]
    #[ignore = "requires LIMO_BAMBU_VERIFY_PROJECT, LIMO_BAMBU_VERIFY_REPORT and installed Bambu 2.8.2.61"]
    fn installed_bambu_verifies_each_owned_plate() {
        let path = std::env::var_os("LIMO_BAMBU_VERIFY_PROJECT")
            .expect("owned writer-produced synthetic project");
        let sidecar =
            std::env::var_os("LIMO_BAMBU_VERIFY_REPORT").expect("matching writer report JSON");
        let bytes = std::fs::read(path).unwrap();
        let summary: serde_json::Value =
            serde_json::from_slice(&std::fs::read(sidecar).unwrap()).unwrap();
        let report: crate::bambu_project::BambuProjectReport =
            serde_json::from_value(summary["report"].clone()).unwrap();
        assert_eq!(report.output_sha256, sha256(&bytes));
        assert!(report.metadata_readback_verified);
        let model =
            serde_json::json!({"print_intent":{"source_document_id":report.source_document_id}})
                .to_string();
        let identity = VerificationIdentity::from_owned_export(
            &bytes,
            &model,
            &serde_json::json!({"placement":report.placement}),
            report.refresh_reference.profile_sha256,
            report.source_document_id,
            None,
        )
        .unwrap();
        let executable = std::env::var_os("LIMO_BAMBU_EXECUTABLE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:/Program Files/Bambu Studio/bambu-studio.exe"));
        let service = LocalSlicerService::default();
        let started = service
            .start(
                bytes,
                identity,
                report.template.plate_count as u32,
                LocalSlicerOptions {
                    executable,
                    timeout_seconds_per_plate: 60,
                },
                "qualification-owner".into(),
            )
            .unwrap();
        let deadline =
            Instant::now() + Duration::from_secs(65 * report.template.plate_count as u64);
        let result = loop {
            let result = service.poll(started.job_id, None).unwrap();
            if !matches!(
                result.state,
                VerificationState::Queued | VerificationState::Running
            ) {
                break result;
            }
            if Instant::now() >= deadline {
                service.cancel(started.job_id).unwrap();
                panic!("Installed-tool verification exceeded its outer bound");
            }
            thread::sleep(Duration::from_millis(50));
        };
        if let Some(output) = std::env::var_os("LIMO_BAMBU_VERIFY_EVIDENCE") {
            std::fs::write(output, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        }
        assert_eq!(
            result.state,
            VerificationState::Completed,
            "{}",
            serde_json::to_string_pretty(&result).unwrap()
        );
        assert!(result.plates.iter().all(|plate| plate.state
            == PlateVerificationState::ToolpathsGenerated
            && plate.exit_status == Some(0)
            && plate.slicer_version.as_deref() == Some(QUALIFIED_VERSION)
            && plate.toolpath_sha256.is_some()
            && plate.native_result.is_some()));
        assert_eq!(result.physical_qualification, "not_run");
    }
}
