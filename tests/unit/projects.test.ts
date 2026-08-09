import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { AppSettingsStore } from '../../src/main/storage/appSettings'
import { ProjectsManager } from '../../src/main/storage/projects'
import { nullLogger } from '../../src/main/logger'

let base: string
let settings: AppSettingsStore
let projects: ProjectsManager

beforeEach(() => {
  base = mkdtempSync(join(tmpdir(), 'tartran-projects-'))
  settings = new AppSettingsStore(base)
  projects = new ProjectsManager(settings, nullLogger)
})

afterEach(() => {
  rmSync(base, { recursive: true, force: true })
})

describe('ProjectsManager', () => {
  it('creates a project with metadata and zero chapters', () => {
    const p = projects.create({ title: 'Novel A', targetLang: 'ar' })
    expect(p.title).toBe('Novel A')
    expect(p.targetLang).toBe('ar')
    expect(p.sourceLang).toBe('zh')
    expect(p.chapterCount).toBe(0)
    expect(settings.registry()).toHaveLength(1)
    expect(existsSync(join(settings.workspacePath(), `Novel A-${p.id.slice(0, 8)}`))).toBe(true)
  })

  it('persists across a fresh manager instance (restart simulation)', () => {
    projects.create({ title: 'Novel A', targetLang: 'en' })
    const reopened = new ProjectsManager(new AppSettingsStore(base), nullLogger)
    const list = reopened.list()
    expect(list).toHaveLength(1)
    expect(list[0].title).toBe('Novel A')
    expect(list[0].targetLang).toBe('en')
  })

  it('updates metadata and persists it', () => {
    const p = projects.create({ title: 'Novel A', targetLang: 'ar' })
    const updated = projects.update(p.id, { author: 'Some Author', targetLang: 'en' })
    expect(updated.author).toBe('Some Author')
    expect(updated.targetLang).toBe('en')

    const reread = new ProjectsManager(new AppSettingsStore(base), nullLogger).get(p.id)
    expect(reread.author).toBe('Some Author')
    expect(reread.targetLang).toBe('en')
  })

  it('deletes the project folder and registry entry', () => {
    const p = projects.create({ title: 'Novel A', targetLang: 'ar' })
    const folder = join(settings.workspacePath(), `Novel A-${p.id.slice(0, 8)}`)
    expect(existsSync(folder)).toBe(true)

    projects.delete(p.id)
    expect(settings.registry()).toHaveLength(0)
    expect(existsSync(folder)).toBe(false)
  })

  it('keeps the same folder when the title is renamed', () => {
    const p = projects.create({ title: 'Novel A', targetLang: 'ar' })
    const folder = join(settings.workspacePath(), `Novel A-${p.id.slice(0, 8)}`)

    const updated = projects.update(p.id, { title: 'Novel B' })
    expect(updated.title).toBe('Novel B')
    expect(settings.registry()[0].title).toBe('Novel B')
    expect(existsSync(folder)).toBe(true)
  })
})
