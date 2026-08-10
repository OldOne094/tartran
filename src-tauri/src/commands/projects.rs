use crate::ipc::{err, ok};
use crate::models::{CreateProjectInput, UpdateProjectInput};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::projects::ProjectsManager;
use crate::AppState;
use serde_json::Value;
use tauri::State;

fn validate_create(input: &CreateProjectInput) -> Option<String> {
    if input.title.trim().is_empty() || input.title.chars().count() > 200 {
        return Some("Invalid project input".into());
    }
    if let Some(author) = &input.author {
        if author.chars().count() > 200 {
            return Some("Invalid project input".into());
        }
    }
    if input.target_lang != "ar" && input.target_lang != "en" {
        return Some("Invalid project input".into());
    }
    if let Some(src) = &input.source_lang {
        if src != "zh" {
            return Some("Invalid project input".into());
        }
    }
    None
}

fn validate_update(patch: &UpdateProjectInput) -> Option<String> {
    if let Some(title) = &patch.title {
        if title.trim().is_empty() || title.chars().count() > 200 {
            return Some("Invalid project update".into());
        }
    }
    if let Some(author) = &patch.author {
        if author.chars().count() > 200 {
            return Some("Invalid project update".into());
        }
    }
    if let Some(lang) = &patch.target_lang {
        if lang != "ar" && lang != "en" {
            return Some("Invalid project update".into());
        }
    }
    None
}

fn manager(state: &AppState) -> ProjectsManager<'_> {
    ProjectsManager::new(AppSettingsStore::new(state.base_dir.clone()), &state.logger)
}

#[tauri::command]
pub fn projects_list(state: State<'_, AppState>) -> Value {
    let manager = manager(&state);
    ok(manager.list())
}

#[tauri::command]
pub fn projects_create(state: State<'_, AppState>, input: CreateProjectInput) -> Value {
    if let Some(msg) = validate_create(&input) {
        return err("INVALID_INPUT", &msg);
    }
    let manager = manager(&state);
    match manager.create(input) {
        Ok(p) => ok(p),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn projects_get(state: State<'_, AppState>, project_id: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.get(&project_id) {
        Ok(p) => ok(p),
        Err(_) => err("NOT_FOUND", "Project not found"),
    }
}

#[tauri::command]
pub fn projects_update(state: State<'_, AppState>, project_id: String, patch: UpdateProjectInput) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project update");
    }
    if let Some(msg) = validate_update(&patch) {
        return err("INVALID_INPUT", &msg);
    }
    let manager = manager(&state);
    match manager.update(&project_id, patch) {
        Ok(p) => ok(p),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[tauri::command]
pub fn projects_delete(state: State<'_, AppState>, project_id: String) -> Value {
    if project_id.trim().is_empty() {
        return err("INVALID_INPUT", "Invalid project id");
    }
    let manager = manager(&state);
    match manager.delete(&project_id) {
        Ok(()) => ok(serde_json::Value::Null),
        Err(e) => err(e.code(), &e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_create_input() {
        assert!(validate_create(&CreateProjectInput {
            title: "".into(),
            author: None,
            target_lang: "ar".into(),
            source_lang: None,
        })
        .is_some());
        assert!(validate_create(&CreateProjectInput {
            title: "Novel".into(),
            author: None,
            target_lang: "fr".into(),
            source_lang: None,
        })
        .is_some());
        assert!(validate_create(&CreateProjectInput {
            title: "Novel".into(),
            author: None,
            target_lang: "ar".into(),
            source_lang: Some("en".into()),
        })
        .is_some());
        assert!(validate_create(&CreateProjectInput {
            title: "Novel".into(),
            author: None,
            target_lang: "en".into(),
            source_lang: None,
        })
        .is_none());
    }
}
