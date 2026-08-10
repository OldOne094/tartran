use crate::error::{AppError, AppResult};
use crate::models::{CreateSuggestionInput, GlossaryEntry, Suggestion, UpdateSuggestionInput};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::project_store::ProjectStore;
use chrono::Utc;
use std::path::PathBuf;

pub struct SuggestionsManager<'a> {
    settings: AppSettingsStore,
    logger: &'a crate::logger::Logger,
}

impl<'a> SuggestionsManager<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a crate::logger::Logger) -> Self {
        SuggestionsManager { settings, logger }
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

    fn to_suggestion(row: &crate::storage::project_store::SuggestionRow) -> Suggestion {
        Suggestion {
            id: row.id.clone(),
            chapter_id: row.chapter_id.clone(),
            zh: row.zh.clone(),
            en: row.en.clone(),
            ar: row.ar.clone(),
            category: row.category.clone(),
            notes: row.notes.clone(),
            context: row.context.clone(),
            status: row.status.clone(),
            created_at: row.created_at.clone(),
        }
    }

    pub fn list(&self, project_id: &str, chapter_id: &str) -> AppResult<Vec<Suggestion>> {
        let store = self.store_for(project_id)?;
        let rows = store.list_suggestions(chapter_id).map_err(|e| AppError::Db(e))?;
        Ok(rows.iter().map(Self::to_suggestion).collect())
    }

    pub fn create(&self, project_id: &str, input: CreateSuggestionInput) -> AppResult<Suggestion> {
        let store = self.store_for(project_id)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        store
            .insert_suggestion(
                &id,
                &input.chapter_id,
                input.zh.trim(),
                input.en.trim(),
                input.ar.trim(),
                input.category.trim(),
                input.notes.trim(),
                input.context.trim(),
                &now,
            )
            .map_err(|e| AppError::CreateFailed(e.to_string()))?;
        let row = store
            .get_suggestion(&id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Suggestion not persisted".into()))?;
        Ok(Self::to_suggestion(&row))
    }

    pub fn update(
        &self,
        project_id: &str,
        suggestion_id: &str,
        patch: UpdateSuggestionInput,
    ) -> AppResult<Suggestion> {
        let store = self.store_for(project_id)?;
        store
            .update_suggestion(
                suggestion_id,
                patch.status.as_deref(),
                patch.en.as_deref().map(str::trim),
                patch.ar.as_deref().map(str::trim),
                patch.category.as_deref().map(str::trim),
                patch.notes.as_deref().map(str::trim),
            )
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let row = store
            .get_suggestion(suggestion_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Suggestion not found".into()))?;
        Ok(Self::to_suggestion(&row))
    }

    /// Approve a suggestion: write it into the glossary (dedupe by zh) and mark approved.
    pub fn approve(&self, project_id: &str, suggestion_id: &str) -> AppResult<GlossaryEntry> {
        let store = self.store_for(project_id)?;
        let row = store
            .get_suggestion(suggestion_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Suggestion not found".into()))?;
        let existing = store
            .list_glossary()
            .map_err(|e| AppError::Db(e))?
            .into_iter()
            .find(|g| g.zh == row.zh);
        let now = Utc::now().to_rfc3339();
        let glossary_id = match existing {
            Some(g) => {
                store
                    .update_glossary(
                        &g.id,
                        None,
                        if row.en.is_empty() { None } else { Some(&row.en) },
                        if row.ar.is_empty() { None } else { Some(&row.ar) },
                        if row.category.is_empty() { None } else { Some(&row.category) },
                        if row.notes.is_empty() { None } else { Some(&row.notes) },
                        None,
                        None,
                        &now,
                    )
                    .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
                g.id
            }
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                store
                    .insert_glossary(
                        &id,
                        &row.zh,
                        &row.en,
                        &row.ar,
                        &row.category,
                        &row.notes,
                        "[]",
                        false,
                        "suggestion",
                        &now,
                    )
                    .map_err(|e| AppError::CreateFailed(e.to_string()))?;
                id
            }
        };
        store
            .update_suggestion(suggestion_id, Some("approved"), None, None, None, None)
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let g = store
            .get_glossary(&glossary_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Glossary entry not found".into()))?;
        self.logger.info(
            "suggestion:approve",
            Some(&serde_json::json!({ "projectId": project_id, "zh": g.zh })),
        );
        Ok(crate::storage::glossary::to_entry_for_test(g))
    }

    pub fn reject(&self, project_id: &str, suggestion_id: &str) -> AppResult<Suggestion> {
        let store = self.store_for(project_id)?;
        store
            .update_suggestion(suggestion_id, Some("rejected"), None, None, None, None)
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let row = store
            .get_suggestion(suggestion_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Suggestion not found".into()))?;
        Ok(Self::to_suggestion(&row))
    }

    pub fn delete(&self, project_id: &str, suggestion_id: &str) -> AppResult<()> {
        let store = self.store_for(project_id)?;
        store
            .delete_suggestion(suggestion_id)
            .map_err(|e| AppError::DeleteFailed(e.to_string()))?;
        Ok(())
    }
}
