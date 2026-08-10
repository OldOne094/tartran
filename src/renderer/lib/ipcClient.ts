import { invoke } from '@tauri-apps/api/core'
import { IPC, type IpcResult, type RendererApi } from '../../shared/ipc'

async function call<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  const result = await invoke<IpcResult<T>>(command, payload)
  if (result && result.ok === false) {
    const error = new Error(result.error.message) as Error & { code?: string }
    error.code = result.error.code
    throw error
  }
  return result.data
}

export const api: RendererApi = {
  projects: {
    list: () => call(IPC.projectsList),
    create: (input) => call(IPC.projectsCreate, { input }),
    get: (projectId) => call(IPC.projectsGet, { projectId }),
    delete: (projectId) => call(IPC.projectsDelete, { projectId }),
    update: (projectId, patch) => call(IPC.projectsUpdate, { projectId, patch })
  },
  settings: {
    get: () => call(IPC.settingsGet),
    update: (patch) => call(IPC.settingsUpdate, { patch }),
    apiKeyStatus: () => call(IPC.settingsApiKeyStatus),
    setApiKey: (apiKey) => call(IPC.settingsApiKeySet, { apiKey }),
    clearApiKey: () => call(IPC.settingsApiKeyClear)
  },
  app: {
    info: () => call(IPC.appInfo)
  },
  chapters: {
    list: (projectId) => call(IPC.chaptersList, { projectId }),
    get: (projectId, chapterId) => call(IPC.chaptersGet, { projectId, chapterId }),
    create: (projectId, input) => call(IPC.chaptersCreate, { projectId, input }),
    update: (projectId, chapterId, patch) => call(IPC.chaptersUpdate, { projectId, chapterId, patch }),
    delete: (projectId, chapterId) => call(IPC.chaptersDelete, { projectId, chapterId }),
    search: (projectId, query) => call(IPC.chaptersSearch, { projectId, query }),
    import: (projectId, input) => call(IPC.chaptersImport, { projectId, input })
  },
  glossary: {
    list: (projectId) => call(IPC.glossaryList, { projectId }),
    create: (projectId, input) => call(IPC.glossaryCreate, { projectId, input }),
    update: (projectId, glossaryId, patch) =>
      call(IPC.glossaryUpdate, { projectId, glossaryId, patch }),
    delete: (projectId, glossaryId) => call(IPC.glossaryDelete, { projectId, glossaryId }),
    search: (projectId, query) => call(IPC.glossarySearch, { projectId, query }),
    replace: (projectId, oldValue, newValue, chapterId) =>
      call(IPC.glossaryReplace, { projectId, oldValue, newValue, chapterId })
  },
  suggestions: {
    list: (projectId, chapterId) => call(IPC.suggestionsList, { projectId, chapterId }),
    create: (projectId, input) => call(IPC.suggestionsCreate, { projectId, input }),
    update: (projectId, suggestionId, patch) =>
      call(IPC.suggestionsUpdate, { projectId, suggestionId, patch }),
    approve: (projectId, suggestionId) =>
      call(IPC.suggestionsApprove, { projectId, suggestionId }),
    reject: (projectId, suggestionId) => call(IPC.suggestionsReject, { projectId, suggestionId }),
    delete: (projectId, suggestionId) => call(IPC.suggestionsDelete, { projectId, suggestionId })
  },
  translation: {
    translateChapter: (projectId, input) =>
      call(IPC.translationTranslateChapter, { projectId, input }),
    models: () => call(IPC.translationModels)
  },
  export: {
    chapterText: (projectId, chapterId) => call(IPC.exportChapterText, { projectId, chapterId }),
    chapterDocx: (projectId, chapterId, targetLang) =>
      call(IPC.exportChapterDocx, { projectId, chapterId, targetLang }),
    glossaryXlsx: (projectId) => call(IPC.exportGlossaryXlsx, { projectId })
  }
}
