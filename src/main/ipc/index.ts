import type { Logger } from '../logger'
import type { AppSettingsStore } from '../storage/appSettings'
import type { ApiKeyStore } from '../storage/apiKeyStore'
import type { ProjectsManager } from '../storage/projects'
import { registerProjects } from './projects.ipc'
import { registerSettings } from './settings.ipc'
import { registerApp } from './app.ipc'

export interface IpcDeps {
  logger: Logger
  settings: AppSettingsStore
  apiKeys: ApiKeyStore
  projects: ProjectsManager
}

export function registerIpc(deps: IpcDeps): void {
  registerProjects({ projects: deps.projects, logger: deps.logger })
  registerSettings({ settings: deps.settings, apiKeys: deps.apiKeys, logger: deps.logger })
  registerApp({ logger: deps.logger })
}
