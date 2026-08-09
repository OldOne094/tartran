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
