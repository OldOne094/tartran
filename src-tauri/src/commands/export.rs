use crate::export::Exporter;
use crate::ipc::{err, ok};
use crate::storage::app_settings::AppSettingsStore;
use crate::AppState;
use serde_json::Value;
use tauri::State;

fn exporter(state: &AppState) -> Exporter<'_> {
    Exporter::new(AppSettingsStore::new(state.base_dir.clone()), &state.logger)
}

#[tauri::command]
pub fn export_chapter_text(state: State<'_, AppState>, project_id: String, chapter_id: String) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid export request");
    }
    match exporter(&state).chapter_clean_text(&project_id, &chapter_id) {
        Ok(f) => ok(f),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn export_chapter_docx(
    state: State<'_, AppState>,
    project_id: String,
    chapter_id: String,
    target_lang: String,
) -> Value {
    if project_id.trim().is_empty() || chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid export request");
    }
    match exporter(&state).chapter_docx(&project_id, &chapter_id, &target_lang) {
        Ok(f) => ok(f),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn export_glossary_xlsx(state: State<'_, AppState>, project_id: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid export request");
    }
    match exporter(&state).glossary_xlsx(&project_id) {
        Ok(f) => ok(f),
        Err(e) => err(e.code(), &e.to_string()),
    }
}
