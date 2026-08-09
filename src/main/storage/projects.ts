import { randomUUID } from 'node:crypto'
import { mkdirSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import type { CreateProjectInput, ProjectSummary, TargetLang, UpdateProjectInput } from '../../shared/types'
import type { AppSettingsStore, RegistryEntry } from './appSettings'
import { openProjectStore } from './projectStore'
import type { Logger } from '../logger'

const FOLDER_SUFFIX_LEN = 8

function nowIso(): string {
  return new Date().toISOString()
}

function asTargetLang(v: unknown): TargetLang {
  return v === 'en' ? 'en' : 'ar'
}

export class ProjectsManager {
  constructor(
    private readonly settings: AppSettingsStore,
    private readonly logger: Logger
  ) {}

  create(input: CreateProjectInput): ProjectSummary {
    const id = randomUUID()
    const now = nowIso()
    const folder = this.folderFor(id, input.title)
    mkdirSync(folder, { recursive: true })

    const store = openProjectStore(folder)
    try {
      store.setMetaMany({
        id,
        title: input.title,
        author: input.author ?? '',
        sourceLang: input.sourceLang ?? 'zh',
        targetLang: input.targetLang,
        createdAt: now,
        updatedAt: now
      })
    } finally {
      store.close()
    }

    this.settings.addRegistry({ id, path: folder, title: input.title, createdAt: now })
    this.logger.info('project:create', { id, title: input.title })
    return this.readSummary(id)
  }

  list(): ProjectSummary[] {
    return this.settings
      .registry()
      .map((entry) => {
        try {
          return this.readSummary(entry.id)
        } catch (e) {
          this.logger.warn('project:unreadable', { id: entry.id, err: String(e) })
          const corrupted: ProjectSummary = {
            id: entry.id,
            title: entry.title,
            author: '',
            sourceLang: 'zh',
            targetLang: 'ar',
            createdAt: entry.createdAt,
            updatedAt: entry.createdAt,
            chapterCount: 0,
            translatedCount: 0,
            reviewedCount: 0,
            corrupted: true
          }
          return corrupted
        }
      })
      .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
  }

  get(id: string): ProjectSummary {
    return this.readSummary(id)
  }

  update(id: string, patch: UpdateProjectInput): ProjectSummary {
    const entry = this.entry(id)
    const store = openProjectStore(entry.path)
    try {
      const meta = store.getMeta()
      store.setMetaMany({
        ...meta,
        ...(patch.title !== undefined ? { title: patch.title } : {}),
        ...(patch.author !== undefined ? { author: patch.author } : {}),
        ...(patch.targetLang !== undefined ? { targetLang: patch.targetLang } : {}),
        updatedAt: nowIso()
      })
    } finally {
      store.close()
    }

    if (patch.title !== undefined && patch.title !== entry.title) {
      this.settings.removeRegistry(id)
      this.settings.addRegistry({ ...entry, title: patch.title })
    }
    return this.readSummary(id)
  }

  delete(id: string): void {
    const entry = this.entry(id)
    rmSync(entry.path, { recursive: true, force: true })
    this.settings.removeRegistry(id)
    this.logger.info('project:delete', { id })
  }

  private folderFor(id: string, title: string): string {
    const safe = title.replace(/[^\p{L}\p{N} _-]/gu, '').slice(0, 40) || 'novel'
    return join(this.settings.workspacePath(), `${safe}-${id.slice(0, FOLDER_SUFFIX_LEN)}`)
  }

  private entry(id: string): RegistryEntry {
    const entry = this.settings.registry().find((e) => e.id === id)
    if (!entry) throw new Error(`Project not found: ${id}`)
    return entry
  }

  private readSummary(id: string): ProjectSummary {
    const entry = this.entry(id)
    const store = openProjectStore(entry.path)
    try {
      const meta = store.getMeta()
      const counts = store.statusCounts()
      const createdAt = meta.createdAt ?? entry.createdAt
      return {
        id,
        title: meta.title ?? entry.title,
        author: meta.author ?? '',
        sourceLang: 'zh',
        targetLang: asTargetLang(meta.targetLang),
        createdAt,
        updatedAt: meta.updatedAt ?? createdAt,
        chapterCount: store.chapterCount(),
        translatedCount: counts.translated,
        reviewedCount: counts.reviewed
      }
    } finally {
      store.close()
    }
  }
}

export function createProjectsManager(
  settings: AppSettingsStore,
  logger: Logger
): ProjectsManager {
  return new ProjectsManager(settings, logger)
}
