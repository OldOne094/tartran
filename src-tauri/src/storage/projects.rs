use crate::error::{AppError, AppResult};
use crate::logger::Logger;
use crate::models::{CreateProjectInput, ProjectSummary, UpdateProjectInput};
use crate::storage::app_settings::{AppSettingsStore, RegistryEntry};
use crate::storage::project_store::ProjectStore;
use chrono::Utc;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

const FOLDER_SUFFIX_LEN: usize = 8;

pub struct ProjectsManager<'a> {
    settings: AppSettingsStore,
    logger: &'a Logger,
}

impl<'a> ProjectsManager<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a Logger) -> Self {
        ProjectsManager { settings, logger }
    }

    pub fn create(&self, input: CreateProjectInput) -> AppResult<ProjectSummary> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let folder = self.folder_for(&id, &input.title);
        fs::create_dir_all(&folder).map_err(|e| AppError::CreateFailed(e.to_string()))?;

        let store =
            ProjectStore::open(&folder.join("novel.db")).map_err(|e| AppError::CreateFailed(e.to_string()))?;
        let meta = [
            ("id", id.as_str()),
            ("title", input.title.as_str()),
            ("author", input.author.as_deref().unwrap_or("")),
            ("sourceLang", input.source_lang.as_deref().unwrap_or("zh")),
            ("targetLang", input.target_lang.as_str()),
            ("createdAt", now.as_str()),
            ("updatedAt", now.as_str()),
        ];
        store
            .set_meta_many(&meta)
            .map_err(|e| AppError::CreateFailed(e.to_string()))?;

        self.settings.add_registry(RegistryEntry {
            id: id.clone(),
            path: folder.to_string_lossy().to_string(),
            title: input.title.clone(),
            created_at: now.clone(),
        });
        self.logger
            .info("project:create", Some(&json!({ "id": id, "title": input.title })));
        self.read_summary(&id)
    }

    pub fn list(&self) -> Vec<ProjectSummary> {
        let mut out: Vec<ProjectSummary> = self
            .settings
            .registry()
            .iter()
            .map(|entry| match self.read_summary(&entry.id) {
                Ok(s) => s,
                Err(e) => {
                    self.logger
                        .warn("project:unreadable", Some(&json!({ "id": entry.id, "err": e.to_string() })));
                    ProjectSummary {
                        id: entry.id.clone(),
                        title: entry.title.clone(),
                        author: String::new(),
                        source_lang: "zh".to_string(),
                        target_lang: "ar".to_string(),
                        created_at: entry.created_at.clone(),
                        updated_at: entry.created_at.clone(),
                        chapter_count: 0,
                        translated_count: 0,
                        reviewed_count: 0,
                        corrupted: Some(true),
                    }
                }
            })
            .collect();
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        out
    }

    pub fn get(&self, id: &str) -> AppResult<ProjectSummary> {
        self.read_summary(id)
    }

    pub fn update(&self, id: &str, patch: UpdateProjectInput) -> AppResult<ProjectSummary> {
        let entry = self.entry(id)?;
        let store = ProjectStore::open(&PathBuf::from(&entry.path).join("novel.db"))
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let mut meta = store
            .get_meta()
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        if let Some(title) = &patch.title {
            meta.insert("title".into(), title.clone());
        }
        if let Some(author) = &patch.author {
            meta.insert("author".into(), author.clone());
        }
        if let Some(target_lang) = &patch.target_lang {
            meta.insert("targetLang".into(), target_lang.clone());
        }
        let updated_at = Utc::now().to_rfc3339();
        meta.insert("updatedAt".into(), updated_at.clone());
        store
            .set_meta_many(&meta.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect::<Vec<_>>())
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;

        if let Some(title) = patch.title {
            if title != entry.title {
                self.settings.remove_registry(id);
                self.settings.add_registry(RegistryEntry {
                    title,
                    ..entry
                });
            }
        }
        self.read_summary(id)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let entry = self.entry(id)?;
        fs::remove_dir_all(&entry.path).map_err(|e| AppError::DeleteFailed(e.to_string()))?;
        self.settings.remove_registry(id);
        self.logger.info("project:delete", Some(&json!({ "id": id })));
        Ok(())
    }

    fn folder_for(&self, id: &str, title: &str) -> PathBuf {
        let safe: String = title
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '_' || *c == '-')
            .take(40)
            .collect();
        let safe = if safe.is_empty() { "novel".to_string() } else { safe };
        self.settings
            .workspace_path()
            .join(format!("{safe}-{}", &id[..FOLDER_SUFFIX_LEN]))
    }

    fn entry(&self, id: &str) -> AppResult<RegistryEntry> {
        self.settings
            .registry()
            .into_iter()
            .find(|e| e.id == id)
            .ok_or_else(|| AppError::NotFound(format!("Project not found: {id}")))
    }

    fn read_summary(&self, id: &str) -> AppResult<ProjectSummary> {
        let entry = self.entry(id)?;
        let store = ProjectStore::open(&PathBuf::from(&entry.path).join("novel.db"))?;
        let meta = store.get_meta()?;
        let counts = store.status_counts()?;
        let created_at = meta.get("createdAt").cloned().unwrap_or_else(|| entry.created_at.clone());
        let updated_at = meta.get("updatedAt").cloned().unwrap_or_else(|| created_at.clone());
        Ok(ProjectSummary {
            id: id.to_string(),
            title: meta.get("title").cloned().unwrap_or_else(|| entry.title.clone()),
            author: meta.get("author").cloned().unwrap_or_default(),
            source_lang: "zh".to_string(),
            target_lang: match meta.get("targetLang").map(|s| s.as_str()) {
                Some("en") => "en",
                _ => "ar",
            }
            .to_string(),
            created_at,
            updated_at,
            chapter_count: store.chapter_count()?,
            translated_count: counts.translated,
            reviewed_count: counts.reviewed,
            corrupted: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCtx {
        _dir: tempfile::TempDir,
        settings: AppSettingsStore,
        logger: Logger,
    }

    impl TestCtx {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let settings = AppSettingsStore::new(dir.path().to_path_buf());
            let logger = Logger::new(dir.path().join("logs"));
            TestCtx {
                _dir: dir,
                settings,
                logger,
            }
        }

        fn projects(&self) -> ProjectsManager<'_> {
            ProjectsManager::new(self.settings.clone(), &self.logger)
        }
    }

    fn create_input(title: &str) -> CreateProjectInput {
        CreateProjectInput {
            title: title.to_string(),
            author: None,
            target_lang: "ar".to_string(),
            source_lang: None,
        }
    }

    #[test]
    fn creates_project_with_metadata_and_zero_chapters() {
        let ctx = TestCtx::new();
        let p = ctx.projects().create(create_input("Novel A")).unwrap();
        assert_eq!(p.title, "Novel A");
        assert_eq!(p.target_lang, "ar");
        assert_eq!(p.source_lang, "zh");
        assert_eq!(p.chapter_count, 0);
        assert_eq!(ctx.settings.registry().len(), 1);
        let folder = ctx
            .settings
            .workspace_path()
            .join(format!("Novel A-{}", &p.id[..8]));
        assert!(folder.exists());
    }

    #[test]
    fn persists_across_fresh_manager() {
        let ctx = TestCtx::new();
        ctx.projects().create(create_input("Novel A")).unwrap();
        let logger = Logger::new(ctx._dir.path().join("logs"));
        let reopened = ProjectsManager::new(ctx.settings.clone(), &logger);
        let list = reopened.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Novel A");
        assert_eq!(list[0].target_lang, "ar");
    }

    #[test]
    fn updates_metadata_and_persists_it() {
        let ctx = TestCtx::new();
        let p = ctx.projects().create(create_input("Novel A")).unwrap();
        let updated = ctx
            .projects()
            .update(
                &p.id,
                UpdateProjectInput {
                    title: None,
                    author: Some("Some Author".into()),
                    target_lang: Some("en".into()),
                },
            )
            .unwrap();
        assert_eq!(updated.author, "Some Author");
        assert_eq!(updated.target_lang, "en");

        let logger = Logger::new(ctx._dir.path().join("logs"));
        let reread = ProjectsManager::new(ctx.settings.clone(), &logger).get(&p.id).unwrap();
        assert_eq!(reread.author, "Some Author");
        assert_eq!(reread.target_lang, "en");
    }

    #[test]
    fn deletes_project_folder_and_registry_entry() {
        let ctx = TestCtx::new();
        let p = ctx.projects().create(create_input("Novel A")).unwrap();
        let folder = ctx
            .settings
            .workspace_path()
            .join(format!("Novel A-{}", &p.id[..8]));
        assert!(folder.exists());

        ctx.projects().delete(&p.id).unwrap();
        assert_eq!(ctx.settings.registry().len(), 0);
        assert!(!folder.exists());
    }

    #[test]
    fn keeps_same_folder_when_title_is_renamed() {
        let ctx = TestCtx::new();
        let p = ctx.projects().create(create_input("Novel A")).unwrap();
        let folder = ctx
            .settings
            .workspace_path()
            .join(format!("Novel A-{}", &p.id[..8]));

        let updated = ctx
            .projects()
            .update(
                &p.id,
                UpdateProjectInput {
                    title: Some("Novel B".into()),
                    author: None,
                    target_lang: None,
                },
            )
            .unwrap();
        assert_eq!(updated.title, "Novel B");
        assert_eq!(ctx.settings.registry()[0].title, "Novel B");
        assert!(folder.exists());
    }
}
