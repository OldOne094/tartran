import { app, ipcMain } from 'electron'
import { IPC } from '../../shared/ipc'
import { ok } from '../../shared/result'
import type { Logger } from '../logger'

export function registerApp(deps: { logger: Logger }): void {
  ipcMain.handle(IPC.appInfo, async () => {
    deps.logger.debug('ipc:app:info')
    return ok({ version: app.getVersion(), userDataPath: app.getPath('userData') })
  })
}
