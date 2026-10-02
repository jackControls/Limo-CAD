//! VERSION is authoritative. Rewrites preserve unrelated dependency versions,
//! formatting and historical release notes; all carriers are checked before writes.
use anyhow::{bail, ensure, Context, Result};
use regex::Regex;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use toml_edit::{value, DocumentMut, Item};

const DOCS: &[&str] = &[
    "README.md",
    "docs/DEVELOPMENT.md",
    "docs/INSTALL.md",
    "docs/OCCT_PACKAGING.md",
    "docs/WINDOWS_PACKAGING.md",
];
const VERSION_PATTERN: &str = r"\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*?)?";

pub fn validate(version: &str) -> Result<()> {
    let parsed: semver::Version = version
        .parse()
        .context("VERSION must hold one semver version")?;
    ensure!(
        parsed.build.is_empty(),
        "VERSION does not support build metadata in package names"
    );
    Ok(())
}

pub fn read(root: &Path) -> Result<String> {
    let version = fs::read_to_string(root.join("VERSION"))?.trim().to_owned();
    validate(&version)?;
    Ok(version)
}

fn string<'a>(item: &'a Item, label: &str) -> Result<&'a str> {
    item.as_str()
        .with_context(|| format!("{label} must be a string"))
}

fn set(item: &mut Item, version: &str) {
    let decor = item.as_value().map(|v| v.decor().clone());
    *item = value(version);
    if let Some(decor) = decor {
        *item.as_value_mut().unwrap().decor_mut() = decor;
    }
}

#[derive(Clone)]
enum Kind {
    Workspace,
    Member,
    Package,
    Lock,
    Json(&'static str),
    NpmLock,
    Container,
    Docs,
    Workflow,
}

#[derive(Clone)]
struct Carrier {
    file: PathBuf,
    kind: Kind,
}

fn inventory(root: &Path) -> Result<(Vec<Carrier>, BTreeSet<String>)> {
    let manifest = fs::read_to_string(root.join("Cargo.toml"))?.parse::<DocumentMut>()?;
    let members = manifest["workspace"]["members"]
        .as_array()
        .context("workspace members must be an array")?;
    let mut carriers = vec![Carrier {
        file: "Cargo.toml".into(),
        kind: Kind::Workspace,
    }];
    let mut names = BTreeSet::new();
    for member in members {
        let file = Path::new(member.as_str().context("workspace member must be a path")?)
            .join("Cargo.toml");
        let doc = fs::read_to_string(root.join(&file))?.parse::<DocumentMut>()?;
        names.insert(string(&doc["package"]["name"], "package name")?.to_owned());
        carriers.push(Carrier {
            file,
            kind: Kind::Member,
        });
    }
    for file in ["src-tauri/Cargo.toml", "mcp-server/Cargo.toml"] {
        let doc = fs::read_to_string(root.join(file))?.parse::<DocumentMut>()?;
        names.insert(string(&doc["package"]["name"], "package name")?.to_owned());
        carriers.push(Carrier {
            file: file.into(),
            kind: Kind::Package,
        });
    }
    for file in [
        "Cargo.lock",
        "src-tauri/Cargo.lock",
        "mcp-server/Cargo.lock",
    ] {
        carriers.push(Carrier {
            file: file.into(),
            kind: Kind::Lock,
        });
    }
    for (file, kind) in [
        ("package.json", Kind::Json("version")),
        ("package-lock.json", Kind::NpmLock),
        ("vcpkg.json", Kind::Json("version-string")),
        ("src/files/nbcad.ts", Kind::Container),
    ] {
        carriers.push(Carrier {
            file: file.into(),
            kind,
        });
    }
    carriers.extend(DOCS.iter().map(|file| Carrier {
        file: (*file).into(),
        kind: Kind::Docs,
    }));
    carriers.push(Carrier {
        file: ".github/workflows/desktop-packages.yml".into(),
        kind: Kind::Workflow,
    });
    Ok((carriers, names))
}

fn document_patterns() -> Vec<Regex> {
    [
        format!(r"(noBS-CAD-)({VERSION_PATTERN})(-windows-|-ubuntu-)"),
        format!(r"(noBS\.CAD_)({VERSION_PATTERN})(_)"),
        format!(r"(/releases/(?:download|tag)/v)({VERSION_PATTERN})([/)#\s]|$)"),
        format!(r"(\bRelease )({VERSION_PATTERN})([^\w.-]|$)"),
        format!(r"(\[)({VERSION_PATTERN})( release\])"),
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("document version pattern"))
    .collect()
}

fn documented(text: &str, version: &str) -> Result<String> {
    let mut next = text.to_owned();
    let mut found = false;
    for pattern in document_patterns() {
        found |= pattern.is_match(text);
        next = pattern
            .replace_all(&next, |capture: &regex::Captures<'_>| {
                format!("{}{version}{}", &capture[1], &capture[3])
            })
            .into_owned();
    }
    ensure!(
        found || !(text.contains("noBS-CAD-") || text.contains("noBS.CAD_")),
        "mentions a packaged file name with no recognizable version"
    );
    Ok(next)
}

fn rewritten(kind: &Kind, text: &str, version: &str, names: &BTreeSet<String>) -> Result<String> {
    match kind {
        Kind::Workspace | Kind::Member | Kind::Package | Kind::Lock => {
            let mut doc = text.parse::<DocumentMut>()?;
            match kind {
                Kind::Workspace => {
                    string(&doc["workspace"]["package"]["version"], "workspace version")?;
                    set(&mut doc["workspace"]["package"]["version"], version);
                }
                Kind::Package => {
                    string(&doc["package"]["version"], "package version")?;
                    set(&mut doc["package"]["version"], version);
                }
                Kind::Member => {
                    let current = &doc["package"]["version"];
                    ensure!(
                        current.as_str().is_some()
                            || current.get("workspace").and_then(Item::as_bool) == Some(true),
                        "member must declare a version or inherit workspace"
                    );
                    if current.get("workspace").and_then(Item::as_bool) != Some(true) {
                        let mut inherited = toml_edit::Table::new();
                        inherited.set_dotted(true);
                        inherited.insert("workspace", value(true));
                        doc["package"]["version"] = Item::Table(inherited);
                    }
                }
                Kind::Lock => {
                    let packages = doc["package"]
                        .as_array_of_tables_mut()
                        .context("lockfile packages")?;
                    let mut count = 0;
                    for package in packages.iter_mut() {
                        let name = string(&package["name"], "lockfile package name")?;
                        // A registry package with the same name is not our local package.
                        if names.contains(name) && !package.contains_key("source") {
                            count += 1;
                            string(&package["version"], "lockfile package version")?;
                            set(&mut package["version"], version);
                        }
                    }
                    ensure!(count > 0, "lockfile records none of the local packages");
                }
                _ => unreachable!(),
            }
            Ok(doc.to_string())
        }
        Kind::Json(_) | Kind::NpmLock => {
            let mut json: Value = serde_json::from_str(text)?;
            ensure!(
                format!("{}\n", serde_json::to_string_pretty(&json)?) == text,
                "JSON formatting cannot round-trip; edit it by hand"
            );
            let field = if let Kind::Json(field) = kind {
                *field
            } else {
                "version"
            };
            ensure!(json[field].is_string(), "JSON {field} must be a string");
            json[field] = version.into();
            if matches!(kind, Kind::NpmLock)
                && json.get("packages").and_then(|v| v.get("")).is_some()
            {
                ensure!(
                    json["packages"][""]["version"].is_string(),
                    "packages[\"\"].version must be a string"
                );
                json["packages"][""]["version"] = version.into();
            }
            Ok(format!("{}\n", serde_json::to_string_pretty(&json)?))
        }
        Kind::Container => {
            let pattern = Regex::new(r"(application_version: ')[^']*(')")?;
            ensure!(
                pattern.is_match(text),
                "container declares no application_version"
            );
            Ok(pattern
                .replace_all(text, |c: &regex::Captures<'_>| {
                    format!("{}{version}{}", &c[1], &c[2])
                })
                .into_owned())
        }
        Kind::Docs => documented(text, version),
        Kind::Workflow => {
            ensure!(
                text.contains("cargo xtask version")
                    || text.contains("Get-Content VERSION")
                    || text.contains("cat VERSION"),
                "package workflow must read VERSION"
            );
            let literal = Regex::new(&format!(r"noBS(?:-CAD-|\.CAD_){VERSION_PATTERN}"))?;
            ensure!(
                !literal.is_match(text),
                "package workflow hard-codes versioned artifact names"
            );
            Ok(text.to_owned())
        }
    }
}

fn plan(root: &Path, version: &str) -> Result<Vec<(PathBuf, String)>> {
    let (carriers, names) = inventory(root)?;
    let mut changes = Vec::new();
    let mut errors = Vec::new();
    for carrier in carriers {
        let result = (|| -> Result<Option<String>> {
            let original = fs::read_to_string(root.join(&carrier.file))?;
            let normalized = original.replace("\r\n", "\n");
            let next = rewritten(&carrier.kind, &normalized, version, &names)?;
            // Read the result again: a rewrite must be complete and idempotent.
            ensure!(
                rewritten(&carrier.kind, &next, version, &names)? == next,
                "version rewrite is not idempotent"
            );
            if next == normalized {
                return Ok(None);
            }
            Ok(Some(if original.contains("\r\n") {
                next.replace('\n', "\r\n")
            } else {
                next
            }))
        })();
        match result {
            Ok(Some(next)) => changes.push((carrier.file, next)),
            Ok(None) => (),
            Err(error) => errors.push(format!("{}: {error:#}", carrier.file.display())),
        }
    }
    ensure!(
        errors.is_empty(),
        "Cannot synchronize VERSION:\n{}",
        errors.join("\n")
    );
    Ok(changes)
}

fn notes(root: &Path, version: &str) -> Result<()> {
    let file = format!("docs/release-notes/v{version}.md");
    let text = fs::read_to_string(root.join(&file))
        .with_context(|| format!("{file}: missing; write release notes with the version bump"))?;
    ensure!(!text.trim().is_empty(), "{file}: is empty");
    Ok(())
}

pub fn run(args: impl Iterator<Item = String>) -> Result<()> {
    let args: Vec<_> = args.collect();
    ensure!(
        args.len() <= 1,
        "usage: cargo xtask version [--check|--sync]"
    );
    let root = super::root();
    let version = read(root)?;
    match args.first().map(String::as_str) {
        None => println!("{version}"),
        Some("--check") => {
            let changes = plan(root, &version)?;
            ensure!(
                changes.is_empty(),
                "VERSION {version} disagrees with:\n{}\nRun cargo xtask version --sync.",
                changes
                    .iter()
                    .map(|(file, _)| file.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            notes(root, &version)?;
            println!("VERSION {version} matches every carrier; release notes are present.");
        }
        Some("--sync") => {
            let changes = plan(root, &version)?;
            for (file, text) in &changes {
                fs::write(root.join(file), text)?;
                println!("Updated {}", file.display());
            }
            println!("VERSION {version}: {} carriers updated.", changes.len());
        }
        Some(other) => bail!("unknown version option {other}; use --check or --sync"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_and_documented_prereleases() {
        for bad in ["", "0.2", "v0.2.0", "0.02.0", "0.2.0 notes", "0.2.0+build"] {
            assert!(validate(bad).is_err());
        }
        validate("0.3.0-rc.1").unwrap();
        let source = "noBS-CAD-0.3.0-rc-1-windows-x64.zip noBS.CAD_0.3.0-beta.2_amd64.deb [0.3.0 release] /releases/tag/v0.3.0 /releases/download/showcase-v0.1.0/bench.mp4 Release 0.3.0**";
        let next = documented(source, "0.4.0-rc.1").unwrap();
        assert!(next.contains("noBS-CAD-0.4.0-rc.1-windows-x64.zip"));
        assert!(next.contains("noBS.CAD_0.4.0-rc.1_amd64.deb"));
        assert!(next.contains("[0.4.0-rc.1 release]"));
        assert!(next.contains("/releases/tag/v0.4.0-rc.1 "));
        assert!(next.contains("showcase-v0.1.0"));
        assert!(next.contains("Release 0.4.0-rc.1**"));
        assert_eq!(documented(&next, "0.4.0-rc.1").unwrap(), next);
        assert!(!documented(&next, "0.4.0").unwrap().contains("rc.1"));
        assert!(documented("noBS-CAD-windows-x64.zip", "0.4.0").is_err());
    }

    #[test]
    fn cargo_rewrites_do_not_change_dependency_versions_or_comments() {
        let source = "[package]\nname = \"nbcad-core\"\nversion = \"0.2.0\" # kept\n[dependencies]\nserde = \"1.0.229\"\n";
        let next = rewritten(&Kind::Package, source, "0.3.0", &BTreeSet::new()).unwrap();
        assert!(next.contains("version = \"0.3.0\" # kept"));
        assert!(next.contains("serde = \"1.0.229\""));
        let inherited = rewritten(&Kind::Member, source, "0.3.0", &BTreeSet::new()).unwrap();
        assert!(inherited.contains("version.workspace = true"));
        let lock = "version = 4\n\n[[package]]\nname = \"nbcad-core\"\nversion = \"0.2.0\"\n\n[[package]]\nname = \"nbcad-core\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n";
        let next = rewritten(
            &Kind::Lock,
            lock,
            "0.3.0",
            &BTreeSet::from(["nbcad-core".into()]),
        )
        .unwrap();
        assert!(next.contains("version = \"0.3.0\""));
        assert!(next.contains("version = \"1.0.0\""));
    }

    #[test]
    fn npm_lock_checks_both_root_versions_and_preserves_key_order() {
        let text = "{\n  \"name\": \"nbcad\",\n  \"version\": \"0.2.0\",\n  \"packages\": {\n    \"\": {\n      \"version\": \"9.9.9\"\n    },\n    \"node_modules/vite\": {\n      \"version\": \"8.3.2\"\n    }\n  }\n}\n";
        let next = rewritten(&Kind::NpmLock, text, "0.2.0", &BTreeSet::new()).unwrap();
        assert_ne!(next, text);
        assert!(next.starts_with("{\n  \"name\":"));
        assert!(next.contains("\"version\": \"8.3.2\""));
        assert!(!next.contains("9.9.9"));
        assert!(rewritten(
            &Kind::NpmLock,
            "{\"version\":\"0.2.0\"}",
            "0.3.0",
            &BTreeSet::new()
        )
        .is_err());
    }

    #[test]
    fn release_bump_preserves_crlf_and_historical_notes() {
        let root = super::super::root();
        let version = read(root).unwrap();
        assert!(plan(root, &version).unwrap().is_empty());
        notes(root, &version).unwrap();
        let temp = tempfile::tempdir().unwrap();
        fs::copy(root.join("VERSION"), temp.path().join("VERSION")).unwrap();
        let (carriers, _) = inventory(root).unwrap();
        for carrier in carriers {
            let target = temp.path().join(&carrier.file);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            let source = fs::read_to_string(root.join(&carrier.file))
                .unwrap()
                .replace("\r\n", "\n");
            fs::write(target, source.replace('\n', "\r\n")).unwrap();
        }
        let historical = temp
            .path()
            .join(format!("docs/release-notes/v{version}.md"));
        fs::create_dir_all(historical.parent().unwrap()).unwrap();
        fs::write(&historical, "historical notes\r\n").unwrap();
        let target_version = if version == "9.9.9" { "9.9.8" } else { "9.9.9" };
        let changes = plan(temp.path(), target_version).unwrap();
        for (file, text) in changes {
            assert!(text.contains("\r\n"));
            fs::write(temp.path().join(file), text).unwrap();
        }
        assert!(plan(temp.path(), target_version).unwrap().is_empty());
        assert_eq!(
            fs::read_to_string(historical).unwrap(),
            "historical notes\r\n"
        );
        assert!(notes(temp.path(), target_version).is_err());
        let file = temp
            .path()
            .join(format!("docs/release-notes/v{target_version}.md"));
        fs::write(&file, " \n").unwrap();
        assert!(notes(temp.path(), target_version).is_err());
        fs::write(&file, "reviewed notes\n").unwrap();
        notes(temp.path(), target_version).unwrap();
    }
}
