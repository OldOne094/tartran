import { ipcMain } from 'electron'
import { z } from 'zod'
import { IPC } from '../../shared/ipc'
import { err, ok } from '../../shared/result'
import type { AppSettingsStore } from '../storage/appSettings'
import type { ApiKeyStore } from '../storage/apiKeyStore'
import type { Logger } from '../logger'

const DEFAULT_KEY_ID = 'default'

const updateSettingsSchema = z
  .object({
    workspacePath: z.string().min(1).max(1000).optional(),
    uiLanguage: z.enum(['en', 'ar']).optional(),
    theme: z.enum(['system', 'light', 'dark']).optional()
  })
  .strict()

const apiKeySchema = z.string().min(8).max(500)

export function registerSettings(deps: {
  settings: AppSettingsStore
  apiKeys: ApiKeyStore
  logger: Logger
}): void {
  ipcMain.handle(IPC.settingsGet, async () => ok(deps.settings.get()))

  ipcMain.handle(IPC.settingsUpdate, async (_event, input: unknown) => {
    const parsed = updateSettingsSchema.safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid settings update')
    try {
      const next = deps.settings.update(parsed.data)
      if (parsed.data.workspacePath) {
        deps.settings.workspacePath()
      }
      return ok(next)
    } catch (e) {
      deps.logger.error('ipc:settings:update', { err: String(e) })
      return err('UPDATE_FAILED', 'Could not save settings')
    }
  })

  ipcMain.handle(IPC.settingsApiKeyStatus, async () => {
    try {
      return ok({ configured: deps.apiKeys.has(DEFAULT_KEY_ID) })
    } catch {
      return err('KEY_STORE_UNAVAILABLE', 'Secure key storage is not available')
    }
  })

  ipcMain.handle(IPC.settingsApiKeySet, async (_event, input: unknown) => {
    const parsed = apiKeySchema.safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid API key')
    try {
      deps.apiKeys.set(DEFAULT_KEY_ID, parsed.data)
      return ok({ configured: true })
    } catch {
      return err('KEY_STORE_UNAVAILABLE', 'Secure key storage is not available on this device')
    }
  })

  ipcMain.handle(IPC.settingsApiKeyClear, async () => {
    try {
      deps.apiKeys.delete(DEFAULT_KEY_ID)
      return ok({ configured: false })
    } catch {
      return err('KEY_STORE_UNAVAILABLE', 'Secure key storage is not available on this device')
    }
  })
}
