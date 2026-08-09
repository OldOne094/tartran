import type {
  AppSettings,
  CreateProjectInput,
  ProjectSummary,
  UpdateProjectInput
} from './types'

export const IPC = {
  projectsList: 'projects:list',
  projectsCreate: 'projects:create',
  projectsGet: 'projects:get',
  projectsDelete: 'projects:delete',
  projectsUpdate: 'projects:update',
  settingsGet: 'settings:get',
  settingsUpdate: 'settings:update',
  settingsApiKeyStatus: 'settings:apiKeyStatus',
  settingsApiKeySet: 'settings:apiKeySet',
  settingsApiKeyClear: 'settings:apiKeyClear',
  appInfo: 'app:info'
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
}
