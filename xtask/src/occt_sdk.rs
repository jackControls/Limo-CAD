//! Reproducible OCCT 7.9 SDK build through a portable Rust entry point.
use anyhow::{bail, ensure, Context, Result};
use std::{
    env, fs,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};

const VERSION: &str = "7_9_3";
const SHA256: &str = "5ecf094ec6b12d5413dfb851d8c3590c354058aee556e32e408bdfbf8c357d57";
const MAX_SOURCE_BYTES: u64 = 256 * 1024 * 1024;
const SETTINGS: &[&str] = &[
    "CMAKE_BUILD_TYPE=Release",
    "BUILD_LIBRARY_TYPE=Shared",
    "BUILD_MODULE_FoundationClasses=ON",
    "BUILD_MODULE_ModelingData=ON",
    "BUILD_MODULE_ModelingAlgorithms=ON",
    "BUILD_MODULE_Visualization=ON",
    "BUILD_MODULE_ApplicationFramework=ON",
    "BUILD_MODULE_DataExchange=ON",
    "BUILD_MODULE_DETools=OFF",
    "BUILD_MODULE_Draw=OFF",
    "BUILD_DOC_Overview=OFF",
    "USE_TCL=OFF",
    "USE_TK=OFF",
    "USE_OPENGL=OFF",
    "USE_GLES2=OFF",
    "USE_FREETYPE=ON",
    "USE_FREEIMAGE=OFF",
    "USE_RAPIDJSON=OFF",
    "USE_DRACO=OFF",
    "USE_TBB=OFF",
    "USE_VTK=OFF",
];

struct Options {
    prefix: PathBuf,
    jobs: usize,
    dry_run: bool,
}
impl Options {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut prefix = None;
        let mut jobs = None;
        let mut dry_run = false;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--prefix" => {
                    ensure!(prefix.is_none(), "duplicate --prefix");
                    prefix = Some(PathBuf::from(args.next().context("missing --prefix path")?));
                }
                "--jobs" => {
                    ensure!(jobs.is_none(), "duplicate --jobs");
                    jobs = Some(args.next().context("missing --jobs")?.parse::<usize>()?);
                }
                "--dry-run" => dry_run = true,
                _ => bail!("use build-occt --prefix PATH [--jobs N] [--dry-run]"),
            }
        }
        let prefix = prefix.context("missing --prefix PATH")?;
        ensure!(!prefix.as_os_str().is_empty(), "empty install prefix");
        let prefix = if prefix.is_absolute() {
            prefix
        } else {
            env::current_dir()?.join(prefix)
        };
        let jobs = jobs
            .or(env::var("CMAKE_BUILD_PARALLEL_LEVEL")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<usize>())
                .transpose()?)
            .unwrap_or(std::thread::available_parallelism().map_or(1, usize::from));
        ensure!(jobs > 0, "--jobs must be greater than zero");
        Ok(Self {
            prefix,
            jobs,
            dry_run,
        })
    }
}

fn configure(options: &Options, source: &std::path::Path, build: &std::path::Path) -> Command {
    let mut command = Command::new("cmake");
    command
        .arg("-S")
        .arg(source)
        .arg("-B")
        .arg(build)
        .args(["-G", "Ninja"])
        .arg(format!("-DINSTALL_DIR={}", options.prefix.display()))
        .arg("-DINSTALL_DIR_LAYOUT=Unix");
    for setting in SETTINGS {
        command.arg(format!("-D{setting}"));
    }
    command
}
fn run_command(command: &mut Command) -> Result<()> {
    let status = command
        .status()
        .with_context(|| format!("start {command:?}"))?;
    ensure!(status.success(), "{command:?} failed ({status})");
    Ok(())
}

pub fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let options = Options::parse(args)?;
    let url =
        format!("https://github.com/Open-Cascade-SAS/OCCT/archive/refs/tags/V{VERSION}.tar.gz");
    if options.dry_run {
        println!(
            "OCCT {} source {url}\nSHA256 {SHA256}\n{:?}\nJobs: {}",
            VERSION.replace('_', "."),
            configure(
                &options,
                std::path::Path::new("SOURCE"),
                std::path::Path::new("BUILD")
            ),
            options.jobs
        );
        return Ok(());
    }
    let work = tempfile::tempdir()?;
    let archive = work.path().join("occt.tar.gz");
    let config = ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(Duration::from_secs(120)))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(&url)
        .header("User-Agent", "noBS-CAD-OCCT-SDK")
        .call()?;
    let bytes = std::io::copy(
        &mut response.body_mut().as_reader().take(MAX_SOURCE_BYTES + 1),
        &mut fs::File::create(&archive)?,
    )?;
    ensure!(
        bytes <= MAX_SOURCE_BYTES,
        "OCCT source archive exceeds the 256 MB limit"
    );
    ensure!(
        crate::hash::file(&archive)? == SHA256,
        "OCCT source checksum differs; refusing extraction"
    );
    tar::Archive::new(flate2::read::GzDecoder::new(fs::File::open(archive)?))
        .unpack(work.path())?;
    let source = work.path().join(format!("OCCT-{VERSION}"));
    ensure!(
        source.join("CMakeLists.txt").is_file(),
        "missing OCCT source tree"
    );
    let build = work.path().join("build");
    run_command(&mut configure(&options, &source, &build))?;
    run_command(
        Command::new("cmake")
            .arg("--build")
            .arg(&build)
            .arg("--parallel")
            .arg(options.jobs.to_string()),
    )?;
    run_command(Command::new("cmake").arg("--install").arg(&build))?;
    let doc = options.prefix.join("share/doc/opencascade");
    fs::create_dir_all(&doc)?;
    let mut copyright = fs::File::create(doc.join("copyright"))?;
    writeln!(copyright,"Open CASCADE Technology {}\nhttps://github.com/Open-Cascade-SAS/OCCT\nCopyright (c) Open CASCADE SAS\n\nOCCT is distributed under the GNU Lesser General Public License version 2.1\nwith the following additional exception.\n",VERSION.replace('_',"."))?;
    std::io::copy(
        &mut fs::File::open(source.join("OCCT_LGPL_EXCEPTION.txt"))?,
        &mut copyright,
    )?;
    fs::copy(source.join("LICENSE_LGPL_21.txt"), doc.join("LGPL-2.1.txt"))?;
    println!(
        "Installed OCCT {} into {}",
        VERSION.replace('_', "."),
        options.prefix.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sdk_recipe_keeps_abi_modules_and_literal_paths_without_a_shell() {
        let options = Options::parse(
            ["--prefix", "SDK path with spaces", "--jobs", "2"]
                .map(str::to_owned)
                .into_iter(),
        )
        .unwrap();
        let command = configure(
            &options,
            std::path::Path::new("source path"),
            std::path::Path::new("build path"),
        );
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(args.contains(&"source path".into()) && args.contains(&"build path".into()));
        assert!(args.contains(&format!("-DINSTALL_DIR={}", options.prefix.display())));
        for module in ["DataExchange", "Visualization", "ApplicationFramework"] {
            assert!(args.contains(&format!("-DBUILD_MODULE_{module}=ON")));
        }
        for module in ["Draw", "DETools"] {
            assert!(args.contains(&format!("-DBUILD_MODULE_{module}=OFF")));
        }
        assert_eq!(options.jobs, 2);
        assert!(Options::parse(
            ["--prefix", "test", "--jobs", "0"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err());
        assert!(Options::parse(
            ["--prefix", "test", "--prefix", "another"]
                .map(str::to_owned)
                .into_iter()
        )
        .is_err());
    }
}
