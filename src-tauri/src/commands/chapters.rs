use crate::ipc::{err, ok};
use crate::models::{CreateChapterInput, ImportChaptersInput, UpdateChapterInput};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::chapters::ChaptersManager;
use crate::AppState;
use serde_json::Value;
use tauri::State;

fn manager(state: &AppState) -> ChaptersManager<'_> {
    ChaptersManager::new(AppSettingsStore::new(state.base_dir.clone()), &state.logger)
}

#[tauri::command]
pub fn chapters_list(state: State<'_, AppState>, project_id: String) -> Value {
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
pub fn chapters_get(state: State<'_, AppState>, project_id: String, chapter_id: String) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid chapter request");
    }
    let manager = manager(&state);
    match manager.get(&project_id, &chapter_id) {
        Ok(ch) => ok(ch),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_get_memory(state: State<'_, AppState>, project_id: String, chapter_id: String) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid chapter request");
    }
    let manager = manager(&state);
    match manager.get_memory(&project_id, &chapter_id) {
        Ok(mem) => ok(mem),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_create(state: State<'_, AppState>, project_id: String, input: CreateChapterInput) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    if input.title.chars().count() > 500 || input.source_text.chars().count() > 2_000_000 {
        return err("INVALID_INPUT", "Invalid chapter input");
    }
    let manager = manager(&state);
    match manager.create(&project_id, input) {
        Ok(ch) => ok(ch),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_update(
    state: State<'_, AppState>,
    project_id: String,
    chapter_id: String,
    patch: UpdateChapterInput,
) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid chapter update");
    }
    let manager = manager(&state);
    match manager.update(&project_id, &chapter_id, patch) {
        Ok(ch) => ok(ch),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_delete(state: State<'_, AppState>, project_id: String, chapter_id: String) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid chapter delete");
    }
    let manager = manager(&state);
    match manager.delete(&project_id, &chapter_id) {
        Ok(()) => ok(serde_json::Value::Null),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_search(state: State<'_, AppState>, project_id: String, query: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.search(&project_id, &query, 50) {
        Ok(results) => ok(results),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn chapters_import(state: State<'_, AppState>, project_id: String, input: ImportChaptersInput) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.import(&project_id, input) {
        Ok(result) => ok(result),
        Err(e) => err(e.code(), &e.to_string()),
    }
}
