use crate::ipc::ok;
use crate::AppState;
use serde_json::json;
use serde_json::Value;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn app_info(app: AppHandle, state: State<'_, AppState>) -> Value {
    let version = app.package_info().version.to_string();
    state.logger.debug("app:info", None);
    ok(json!({
        "version": version,
        "userDataPath": state.base_dir.to_string_lossy()
    }))
}
