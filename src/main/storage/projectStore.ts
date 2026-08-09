import Database from 'better-sqlite3'
import { join } from 'node:path'

const SCHEMA = `
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
  source_text, translation, tokenize='trigram'
);

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
  zh, en, ar, tokenize='trigram'
);

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

PRAGMA user_version = 1;
`

export interface ChapterStatusCounts {
  translated: number
  reviewed: number
}

export class ProjectStore {
  private readonly db: Database.Database

  private constructor(path: string) {
    this.db = new Database(path)
    this.db.pragma('journal_mode = WAL')
    this.db.exec(SCHEMA)
  }

  static open(path: string): ProjectStore {
    return new ProjectStore(path)
  }

  getMeta(): Record<string, string> {
    const rows = this.db.prepare('SELECT key, value FROM meta').all() as Array<{
      key: string
      value: string
    }>
    return Object.fromEntries(rows.map((r) => [r.key, r.value]))
  }

  setMetaMany(entries: Record<string, string>): void {
    const stmt = this.db.prepare(
      'INSERT INTO meta(key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value'
    )
    const tx = this.db.transaction((rows: Array<[string, string]>) => {
      for (const [k, v] of rows) stmt.run(k, v)
    })
    tx(Object.entries(entries))
  }

  chapterCount(): number {
    return (this.db.prepare('SELECT count(*) AS c FROM chapters').get() as { c: number }).c
  }

  statusCounts(): ChapterStatusCounts {
    const row = this.db
      .prepare(
        `SELECT
           sum(CASE WHEN status IN ('translated','reviewed','exported') THEN 1 ELSE 0 END) AS translated,
           sum(CASE WHEN status IN ('reviewed','exported') THEN 1 ELSE 0 END) AS reviewed
         FROM chapters`
      )
      .get() as { translated: number | null; reviewed: number | null }
    return { translated: row.translated ?? 0, reviewed: row.reviewed ?? 0 }
  }

  close(): void {
    this.db.close()
  }
}

export function openProjectStore(projectDir: string): ProjectStore {
  return ProjectStore.open(join(projectDir, 'novel.db'))
}
