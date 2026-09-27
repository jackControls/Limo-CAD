//! Disposable owned-window launcher shared with the established platform
//! input handshake. The existing drawing fixture keeps its exact PID proof.
use super::*;
use anyhow::bail;
use std::{ffi::OsString, process::Command};

struct PrivateEnvironment(Vec<(&'static str, Option<OsString>)>);
pub(super) fn verify_display() -> Result<()> {
    let status = Command::new("python3")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("platform/native-drawing-linux.py"))
        .arg("--verify-private-display")
        .status()
        .context("Verify private drawing Xvfb display")?;
    ensure!(
        status.success(),
        "Drawing input requires a private Xvfb display owned by this fixture"
    );
    Ok(())
}
impl PrivateEnvironment {
    fn set(root: &Path) -> Self {
        let values = [
            ("NBCAD_SESSION_DIR", root.join("sessions")),
            ("NBCAD_CONFIG_DIR", root.join("config")),
        ];
        let mut saved = Vec::new();
        for (key, value) in values {
            saved.push((key, std::env::var_os(key)));
            std::env::set_var(key, value);
        }
        Self(saved)
    }
}
impl Drop for PrivateEnvironment {
    fn drop(&mut self) {
        for (key, prior) in self.0.drain(..) {
            if let Some(value) = prior {
                std::env::set_var(key, value)
            } else {
                std::env::remove_var(key)
            }
        }
    }
}

pub(in super::super) fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut server = None;
    let mut out = None;
    let mut desktop = false;
    let mut authoring = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--server" => {
                server = Some(PathBuf::from(args.next().context("Missing --server path")?))
            }
            "--out" => out = Some(PathBuf::from(args.next().context("Missing --out path")?)),
            "--desktop-input" => desktop = true,
            "--authoring-input" => authoring = true,
            _ => bail!("Unknown owned drawing option {arg}"),
        }
    }
    ensure!(
        desktop && cfg!(target_os = "linux"),
        "Use native-drawing-platform --desktop-input only on the disposable Linux Xvfb runner"
    );
    verify_display()?;
    let server = server
        .context("Use --server for the dev-bevy-host binary")?
        .canonicalize()?;
    let out = out.context("Use --out for an empty evidence root")?;
    ensure!(
        out.is_absolute() && (!out.exists() || fs::read_dir(&out)?.next().is_none()),
        "Choose a fresh absolute evidence root; preserve previous results"
    );
    fs::create_dir_all(&out)?;
    let out = out.canonicalize()?;
    let _environment = PrivateEnvironment::set(&out);
    let sessions = out.join("sessions");
    fs::create_dir(&sessions)?;
    let result = (|| {
        let mut command = Command::new(&server);
        command.current_dir(&sessions);
        let mut host = Client::start_command(command, Some(Duration::from_secs(45)))?;
        fs::write(
            out.join("host.json"),
            serde_json::to_vec_pretty(&json!({"pid":host.process_id(),"exe":server}))?,
        )?;
        let session = crate::native_platform_test::wait_for_owned_window(&mut host, &sessions)?;
        host.call("cad_attach", json!({"session_id":session}))?;
        crate::native_platform_test::wait_for_interface(&mut host, &session)?;
        fs::write(out.join("session.txt"), &session)?;
        // Keep the owned GUI client alive while the established blank-document
        // fixture attaches its headless transport through the private registry.
        let mut fixture_args = vec![
            "--server".into(),
            server.to_string_lossy().into_owned(),
            "--session".into(),
            session,
            "--out".into(),
            out.join("evidence").to_string_lossy().into_owned(),
            "--desktop-input".into(),
        ];
        if authoring {
            fixture_args.push("--authoring-input".into());
        }
        crate::native_drawing_annotations_test::run_navigation(fixture_args.into_iter())?;
        ensure!(
            host.is_running()?,
            "Owned native host exited during drawing input validation"
        );
        Ok::<_, anyhow::Error>(
            json!({"status":"passed","platform":std::env::consts::OS,"pid":host.process_id(),
            "evidence":"evidence/native-drawing-navigation.json","annotation_os_input":authoring,
            "not_proven":["Touchpad pinch","Monitor DPI transition","Wayland","macOS paper gestures"]}),
        )
    })();
    let report = match &result {
        Ok(report) => report.clone(),
        Err(error) => {
            json!({"status":"failed","error":format!("{error:#}"),"platform":std::env::consts::OS})
        }
    };
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    result.map(|_| ())
}
