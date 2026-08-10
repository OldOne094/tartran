use crate::error::{AppError, AppResult};
use crate::models::{
    CreateGlossaryInput, GlossaryEntry, GlossarySearchResult, UpdateGlossaryInput,
};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::project_store::{GlossaryRow, ProjectStore};
use chrono::Utc;
use std::path::PathBuf;

pub struct GlossaryManager<'a> {
    settings: AppSettingsStore,
    logger: &'a crate::logger::Logger,
}

impl<'a> GlossaryManager<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a crate::logger::Logger) -> Self {
        GlossaryManager { settings, logger }
    }

    fn store_for(&self, project_id: &str) -> AppResult<ProjectStore> {
        let entry = self
            .settings
            .registry()
            .into_iter()
            .find(|e| e.id == project_id)
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {project_id}")))?;
        ProjectStore::open(&PathBuf::from(&entry.path).join("novel.db"))
            .map_err(|e| AppError::Db(e))
    }

    fn to_entry(row: &GlossaryRow) -> GlossaryEntry {
        to_glossary_entry(row)
    }

    pub fn list(&self, project_id: &str) -> AppResult<Vec<GlossaryEntry>> {
        let store = self.store_for(project_id)?;
        let rows = store.list_glossary().map_err(|e| AppError::Db(e))?;
        Ok(rows.iter().map(Self::to_entry).collect())
    }

    pub fn create(&self, project_id: &str, input: CreateGlossaryInput) -> AppResult<GlossaryEntry> {
        let store = self.store_for(project_id)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let aliases = serde_json::to_string(&input.aliases).unwrap_or_else(|_| "[]".into());
        store
            .insert_glossary(
                &id,
                input.zh.trim(),
                input.en.trim(),
                input.ar.trim(),
                input.category.trim(),
                input.notes.trim(),
                &aliases,
                false,
                "manual",
                &now,
            )
            .map_err(|e| AppError::CreateFailed(e.to_string()))?;
        self.logger.info("glossary:create", Some(&serde_json::json!({ "id": id, "zh": input.zh })));
        let row = store.get_glossary(&id).map_err(|e| AppError::Db(e))?;
        row.map(|r| Self::to_entry(&r))
            .ok_or_else(|| AppError::CreateFailed("Glossary entry not persisted".into()))
    }

    pub fn update(
        &self,
        project_id: &str,
        glossary_id: &str,
        patch: UpdateGlossaryInput,
    ) -> AppResult<GlossaryEntry> {
        let store = self.store_for(project_id)?;
        let now = Utc::now().to_rfc3339();
        let aliases = patch.aliases.as_ref().map(|a| serde_json::to_string(a).unwrap_or_else(|_| "[]".into()));
        store
            .update_glossary(
                glossary_id,
                patch.zh.as_deref().map(str::trim),
                patch.en.as_deref().map(str::trim),
                patch.ar.as_deref().map(str::trim),
                patch.category.as_deref().map(str::trim),
                patch.notes.as_deref().map(str::trim),
                aliases.as_deref(),
                patch.locked,
                &now,
            )
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let row = store
            .get_glossary(glossary_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Glossary entry not found".into()))?;
        Ok(Self::to_entry(&row))
    }

    pub fn delete(&self, project_id: &str, glossary_id: &str) -> AppResult<()> {
        let store = self.store_for(project_id)?;
        store
            .delete_glossary(glossary_id)
            .map_err(|e| AppError::DeleteFailed(e.to_string()))?;
        self.logger.info("glossary:delete", Some(&serde_json::json!({ "id": glossary_id })));
        Ok(())
    }

    pub fn search(&self, project_id: &str, query: &str, limit: usize) -> AppResult<Vec<GlossarySearchResult>> {
        let store = self.store_for(project_id)?;
        let rows = store
            .search_glossary(query, limit)
            .map_err(|e| AppError::Db(e))?;
        Ok(rows
            .iter()
            .map(|r| GlossarySearchResult {
                id: r.id.clone(),
                zh: r.zh.clone(),
                en: r.en.clone(),
                ar: r.ar.clone(),
                category: r.category.clone(),
                snippet: snippet_for(&r.zh, query),
            })
            .collect())
    }

    /// Terms from glossary whose zh/alias appears in the source text (longest-match).
    pub fn detect_terms(&self, project_id: &str, source_text: &str, limit: usize) -> AppResult<Vec<GlossaryEntry>> {
        let store = self.store_for(project_id)?;
        let rows = store.list_glossary().map_err(|e| AppError::Db(e))?;
        let mut found: Vec<GlossaryEntry> = Vec::new();
        for row in &rows {
            let mut matched = source_text.contains(&row.zh);
            if !matched {
                if let Ok(aliases) = serde_json::from_str::<Vec<String>>(&row.aliases) {
                    matched = aliases.iter().any(|a| !a.is_empty() && source_text.contains(a));
                }
            }
            if matched {
                found.push(Self::to_entry(row));
            }
            if found.len() >= limit {
                break;
            }
        }
        found.sort_by(|a, b| b.zh.chars().count().cmp(&a.zh.chars().count()));
        Ok(found)
    }
}

fn to_glossary_entry(row: &GlossaryRow) -> GlossaryEntry {
    GlossaryEntry {
        id: row.id.clone(),
        zh: row.zh.clone(),
        en: row.en.clone(),
        ar: row.ar.clone(),
        category: row.category.clone(),
        notes: row.notes.clone(),
        aliases: serde_json::from_str(&row.aliases).unwrap_or_default(),
        locked: row.locked,
        source: row.source.clone(),
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

pub(crate) fn to_entry_for_test(row: GlossaryRow) -> GlossaryEntry {
    to_glossary_entry(&row)
}

fn snippet_for(text: &str, query: &str) -> String {
    if query.is_empty() {
        return text.chars().take(120).collect();
    }
    if let Some(pos) = text.find(query) {
        let start = pos.saturating_sub(20);
        let end = (pos + query.len() + 60).min(text.len());
        let s = &text[start..end];
        let mut out = String::new();
        if start > 0 {
            out.push('…');
        }
        out.push_str(s);
        if end < text.len() {
            out.push('…');
        }
        out
    } else {
        text.chars().take(120).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> (tempfile::TempDir, AppSettingsStore, crate::logger::Logger) {
        let dir = tempfile::tempdir().unwrap();
        let settings = AppSettingsStore::new(dir.path().to_path_buf());
        let logger = crate::logger::Logger::new(dir.path().join("logs"));
        (dir, settings, logger)
    }

    fn open_store(settings: &AppSettingsStore) -> ProjectStore {
        let ws = settings.workspace_path();
        ProjectStore::open(&ws.join("novel.db")).unwrap()
    }

    #[test]
    fn creates_and_lists_entries() {
        let (_dir, settings, logger) = ctx();
        let project = open_store(&settings);
        let pid = "proj1";
        let _ = project; // ensure schema created
        let manager = GlossaryManager::new(settings.clone(), &logger);
        // seed a registry entry pointing at the workspace db
        let ws = settings.workspace_path();
        settings.add_registry(crate::storage::app_settings::RegistryEntry {
            id: pid.into(),
            path: ws.to_string_lossy().to_string(),
            title: "Novel".into(),
            created_at: "now".into(),
        });
        let entry = manager
            .create(
                pid,
                CreateGlossaryInput {
                    zh: "剑客".into(),
                    en: "swordsman".into(),
                    ar: "سياف".into(),
                    category: "character".into(),
                    notes: String::new(),
                    aliases: vec!["剑客大师".into()],
                },
            )
            .unwrap();
        assert_eq!(entry.zh, "剑客");
        assert_eq!(manager.list(pid).unwrap().len(), 1);
    }
}
