import { mkdirSync, readFileSync, writeFileSync, renameSync } from 'node:fs'
import { join } from 'node:path'

export type UiLanguage = 'en' | 'ar'
export type Theme = 'system' | 'light' | 'dark'

export interface AppSettingsData {
  workspacePath: string
  uiLanguage: UiLanguage
  theme: Theme
}

export interface RegistryEntry {
  id: string
  path: string
  title: string
  createdAt: string
}

const DEFAULTS: AppSettingsData = {
  workspacePath: '',
  uiLanguage: 'en',
  theme: 'system'
}

function readJsonSafe<T>(file: string, fallback: T): T {
  try {
    return JSON.parse(readFileSync(file, 'utf-8')) as T
  } catch {
    return fallback
  }
}

function writeJsonAtomic(file: string, value: unknown): void {
  const tmp = `${file}.tmp`
  writeFileSync(tmp, JSON.stringify(value, null, 2), 'utf-8')
  renameSync(tmp, file)
}

export class AppSettingsStore {
  private readonly settingsPath: string
  private readonly registryPath: string

  constructor(private readonly baseDir: string) {
    mkdirSync(baseDir, { recursive: true })
    this.settingsPath = join(baseDir, 'settings.json')
    this.registryPath = join(baseDir, 'projects.json')
  }

  get(): AppSettingsData {
    const raw = readJsonSafe<Partial<AppSettingsData>>(this.settingsPath, {})
    return {
      workspacePath: typeof raw.workspacePath === 'string' ? raw.workspacePath : DEFAULTS.workspacePath,
      uiLanguage: raw.uiLanguage === 'ar' ? 'ar' : 'en',
      theme: raw.theme === 'dark' || raw.theme === 'light' ? raw.theme : 'system'
    }
  }

  update(patch: Partial<AppSettingsData>): AppSettingsData {
    const next = { ...this.get(), ...patch }
    writeJsonAtomic(this.settingsPath, next)
    return next
  }

  workspacePath(): string {
    const ws = this.get().workspacePath || join(this.baseDir, 'Projects')
    mkdirSync(ws, { recursive: true })
    return ws
  }

  registry(): RegistryEntry[] {
    return readJsonSafe<RegistryEntry[]>(this.registryPath, [])
  }

  saveRegistry(entries: RegistryEntry[]): void {
    writeJsonAtomic(this.registryPath, entries)
  }

  addRegistry(entry: RegistryEntry): void {
    this.saveRegistry([...this.registry().filter((e) => e.id !== entry.id), entry])
  }

  removeRegistry(id: string): void {
    this.saveRegistry(this.registry().filter((e) => e.id !== id))
  }
}

export function createAppSettings(baseDir: string): AppSettingsStore {
  return new AppSettingsStore(baseDir)
}
