//! One host-independent entry point for the browser's Rust engine bundle.
use anyhow::{bail, Context, Result};
use std::{path::Path, process::Command};

pub fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let mut profile = None;
    for argument in args {
        match argument.as_str() {
            "--dev" | "--release" if profile.is_none() => profile = Some(argument),
            "--help" | "-h" => {
                println!("cargo xtask build-wasm [--dev|--release] (default: release)\nRequires wasm-pack and the wasm32-unknown-unknown Rust target.");
                return Ok(());
            }
            _ => bail!("Unknown or duplicate build-wasm option {argument}"),
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let status = Command::new("wasm-pack")
        .current_dir(root)
        .args([
            "build",
            "crates/wasm",
            "--target",
            "web",
            "--out-dir",
            "../../src/engine-wasm/pkg",
            "--out-name",
            "nbcad_wasm",
            // This bundle is imported directly; it is not an npm package.
            "--no-pack",
        ])
        .arg(profile.as_deref().unwrap_or("--release"))
        .args(["--", "--locked"])
        .status()
        .context("Run wasm-pack; install it with cargo install wasm-pack --locked")?;
    if !status.success() {
        bail!("Browser engine build failed ({status})");
    }
    Ok(())
}
