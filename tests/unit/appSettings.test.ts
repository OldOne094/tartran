import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { AppSettingsStore } from '../../src/main/storage/appSettings'

let base: string

beforeEach(() => {
  base = mkdtempSync(join(tmpdir(), 'tartran-settings-'))
})

afterEach(() => {
  rmSync(base, { recursive: true, force: true })
})

describe('AppSettingsStore', () => {
  it('loads defaults on a fresh directory', () => {
    const store = new AppSettingsStore(base)
    expect(store.get()).toEqual({ workspacePath: '', uiLanguage: 'en', theme: 'system' })
  })

  it('updates and persists settings', () => {
    const store = new AppSettingsStore(base)
    store.update({ uiLanguage: 'ar', theme: 'dark' })
    const reopened = new AppSettingsStore(base)
    expect(reopened.get().uiLanguage).toBe('ar')
    expect(reopened.get().theme).toBe('dark')
  })

  it('ignores invalid persisted values and falls back to defaults', () => {
    const store = new AppSettingsStore(base)
    store.update({ uiLanguage: 'fr' as never })
    const reopened = new AppSettingsStore(base)
    expect(reopened.get().uiLanguage).toBe('en')
  })

  it('creates the workspace folder on demand', () => {
    const store = new AppSettingsStore(base)
    const ws = store.workspacePath()
    expect(existsSync(ws)).toBe(true)
  })

  it('adds and removes registry entries', () => {
    const store = new AppSettingsStore(base)
    const entry = { id: 'p1', path: join(base, 'p1'), title: 'Novel', createdAt: 'now' }
    store.addRegistry(entry)
    expect(store.registry()).toHaveLength(1)
    store.addRegistry({ ...entry, id: 'p1' })
    expect(store.registry()).toHaveLength(1)
    store.removeRegistry('p1')
    expect(store.registry()).toHaveLength(0)
  })
})
