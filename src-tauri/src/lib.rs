mod commands;
mod error;
mod export;
mod ipc;
mod llm;
mod logger;
mod models;
mod pipeline;
mod storage;

use llm::rate_limiter::RateLimiter;
use logger::Logger;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub base_dir: PathBuf,
    pub logger: Logger,
    pub rate_limiter: Mutex<RateLimiter>,
}

fn resolve_base_dir(app: &tauri::AppHandle) -> PathBuf {
    let override_dir = std::env::var("TARTRAN_USER_DATA")
        .ok()
        .filter(|s| !s.is_empty());
    override_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            app.path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("tartran"))
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let base_dir = resolve_base_dir(app.handle());
            let logger = Logger::new(base_dir.join("logs"));
            let rate_limiter = Mutex::new(RateLimiter::new(30.0));
            app.manage(AppState {
                base_dir,
                logger,
                rate_limiter,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::projects_list,
            commands::projects::projects_create,
            commands::projects::projects_get,
            commands::projects::projects_update,
            commands::projects::projects_delete,
            commands::settings::settings_get,
            commands::settings::settings_update,
            commands::settings::settings_api_key_status,
            commands::settings::settings_api_key_set,
            commands::settings::settings_api_key_clear,
            commands::app::app_info,
            commands::chapters::chapters_list,
            commands::chapters::chapters_get,
            commands::chapters::chapters_get_memory,
            commands::chapters::chapters_create,
            commands::chapters::chapters_update,
            commands::chapters::chapters_delete,
            commands::chapters::chapters_search,
            commands::chapters::chapters_import,
            commands::glossary::glossary_list,
            commands::glossary::glossary_create,
            commands::glossary::glossary_update,
            commands::glossary::glossary_delete,
            commands::glossary::glossary_search,
            commands::suggestions::suggestions_list,
            commands::suggestions::suggestions_create,
            commands::suggestions::suggestions_update,
            commands::suggestions::suggestions_approve,
            commands::suggestions::suggestions_reject,
            commands::suggestions::suggestions_delete,
            commands::translation::translation_translate_chapter,
            commands::translation::translation_models,
            commands::export::export_chapter_text,
            commands::export::export_chapter_docx,
            commands::export::export_glossary_xlsx
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
