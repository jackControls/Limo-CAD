//! Bounded semantic switching measurements on an owned disposable Xvfb desktop.
//! The same archived inputs can drive old React and new native executables.
//! Request completion is measured; GPU presentation and physical Alt+Tab are not.
use crate::{
    native_fixture::{controls, ui},
    native_platform_test::{wait_for_interface, wait_for_owned_window},
    replay::Client,
};
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

struct Options {
    server: PathBuf,
    out: PathBuf,
    models: [PathBuf; 2],
    shell: String,
    commit: String,
    profile: String,
    cycles: usize,
    instances: usize,
}
impl Options {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut values = HashMap::new();
        while let Some(key) = args.next() {
            ensure!(
                [
                    "--server",
                    "--out",
                    "--model-a",
                    "--model-b",
                    "--shell",
                    "--commit",
                    "--profile",
                    "--cycles",
                    "--instances"
                ]
                .contains(&key.as_str()),
                "Unknown switching option {key}"
            );
            let value = args
                .next()
                .with_context(|| format!("Missing {key} value"))?;
            ensure!(
                values.insert(key.clone(), value).is_none(),
                "Duplicate {key}"
            );
        }
        let required = |key| {
            values
                .get(key)
                .cloned()
                .with_context(|| format!("Use {key}"))
        };
        let cycles = values
            .get("--cycles")
            .map_or(Ok(20), |v| v.parse::<usize>())?;
        let instances = values
            .get("--instances")
            .map_or(Ok(1), |v| v.parse::<usize>())?;
        ensure!(
            (1..=50).contains(&cycles) && (1..=2).contains(&instances),
            "Use 1-50 cycles and 1-2 instances"
        );
        let shell = required("--shell")?;
        ensure!(
            shell == "react" || shell == "native",
            "--shell must be react or native"
        );
        let commit = required("--commit")?;
        ensure!(
            commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()),
            "--commit needs the exact 40-character build source SHA"
        );
        Ok(Self {
            server: PathBuf::from(required("--server")?).canonicalize()?,
            out: PathBuf::from(required("--out")?),
            models: [
                PathBuf::from(required("--model-a")?).canonicalize()?,
                PathBuf::from(required("--model-b")?).canonicalize()?,
            ],
            shell,
            commit,
            profile: required("--profile")?,
            cycles,
            instances,
        })
    }
}

fn verify_private_display() -> Result<()> {
    ensure!(
        cfg!(target_os = "linux"),
        "Switching measurements only launch on disposable Linux Xvfb"
    );
    let status = Command::new("python3")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("platform/native-drawing-linux.py"))
        .arg("--verify-private-display")
        .status()
        .context("Verify owned Xvfb")?;
    ensure!(
        status.success(),
        "The Xvfb display must belong to this fixture's process tree"
    );
    Ok(())
}
fn hash(path: &Path) -> Result<String> {
    let result = Command::new("sha256sum").arg("--").arg(path).output()?;
    ensure!(result.status.success(), "Could not hash {}", path.display());
    let text = String::from_utf8(result.stdout)?;
    let digest = text.split_whitespace().next().context("Missing SHA256")?;
    ensure!(
        digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid SHA256 output"
    );
    Ok(digest.into())
}
fn model(client: &mut Client) -> Result<Value> {
    let text = client.call("cad_project_model", json!({}))?;
    serde_json::from_str(text.as_str().context("Project model missing")?)
        .context("Parse project model")
}
fn reattach(client: &mut Client, response: &Value) -> Result<()> {
    let session = response["active_session_id"]
        .as_str()
        .context("Transition has no active session receipt")?;
    client.call("cad_attach", json!({"session_id":session}))?;
    Ok(())
}
fn target(client: &mut Client, matches: impl Fn(&Value) -> bool) -> Result<Value> {
    let observed = ui(client, json!({"action":"inspect"}))?;
    let found = controls(&observed)
        .filter(|c| c["disabled"] == false && matches(c))
        .collect::<Vec<_>>();
    ensure!(
        found.len() == 1,
        "Expected one enabled switching control, found {found:?}"
    );
    Ok(found[0].clone())
}
fn click(client: &mut Client, label: &str) -> Result<Value> {
    let control = target(client, |c| c["label"] == label)?;
    ui(client, json!({"action":"click", "target":control["id"]}))
}
fn proc_sample(pid: u32) -> Value {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    let columns = stat
        .rsplit_once(')')
        .map(|(_, tail)| tail.split_whitespace().collect::<Vec<_>>())
        .unwrap_or_default();
    let number = |index: usize| columns.get(index).and_then(|s| s.parse::<u64>().ok());
    json!({"user_ticks":number(11),"system_ticks":number(12),"rss_pages":number(21),
        "io":fs::read_to_string(format!("/proc/{pid}/io")).ok()})
}
struct Host {
    client: Client,
    labels: [String; 2],
    models: [Value; 2],
}
fn launch(options: &Options, index: usize, inputs: &[PathBuf; 2]) -> Result<Host> {
    let directory = options.out.join(format!("instance-{index}"));
    fs::create_dir(&directory)?;
    let sessions = options.out.join("sessions");
    let mut command = Command::new(&options.server);
    command
        .current_dir(&directory)
        .env("NBCAD_SESSION_DIR", &sessions)
        .env("NBCAD_CONFIG_DIR", options.out.join("config"));
    let mut client =
        Client::start_command_logged(command, Some(Duration::from_secs(45)), &directory)?;
    let session = wait_for_owned_window(&mut client, &sessions)?;
    client.call("cad_attach", json!({"session_id":session}))?;
    wait_for_interface(&mut client, &session)?;
    fs::write(
        directory.join("initial-ui.json"),
        serde_json::to_vec_pretty(&ui(&mut client, json!({"action":"inspect"}))?)?,
    )?;
    ensure!(
        client.call("cad_document", json!({}))?["features"]
            .as_array()
            .is_some_and(Vec::is_empty),
        "Owned startup document is not blank"
    );
    let mut labels = [String::new(), String::new()];
    let mut models = [Value::Null, Value::Null];
    for n in 0..2 {
        if n != 0 {
            let created = click(&mut client, "New design")?;
            reattach(&mut client, &created)?;
        }
        let opened = ui(
            &mut client,
            json!({"action":"file","command":"open","path":inputs[n]}),
        )?;
        reattach(&mut client, &opened)?;
        models[n] = model(&mut client)?;
        ensure!(
            models[n]["document"]["features"]
                .as_array()
                .is_some_and(|f| !f.is_empty())
                || client.call("cad_document", json!({}))?["features"]
                    .as_array()
                    .is_some_and(|f| !f.is_empty()),
            "Switching input needs a real nonempty design"
        );
        let document = client.call("cad_document", json!({}))?;
        labels[n] = if options.shell == "native" {
            document["name"]
                .as_str()
                .context("Loaded document name")?
                .into()
        } else {
            inputs[n]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        };
        fs::write(
            directory.join(format!("loaded-{n}.json")),
            serde_json::to_vec_pretty(&models[n])?,
        )?;
    }
    ensure!(
        labels[0] != labels[1],
        "Inputs must have distinct document names"
    );
    fs::write(
        directory.join("host.json"),
        serde_json::to_vec_pretty(
            &json!({"pid":client.process_id(),"exe":options.server,"labels":labels}),
        )?,
    )?;
    Ok(Host {
        client,
        labels,
        models,
    })
}

fn measure(
    options: &Options,
    raw: &mut fs::File,
    host: &mut Host,
    instance: usize,
    cycle: usize,
    n: usize,
    warmup: bool,
) -> Result<f64> {
    // Snapshot lookup is outside the timed action. Retained IDs are never
    // reused across another inspect or document transition.
    let selected = target(&mut host.client, |c| {
        c["label"] == host.labels[n]
            && if options.shell == "react" {
                c["role"] == "tab"
            } else {
                c["surface"] == "document/session"
            }
    })?;
    let before = proc_sample(host.client.process_id());
    let started = Instant::now();
    let result = ui(
        &mut host.client,
        json!({"action":"click","target":selected["id"]}),
    );
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    let after = proc_sample(host.client.process_id());
    let receipt = result.as_ref().map(|value| json!({"status":value["status"],"active_session_id":value["active_session_id"],
        "document_id":value["document_id"],"presented":value["presented"],"render_status":value["render_status"],
        "native_layout_revision":value["native_layout_revision"],"native_submitted_revision":value["native_submitted_revision"]})).unwrap_or(Value::Null);
    writeln!(
        raw,
        "{}",
        json!({"kind":"tab","instance":instance,"cycle":cycle,"target":n,"warmup":warmup,
        "elapsed_ms":elapsed_ms,"cpu_before":before,"cpu_after":after,"control":selected,"receipt":receipt,
        "error":result.as_ref().err().map(|e| format!("{e:#}"))})
    )?;
    raw.flush()?;
    let response = result?;
    reattach(&mut host.client, &response)?;
    let current = model(&mut host.client)?;
    if current != host.models[n] {
        fs::write(
            options
                .out
                .join(format!("changed-{instance}-{cycle}-{n}.json")),
            serde_json::to_vec_pretty(&current)?,
        )?;
        bail!("Switch changed the exact model in instance {instance}, cycle {cycle}, target {n}");
    }
    Ok(elapsed_ms)
}
fn statistics(values: &[f64]) -> Value {
    if values.is_empty() {
        return Value::Null;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile =
        |percent: usize| sorted[(sorted.len() * percent).div_ceil(100).saturating_sub(1)];
    let middle = sorted.len() / 2;
    let median = if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.
    } else {
        sorted[middle]
    };
    json!({"count":sorted.len(),"minimum_ms":sorted[0],"median_ms":median,"p95_ms":percentile(95),"maximum_ms":sorted[sorted.len()-1]})
}
pub(super) fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let mut options = Options::parse(args)?;
    verify_private_display()?; // Must precede even the first GUI child.
    ensure!(
        options.out.is_absolute()
            && (!options.out.exists() || fs::read_dir(&options.out)?.next().is_none()),
        "Choose an empty absolute evidence directory"
    );
    fs::create_dir_all(&options.out)?;
    options.out = options.out.canonicalize()?;
    fs::create_dir(options.out.join("sessions"))?;
    fs::create_dir(options.out.join("config"))?;
    let inputs = [
        options.out.join("Switch-A.nbcad"),
        options.out.join("Switch-B.nbcad"),
    ];
    for (source, destination) in options.models.iter().zip(&inputs) {
        let bytes = fs::read(source)?;
        let _ = crate::project_archive::model(&bytes)?;
        fs::write(destination, bytes)?;
    }
    let metadata = json!({"declared_commit":options.commit,"declared_build_profile":options.profile,"shell":options.shell,
        "binary_sha256":hash(&options.server)?,"input_sha256":[hash(&inputs[0])?,hash(&inputs[1])?],
        "cycles":options.cycles,"instances":options.instances,"warmup_cycles":2,"request_deadline_seconds":45,
        "measurement_budget_seconds":900,
        "requested_x11_scale":std::env::var("WINIT_X11_SCALE_FACTOR").ok(),"display":std::env::var("DISPLAY").ok(),
        "measurement":"semantic click request to application acknowledgment; inspect, attach, model verification and focus requests excluded",
        "not_proven":["physical input latency","compositor presentation","GPU time or memory","hardware/monitor DPI","user-reported cause"],
        "comparison_warning":"React acknowledges visible UI; native separately reports GPU submission. These are not equivalent presentation receipts."});
    fs::write(
        options.out.join("metadata.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;
    let mut raw = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(options.out.join("samples.jsonl"))?;
    let result = (|| {
        let mut hosts = (0..options.instances)
            .map(|i| launch(&options, i, &inputs))
            .collect::<Result<Vec<_>>>()?;
        let mut samples = vec![Vec::new(); options.instances];
        let deadline = Instant::now() + Duration::from_secs(900);
        for cycle in 0..options.cycles + 2 {
            for n in 0..2 {
                for (instance, host) in hosts.iter_mut().enumerate() {
                    ensure!(
                        Instant::now() < deadline,
                        "Switching measurement exceeded its 15-minute budget"
                    );
                    if options.instances == 2 {
                        let start = Instant::now();
                        let response = ui(
                            &mut host.client,
                            json!({"action":"window","mode":"foreground"}),
                        );
                        writeln!(
                            raw,
                            "{}",
                            json!({"kind":"foreground","instance":instance,"cycle":cycle,"target":n,"warmup":cycle<2,
                            "elapsed_ms":start.elapsed().as_secs_f64()*1000.,"receipt":response.as_ref().ok(),"error":response.as_ref().err().map(|e|format!("{e:#}"))})
                        )?;
                        raw.flush()?;
                        let foreground = response?;
                        ensure!(
                            foreground["value"]["focused"] == true
                                || foreground["window"]["focused"] == true,
                            "Owned instance did not report foreground focus: {foreground}"
                        );
                    }
                    let elapsed = measure(&options, &mut raw, host, instance, cycle, n, cycle < 2)?;
                    if cycle >= 2 {
                        samples[instance].push(elapsed);
                    }
                }
            }
        }
        Ok::<_, anyhow::Error>(
            json!({"completed":true,"exact_models_preserved":true,"performance_acceptance":"not established",
            "statistics":samples.iter().map(|s|statistics(s)).collect::<Vec<_>>()}),
        )
    })();
    let report = match &result {
        Ok(value) => value.clone(),
        Err(error) => {
            json!({"completed":false,"error":format!("{error:#}"),"performance_acceptance":"not established"})
        }
    };
    fs::write(
        options.out.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_summary_preserves_tail_and_empty_samples() {
        assert!(statistics(&[]).is_null());
        let samples = (1..=20).rev().map(|n| n as f64).collect::<Vec<_>>();
        assert_eq!(
            statistics(&samples),
            json!({"count":20,"minimum_ms":1.,"median_ms":10.5,"p95_ms":19.,"maximum_ms":20.})
        );
    }
}
