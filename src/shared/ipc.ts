import type {
  AppSettings,
  ChapterDetail,
  ChapterSearchResult,
  ChapterSummary,
  CreateChapterInput,
  CreateGlossaryInput,
  CreateProjectInput,
  CreateSuggestionInput,
  ExportFile,
  GlossaryEntry,
  GlossaryReplaceResult,
  GlossarySearchResult,
  ImportChaptersInput,
  ImportChaptersResult,
  ModelInfo,
  ProjectSummary,
  Suggestion,
  TranslateChapterInput,
  TranslateResult,
  UpdateChapterInput,
  UpdateGlossaryInput,
  UpdateProjectInput,
  UpdateSuggestionInput
} from './types'

export const IPC = {
  projectsList: 'projects_list',
  projectsCreate: 'projects_create',
  projectsGet: 'projects_get',
  projectsDelete: 'projects_delete',
  projectsUpdate: 'projects_update',
  settingsGet: 'settings_get',
  settingsUpdate: 'settings_update',
  settingsApiKeyStatus: 'settings_api_key_status',
  settingsApiKeySet: 'settings_api_key_set',
  settingsApiKeyClear: 'settings_api_key_clear',
  appInfo: 'app_info',
  chaptersList: 'chapters_list',
  chaptersGet: 'chapters_get',
  chaptersCreate: 'chapters_create',
  chaptersUpdate: 'chapters_update',
  chaptersDelete: 'chapters_delete',
  chaptersSearch: 'chapters_search',
  chaptersImport: 'chapters_import',
  glossaryList: 'glossary_list',
  glossaryCreate: 'glossary_create',
  glossaryUpdate: 'glossary_update',
  glossaryDelete: 'glossary_delete',
  glossarySearch: 'glossary_search',
  glossaryReplace: 'glossary_replace',
  suggestionsList: 'suggestions_list',
  suggestionsCreate: 'suggestions_create',
  suggestionsUpdate: 'suggestions_update',
  suggestionsApprove: 'suggestions_approve',
  suggestionsReject: 'suggestions_reject',
  suggestionsDelete: 'suggestions_delete',
  translationTranslateChapter: 'translation_translate_chapter',
  translationModels: 'translation_models',
  exportChapterText: 'export_chapter_text',
  exportChapterDocx: 'export_chapter_docx',
  exportGlossaryXlsx: 'export_glossary_xlsx'
} as const

export type IpcResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: { code: string; message: string } }

export interface RendererApi {
  projects: {
    list(): Promise<ProjectSummary[]>
    create(input: CreateProjectInput): Promise<ProjectSummary>
    get(projectId: string): Promise<ProjectSummary>
    delete(projectId: string): Promise<void>
    update(projectId: string, patch: UpdateProjectInput): Promise<ProjectSummary>
  }
  settings: {
    get(): Promise<AppSettings>
    update(patch: Partial<AppSettings>): Promise<AppSettings>
    apiKeyStatus(): Promise<{ configured: boolean }>
    setApiKey(apiKey: string): Promise<{ configured: boolean }>
    clearApiKey(): Promise<{ configured: boolean }>
  }
  app: {
    info(): Promise<{ version: string; userDataPath: string }>
  }
  chapters: {
    list(projectId: string): Promise<ChapterSummary[]>
    get(projectId: string, chapterId: string): Promise<ChapterDetail>
    create(projectId: string, input: CreateChapterInput): Promise<ChapterSummary>
    update(projectId: string, chapterId: string, patch: UpdateChapterInput): Promise<ChapterSummary>
    delete(projectId: string, chapterId: string): Promise<void>
    search(projectId: string, query: string): Promise<ChapterSearchResult[]>
    import(projectId: string, input: ImportChaptersInput): Promise<ImportChaptersResult>
  }
  glossary: {
    list(projectId: string): Promise<GlossaryEntry[]>
    create(projectId: string, input: CreateGlossaryInput): Promise<GlossaryEntry>
    update(projectId: string, glossaryId: string, patch: UpdateGlossaryInput): Promise<GlossaryEntry>
    delete(projectId: string, glossaryId: string): Promise<void>
    search(projectId: string, query: string): Promise<GlossarySearchResult[]>
    replace(
      projectId: string,
      oldValue: string,
      newValue: string,
      chapterId?: string
    ): Promise<GlossaryReplaceResult>
  }
  suggestions: {
    list(projectId: string, chapterId: string): Promise<Suggestion[]>
    create(projectId: string, input: CreateSuggestionInput): Promise<Suggestion>
    update(projectId: string, suggestionId: string, patch: UpdateSuggestionInput): Promise<Suggestion>
    approve(projectId: string, suggestionId: string): Promise<GlossaryEntry>
    reject(projectId: string, suggestionId: string): Promise<Suggestion>
    delete(projectId: string, suggestionId: string): Promise<void>
  }
  translation: {
    translateChapter(projectId: string, input: TranslateChapterInput): Promise<TranslateResult>
    models(): Promise<ModelInfo[]>
  }
  export: {
    chapterText(projectId: string, chapterId: string): Promise<ExportFile>
    chapterDocx(projectId: string, chapterId: string, targetLang: string): Promise<ExportFile>
    glossaryXlsx(projectId: string): Promise<ExportFile>
  }
}
