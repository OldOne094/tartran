use crate::ipc::{err, ok};
use crate::models::{CreateSuggestionInput, UpdateSuggestionInput};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::suggestions::SuggestionsManager;
use crate::AppState;
use serde_json::Value;
use tauri::State;

fn manager(state: &AppState) -> SuggestionsManager<'_> {
    SuggestionsManager::new(AppSettingsStore::new(state.base_dir.clone()), &state.logger)
}

#[tauri::command]
pub fn suggestions_list(state: State<'_, AppState>, project_id: String, chapter_id: String) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid suggestions request");
    }
    let manager = manager(&state);
    match manager.list(&project_id, &chapter_id) {
        Ok(list) => ok(list),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn suggestions_create(state: State<'_, AppState>, project_id: String, input: CreateSuggestionInput) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.create(&project_id, input) {
        Ok(s) => ok(s),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn suggestions_update(
    state: State<'_, AppState>,
    project_id: String,
    suggestion_id: String,
    patch: UpdateSuggestionInput,
) -> Value {
    if project_id.trim().is_empty() || suggestion_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid suggestion update");
    }
    let manager = manager(&state);
    match manager.update(&project_id, &suggestion_id, patch) {
        Ok(s) => ok(s),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn suggestions_approve(state: State<'_, AppState>, project_id: String, suggestion_id: String) -> Value {
    if project_id.trim().is_empty() || suggestion_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid suggestion");
    }
    let manager = manager(&state);
    match manager.approve(&project_id, &suggestion_id) {
        Ok(entry) => ok(entry),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn suggestions_reject(state: State<'_, AppState>, project_id: String, suggestion_id: String) -> Value {
    if project_id.trim().is_empty() || suggestion_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid suggestion");
    }
    let manager = manager(&state);
    match manager.reject(&project_id, &suggestion_id) {
        Ok(s) => ok(s),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn suggestions_delete(state: State<'_, AppState>, project_id: String, suggestion_id: String) -> Value {
    if project_id.trim().is_empty() || suggestion_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid suggestion delete");
    }
    let manager = manager(&state);
    match manager.delete(&project_id, &suggestion_id) {
        Ok(()) => ok(serde_json::Value::Null),
        Err(e) => err(e.code(), &e.to_string()),
    }
}
