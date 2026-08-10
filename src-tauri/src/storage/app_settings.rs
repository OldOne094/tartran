use crate::models::{AppSettings, UpdateSettingsInput};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryEntry {
    pub id: String,
    pub path: String,
    pub title: String,
    pub created_at: String,
}

#[derive(Clone)]
pub struct AppSettingsStore {
    base_dir: PathBuf,
    settings_path: PathBuf,
    registry_path: PathBuf,
}

impl AppSettingsStore {
    pub fn new(base_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&base_dir);
        let settings_path = base_dir.join("settings.json");
        let registry_path = base_dir.join("projects.json");
        AppSettingsStore {
            base_dir,
            settings_path,
            registry_path,
        }
    }

    pub fn get(&self) -> AppSettings {
        let raw = read_json_safe::<serde_json::Value>(&self.settings_path, serde_json::json!({}));
        let workspace_path = match raw.get("workspacePath").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => String::new(),
        };
        let ui_language = match raw.get("uiLanguage").and_then(|v| v.as_str()) {
            Some("ar") => "ar",
            _ => "en",
        };
        let theme = match raw.get("theme").and_then(|v| v.as_str()) {
            Some("dark") => "dark",
            Some("light") => "light",
            _ => "system",
        };
        AppSettings {
            workspace_path,
            ui_language: ui_language.to_string(),
            theme: theme.to_string(),
        }
    }

    pub fn update(&self, patch: &UpdateSettingsInput) -> AppSettings {
        let current = self.get();
        let next = AppSettings {
            workspace_path: patch.workspace_path.clone().unwrap_or(current.workspace_path),
            ui_language: patch.ui_language.clone().unwrap_or(current.ui_language),
            theme: patch.theme.clone().unwrap_or(current.theme),
        };
        write_json_atomic(&self.settings_path, &next);
        next
    }

    pub fn workspace_path(&self) -> PathBuf {
        let ws = self.get().workspace_path;
        let ws = if ws.is_empty() {
            self.base_dir.join("Projects")
        } else {
            PathBuf::from(ws)
        };
        let _ = fs::create_dir_all(&ws);
        ws
    }

    pub fn registry(&self) -> Vec<RegistryEntry> {
        read_json_safe::<Vec<RegistryEntry>>(&self.registry_path, Vec::new())
    }

    pub fn save_registry(&self, entries: &[RegistryEntry]) {
        write_json_atomic(&self.registry_path, &entries);
    }

    pub fn add_registry(&self, entry: RegistryEntry) {
        let mut entries: Vec<RegistryEntry> = self
            .registry()
            .into_iter()
            .filter(|e| e.id != entry.id)
            .collect();
        entries.push(entry);
        self.save_registry(&entries);
    }

    pub fn remove_registry(&self, id: &str) {
        let entries: Vec<RegistryEntry> = self.registry().into_iter().filter(|e| e.id != id).collect();
        self.save_registry(&entries);
    }
}

fn read_json_safe<T: serde::de::DeserializeOwned>(file: &Path, fallback: T) -> T {
    fs::read_to_string(file)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(fallback)
}

fn write_json_atomic(file: &Path, value: &impl serde::Serialize) {
    let tmp = file.with_extension("json.tmp");
    if let Ok(s) = serde_json::to_string_pretty(value) {
        if fs::write(&tmp, s).is_ok() {
            let _ = fs::rename(&tmp, file);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_dir() -> PathBuf {
        let dir = tempfile::tempdir().unwrap();
        dir.into_path()
    }

    #[test]
    fn loads_defaults_on_a_fresh_directory() {
        let store = AppSettingsStore::new(base_dir());
        let s = store.get();
        assert_eq!(s.workspace_path, "");
        assert_eq!(s.ui_language, "en");
        assert_eq!(s.theme, "system");
    }

    #[test]
    fn updates_and_persists_settings() {
        let store = AppSettingsStore::new(base_dir());
        store.update(&UpdateSettingsInput {
            workspace_path: None,
            ui_language: Some("ar".into()),
            theme: Some("dark".into()),
        });
        let reopened = AppSettingsStore::new(store.base_dir.clone());
        assert_eq!(reopened.get().ui_language, "ar");
        assert_eq!(reopened.get().theme, "dark");
    }

    #[test]
    fn ignores_invalid_persisted_values() {
        let store = AppSettingsStore::new(base_dir());
        fs::write(&store.settings_path, r#"{"uiLanguage":"fr","theme":"neon"}"#).unwrap();
        let reopened = AppSettingsStore::new(store.base_dir.clone());
        assert_eq!(reopened.get().ui_language, "en");
        assert_eq!(reopened.get().theme, "system");
    }

    #[test]
    fn creates_workspace_folder_on_demand() {
        let store = AppSettingsStore::new(base_dir());
        let ws = store.workspace_path();
        assert!(ws.exists());
    }

    #[test]
    fn adds_and_removes_registry_entries() {
        let store = AppSettingsStore::new(base_dir());
        let entry = RegistryEntry {
            id: "p1".into(),
            path: "C:/tmp/p1".into(),
            title: "Novel".into(),
            created_at: "now".into(),
        };
        store.add_registry(entry.clone());
        assert_eq!(store.registry().len(), 1);
        store.add_registry(entry);
        assert_eq!(store.registry().len(), 1);
        store.remove_registry("p1");
        assert_eq!(store.registry().len(), 0);
    }
}
