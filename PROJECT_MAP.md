# PROJECT_MAP.md

> صيغة العمليات الصادرة عبر مبدأ **Think Before Coding**. كل تغيير معماري جوهري يجب أن يُسجّل هنا.
> Last architecture review: **2026-08-09** — Milestone 1 (Foundation).

## [TECH_STACK]

Versions verified 2026-08-09 against npm registry + official docs (protocol: time-awareness).

| Layer | Choice | Version | Why |
|---|---|---|---|
| Shell | Electron | 43.3.0 (latest stable) | Full Node in main; local-first; mature |
| Build | electron-vite | 5.0.0 | Standardized Electron+Vite build (main/preload/renderer) |
| | Vite | 8.2.1 | |
| Language | TypeScript | 7.0.2 (latest stable, native compiler) | Fallback to 5.9.x if any tool friction (see PENDING) |
| UI | React + react-dom | 19.2.8 | |
| | Tailwind CSS | 4.3.3 (v4) | CSS-first config, logical props for RTL |
| | lucide-react | 1.31.0 | Icons |
| State | @tanstack/react-query | 5.101.4 | Data over IPC: caching, mutations, invalidation |
| Storage | better-sqlite3 | 13.0.3 | N-API prebuilds ship Electron 43 binaries (no rebuild); sync API; FTS5 trigram for CJK |
| LLM | @google/genai | 2.16.0 | Official GA SDK. Legacy `@google/generative-ai` deprecated (EOL 2025-11-30). |
| Export | docx | 9.7.1 | Pure JS DOCX |
| | exceljs | 4.4.0 | Pure JS XLSX |
| Validation | zod | 4.4.3 | IPC payloads + settings + model config |
| Testing | vitest | 4.1.10 | unit/integration |
| | @playwright/test | 1.62.1 | Electron E2E |
| Packaging | electron-builder | 26.15.3 | NSIS + portable (M6) |
| IDs/Dates | crypto.randomUUID / ISO-8601 | built-in | zero deps |

Rejected alternatives (verified by experiment):
- `node:sqlite` — still **experimental** in Node 24 (warning emitted); not for core storage.
- `@sqlite.org/sqlite-wasm` — FTS5 trigram works, but **file persistence failed on Windows/Node test** (SQLITE_CANTOPEN on reopen).

## [SYSTEM_FLOW]

```
Renderer (React) ──typed IPC──▶ Main Process ──▶ SQLite (per project)
     │                             │
     │                             └── LLM Pipeline (Gemini) ──▶ @google/genai
     └── Export: DOCX / XLSX / copy (clean text)
```

## [ARCHITECTURE]

- **Electron main process owns everything** (storage, LLM, export, secrets). Renderer has zero Node access.
- `contextIsolation: true`, `sandbox: true`, `nodeIntegration: false`, CSP set.
- Feature-oriented module layout (no random `utils/helpers/services` folders).
- IPC returns a result envelope `{ ok, data } | { ok: false, error }` — every handler validates input with zod.

## [PROJECT_STRUCTURE]

```
tartran/
├─ electron.vite.config.ts  electron-builder.yml  vitest.config.ts  playwright.config.ts
├─ src/
│  ├─ main/
│  │  ├─ index.ts               # bootstrap, window, security
│  │  ├─ logger.ts              # async file logs, redaction
│  │  ├─ safeStorageCipher.ts   # Electron safeStorage → KeyCipher
│  │  ├─ ipc/                   # projects / settings / app handlers
│  │  ├─ storage/               # appSettings, apiKeyStore, projectStore, projects
│  │  ├─ llm/                   # (M4) provider + gemini + registry
│  │  ├─ pipeline/              # (M4) translator, promptBuilder, termDetector, validation
│  │  └─ export/                # (M5) docx, xlsx
│  ├─ preload/index.ts          # typed contextBridge API
│  ├─ renderer/                 # React app (features + i18n + lib + components)
│  └─ shared/                   # types, IPC contract, result helpers, chapter statuses
└─ tests/                       # unit/ + e2e/
```

## [DATA_MODEL]

Per-project SQLite file: `<workspace>/Projects/<safe-title>-<id8>/novel.db`
- `meta(key PK, value)` — id, title, author, sourceLang, targetLang, createdAt, updatedAt.
- `chapters(id, number, title, source_text, translation, status, word_count, created_at, updated_at, translated_at)` + FTS5 `chapters_fts` (trigram).
- `glossary(id, zh, en, ar, category, notes, aliases JSON, locked, source, created_at, updated_at)` + FTS5 `glossary_fts`.
- `suggestions(id, chapter_id, zh, en, ar, category, notes, context, status, created_at)` — pending human review.

Chapter statuses: `imported → translating → translated → reviewed → exported`.

Global (app `userData`): `settings.json`, `projects.json` (registry), `api-keys.json` (encrypted), `logs/`.

## [LLM_ARCHITECTURE]

- `LLMProvider` interface (`translate`, `listModels`); only **GeminiProvider** implemented now.
- Gemini facts (official docs, 2026-08):
  - Default model: `gemini-3.6-flash` (free-tier input/output; 1M context; 65K max output).
  - Budget model: `gemini-3.1-flash-lite` (30 RPM class).
  - `gemini-2.5-flash` / `gemini-2.5-flash-lite`: **deprecated, shutdown 2026-10-16** → NOT offered.
  - Pro models have **no free tier** since 2026-04-01.
  - Google no longer publishes fixed free-quota numbers; limits vary per account (AI Studio). System treats live 429s as the source of truth and keeps configurable per-model RPM/RPD defaults.

## [TRANSLATION_PIPELINE]

```
source → termDetector (longest-match zh + aliases, en/ar on translation)
       → promptBuilder (system + project instructions + relevant glossary + optional context + source)
       → rateLimiter (token bucket) → GeminiProvider
       → validation (zod; JSON fallback = plain-text retry)
       → suggestions (pending) → user approval → glossary
```
- Only terms actually present in the chapter are sent (capped), never the whole glossary.
- Oversized chapters chunk at paragraph level (deterministic).

## [GLOSSARY_SYSTEM]

- Fields: `zh | en | ar | category | notes` + `aliases`, `locked`, `source`.
- Detection is string matching (not FTS). FTS is for user search.
- No term is committed to the glossary automatically — always via user review of `suggestions`.

## [EXPORT_SYSTEM]

- DOCX: `docx` lib; title + paragraphs; RTL for Arabic (verify exact API in M5).
- XLSX: `exceljs`; columns `Chinese | English | Arabic | Category | Notes`.
- Clean copy (no metadata/debug).

## [INTEGRATIONS]

- Google Docs/Sheets: **deferred**. Interface-only design; no fake provider. Local DOCX/XLSX is the complete v1 path.

## [SECURITY]

- API keys encrypted via Electron `safeStorage` (DPAPI on Windows); never sent to renderer, never logged.
- zod on every IPC input; path guards to be added with M2 file operations.
- Logger redacts key-like patterns and truncates long strings (chapters logged as lengths).

## [TESTING]

- `npm test` — vitest unit/integration: projects persistence/CRUD, settings, api key store (M1). Storage, pipeline, rate limiter, export round-trips to follow.
- `npm run test:e2e` — Playwright + Electron: project lifecycle (create → relaunch → delete).

## [ORPHANS & PENDING]

- Google Docs/Sheets integration: designed, not implemented (deferred milestone).
- CodeMirror editor upgrade: deferred until measured perf need.
- Security path guards (`security.ts`): pending M2 file operations.
- API-key UI in Settings: pending M4 (storage layer + IPC exist since M1).
- Theme dark variants: partial polish, complete in M6.
- TypeScript 7.0.2: if any tooling friction in later milestones, pin TS 5.9.x.
- Full Arabic i18n coverage across all UI strings: maintained per milestone; audit in M6.
- Known limitation: free-tier Gemini quotas vary per account; the app must surface 429 messages clearly (M4).

### Milestone progress

- [x] **M1 — Project Foundation**: app boots; project CRUD persists across restarts; secure key storage scaffold; bilingual i18n scaffold.
- [ ] M2 — Chapters (import/split/search/editor/statuses)
- [ ] M3 — Glossary (CRUD/search/detection/prompt inclusion)
- [ ] M4 — Gemini integration + translation pipeline + suggestions review
- [ ] M5 — Export (DOCX/XLSX/copy)
- [ ] M6 — Polish, Arabic/RTL completion, packaging
