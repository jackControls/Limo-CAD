//! Shared constraint icon vocabulary stays consistent across the independent
//! browser and native interfaces. No desktop IPC decoder is involved.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read(relative: &str) -> String {
    let path = repo_path(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// Single-quoted strings inside `text[start..end]`, where `end` is the first
/// `;` after `start`.
fn quoted_kinds(source: &str, start_marker: &str) -> BTreeSet<String> {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing {start_marker}"))
        + start_marker.len();
    let end = source[start..]
        .find(';')
        .unwrap_or_else(|| panic!("unterminated {start_marker}"))
        + start;
    source[start..end]
        .split('\'')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// Serde's snake_case form of the variants declared in `pub enum {name}`.
fn enum_variants(source: &str, name: &str) -> BTreeSet<String> {
    let header = format!("pub enum {name} {{");
    let start = source
        .find(&header)
        .unwrap_or_else(|| panic!("missing {header}"))
        + header.len();
    let mut variants = BTreeSet::new();
    for line in source[start..].lines() {
        let line = line.trim();
        if line == "}" {
            break;
        }
        let identifier = line.trim_end_matches(',').trim();
        if identifier.is_empty()
            || identifier.starts_with("///")
            || identifier.starts_with("//")
            || !identifier
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            continue;
        }
        variants.insert(snake_case(identifier));
    }
    assert!(!variants.is_empty(), "{name} parsed as empty");
    variants
}

fn snake_case(identifier: &str) -> String {
    let mut out = String::new();
    for (index, character) in identifier.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    out
}

#[test]
fn every_frontend_constraint_icon_has_a_native_variant() {
    let frontend = quoted_kinds(
        &read("src/sketch/constraintIcons.tsx"),
        "export type ConstraintIconKind =",
    );
    let native = enum_variants(
        &read("src-tauri/src/native_viewport/mod.rs"),
        "ViewportConstraintIcon",
    );
    assert_eq!(
        frontend, native,
        "the browser and native interfaces must cover the same constraint icon kinds"
    );
}

