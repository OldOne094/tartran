import { app, BrowserWindow, shell } from 'electron'
import { join } from 'node:path'
import { createLogger } from './logger'
import { createAppSettings } from './storage/appSettings'
import { ApiKeyStore } from './storage/apiKeyStore'
import { createProjectsManager } from './storage/projects'
import { safeStorageCipher } from './safeStorageCipher'
import { registerIpc } from './ipc'

function resolveBaseDir(): string {
  const override = process.env.TARTRAN_USER_DATA
  return override && override.length > 0 ? override : app.getPath('userData')
}

function createWindow(): BrowserWindow {
  const win = new BrowserWindow({
    width: 1280,
    height: 820,
    minWidth: 960,
    minHeight: 600,
    show: false,
    title: 'TarTran',
    backgroundColor: '#f8fafc',
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      sandbox: true,
      nodeIntegration: false
    }
  })

  win.on('ready-to-show', () => win.show())

  win.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url)
    return { action: 'deny' }
  })

  if (process.env.ELECTRON_RENDERER_URL) {
    void win.loadURL(process.env.ELECTRON_RENDERER_URL)
  } else {
    void win.loadFile(join(__dirname, '../renderer/index.html'))
  }

  return win
}

void app.whenReady().then(() => {
  const baseDir = resolveBaseDir()
  const logger = createLogger(join(baseDir, 'logs'))
  const settings = createAppSettings(baseDir)
  const apiKeys = new ApiKeyStore(baseDir, safeStorageCipher)
  const projects = createProjectsManager(settings, logger)

  registerIpc({ logger, settings, apiKeys, projects })
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })

  logger.info('app:ready', { version: app.getVersion(), baseDir })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})
