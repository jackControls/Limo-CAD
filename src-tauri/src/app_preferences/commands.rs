//! Tauri adapter for the shared application preference store.

use super::{LegacyPreferences, Preferences, Store};
use tauri::Emitter;

#[tauri::command]
pub(crate) async fn app_preferences_load(app: tauri::AppHandle) -> Result<Preferences, String> {
    let identifier = app.config().identifier.clone();
    tauri::async_runtime::spawn_blocking(move || Store::for_application(&identifier)?.read())
        .await
        .map_err(|error| format!("Application preference worker failed: {error}"))?
}

#[tauri::command]
pub(crate) async fn app_preferences_patch(
    app: tauri::AppHandle,
    patch: Preferences,
) -> Result<Preferences, String> {
    let identifier = app.config().identifier.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        Store::for_application(&identifier)?.patch(patch)
    })
    .await
    .map_err(|error| format!("Application preference worker failed: {error}"))??;
    // A notification is only an invalidation hint. Its failure cannot turn a
    // durable save into an error that encourages the caller to rewrite it.
    let _ = app.emit("app-preferences-changed", ());
    Ok(saved)
}

#[tauri::command]
pub(crate) async fn app_preferences_import_legacy(
    app: tauri::AppHandle,
    legacy: LegacyPreferences,
) -> Result<Preferences, String> {
    let identifier = app.config().identifier.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        Store::for_application(&identifier)?.import_legacy(legacy)
    })
    .await
    .map_err(|error| format!("Application preference worker failed: {error}"))??;
    let _ = app.emit("app-preferences-changed", ());
    Ok(saved)
}
