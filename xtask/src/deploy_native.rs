//! Build the desktop once, then use that installed executable for both GUI and MCP.
use crate::package::{ordinary_directory, ordinary_file};
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const MANIFEST: &str = "runtime-manifest.json";
const EXECUTABLE: &str = if cfg!(windows) {
    "Limo-CAD.exe"
} else {
    "Limo-CAD"
};

#[derive(Debug, Default)]
pub(crate) struct BuildOptions {
    pub restart: bool,
    pub release: bool,
    pub occt_root: Option<PathBuf>,
    pub jobs: Option<usize>,
    rebind_source: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Source {
    checkout: PathBuf,
    revision: String,
    modified: bool,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
struct Payload {
    path: PathBuf,
    sha256: String,
}

#[derive(Serialize, Deserialize)]
struct RuntimeManifest {
    schema: u32,
    source: Source,
    build: String,
    occt_sdk: PathBuf,
    executable: String,
    executable_sha256: String,
    installed_unix_seconds: u64,
    payload: Vec<Payload>,
}

/// The only installed application executable; lookup never falls back to Cargo output.
pub(crate) fn installed_executable() -> Result<PathBuf> {
    let owner = if cfg!(windows) {
        PathBuf::from(env::var_os("LOCALAPPDATA").context("LOCALAPPDATA is missing")?)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(env::var_os("HOME").context("HOME is missing")?)
            .join("Library/Application Support")
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".local/share")
            })
    };
    ensure!(
        owner.is_absolute(),
        "Local application data directory must be absolute"
    );
    Ok(owner.join("limo-cad/bevy").join(EXECUTABLE))
}

pub(crate) fn managed_install_exists() -> Result<bool> {
    Ok(installed_executable()?.with_file_name(MANIFEST).is_file())
}

/// Return the managed checkout without rejecting pending changes that must be rebuilt.
pub(crate) fn managed_source_checkout() -> Result<PathBuf> {
    let _deployment = lock_runtime()?;
    let manifest = read_manifest()?;
    let checkout = manifest.source.checkout;
    ensure!(
        checkout.is_absolute(),
        "Managed source checkout must be absolute"
    );
    ordinary_directory(&checkout)?;
    ensure!(
        checkout.canonicalize()? == checkout,
        "Managed source checkout is redirected"
    );
    let top = String::from_utf8(git(&checkout, &["rev-parse", "--show-toplevel"])?)?;
    ensure!(Path::new(top.trim()).canonicalize()? == checkout,
        "Managed source checkout is no longer a Git checkout root; explicitly deploy-native from the intended checkout");
    Ok(checkout)
}

fn read_manifest() -> Result<RuntimeManifest> {
    let path = installed_executable()?.with_file_name(MANIFEST);
    ordinary_file(&path)
        .context("No ordinary managed runtime manifest; run cargo xtask deploy-native")?;
    let manifest: RuntimeManifest = serde_json::from_slice(&fs::read(path)?)?;
    ensure!(
        manifest.schema == 1 && manifest.executable == EXECUTABLE,
        "Unsupported local runtime manifest"
    );
    Ok(manifest)
}

/// Verify provenance and every installed payload hash before selecting a managed runtime.
pub(crate) fn verify_installed_runtime() -> Result<PathBuf> {
    let _deployment = lock_runtime()?;
    verify_runtime()
}

fn verify_runtime() -> Result<PathBuf> {
    let executable = installed_executable()?;
    let directory = executable.parent().context("installed executable parent")?;
    ordinary_directory(directory)?;
    let manifest = read_manifest()?;
    ensure!(
        snapshot(&manifest.source.checkout)? == manifest.source,
        "Installed runtime is stale for its source checkout; run cargo xtask deploy-native"
    );
    ensure!(
        manifest
            .payload
            .iter()
            .any(|file| file.path == Path::new(EXECUTABLE)),
        "Runtime manifest omits its executable"
    );
    for file in &manifest.payload {
        ensure!(safe_relative(&file.path), "Unsafe runtime payload path");
        ordinary_file(&directory.join(&file.path))?;
        ensure!(
            crate::hash::file(&directory.join(&file.path))? == file.sha256,
            "Installed runtime changed: {}; run cargo xtask deploy-native",
            file.path.display()
        );
    }
    ensure!(
        crate::hash::file(&executable)? == manifest.executable_sha256,
        "Installed executable does not match its provenance"
    );
    require_one_executable(directory)?;
    Ok(executable)
}

/// Build first and promote the compiler-reported artifact, without configuring clients.
/// A live installed runtime blocks promotion unless `restart` explicitly discards its work.
pub(crate) fn prepare_runtime(repo_root: &Path, options: &BuildOptions) -> Result<PathBuf> {
    ensure!(
        cfg!(windows),
        "deploy-native currently stages Windows runtimes; Unix runtime bundling is not implemented"
    );
    let _deployment = lock_runtime()?;
    let source = snapshot(repo_root)?;
    let marker = installed_executable()?.with_file_name(MANIFEST);
    match fs::symlink_metadata(marker) {
        Ok(_) if !options.rebind_source => ensure!(read_manifest()?.source.checkout == source.checkout,
            "Automatic builds must use the managed source checkout; only explicit deploy-native may rebind it"),
        Ok(_) => {},
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(error) => return Err(error.into()),
    }
    let sdk = windows_sdk(&source.checkout, options)?;
    let (artifact, artifact_sha256) = build(&source, options, &sdk)?;
    ensure!(
        snapshot(&source.checkout)? == source,
        "Source changed during the build; nothing was installed. Run deploy-native again"
    );
    let executable = installed_executable()?;
    let directory = executable.parent().context("installed executable parent")?;
    let owner = directory.parent().context("installation owner")?;
    let stage = tempfile::Builder::new()
        .prefix(".native-stage-")
        .tempdir_in(owner)?;
    crate::package::stage_runtime(
        &source.checkout,
        &artifact,
        &sdk,
        &crate::package::runtime_bin(&sdk)?,
        stage.path(),
    )?;
    let payload = payload(stage.path())?;
    let executable_sha256 = crate::hash::file(&stage.path().join(EXECUTABLE))?;
    ensure!(
        executable_sha256 == artifact_sha256,
        "Cargo artifact changed during staging"
    );
    let manifest = RuntimeManifest {
        schema: 1,
        source,
        build: if options.release {
            "release"
        } else {
            "incremental-local-opt1-third-party-opt3"
        }
        .into(),
        occt_sdk: sdk.clone(),
        executable: EXECUTABLE.into(),
        executable_sha256,
        installed_unix_seconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        payload,
    };
    fs::write(
        stage.path().join(MANIFEST),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    ensure!(
        snapshot(&manifest.source.checkout)? == manifest.source,
        "Source changed while staging; nothing was installed"
    );
    create_directory(directory)?;
    require_one_executable(directory)?;
    let unchanged = manifest
        .payload
        .iter()
        .try_fold(true, |same, file| -> Result<bool> {
            let path = directory.join(&file.path);
            if !path.exists() {
                return Ok(false);
            }
            ordinary_file(&path)?;
            Ok(same && crate::hash::file(&path)? == file.sha256)
        })?;
    if !unchanged {
        stop_runtime(&executable, options.restart)?;
        for file in &manifest.payload {
            if file.path != Path::new(EXECUTABLE) {
                replace_file(&stage.path().join(&file.path), &directory.join(&file.path))?;
            }
        }
        replace_file(&stage.path().join(EXECUTABLE), &executable)?;
    }
    replace_file(&stage.path().join(MANIFEST), &directory.join(MANIFEST))?;
    let installed = verify_runtime()?;
    ensure!(snapshot(&manifest.source.checkout)? == manifest.source, "Source changed during promotion; runtime was installed but will not be launched. Run deploy-native again");
    println!(
        "Installed {} from {} (sha256 {})",
        installed.display(),
        manifest.source.revision,
        manifest.executable_sha256
    );
    Ok(installed)
}

pub(crate) fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    let mut options = BuildOptions::default();
    let mut clients = None;
    let mut launch = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--restart" if !options.restart => options.restart = true,
            "--release" if !options.release => options.release = true,
            "--launch" if !launch => launch = true,
            "--occt-root" if options.occt_root.is_none() => {
                options.occt_root = Some(args.next().context("--occt-root needs a path")?.into())
            }
            "--jobs" if options.jobs.is_none() => {
                let jobs = args
                    .next()
                    .context("--jobs needs a positive count")?
                    .parse()?;
                ensure!(jobs > 0, "--jobs must be positive");
                options.jobs = Some(jobs);
            }
            "--clients" if clients.is_none() => {
                clients = Some(
                    args.next()
                        .context("--clients needs a comma-separated list")?,
                )
            }
            "--help" | "-h" => {
                println!("cargo xtask deploy-native [--restart] [--release] [--occt-root PATH] [--jobs N] [--clients LIST] [--launch]\n\nBuilds and installs one Windows GUI/MCP executable. Default: incremental local crates opt1, third-party dependencies opt3.\n--restart terminates only the canonical installed runtime, discarding unsaved work.\nClient registration defaults to codex,cursor,vscode,claude,opencode. Launch occurs after verification and registration.");
                return Ok(());
            }
            _ => bail!("Unknown or duplicate deploy-native option {arg}"),
        }
    }
    let clients = clients.unwrap_or_else(|| "codex,cursor,vscode,claude,opencode".into());
    options.rebind_source = true;
    let binary = installed_executable()?;
    let install = crate::install_mcp::Options::parse(
        [
            "--clients".into(),
            clients,
            "--binary".into(),
            binary.to_string_lossy().into_owned(),
            "--in-place".into(),
            "--server-arg".into(),
            "--headless".into(),
            "--desktop".into(),
            binary.to_string_lossy().into_owned(),
        ]
        .into_iter(),
    )?;
    let executable = prepare_runtime(crate::build_tools::root(), &options)?;
    crate::install_mcp::run(install)?;
    if launch {
        let mut command = Command::new(&executable);
        command
            .current_dir(executable.parent().context("installed runtime parent")?)
            .env("LIMO_CAD_LOCAL_RUNTIME", &executable)
            .env("LIMO_CAD_DESKTOP_BIN", &executable);
        if cfg!(windows) {
            let temporary =
                PathBuf::from(env::var_os("LOCALAPPDATA").context("LOCALAPPDATA")?).join("Temp");
            command.env("TEMP", &temporary).env("TMP", &temporary);
        }
        command.spawn().context("Launch installed CAD")?;
    }
    Ok(())
}

fn windows_sdk(root: &Path, options: &BuildOptions) -> Result<PathBuf> {
    let previous_sdk = installed_executable()
        .ok()
        .and_then(|path| fs::read(path.with_file_name(MANIFEST)).ok())
        .and_then(|bytes| serde_json::from_slice::<RuntimeManifest>(&bytes).ok())
        .filter(|manifest| manifest.schema == 1)
        .map(|manifest| manifest.occt_sdk);
    let roots = crate::build_tools::sdk::roots(
        "windows",
        env::consts::ARCH,
        root,
        options
            .occt_root
            .clone()
            .or_else(|| env::var_os("OCCT_ROOT").map(PathBuf::from))
            .or(previous_sdk),
        env::var_os("VCPKG_INSTALLED_DIR").map(PathBuf::from),
        env::var("VCPKG_TARGET_TRIPLET").ok(),
    )
    .map_err(anyhow::Error::msg)?;
    let sdk = roots
        .into_iter()
        .find(|path| crate::package::runtime_bin(path).is_ok())
        .context("No Windows OCCT runtime SDK; use --occt-root PATH")?
        .canonicalize()?;
    crate::build_tools::sdk::resolve(
        std::slice::from_ref(&sdk),
        "windows",
        env::consts::ARCH,
        None,
    )
    .map_err(anyhow::Error::msg)?;
    Ok(sdk)
}

fn build(source: &Source, options: &BuildOptions, sdk: &Path) -> Result<(PathBuf, String)> {
    let target = match env::consts::ARCH {
        "x86_64" => "x86_64-pc-windows-msvc",
        "aarch64" => "aarch64-pc-windows-msvc",
        arch => bail!("Unsupported Windows deployment architecture {arch}"),
    };
    let mut command = crate::build_tools::cargo();
    command.current_dir(&source.checkout).args([
        "build",
        "--locked",
        "--release",
        "--manifest-path",
        "desktop/Cargo.toml",
        "--bin",
        "limo-cad",
        "--message-format=json-render-diagnostics",
        "--config",
        "profile.release.incremental=true",
        "--config",
        "profile.release.opt-level=3",
    ]);
    command.args(["--target", target]);
    if !options.release {
        for package in local_packages(&source.checkout)? {
            command.args([
                "--config",
                &format!("profile.release.package.{package}.opt-level=1"),
            ]);
        }
    }
    if let Some(jobs) = options.jobs {
        command.args(["--jobs", &jobs.to_string()]);
    }
    command
        .env("OCCT_ROOT", sdk)
        .env("LIMO_CAD_BUILD_REVISION", &source.revision)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    let mut child = command.spawn().context("Start locked desktop build")?;
    let mut artifact = None;
    let output = child.stdout.take().context("Cargo stdout")?;
    let read = (|| -> Result<()> {
        for line in BufReader::new(output).lines() {
            let line = line?;
            let Ok(message) = serde_json::from_str::<serde_json::Value>(&line) else {
                eprintln!("{line}");
                continue;
            };
            if message["reason"] == "compiler-artifact" && message["target"]["name"] == "limo-cad" {
                if let Some(path) = message["executable"].as_str() {
                    let path = PathBuf::from(path);
                    ordinary_file(&path)?;
                    let sha256 = crate::hash::file(&path)?;
                    artifact = Some((path, sha256));
                }
            }
            if let Some(rendered) = message["message"]["rendered"].as_str() {
                eprint!("{rendered}");
            }
        }
        Ok(())
    })();
    if let Err(error) = read {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    ensure!(
        child.wait()?.success(),
        "Desktop build failed; installed runtime was preserved"
    );
    let artifact = artifact.context("Cargo did not report its desktop executable")?;
    Ok(artifact)
}

fn local_packages(root: &Path) -> Result<Vec<String>> {
    let mut manifests = vec![
        root.join("desktop/Cargo.toml"),
        root.join("mcp-server/Cargo.toml"),
    ];
    for entry in fs::read_dir(root.join("crates"))? {
        let path = entry?.path().join("Cargo.toml");
        if path.is_file() {
            manifests.push(path);
        }
    }
    let mut packages = Vec::new();
    for path in manifests {
        ordinary_file(&path)?;
        let manifest = fs::read_to_string(&path)?.parse::<toml_edit::DocumentMut>()?;
        let package = manifest["package"]["name"]
            .as_str()
            .context("Local crate name")?;
        ensure!(
            package
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "Unsupported local package name"
        );
        packages.push(package.to_owned());
    }
    packages.sort();
    packages.dedup();
    Ok(packages)
}

/// The kernel releases this guard on close or process exit; no stale-PID deletion is needed.
fn lock_runtime() -> Result<fs::File> {
    let executable = installed_executable()?;
    let owner = executable
        .parent()
        .and_then(Path::parent)
        .context("runtime installation owner")?;
    create_directory(owner)?;
    let path = owner.join(".native-deploy.lock");
    let file = match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ordinary_file(&path)?;
            fs::OpenOptions::new().read(true).write(true).open(&path)?
        }
        Err(error) => return Err(error.into()),
    };
    ordinary_file(&path)?;
    file.try_lock().context(
        "Another local CAD build or promotion holds the runtime lock; retry after it finishes",
    )?;
    Ok(file)
}

fn create_directory(path: &Path) -> Result<()> {
    let mut existing = path;
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => {
                ordinary_directory(existing)?;
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                existing = existing
                    .parent()
                    .context("Runtime path has no existing owner")?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    fs::create_dir_all(path)?;
    ordinary_directory(path)
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .output()?;
    ensure!(
        output.status.success(),
        "Git source snapshot failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

fn snapshot(root: &Path) -> Result<Source> {
    let checkout = root.canonicalize()?;
    let top = String::from_utf8(git(&checkout, &["rev-parse", "--show-toplevel"])?)?;
    ensure!(
        Path::new(top.trim()).canonicalize()? == checkout,
        "Deployment requires the source checkout root"
    );
    let revision = String::from_utf8(git(&checkout, &["rev-parse", "HEAD"])?)?
        .trim()
        .to_owned();
    ensure!(
        revision.len() == 40 && revision.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid Git HEAD"
    );
    let diff = git(
        &checkout,
        &["diff", "--binary", "--no-ext-diff", "HEAD", "--"],
    )?;
    let files = git(
        &checkout,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    let mut digest = Sha256::new();
    digest.update(revision.as_bytes());
    digest.update(&diff);
    let mut names = String::from_utf8(files)?
        .split('\0')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    for name in names {
        let relative = Path::new(&name);
        ensure!(safe_relative(relative), "Unsafe source path in Git index");
        digest.update(name.as_bytes());
        digest.update([0]);
        let path = checkout.join(relative);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                digest.update(fs::read_link(path)?.as_os_str().as_encoded_bytes())
            }
            Ok(metadata) if metadata.is_file() => {
                digest.update(crate::hash::file(&path)?.as_bytes())
            }
            Ok(_) => bail!("Source path is not a file: {}", path.display()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => digest.update(b"deleted"),
            Err(error) => return Err(error.into()),
        }
        digest.update([0]);
    }
    let modified = !git(
        &checkout,
        &["status", "--porcelain", "--untracked-files=all"],
    )?
    .is_empty();
    Ok(Source {
        checkout,
        revision,
        modified,
        sha256: crate::hash::hex(&digest.finalize()),
    })
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn require_one_executable(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            ensure!(entry.file_name() == EXECUTABLE, "Retired executable alias remains at {}; remove the audited alias before deployment", entry.path().display());
        }
    }
    Ok(())
}

fn payload(directory: &Path) -> Result<Vec<Payload>> {
    fn visit(owner: &Path, directory: &Path, files: &mut Vec<Payload>) -> Result<()> {
        ordinary_directory(directory)?;
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                visit(owner, &path, files)?;
            } else {
                ordinary_file(&path)?;
                files.push(Payload {
                    path: path.strip_prefix(owner)?.into(),
                    sha256: crate::hash::file(&path)?,
                });
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(directory, directory, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn replace_file(source: &Path, destination: &Path) -> Result<()> {
    let owner = destination.parent().context("payload owner")?;
    create_directory(owner)?;
    if destination.exists() {
        ordinary_file(destination)?;
    }
    let mut temporary = tempfile::NamedTempFile::new_in(owner)?;
    std::io::copy(&mut fs::File::open(source)?, &mut temporary)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(destination)
        .map_err(|error| error.error)
        .with_context(|| {
            format!(
                "Cannot replace {}; close the installed CAD/MCP runtime or use --restart",
                destination.display()
            )
        })?;
    Ok(())
}

fn stop_runtime(executable: &Path, restart: bool) -> Result<()> {
    let Ok(canonical) = executable.canonicalize() else {
        return Ok(());
    };
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    let processes = system
        .processes()
        .iter()
        .filter(|(_, process)| {
            process
                .exe()
                .and_then(|path| path.canonicalize().ok())
                .is_some_and(|path| path == canonical)
        })
        .map(|(pid, process)| (*pid, process.start_time()))
        .collect::<Vec<_>>();
    ensure!(
        processes.is_empty() || restart,
        "Installed CAD/MCP is running; close it or pass --restart to discard unsaved work"
    );
    for (pid, start) in &processes {
        let process = system
            .process(*pid)
            .context("Runtime process disappeared")?;
        ensure!(
            process.start_time() == *start && process.kill(),
            "Could not stop installed runtime PID {pid}"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        system.refresh_processes(ProcessesToUpdate::All, true);
        if processes.iter().all(|(pid, start)| {
            system
                .process(*pid)
                .is_none_or(|process| process.start_time() != *start)
        }) {
            return Ok(());
        }
        ensure!(
            Instant::now() < deadline,
            "Installed runtime did not stop; nothing was promoted"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
