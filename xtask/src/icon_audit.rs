//! Product icon registry/provenance validation, independent of npm installation.
use anyhow::{ensure, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs};

pub fn run(mut args: impl Iterator<Item = String>) -> Result<()> {
    ensure!(args.next().is_none(), "audit-icons takes no arguments");
    let root = crate::release_tooling::root();
    let source = fs::read_to_string(root.join("src/components/icons.tsx"))?.replace("\r\n", "\n");
    let cam =
        fs::read_to_string(root.join("src/components/cam/CamToolIcon.tsx"))?.replace("\r\n", "\n");
    let brand = fs::read_to_string(root.join("public/app-icon.svg"))?.replace("\r\n", "\n");
    let provenance =
        fs::read_to_string(root.join("docs/ICON_PROVENANCE.md"))?.replace("\r\n", "\n");
    let block = |source: &str, prefix: &str, suffix: &str| -> Result<String> {
        Ok(source
            .split_once(prefix)
            .and_then(|(_, rest)| rest.split_once(suffix))
            .ok_or_else(|| anyhow::anyhow!("missing icon inventory block {prefix}"))?
            .0
            .to_owned())
    };
    let glyphs = block(
        &source,
        "const GLYPHS: Record<string, ReactNode> = {",
        "\n};\n\n/** Stable inventory",
    )?;
    let cam_glyphs = block(
        &cam,
        "const CAM_GLYPHS: Record<CamIconId, ReactNode> = {",
        "\n};\n\n/** Stable inventory",
    )?;
    let inventory = block(
        &provenance,
        "<!-- custom-icon-inventory:start -->",
        "<!-- custom-icon-inventory:end -->",
    )?;
    let id = Regex::new(r"(?m)^ {2}([A-Za-z][A-Za-z0-9]*):")?;
    let ids: Vec<String> = id
        .captures_iter(&glyphs)
        .chain(id.captures_iter(&cam_glyphs))
        .map(|c| c[1].to_owned())
        .collect();
    let actual: BTreeSet<_> = ids.iter().cloned().collect();
    let documented: BTreeSet<_> = Regex::new(r"`([A-Za-z][A-Za-z0-9]*)`")?
        .captures_iter(&inventory)
        .map(|c| c[1].to_owned())
        .collect();
    ensure!(ids.len() == actual.len(), "duplicate source icon IDs");
    ensure!(
        actual == documented,
        "icon provenance differs: undocumented {:?}; stale {:?}",
        actual.difference(&documented).collect::<Vec<_>>(),
        documented.difference(&actual).collect::<Vec<_>>()
    );
    for pattern in [
        r"(?i)<image\b",
        r"(?i)\b(?:href|xlinkHref)\s*=",
        r"(?i)\bdata:image/",
        r#"(?i)\bfrom\s+['"][^'"]+\.(?:svg|png|jpe?g|webp)['"]"#,
    ] {
        let pattern = Regex::new(pattern)?;
        ensure!(
            !pattern.is_match(&glyphs) && !pattern.is_match(&cam),
            "disallowed custom-icon asset"
        );
    }
    let forbidden = Regex::new(
        r"(?i)<(?:image|script|foreignObject)\b|\bon\w+\s*=|\b(?:href|xlink:href)\s*=|data:image/",
    )?;
    for entry in fs::read_dir(root.join("src/assets/ribbon-icons"))? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "LICENSE.lucide" {
            continue;
        }
        ensure!(
            entry.file_type()?.is_file() && entry.path().extension().is_some_and(|s| s == "svg"),
            "invalid shared ribbon vector {}",
            name.to_string_lossy()
        );
        let svg = fs::read_to_string(entry.path())?;
        ensure!(
            svg.contains("viewBox=\"0 0 24 24\"") && !forbidden.is_match(&svg),
            "invalid or executable shared ribbon vector {}",
            name.to_string_lossy()
        );
    }
    ensure!(
        brand.contains("noBS CAD NB monogram"),
        "canonical mark lacks provenance title"
    );
    ensure!(
        !Regex::new(r"(?i)<image\b|(?:href|xlink:href)\s*=|data:image/")?.is_match(&brand),
        "canonical mark references an external image"
    );
    println!("Icon provenance OK: {} custom glyphs", actual.len());
    for (name, text) in [
        ("icons.tsx", source),
        ("CamToolIcon.tsx", cam),
        ("app-icon.svg", brand),
    ] {
        println!(
            "{name} sha256 {}",
            crate::repository_ci::hex(&Sha256::digest(text.as_bytes()))
        );
    }
    Ok(())
}
