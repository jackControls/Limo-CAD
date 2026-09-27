//! One per-user configuration root for the React and native desktop hosts.
use std::path::PathBuf;

pub(crate) fn directory(identifier: &str) -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|root| root.join(identifier))
        .ok_or_else(|| "Could not resolve the per-user config directory".into())
}

#[cfg(feature = "dev-bevy-host")]
pub(crate) fn native_directory() -> Result<PathBuf, String> {
    let config: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tauri.conf.json"
    )))
    .map_err(|error| format!("Invalid desktop configuration: {error}"))?;
    directory(
        config["identifier"]
            .as_str()
            .ok_or("Desktop application identity is missing")?,
    )
}
