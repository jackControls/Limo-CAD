//! Qualify checked matrix allocation/copy behavior in the actual OCCT runtime.
use anyhow::{ensure, Context, Result};
use std::{env, fs, path::Path, process::Command};

pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    ensure!(
        args.next().as_deref() == Some("--prefix"),
        "use verify-occt-storage --prefix SDK"
    );
    let prefix = args.next().context("missing SDK path")?;
    ensure!(
        args.next().is_none(),
        "unexpected storage verification argument"
    );
    verify(Path::new(&prefix))
}

pub fn verify(prefix: &Path) -> Result<()> {
    let sdk = crate::build_tools::sdk::resolve(
        &[prefix.to_path_buf()],
        env::consts::OS,
        env::consts::ARCH,
        None,
    )
    .map_err(anyhow::Error::msg)?;
    verify_header(&sdk.include)?;
    let staging = tempfile::tempdir()?;
    let source = staging.path().join("source");
    fs::create_dir(&source)?;
    for (name, content) in [
        (
            "CMakeLists.txt",
            include_bytes!("../../native/occt-overlay/CMakeLists.txt").as_slice(),
        ),
        (
            "math-double-tab-check.cxx",
            include_bytes!("../../native/occt-overlay/math-double-tab-check.cxx").as_slice(),
        ),
    ] {
        fs::write(source.join(name), content)?;
    }
    let build = staging.path().join("build");
    crate::build_tools::run(
        Command::new("cmake")
            .arg("-S")
            .arg(&source)
            .arg("-B")
            .arg(&build)
            .args(["-G", "Ninja", "-DCMAKE_BUILD_TYPE=Release"])
            .arg(format!("-DOCCT_INCLUDE={}", sdk.include.display()))
            .arg(format!("-DOCCT_LIB={}", sdk.lib.display())),
    )?;
    crate::build_tools::run(
        Command::new("cmake")
            .arg("--build")
            .arg(&build)
            .args(["--parallel", "1"]),
    )?;
    let mut probe = Command::new(build.join(if cfg!(windows) {
        "limo_occt_storage_check.exe"
    } else {
        "limo_occt_storage_check"
    }));
    let variable = if cfg!(windows) {
        "PATH"
    } else if cfg!(target_os = "macos") {
        "DYLD_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    };
    let mut search = vec![
        prefix.join("bin"),
        sdk.lib.parent().context("OCCT library parent")?.join("bin"),
        sdk.lib,
    ];
    if let Some(existing) = env::var_os(variable) {
        search.extend(env::split_paths(&existing));
    }
    probe.env(variable, env::join_paths(search)?);
    crate::build_tools::run(&mut probe)
}

pub fn verify_header(include: &Path) -> Result<()> {
    let expected = include_str!("../../native/occt-overlay/opencascade/math_DoubleTab.lxx")
        .replace("\r\n", "\n");
    let installed = fs::read_to_string(include.join("math_DoubleTab.lxx"))?
        .replace("\r\n", "\n")
        .replace(
            "#include \"Standard_OutOfRange.hxx\"",
            "#include <Standard_OutOfRange.hxx>",
        );
    ensure!(installed == expected, "OCCT SDK lacks the reviewed checked matrix header; rebuild with cargo xtask build-occt or the pinned vcpkg overlay");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_header_accepts_vcpkg_formatting_and_rejects_changed_storage() {
        let staging = tempfile::tempdir().unwrap();
        let header = staging.path().join("math_DoubleTab.lxx");
        let reviewed = include_str!("../../native/occt-overlay/opencascade/math_DoubleTab.lxx")
            .replace("\r\n", "\n");
        fs::write(&header, &reviewed).unwrap();
        verify_header(staging.path()).unwrap();
        let packaged = reviewed
            .replace(
                "#include <Standard_OutOfRange.hxx>",
                "#include \"Standard_OutOfRange.hxx\"",
            )
            .replace('\n', "\r\n");
        fs::write(&header, packaged).unwrap();
        verify_header(staging.path()).unwrap();
        fs::write(
            &header,
            reviewed.replace("const std::size_t count", "const int count"),
        )
        .unwrap();
        assert!(verify_header(staging.path()).is_err());
        fs::write(&header, "old storage header").unwrap();
        assert!(verify_header(staging.path()).is_err());
    }
}
