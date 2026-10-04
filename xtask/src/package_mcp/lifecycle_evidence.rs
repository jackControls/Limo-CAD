//! Bounded artifact copies of this fixture's exclusively owned session tree.
use super::*;
use std::io::{self, Write};

pub(super) fn stage(sessions: &Path, name: &str, pid: Option<u32>) -> Result<()> {
    let logs = sessions.join("_ui").join("fixture-logs");
    fs::create_dir_all(&logs)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs.join("stages.jsonl"))?;
    writeln!(
        file,
        "{}",
        json!({"stage":name,"pid":pid,"unix_ms":SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()})
    )?;
    Ok(())
}

pub(super) fn retain(source: &Path, report: Option<&Path>) -> Result<PathBuf> {
    let Some(report) = report else {
        return Ok(source.to_owned());
    };
    let name = report
        .file_name()
        .context("Lifecycle report has no filename")?
        .to_string_lossy();
    let destination = report.with_file_name(format!("{name}.lifecycle"));
    fs::create_dir(&destination)
        .context("Create new lifecycle evidence directory without replacing prior evidence")?;
    let mut budget = Budget {
        entries: 1024,
        bytes: 32 * 1024 * 1024,
        per_file: 8 * 1024 * 1024,
    };
    let mut manifest = Vec::new();
    copy_tree(
        source,
        &destination,
        Path::new(""),
        0,
        &mut budget,
        &mut manifest,
    )?;
    fs::write(
        destination.join("retention.json"),
        serde_json::to_vec_pretty(&json!({
            "source":source,"limits":{"entries":1024,"bytes":32*1024*1024,"per_file_bytes":8*1024*1024,"depth":12},
            "entries":manifest,
        }))?,
    )?;
    Ok(destination)
}

struct Budget {
    entries: usize,
    bytes: u64,
    per_file: u64,
}

fn copy_tree(
    source: &Path,
    destination: &Path,
    relative: &Path,
    depth: usize,
    budget: &mut Budget,
    manifest: &mut Vec<Value>,
) -> Result<()> {
    if depth > 12 || budget.entries == 0 {
        manifest.push(json!({"path":relative,"skipped":"entry/depth limit"}));
        return Ok(());
    }
    for entry in fs::read_dir(source.join(relative))? {
        if budget.entries == 0 {
            manifest.push(json!({"path":relative,"skipped":"remaining entries"}));
            break;
        }
        budget.entries -= 1;
        let entry = entry?;
        if depth == 0 && entry.file_name().to_string_lossy().starts_with("webview-") {
            manifest.push(json!({"path":entry.file_name(),"skipped":"browser profile"}));
            continue;
        }
        let relative = relative.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            fs::create_dir(destination.join(&relative))?;
            copy_tree(source, destination, &relative, depth + 1, budget, manifest)?;
        } else if kind.is_file() {
            let original_bytes = entry.metadata()?.len();
            let limit = budget.per_file.min(budget.bytes);
            let mut input = fs::File::open(entry.path())?.take(limit);
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(&relative))?;
            let retained_bytes = io::copy(&mut input, &mut output)?;
            budget.bytes -= retained_bytes;
            manifest.push(json!({"path":relative,"original_bytes":original_bytes,"retained_bytes":retained_bytes,"truncated":retained_bytes<original_bytes}));
        } else {
            manifest.push(json!({"path":relative,"skipped":"not a regular file or directory"}));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_keeps_owned_publication_and_stage_without_profiles_or_overwrites() {
        let root = SessionDirectory::create().unwrap();
        let source = root.0.join("source");
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("session")).unwrap();
        fs::write(source.join("session/heartbeat.json"), "{\"generation\":7}").unwrap();
        fs::create_dir(source.join("webview-0")).unwrap();
        fs::write(source.join("webview-0/profile"), "excluded profile").unwrap();
        stage(&source, "saving-baseline-document", Some(42)).unwrap();
        let report = root.0.join("desktop.json");
        let retained = retain(&source, Some(&report)).unwrap();
        assert_eq!(retained, root.0.join("desktop.json.lifecycle"));
        assert_eq!(
            fs::read(retained.join("session/heartbeat.json")).unwrap(),
            b"{\"generation\":7}"
        );
        assert!(
            fs::read_to_string(retained.join("_ui/fixture-logs/stages.jsonl"))
                .unwrap()
                .contains("saving-baseline-document")
        );
        assert!(!retained.join("webview-0").exists());
        let manifest = fs::read(retained.join("retention.json")).unwrap();
        assert!(retain(&source, Some(&report)).is_err());
        assert_eq!(fs::read(retained.join("retention.json")).unwrap(), manifest);
        assert_eq!(retain(&source, None).unwrap(), source);
        assert!(source.join("webview-0/profile").is_file());
        fs::remove_dir_all(&root.0).unwrap();
    }

    #[test]
    fn retained_child_logs_and_models_obey_byte_limits() {
        let root = SessionDirectory::create().unwrap();
        let source = root.0.join("source");
        let destination = root.0.join("copy");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::create_dir(source.join("child")).unwrap();
        fs::write(source.join("child/stderr.log"), "123456789").unwrap();
        fs::write(source.join("model.json"), "abcdefghi").unwrap();
        let mut budget = Budget {
            entries: 10,
            bytes: 8,
            per_file: 5,
        };
        let mut manifest = Vec::new();
        copy_tree(
            &source,
            &destination,
            Path::new(""),
            0,
            &mut budget,
            &mut manifest,
        )
        .unwrap();
        assert_eq!(
            manifest
                .iter()
                .filter_map(|v| v["retained_bytes"].as_u64())
                .sum::<u64>(),
            8
        );
        assert!(manifest.iter().all(|v| v["truncated"] == true));
        assert_eq!(
            fs::read(source.join("child/stderr.log")).unwrap(),
            b"123456789"
        );
        assert!(destination.join("child/stderr.log").is_file());
        assert!(destination.join("model.json").is_file());
        fs::remove_dir_all(&root.0).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn retention_does_not_follow_links_outside_owned_session() {
        let root = SessionDirectory::create().unwrap();
        let source = root.0.join("source");
        let destination = root.0.join("copy");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(root.0.join("outside"), "not owned session evidence").unwrap();
        std::os::unix::fs::symlink(root.0.join("outside"), source.join("linked")).unwrap();
        let mut budget = Budget {
            entries: 10,
            bytes: 100,
            per_file: 100,
        };
        let mut manifest = Vec::new();
        copy_tree(
            &source,
            &destination,
            Path::new(""),
            0,
            &mut budget,
            &mut manifest,
        )
        .unwrap();
        assert!(!destination.join("linked").exists());
        assert!(manifest[0]["skipped"].is_string());
        fs::remove_dir_all(&root.0).unwrap();
    }
}
