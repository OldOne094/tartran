use crate::text::count_units;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chapters (
  id           TEXT PRIMARY KEY,
  number       INTEGER NOT NULL,
  title        TEXT NOT NULL DEFAULT '',
  source_text  TEXT NOT NULL DEFAULT '',
  translation  TEXT NOT NULL DEFAULT '',
  status       TEXT NOT NULL DEFAULT 'imported',
  word_count   INTEGER NOT NULL DEFAULT 0,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  translated_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_chapters_number ON chapters(number);
CREATE INDEX IF NOT EXISTS idx_chapters_status ON chapters(status);

CREATE VIRTUAL TABLE IF NOT EXISTS chapters_fts USING fts5(
  source_text, translation, content='chapters', content_rowid='rowid', tokenize='trigram'
);
CREATE TRIGGER IF NOT EXISTS chapters_fts_ai AFTER INSERT ON chapters BEGIN
  INSERT INTO chapters_fts(rowid, source_text, translation)
  VALUES (new.rowid, new.source_text, new.translation);
END;
CREATE TRIGGER IF NOT EXISTS chapters_fts_ad AFTER DELETE ON chapters BEGIN
  INSERT INTO chapters_fts(chapters_fts, rowid, source_text, translation)
  VALUES ('delete', old.rowid, old.source_text, old.translation);
END;
CREATE TRIGGER IF NOT EXISTS chapters_fts_au AFTER UPDATE ON chapters BEGIN
  INSERT INTO chapters_fts(chapters_fts, rowid, source_text, translation)
  VALUES ('delete', old.rowid, old.source_text, old.translation);
  INSERT INTO chapters_fts(rowid, source_text, translation)
  VALUES (new.rowid, new.source_text, new.translation);
END;

CREATE TABLE IF NOT EXISTS glossary (
  id         TEXT PRIMARY KEY,
  zh         TEXT NOT NULL,
  en         TEXT NOT NULL DEFAULT '',
  ar         TEXT NOT NULL DEFAULT '',
  category   TEXT NOT NULL DEFAULT '',
  notes      TEXT NOT NULL DEFAULT '',
  aliases    TEXT NOT NULL DEFAULT '[]',
  locked     INTEGER NOT NULL DEFAULT 0,
  source     TEXT NOT NULL DEFAULT 'manual',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS glossary_fts USING fts5(
  zh, en, ar, content='glossary', content_rowid='rowid', tokenize='trigram'
);
CREATE TRIGGER IF NOT EXISTS glossary_fts_ai AFTER INSERT ON glossary BEGIN
  INSERT INTO glossary_fts(rowid, zh, en, ar)
  VALUES (new.rowid, new.zh, new.en, new.ar);
END;
CREATE TRIGGER IF NOT EXISTS glossary_fts_ad AFTER DELETE ON glossary BEGIN
  INSERT INTO glossary_fts(glossary_fts, rowid, zh, en, ar)
  VALUES ('delete', old.rowid, old.zh, old.en, old.ar);
END;
CREATE TRIGGER IF NOT EXISTS glossary_fts_au AFTER UPDATE ON glossary BEGIN
  INSERT INTO glossary_fts(glossary_fts, rowid, zh, en, ar)
  VALUES ('delete', old.rowid, old.zh, old.en, old.ar);
  INSERT INTO glossary_fts(rowid, zh, en, ar)
  VALUES (new.rowid, new.zh, new.en, new.ar);
END;

CREATE TABLE IF NOT EXISTS suggestions (
  id         TEXT PRIMARY KEY,
  chapter_id TEXT NOT NULL,
  zh         TEXT NOT NULL,
  en         TEXT NOT NULL DEFAULT '',
  ar         TEXT NOT NULL DEFAULT '',
  category   TEXT NOT NULL DEFAULT '',
  notes      TEXT NOT NULL DEFAULT '',
  context    TEXT NOT NULL DEFAULT '',
  status     TEXT NOT NULL DEFAULT 'pending',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_suggestions_status ON suggestions(status);

CREATE TABLE IF NOT EXISTS chapter_summaries (
  chapter_id     TEXT PRIMARY KEY,
  chapter_number INTEGER NOT NULL,
  summary        TEXT NOT NULL DEFAULT '',
  model          TEXT NOT NULL DEFAULT '',
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_chapter_summaries_number ON chapter_summaries(chapter_number);

PRAGMA user_version = 1;
"#;

pub struct ChapterStatusCounts {
    pub translated: i64,
    pub reviewed: i64,
}

pub struct ChapterRow {
    pub id: String,
    pub number: i64,
    pub title: String,
    pub source_text: String,
    pub translation: String,
    pub status: String,
    pub word_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub translated_at: Option<String>,
}

pub struct GlossaryRow {
    pub id: String,
    pub zh: String,
    pub en: String,
    pub ar: String,
    pub category: String,
    pub notes: String,
    pub aliases: String,
    pub locked: bool,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct SuggestionRow {
    pub id: String,
    pub chapter_id: String,
    pub zh: String,
    pub en: String,
    pub ar: String,
    pub category: String,
    pub notes: String,
    pub context: String,
    pub status: String,
    pub created_at: String,
}

impl ChapterRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(ChapterRow {
            id: row.get(0)?,
            number: row.get(1)?,
            title: row.get(2)?,
            source_text: row.get(3)?,
            translation: row.get(4)?,
            status: row.get(5)?,
            word_count: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
            translated_at: row.get(9)?,
        })
    }
}

impl GlossaryRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(GlossaryRow {
            id: row.get(0)?,
            zh: row.get(1)?,
            en: row.get(2)?,
            ar: row.get(3)?,
            category: row.get(4)?,
            notes: row.get(5)?,
            aliases: row.get(6)?,
            locked: row.get(7)?,
            source: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }
}

impl SuggestionRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(SuggestionRow {
            id: row.get(0)?,
            chapter_id: row.get(1)?,
            zh: row.get(2)?,
            en: row.get(3)?,
            ar: row.get(4)?,
            category: row.get(5)?,
            notes: row.get(6)?,
            context: row.get(7)?,
            status: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}

pub struct ChapterSummaryRow {
    pub chapter_id: String,
    pub chapter_number: i64,
    pub summary: String,
    pub model: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ChapterSummaryRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(ChapterSummaryRow {
            chapter_id: row.get(0)?,
            chapter_number: row.get(1)?,
            summary: row.get(2)?,
            model: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}

pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(SCHEMA)?;
        Ok(ProjectStore { conn })
    }

    pub fn get_meta(&self) -> rusqlite::Result<HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM meta")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }

    pub fn set_meta_many(&self, entries: &[(impl AsRef<str>, impl AsRef<str>)]) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO meta(key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )?;
            for (k, v) in entries {
                stmt.execute(params![k.as_ref(), v.as_ref()])?;
            }
        }
        tx.commit()
    }

    pub fn chapter_count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT count(*) AS c FROM chapters", [], |row| row.get(0))
    }

    pub fn status_counts(&self) -> rusqlite::Result<ChapterStatusCounts> {
        self.conn
            .query_row(
                "SELECT
                   sum(CASE WHEN status IN ('translated','reviewed','exported') THEN 1 ELSE 0 END) AS translated,
                   sum(CASE WHEN status IN ('reviewed','exported') THEN 1 ELSE 0 END) AS reviewed
                 FROM chapters",
                [],
                |row| {
                    Ok(ChapterStatusCounts {
                        translated: row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                        reviewed: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    })
                },
            )
    }

    pub fn max_chapter_number(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT coalesce(max(number), 0) FROM chapters", [], |row| row.get(0))
    }

    pub fn insert_chapter(
        &self,
        number: i64,
        title: &str,
        source_text: &str,
        now: &str,
    ) -> rusqlite::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let word_count = count_units(source_text);
        self.conn.execute(
            "INSERT INTO chapters (id, number, title, source_text, translation, status, word_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '', 'imported', ?5, ?6, ?6)",
            params![id, number, title, source_text, word_count, now],
        )?;
        Ok(id)
    }

    pub fn list_chapters(&self) -> rusqlite::Result<Vec<ChapterRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, number, title, source_text, translation, status, word_count, created_at, updated_at, translated_at
             FROM chapters ORDER BY number ASC, created_at ASC",
        )?;
        let rows = stmt.query_map([], ChapterRow::from_row)?;
        rows.collect()
    }

    pub fn get_chapter(&self, id: &str) -> rusqlite::Result<Option<ChapterRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, number, title, source_text, translation, status, word_count, created_at, updated_at, translated_at
             FROM chapters WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], ChapterRow::from_row)?;
        rows.next().transpose()
    }

    pub fn update_chapter(
        &self,
        id: &str,
        title: Option<&str>,
        source_text: Option<&str>,
        translation: Option<&str>,
        status: Option<&str>,
        now: &str,
    ) -> rusqlite::Result<()> {
        let current = self.get_chapter(id)?;
        let Some(current) = current else {
            return Ok(());
        };
        let next_title = title.unwrap_or(&current.title).to_string();
        let next_source = source_text.unwrap_or(&current.source_text).to_string();
        let next_translation = translation.unwrap_or(&current.translation).to_string();
        let next_status = status.unwrap_or(&current.status).to_string();
        let word_count = count_units(&next_source);
        let translated_at = if translation.is_some() {
            Some(now.to_string())
        } else {
            current.translated_at.clone()
        };
        self.conn.execute(
            "UPDATE chapters
             SET title = ?1, source_text = ?2, translation = ?3, status = ?4,
                 word_count = ?5, updated_at = ?6, translated_at = ?7
             WHERE id = ?8",
            params![
                next_title,
                next_source,
                next_translation,
                next_status,
                word_count,
                now,
                translated_at,
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_chapter(&self, id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM chapters WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Replace `old` with `new` inside chapter translations (all chapters, or a
    /// single chapter when `chapter_id` is given). Returns the number of chapters
    /// whose translation actually changed.
    pub fn replace_in_translations(
        &self,
        old: &str,
        new: &str,
        chapter_id: Option<&str>,
        now: &str,
    ) -> rusqlite::Result<usize> {
        if old.trim().is_empty() || old == new {
            return Ok(0);
        }
        let chapters = match chapter_id {
            Some(id) => {
                let row = self.get_chapter(id)?;
                vec![row.ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?]
            }
            None => self.list_chapters()?,
        };
        let mut changed = 0usize;
        for c in &chapters {
            if c.translation.is_empty() || !c.translation.contains(old) {
                continue;
            }
            let next = c.translation.replace(old, new);
            self.update_chapter(&c.id, None, None, Some(&next), None, now)?;
            changed += 1;
        }
        Ok(changed)
    }

    pub fn search_chapters(
        &self,
        query: &str,
        limit: usize,
    ) -> rusqlite::Result<Vec<(String, i64, String, String)>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        if query.chars().count() >= 3 {
            let phrase = query.replace('"', "\"\"");
            let match_expr = format!("\"{phrase}\"");
            let sql = format!(
                "SELECT c.id, c.number, c.title, snippet(chapters_fts, 0, '[', ']', '…', 8)
                 FROM chapters_fts JOIN chapters c ON c.rowid = chapters_fts.rowid
                 WHERE chapters_fts MATCH ?1
                 ORDER BY rank LIMIT ?2"
            );
            if let Ok(mut stmt) = self.conn.prepare(&sql) {
                let rows = stmt.query_map(
                    params![match_expr, limit as i64],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?)),
                );
                if let Ok(rows) = rows {
                    out.extend(rows.flatten());
                }
            }
        }
        if out.len() < limit {
            let like = format!("%{}%", query.replace('%', r"\%").replace('_', r"\_"));
            let needed = limit - out.len();
            let mut stmt = self.conn.prepare(
                "SELECT id, number, title, substr(source_text, 1, 160) FROM chapters
                 WHERE title LIKE ?1 ESCAPE '\\' OR source_text LIKE ?1 ESCAPE '\\'
                 ORDER BY number ASC LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![like, needed as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))
            })?;
            out.extend(rows.flatten());
        }
        Ok(out)
    }

    pub fn list_glossary(&self) -> rusqlite::Result<Vec<GlossaryRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, zh, en, ar, category, notes, aliases, locked, source, created_at, updated_at
             FROM glossary ORDER BY zh COLLATE NOCASE ASC",
        )?;
        let rows = stmt.query_map([], GlossaryRow::from_row)?;
        rows.collect()
    }

    pub fn insert_glossary(
        &self,
        id: &str,
        zh: &str,
        en: &str,
        ar: &str,
        category: &str,
        notes: &str,
        aliases: &str,
        locked: bool,
        source: &str,
        now: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO glossary (id, zh, en, ar, category, notes, aliases, locked, source, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)",
            params![id, zh, en, ar, category, notes, aliases, locked, source, now],
        )?;
        Ok(())
    }

    pub fn update_glossary(
        &self,
        id: &str,
        zh: Option<&str>,
        en: Option<&str>,
        ar: Option<&str>,
        category: Option<&str>,
        notes: Option<&str>,
        aliases: Option<&str>,
        locked: Option<bool>,
        now: &str,
    ) -> rusqlite::Result<()> {
        let current = self.get_glossary(id)?;
        let Some(current) = current else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE glossary SET zh=?1, en=?2, ar=?3, category=?4, notes=?5, aliases=?6, locked=?7, updated_at=?8
             WHERE id=?9",
            params![
                zh.unwrap_or(&current.zh),
                en.unwrap_or(&current.en),
                ar.unwrap_or(&current.ar),
                category.unwrap_or(&current.category),
                notes.unwrap_or(&current.notes),
                aliases.unwrap_or(&current.aliases),
                locked.unwrap_or(current.locked),
                now,
                id
            ],
        )?;
        Ok(())
    }

    pub fn get_glossary(&self, id: &str) -> rusqlite::Result<Option<GlossaryRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, zh, en, ar, category, notes, aliases, locked, source, created_at, updated_at
             FROM glossary WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], GlossaryRow::from_row)?;
        rows.next().transpose()
    }

    pub fn delete_glossary(&self, id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM glossary WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn search_glossary(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<GlossaryRow>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        if query.chars().count() >= 3 {
            let phrase = query.replace('"', "\"\"");
            let match_expr = format!("\"{phrase}\"");
            let sql = format!(
                "SELECT g.id, g.zh, g.en, g.ar, g.category, g.notes, g.aliases, g.locked, g.source, g.created_at, g.updated_at
                 FROM glossary_fts JOIN glossary g ON g.rowid = glossary_fts.rowid
                 WHERE glossary_fts MATCH ?1
                 ORDER BY rank LIMIT ?2"
            );
            if let Ok(mut stmt) = self.conn.prepare(&sql) {
                let rows = stmt.query_map(
                    params![match_expr, limit as i64],
                    GlossaryRow::from_row,
                );
                if let Ok(rows) = rows {
                    out.extend(rows.flatten());
                }
            }
        }
        if out.len() < limit {
            let like = format!("%{}%", query.replace('%', r"\%").replace('_', r"\_"));
            let needed = limit - out.len();
            let mut stmt = self.conn.prepare(
                "SELECT id, zh, en, ar, category, notes, aliases, locked, source, created_at, updated_at
                 FROM glossary
                 WHERE zh LIKE ?1 ESCAPE '\\' OR en LIKE ?1 ESCAPE '\\' OR ar LIKE ?1 ESCAPE '\\' OR aliases LIKE ?1 ESCAPE '\\'
                 ORDER BY zh COLLATE NOCASE ASC LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![like, needed as i64], GlossaryRow::from_row)?;
            out.extend(rows.flatten());
        }
        Ok(out)
    }

    pub fn list_suggestions(&self, chapter_id: &str) -> rusqlite::Result<Vec<SuggestionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chapter_id, zh, en, ar, category, notes, context, status, created_at
             FROM suggestions WHERE chapter_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![chapter_id], SuggestionRow::from_row)?;
        rows.collect()
    }

    pub fn insert_suggestion(
        &self,
        id: &str,
        chapter_id: &str,
        zh: &str,
        en: &str,
        ar: &str,
        category: &str,
        notes: &str,
        context: &str,
        now: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO suggestions (id, chapter_id, zh, en, ar, category, notes, context, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9)",
            params![id, chapter_id, zh, en, ar, category, notes, context, now],
        )?;
        Ok(())
    }

    pub fn get_suggestion(&self, id: &str) -> rusqlite::Result<Option<SuggestionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chapter_id, zh, en, ar, category, notes, context, status, created_at
             FROM suggestions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], SuggestionRow::from_row)?;
        rows.next().transpose()
    }

    pub fn update_suggestion(
        &self,
        id: &str,
        status: Option<&str>,
        zh: Option<&str>,
        en: Option<&str>,
        ar: Option<&str>,
        category: Option<&str>,
        notes: Option<&str>,
    ) -> rusqlite::Result<()> {
        let current = self.get_suggestion(id)?;
        let Some(current) = current else {
            return Ok(());
        };
        self.conn.execute(
            "UPDATE suggestions SET status=?1, zh=?2, en=?3, ar=?4, category=?5, notes=?6 WHERE id=?7",
            params![
                status.unwrap_or(&current.status),
                zh.unwrap_or(&current.zh),
                en.unwrap_or(&current.en),
                ar.unwrap_or(&current.ar),
                category.unwrap_or(&current.category),
                notes.unwrap_or(&current.notes),
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_suggestion(&self, id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM suggestions WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Insert or refresh the AI-generated summary for a chapter.
    pub fn upsert_chapter_summary(
        &self,
        chapter_id: &str,
        chapter_number: i64,
        summary: &str,
        model: &str,
        now: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO chapter_summaries (chapter_id, chapter_number, summary, model, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(chapter_id) DO UPDATE SET
               chapter_number = excluded.chapter_number,
               summary = excluded.summary,
               model = excluded.model,
               updated_at = excluded.updated_at",
            params![chapter_id, chapter_number, summary, model, now],
        )?;
        Ok(())
    }

    pub fn get_chapter_summary(&self, chapter_id: &str) -> rusqlite::Result<Option<ChapterSummaryRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chapter_id, chapter_number, summary, model, created_at, updated_at
             FROM chapter_summaries WHERE chapter_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![chapter_id], ChapterSummaryRow::from_row)?;
        rows.next().transpose()
    }

    /// Summaries of the most recent chapters strictly before `before_number`, ascending by number.
    pub fn list_summaries_before(
        &self,
        before_number: i64,
        limit: usize,
    ) -> rusqlite::Result<Vec<ChapterSummaryRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT chapter_id, chapter_number, summary, model, created_at, updated_at
             FROM chapter_summaries
             WHERE chapter_number < ?1
             ORDER BY chapter_number DESC, updated_at DESC
             LIMIT ?2",
        )?;
        let mut rows = stmt
            .query_map(params![before_number, limit as i64], ChapterSummaryRow::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.reverse();
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_temp() -> (tempfile::TempDir, ProjectStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProjectStore::open(&dir.path().join("novel.db")).unwrap();
        (dir, store)
    }

    #[test]
    fn opens_schema_and_sets_meta() {
        let (_dir, store) = open_temp();
        assert!(store.get_meta().unwrap().is_empty());
        store
            .set_meta_many(&[("id", "abc"), ("title", "Novel")])
            .unwrap();
        let meta = store.get_meta().unwrap();
        assert_eq!(meta.get("title").unwrap(), "Novel");
    }

    #[test]
    fn fts5_trigram_table_created() {
        let (_dir, store) = open_temp();
        let tables: Vec<String> = store
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_fts'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(tables.len(), 2);
        assert!(tables.contains(&"chapters_fts".to_string()));
        assert!(tables.contains(&"glossary_fts".to_string()));
    }

    #[test]
    fn fts5_trigram_matches_cjk_substring() {
        let (_dir, store) = open_temp();
        store
            .conn
            .execute(
                "INSERT INTO chapters(id, number, title, source_text, created_at, updated_at)
                 VALUES ('c1', 1, '', '第一章：剑客的觉醒', 'now', 'now')",
                [],
            )
            .unwrap();
        let matches: i64 = store
            .conn
            .query_row(
                "SELECT count(*) FROM chapters_fts WHERE chapters_fts MATCH ?1",
                params!["剑客的"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(matches, 1);
    }

    #[test]
    fn replace_updates_translation_and_scopes_to_chapter() {
        let (_dir, store) = open_temp();
        let c1 = store.insert_chapter(1, "C1", "第一章", "t").unwrap();
        let c2 = store.insert_chapter(2, "C2", "第二章", "t").unwrap();
        store
            .update_chapter(&c1, None, None, Some("كان البطلُ يغسل السيف الجديد"), Some("translated"), "t1")
            .unwrap();
        store
            .update_chapter(&c2, None, None, Some("صعد الجبل"), Some("translated"), "t2")
            .unwrap();

        let changed = store.replace_in_translations("السيف", "الخنجَر", None, "t3").unwrap();
        assert_eq!(changed, 1);
        assert!(store.get_chapter(&c1).unwrap().unwrap().translation.contains("الخنجَر"));
        assert!(!store.get_chapter(&c1).unwrap().unwrap().translation.contains("السيف"));

        let scoped = store.replace_in_translations("الجبل", "الوادي", Some(&c2), "t4").unwrap();
        assert_eq!(scoped, 1);
        assert!(store.get_chapter(&c2).unwrap().unwrap().translation.contains("الوادي"));
        let scoped_miss = store.replace_in_translations("الجبل", "الوادي", Some(&c1), "t5").unwrap();
        assert_eq!(scoped_miss, 0);
    }

    #[test]
    fn word_count_is_adaptive_by_script() {
        let (_dir, store) = open_temp();
        let c_zh = store.insert_chapter(1, "C1", "他睁开了眼睛", "t").unwrap();
        let c_en = store.insert_chapter(2, "C2", "He opened his eyes and saw the mountain.", "t").unwrap();
        assert_eq!(store.get_chapter(&c_zh).unwrap().unwrap().word_count, 6);
        assert_eq!(store.get_chapter(&c_en).unwrap().unwrap().word_count, 8);
    }

    #[test]
    fn chapter_summaries_upsert_and_list_before() {
        let (_dir, store) = open_temp();
        store
            .upsert_chapter_summary("c1", 1, "الفصل الأول: بطل ينفض غبار النوم.", "gemini", "t1")
            .unwrap();
        store
            .upsert_chapter_summary("c2", 2, "الفصل الثاني: مواجهة السياف القديم.", "gemini", "t2")
            .unwrap();
        store
            .upsert_chapter_summary("c3", 3, "الفصل الثالث: هرب البطل من المدينة.", "gemini", "t3")
            .unwrap();

        let got = store.get_chapter_summary("c2").unwrap().unwrap();
        assert_eq!(got.chapter_number, 2);
        assert_eq!(got.model, "gemini");

        let before = store.list_summaries_before(4, 2).unwrap();
        assert_eq!(before.len(), 2);
        assert_eq!(before[0].chapter_number, 2);
        assert_eq!(before[1].chapter_number, 3);

        store
            .upsert_chapter_summary("c3", 3, "الفصل الثالث منقح.", "gemini", "t4")
            .unwrap();
        let before2 = store.list_summaries_before(4, 10).unwrap();
        assert_eq!(before2.len(), 3);
        assert_eq!(before2[2].summary, "الفصل الثالث منقح.");
    }
}
