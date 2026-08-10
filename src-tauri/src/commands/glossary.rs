use crate::ipc::{err, ok};
use crate::models::{CreateGlossaryInput, UpdateGlossaryInput};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::glossary::GlossaryManager;
use crate::AppState;
use serde_json::Value;
use tauri::State;

fn manager(state: &AppState) -> GlossaryManager<'_> {
    GlossaryManager::new(AppSettingsStore::new(state.base_dir.clone()), &state.logger)
}

#[tauri::command]
pub fn glossary_list(state: State<'_, AppState>, project_id: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.list(&project_id) {
        Ok(list) => ok(list),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn glossary_create(state: State<'_, AppState>, project_id: String, input: CreateGlossaryInput) -> Value {
    if project_id.trim().is_empty() || input.zh.trim().is_empty() || input.zh.chars().count() > 200 {
        return err("INVALID_INPUT", "Invalid glossary entry");
    }
    let manager = manager(&state);
    match manager.create(&project_id, input) {
        Ok(entry) => ok(entry),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn glossary_update(
    state: State<'_, AppState>,
    project_id: String,
    glossary_id: String,
    patch: UpdateGlossaryInput,
) -> Value {
    if project_id.trim().is_empty() || glossary_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid glossary update");
    }
    let manager = manager(&state);
    match manager.update(&project_id, &glossary_id, patch) {
        Ok(entry) => ok(entry),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn glossary_delete(state: State<'_, AppState>, project_id: String, glossary_id: String) -> Value {
    if project_id.trim().is_empty() || glossary_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid glossary delete");
    }
    let manager = manager(&state);
    match manager.delete(&project_id, &glossary_id) {
        Ok(()) => ok(serde_json::Value::Null),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn glossary_search(state: State<'_, AppState>, project_id: String, query: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.search(&project_id, &query, 50) {
        Ok(results) => ok(results),
        Err(e) => err(e.code(), &e.to_string()),
    }
}
