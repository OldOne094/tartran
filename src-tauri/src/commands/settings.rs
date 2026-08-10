use crate::ipc::{err, ok};
use crate::models::{AppSettings, UpdateSettingsInput};
use crate::storage::app_settings::{AppSettingsStore, TEMPERATURE_MAX, TEMPERATURE_MIN};
use crate::storage::key_store::{KeyStore, OsKeyring};
use crate::AppState;
use serde_json::json;
use serde_json::Value;
use tauri::State;

const DEFAULT_KEY_ID: &str = "default";

fn validate_settings_update(patch: &UpdateSettingsInput) -> Option<String> {
    if let Some(w) = &patch.workspace_path {
        if w.trim().is_empty() || w.chars().count() > 1000 {
            return Some("Invalid settings update".into());
        }
    }
    if let Some(lang) = &patch.ui_language {
        if lang != "en" && lang != "ar" {
            return Some("Invalid settings update".into());
        }
    }
    if let Some(theme) = &patch.theme {
        if theme != "system" && theme != "light" && theme != "dark" {
            return Some("Invalid settings update".into());
        }
    }
    if let Some(t) = &patch.temperature {
        if !t.is_finite() || *t < TEMPERATURE_MIN || *t > TEMPERATURE_MAX {
            return Some("Invalid settings update".into());
        }
    }
    None
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> Value {
    let settings = AppSettingsStore::new(state.base_dir.clone());
    ok(settings.get())
}

#[tauri::command]
pub fn settings_update(state: State<'_, AppState>, patch: UpdateSettingsInput) -> Value {
    if let Some(msg) = validate_settings_update(&patch) {
        return err("INVALID_INPUT", &msg);
    }
    let settings = AppSettingsStore::new(state.base_dir.clone());
    let next: AppSettings = settings.update(&patch);
    if patch.workspace_path.is_some() {
        settings.workspace_path();
    }
    ok(next)
}

fn keyring_store() -> OsKeyring {
    OsKeyring::new("tartran")
}

#[tauri::command]
pub fn settings_api_key_status(state: State<'_, AppState>) -> Value {
    let store = keyring_store();
    match store.has(DEFAULT_KEY_ID) {
        true => ok(json!({ "configured": true })),
        false => {
            state.logger.debug("api_key:status:unconfigured", None);
            ok(json!({ "configured": false }))
        }
    }
}

#[tauri::command]
pub fn settings_api_key_set(state: State<'_, AppState>, api_key: String) -> Value {
    if api_key.chars().count() < 8 || api_key.chars().count() > 500 {
        return err("INVALID_INPUT", "Invalid API key");
    }
    let store = keyring_store();
    match store.set(DEFAULT_KEY_ID, &api_key) {
        Ok(()) => {
            state.logger.info("api_key:set", None);
            ok(json!({ "configured": true }))
        }
        Err(_) => err("KEY_STORE_UNAVAILABLE", "Secure key storage is not available on this device"),
    }
}

#[tauri::command]
pub fn settings_api_key_clear(state: State<'_, AppState>) -> Value {
    let store = keyring_store();
    store.delete(DEFAULT_KEY_ID);
    state.logger.info("api_key:cleared", None);
    ok(json!({ "configured": false }))
}
