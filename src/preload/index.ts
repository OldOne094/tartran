import { contextBridge, ipcRenderer } from 'electron'
import { IPC, type IpcResult, type RendererApi } from '../shared/ipc'

async function invoke<T>(channel: string, payload?: unknown): Promise<T> {
  const result = (await ipcRenderer.invoke(channel, payload)) as IpcResult<T>
  if (result && result.ok === false) {
    const error = new Error(result.error.message) as Error & { code?: string }
    error.code = result.error.code
    throw error
  }
  return result.data
}

const api: RendererApi = {
  projects: {
    list: () => invoke(IPC.projectsList),
    create: (input) => invoke(IPC.projectsCreate, input),
    get: (projectId) => invoke(IPC.projectsGet, projectId),
    delete: (projectId) => invoke(IPC.projectsDelete, projectId),
    update: (projectId, patch) => invoke(IPC.projectsUpdate, { projectId, patch })
  },
  settings: {
    get: () => invoke(IPC.settingsGet),
    update: (patch) => invoke(IPC.settingsUpdate, patch),
    apiKeyStatus: () => invoke(IPC.settingsApiKeyStatus),
    setApiKey: (apiKey) => invoke(IPC.settingsApiKeySet, apiKey),
    clearApiKey: () => invoke(IPC.settingsApiKeyClear)
  },
  app: {
    info: () => invoke(IPC.appInfo)
  }
}

contextBridge.exposeInMainWorld('api', api)
