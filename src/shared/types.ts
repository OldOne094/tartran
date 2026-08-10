export type UiLanguage = 'en' | 'ar'
export type Theme = 'system' | 'light' | 'dark'
export type SourceLang = 'zh'
export type TargetLang = 'ar' | 'en'

export interface AppSettings {
  workspacePath: string
  uiLanguage: UiLanguage
  theme: Theme
}

export interface ProjectMeta {
  id: string
  title: string
  author: string
  sourceLang: SourceLang
  targetLang: TargetLang
  createdAt: string
  updatedAt: string
}

export interface ProjectSummary extends ProjectMeta {
  chapterCount: number
  translatedCount: number
  reviewedCount: number
  corrupted?: boolean
}

export interface CreateProjectInput {
  title: string
  author?: string
  targetLang: TargetLang
  sourceLang?: SourceLang
}

export interface UpdateProjectInput {
  title?: string
  author?: string
  targetLang?: TargetLang
}

export type ChapterStatus = 'imported' | 'translating' | 'translated' | 'reviewed' | 'exported'

export interface ChapterSummary {
  id: string
  number: number
  title: string
  wordCount: number
  status: ChapterStatus
  createdAt: string
  updatedAt: string
  translatedAt?: string
}

export interface ChapterDetail {
  id: string
  number: number
  title: string
  sourceText: string
  translation: string
  wordCount: number
  status: ChapterStatus
  createdAt: string
  updatedAt: string
  translatedAt?: string
}

export interface CreateChapterInput {
  number?: number
  title: string
  sourceText: string
}

export interface UpdateChapterInput {
  title?: string
  sourceText?: string
  translation?: string
  status?: ChapterStatus
}

export interface ImportChaptersInput {
  text: string
  splitBy?: 'auto' | 'marker' | 'paragraphs'
}

export interface ImportChaptersResult {
  imported: number
  skipped: number
  chapters: ChapterSummary[]
}

export interface ChapterSearchResult {
  id: string
  number: number
  title: string
  snippet: string
}

export interface GlossaryEntry {
  id: string
  zh: string
  en: string
  ar: string
  category: string
  notes: string
  aliases: string[]
  locked: boolean
  source: string
  createdAt: string
  updatedAt: string
}

export interface CreateGlossaryInput {
  zh: string
  en?: string
  ar?: string
  category?: string
  notes?: string
  aliases?: string[]
}

export interface UpdateGlossaryInput {
  zh?: string
  en?: string
  ar?: string
  category?: string
  notes?: string
  aliases?: string[]
  locked?: boolean
}

export interface GlossarySearchResult {
  id: string
  zh: string
  en: string
  ar: string
  category: string
  snippet: string
}

export type SuggestionStatus = 'pending' | 'approved' | 'rejected'

export interface Suggestion {
  id: string
  chapterId: string
  zh: string
  en: string
  ar: string
  category: string
  notes: string
  context: string
  status: SuggestionStatus
  createdAt: string
}

export interface CreateSuggestionInput {
  chapterId: string
  zh: string
  en?: string
  ar?: string
  category?: string
  notes?: string
  context?: string
}

export interface UpdateSuggestionInput {
  status?: SuggestionStatus
  en?: string
  ar?: string
  category?: string
  notes?: string
}

export interface TranslateChapterInput {
  chapterId: string
  model?: string
}

export interface TranslateResult {
  chapterId: string
  translation: string
  suggestions: Suggestion[]
  model: string
  durationMs: number
  tokensUsed: number
  chunkCount: number
}

export interface TranslationProgress {
  chapterId: string
  current: number
  total: number
  percent: number
}

export interface ModelInfo {
  id: string
  label: string
  description: string
}

export interface ExportFile {
  name: string
  mime: string
  dataBase64: string
}
