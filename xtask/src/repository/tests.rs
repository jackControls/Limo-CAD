use super::*;

#[test]
fn links_preserve_artifacts_and_distinct_repositories() {
    let from = "jackControls/noBS-CAD";
    let to = "limo-cad/limo-cad";
    for prefix in [
        "https://github.com/",
        "git+https://github.com/",
        "https://raw.githubusercontent.com/",
        "https://api.github.com/repos/",
        "git@github.com:",
        "https://img.shields.io/github/actions/workflow/status/",
        "https://img.shields.io/github/v/release/",
        "`",
    ] {
        for tail in ["", "/main/a.rs", ".git", ".git\"", "?label=release", "`"] {
            assert_eq!(
                retarget(&format!("{prefix}{from}{tail}"), from, to, None).unwrap(),
                format!("{prefix}{to}{tail}")
            );
        }
        for tail in ["-fork", ".gitx", "_more"] {
            let text = format!("{prefix}{from}{tail}");
            assert_eq!(retarget(&text, from, to, None).unwrap(), text);
        }
    }
    let artifact = format!("https://github.com/{from}/releases/download/v1/noBS-CAD-1-windows.zip");
    assert_eq!(
        retarget(&artifact, from, to, None).unwrap(),
        artifact.replace(from, to)
    );
    assert_eq!(
        retarget(
            "https://JACKCONTROLS.github.io/noBS-CAD/open.html",
            from,
            to,
            Some("limo.example/cad")
        )
        .unwrap(),
        "https://limo.example/cad/open.html"
    );
}

#[test]
fn migration_skips_history_binaries_lockfiles_and_its_fixtures() {
    for file in [
        "docs/release-notes/v1.md",
        "Cargo.lock",
        "a/Cargo.lock",
        "a.png",
        "a.PDF",
        "xtask/src/repository/tests.rs",
    ] {
        assert!(skipped(file));
    }
    for file in ["REPOSITORY", "README.md", "Cargo.toml"] {
        assert!(!skipped(file));
    }
    for slug in ["o/r", "owner/repo.name"] {
        assert!(valid_slug(slug));
    }
    for slug in ["../repo", "o/../r", "o/", "/r", "o/re po"] {
        assert!(!valid_slug(slug));
    }
}
