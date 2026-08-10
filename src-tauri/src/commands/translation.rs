use crate::ipc::{err, ok};
use crate::llm::available_models;
use crate::models::TranslateChapterInput;
use crate::pipeline::TranslationPipeline;
use crate::storage::app_settings::AppSettingsStore;
use crate::AppState;
use serde_json::json;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn translation_translate_chapter(
    app: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    input: TranslateChapterInput,
) -> Value {
    if project_id.trim().is_empty() || input.chapter_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid translation request");
    }
    let chapter_id = input.chapter_id.clone();
    let pipeline = TranslationPipeline::new(
        AppSettingsStore::new(state.base_dir.clone()),
        &state.logger,
    );
    let on_progress = move |current: usize, total: usize| {
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0).round() as u8
        } else {
            100
        };
        let _ = app.emit(
            "translation:progress",
            json!({ "chapterId": chapter_id, "current": current, "total": total, "percent": percent }),
        );
    };
    match pipeline.translate_chapter(&project_id, &input, &state.rate_limiter, &on_progress) {
        Ok(result) => ok(result),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn translation_models() -> Value {
    ok(available_models())
}
