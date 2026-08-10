use crate::error::{AppError, AppResult};
use crate::models::{
    ChapterDetail, ChapterMemory, ChapterSearchResult, ChapterSummary, CreateChapterInput,
    ImportChaptersInput, ImportChaptersResult, UpdateChapterInput,
};
use crate::storage::app_settings::AppSettingsStore;
use crate::storage::project_store::{ChapterRow, ProjectStore};
use chrono::Utc;
use std::path::PathBuf;

const CHAPTER_MARKER_RE: &str = r"(?m)^\s*(第\s*[0-9一二三四五六七八九十百千万零〇]+\s*[章回节卷篇]|(?:chapter|CHAPTER|Chapter)\s*\d+)\s*[:：.\-—]?\s*";

pub struct ChaptersManager<'a> {
    settings: AppSettingsStore,
    logger: &'a crate::logger::Logger,
}

impl<'a> ChaptersManager<'a> {
    pub fn new(settings: AppSettingsStore, logger: &'a crate::logger::Logger) -> Self {
        ChaptersManager { settings, logger }
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

    fn summary(row: &ChapterRow) -> ChapterSummary {
        ChapterSummary {
            id: row.id.clone(),
            number: row.number,
            title: row.title.clone(),
            word_count: row.word_count,
            status: row.status.clone(),
            created_at: row.created_at.clone(),
            updated_at: row.updated_at.clone(),
            translated_at: row.translated_at.clone(),
        }
    }

    pub fn list(&self, project_id: &str) -> AppResult<Vec<ChapterSummary>> {
        let store = self.store_for(project_id)?;
        let rows = store.list_chapters().map_err(|e| AppError::Db(e))?;
        Ok(rows.iter().map(Self::summary).collect())
    }

    pub fn get(&self, project_id: &str, chapter_id: &str) -> AppResult<ChapterDetail> {
        let store = self.store_for(project_id)?;
        let row = store
            .get_chapter(chapter_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Chapter not found".into()))?;
        Ok(ChapterDetail {
            id: row.id,
            number: row.number,
            title: row.title,
            source_text: row.source_text,
            translation: row.translation,
            word_count: row.word_count,
            status: row.status,
            created_at: row.created_at,
            updated_at: row.updated_at,
            translated_at: row.translated_at,
        })
    }

    pub fn get_memory(&self, project_id: &str, chapter_id: &str) -> AppResult<Option<ChapterMemory>> {
        let store = self.store_for(project_id)?;
        let row = store
            .get_chapter_summary(chapter_id)
            .map_err(|e| AppError::Db(e))?;
        Ok(row.map(|r| ChapterMemory {
            chapter_id: r.chapter_id,
            chapter_number: r.chapter_number,
            summary: r.summary,
            model: r.model,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub fn create(&self, project_id: &str, input: CreateChapterInput) -> AppResult<ChapterSummary> {
        let store = self.store_for(project_id)?;
        let number = match input.number {
            Some(n) => n,
            None => store.max_chapter_number().map_err(|e| AppError::Db(e))? + 1,
        };
        let now = Utc::now().to_rfc3339();
        let id = store
            .insert_chapter(number, &input.title, &input.source_text, &now)
            .map_err(|e| AppError::CreateFailed(e.to_string()))?;
        let row = store.get_chapter(&id).map_err(|e| AppError::Db(e))?;
        row.map(|r| Self::summary(&r))
            .ok_or_else(|| AppError::CreateFailed("Chapter not persisted".into()))
    }

    pub fn update(&self, project_id: &str, chapter_id: &str, patch: UpdateChapterInput) -> AppResult<ChapterSummary> {
        let store = self.store_for(project_id)?;
        let now = Utc::now().to_rfc3339();
        store
            .update_chapter(
                chapter_id,
                patch.title.as_deref(),
                patch.source_text.as_deref(),
                patch.translation.as_deref(),
                patch.status.as_deref(),
                &now,
            )
            .map_err(|e| AppError::UpdateFailed(e.to_string()))?;
        let row = store
            .get_chapter(chapter_id)
            .map_err(|e| AppError::Db(e))?
            .ok_or_else(|| AppError::NotFound("Chapter not found".into()))?;
        Ok(Self::summary(&row))
    }

    pub fn delete(&self, project_id: &str, chapter_id: &str) -> AppResult<()> {
        let store = self.store_for(project_id)?;
        store
            .delete_chapter(chapter_id)
            .map_err(|e| AppError::DeleteFailed(e.to_string()))?;
        Ok(())
    }

    pub fn search(&self, project_id: &str, query: &str, limit: usize) -> AppResult<Vec<ChapterSearchResult>> {
        let store = self.store_for(project_id)?;
        let rows = store
            .search_chapters(query, limit)
            .map_err(|e| AppError::Db(e))?;
        Ok(rows
            .into_iter()
            .map(|(id, number, title, snippet)| ChapterSearchResult {
                id,
                number,
                title,
                snippet,
            })
            .collect())
    }

    pub fn import(&self, project_id: &str, input: ImportChaptersInput) -> AppResult<ImportChaptersResult> {
        let store = self.store_for(project_id)?;
        let text = input.text.trim();
        if text.is_empty() {
            return Err(AppError::InvalidInput("Import text is empty".into()));
        }
        let parts = split_into_chapters(text, &input.split_by);
        let mut start = store.max_chapter_number().map_err(|e| AppError::Db(e))? + 1;
        let now = Utc::now().to_rfc3339();
        let mut imported = 0usize;
        let mut skipped = 0usize;
        let mut created: Vec<ChapterSummary> = Vec::new();
        for (title, body) in parts {
            if body.trim().is_empty() {
                skipped += 1;
                continue;
            }
            let id = store
                .insert_chapter(start, &title, &body, &now)
                .map_err(|e| AppError::CreateFailed(e.to_string()))?;
            start += 1;
            imported += 1;
            if let Ok(Some(row)) = store.get_chapter(&id) {
                created.push(Self::summary(&row));
            }
        }
        self.logger
            .info("chapters:import", Some(&serde_json::json!({ "projectId": project_id, "imported": imported, "skipped": skipped })));
        Ok(ImportChaptersResult {
            imported,
            skipped,
            chapters: created,
        })
    }
}

pub fn split_into_chapters(text: &str, mode: &str) -> Vec<(String, String)> {
    let re = regex::Regex::new(CHAPTER_MARKER_RE).unwrap();
    match mode {
        "marker" => split_by_marker(text, &re),
        "paragraphs" => split_by_paragraphs(text),
        _ => {
            let marked = split_by_marker(text, &re);
            if marked.len() >= 2 {
                marked
            } else {
                split_by_paragraphs(text)
            }
        }
    }
}

fn split_by_marker(text: &str, re: &regex::Regex) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current_title = String::new();
    let mut current_body = String::new();
    for line in text.lines() {
        if re.is_match(line) {
            if !current_title.is_empty() || !current_body.trim().is_empty() {
                out.push((std::mem::take(&mut current_title), std::mem::take(&mut current_body)));
            }
            current_title = line.trim().to_string();
        } else if current_title.is_empty() {
            current_title = "Chapter".to_string();
            current_body.push_str(line);
            current_body.push('\n');
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if !current_title.is_empty() || !current_body.trim().is_empty() {
        out.push((current_title, current_body));
    }
    if out.is_empty() {
        out.push(("Chapter".to_string(), text.to_string()));
    }
    out
}

fn split_by_paragraphs(text: &str) -> Vec<(String, String)> {
    const CHUNK: usize = 40;
    let paragraphs: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if paragraphs.is_empty() {
        return vec![("Chapter".to_string(), text.to_string())];
    }
    let mut out = Vec::new();
    let mut chunks: Vec<Vec<&str>> = paragraphs
        .chunks(CHUNK)
        .map(|c| c.to_vec())
        .collect();
    if chunks.is_empty() {
        chunks.push(paragraphs);
    }
    for (i, chunk) in chunks.iter().enumerate() {
        out.push((format!("Chapter {}", i + 1), chunk.join("\n")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_by_markers() {
        let text = "第一章 觉醒\n\n他睁开了眼。\n\n第二章 逃亡\n\n他跑了。\n";
        let parts = split_into_chapters(text, "marker");
        assert_eq!(parts.len(), 2);
        assert!(parts[0].0.contains("第一章"));
        assert!(parts[1].0.contains("第二章"));
        assert!(parts[0].1.contains("他睁开了眼"));
    }

    #[test]
    fn splits_into_paragraph_chunks_when_no_markers() {
        let mut body = String::new();
        for i in 0..90 {
            body.push_str(&format!("段落第{i}行。\n"));
        }
        let parts = split_into_chapters(&body, "auto");
        assert!(parts.len() >= 2);
        assert!(parts.iter().all(|(_, b)| b.chars().count() > 0));
    }
}
