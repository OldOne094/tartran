import { useEffect, useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { open } from '@tauri-apps/plugin-dialog'
import { check } from '@tauri-apps/plugin-updater'
import { FolderOpen, Gauge, KeyRound, Languages, RefreshCw, Save } from 'lucide-react'
import type { Theme } from '../../../shared/types'
import { useI18n, useT } from '../../i18n/I18nProvider'
import { api } from '../../lib/ipcClient'
import { useSettings } from '../../lib/queries'
import { Button, Card, Field, Input, Select } from '../../components/ui'
function useSystemTheme(): 'light' | 'dark' {
  const [mode, setMode] = useState<'light' | 'dark'>(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  )
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = (e: MediaQueryListEvent): void => setMode(e.matches ? 'dark' : 'light')
    mq.addEventListener('change', onChange)
    return () => mq.removeEventListener('change', onChange)
  }, [])
  return mode
}

export function SettingsPage(): ReactNode {
  const t = useT()
  const { locale, setLocale } = useI18n()
  const { data, isLoading } = useSettings()
  const queryClient = useQueryClient()

  const [workspace, setWorkspace] = useState('')
  const [theme, setTheme] = useState<Theme>('system')
  const [removeTashkeel, setRemoveTashkeel] = useState(false)
  const [temperature, setTemperature] = useState(0.7)
  const [apiKeyStatus, setApiKeyStatus] = useState<'loading' | 'configured' | 'none'>('loading')
  const [apiKeyInput, setApiKeyInput] = useState('')
  const [updateState, setUpdateState] = useState<
    | { kind: 'idle' }
    | { kind: 'checking' }
    | { kind: 'none' }
    | { kind: 'available'; version: string }
    | { kind: 'installing'; version: string }
    | { kind: 'installed' }
    | { kind: 'error' }
  >({ kind: 'idle' })
  const systemTheme = useSystemTheme()

  const apiKeyMutation = useMutation({
    mutationFn: (key: string) => api.settings.setApiKey(key),
    onSuccess: () => {
      setApiKeyInput('')
      setApiKeyStatus('configured')
      void queryClient.invalidateQueries({ queryKey: ['apiKey'] })
      window.setTimeout(() => apiKeyMutation.reset(), 2000)
    }
  })

  const clearApiKeyMutation = useMutation({
    mutationFn: () => api.settings.clearApiKey(),
    onSuccess: () => {
      setApiKeyStatus('none')
      void queryClient.invalidateQueries({ queryKey: ['apiKey'] })
    }
  })

  useEffect(() => {
    if (data) {
      setWorkspace(data.workspacePath)
      setTheme(data.theme)
      setRemoveTashkeel(data.removeTashkeel)
      setTemperature(data.temperature)
    }
  }, [data])

  useEffect(() => {
    api.settings
      .apiKeyStatus()
      .then((s) => setApiKeyStatus(s.configured ? 'configured' : 'none'))
      .catch(() => setApiKeyStatus('none'))
  }, [])

  useEffect(() => {
    const resolved = theme === 'system' ? systemTheme : theme
    document.documentElement.classList.toggle('dark', resolved === 'dark')
  }, [theme, systemTheme])

  const updateMutation = useMutation({
    mutationFn: (patch: {
      workspacePath?: string
      theme?: Theme
      removeTashkeel?: boolean
      temperature?: number
    }) => api.settings.update(patch),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['settings'] })
      window.setTimeout(() => updateMutation.reset(), 2000)
    }
  })

  const saveWorkspace = (): void => {
    updateMutation.mutate({ workspacePath: workspace.trim() || undefined })
  }

  const browseWorkspace = async (): Promise<void> => {
    const dir = await open({ directory: true, multiple: false })
    if (typeof dir === 'string' && dir.trim()) {
      setWorkspace(dir)
    }
  }

  const checkForUpdates = async (): Promise<void> => {
    setUpdateState({ kind: 'checking' })
    try {
      const update = await check()
      if (update) {
        setUpdateState({ kind: 'available', version: update.version })
      } else {
        setUpdateState({ kind: 'none' })
      }
    } catch {
      setUpdateState({ kind: 'error' })
    }
  }

  const installUpdate = async (version: string): Promise<void> => {
    setUpdateState({ kind: 'installing', version })
    try {
      const update = await check()
      if (update) {
        await update.downloadAndInstall()
        setUpdateState({ kind: 'installed' })
      } else {
        setUpdateState({ kind: 'none' })
      }
    } catch {
      setUpdateState({ kind: 'error' })
    }
  }

  return (
    <div className="mx-auto max-w-2xl px-6 py-8">
      <h1 className="mb-6 text-xl font-semibold text-slate-900 dark:text-slate-100">
        {t('settings.title')}
      </h1>

      {isLoading ? (
        <p className="text-sm text-slate-500 dark:text-slate-400">{t('common.loading')}</p>
      ) : (
        <div className="flex flex-col gap-5">
          <Card>
            <h2 className="mb-4 text-sm font-semibold text-slate-900 dark:text-slate-100">
              {t('settings.workspace')}
            </h2>
            <div className="flex flex-col gap-2">
              <Field label={t('settings.workspace')} hint={t('settings.workspaceHint')}>
                <div className="flex gap-2">
                  <Input
                    value={workspace}
                    onChange={(e) => setWorkspace(e.target.value)}
                    aria-label={t('settings.workspace')}
                  />
                  <Button
                    type="button"
                    variant="secondary"
                    onClick={() => void browseWorkspace()}
                    aria-label={t('settings.browse')}
                    title={t('settings.browse')}
                  >
                    <FolderOpen className="size-4" />
                    {t('settings.browse')}
                  </Button>
                </div>
              </Field>
              <div className="flex items-center gap-3">
                <Button onClick={saveWorkspace} disabled={updateMutation.isPending}>
                  <Save className="size-4" />
                  {t('settings.save')}
                </Button>
                {updateMutation.isSuccess ? (
                  <span className="text-xs text-emerald-600 dark:text-emerald-400">
                    {t('settings.saved')}
                  </span>
                ) : null}
              </div>
            </div>
          </Card>

          <Card>
            <h2 className="mb-4 text-sm font-semibold text-slate-900 dark:text-slate-100">
              {t('settings.language')}
            </h2>
            <div className="flex gap-2">
              {(['en', 'ar'] as const).map((l) => (
                <Button
                  key={l}
                  variant={locale === l ? 'primary' : 'secondary'}
                  className="flex-1"
                  onClick={() => setLocale(l)}
                >
                  <Languages className="size-4" />
                  {l === 'en' ? t('create.english') : t('create.arabic')}
                </Button>
              ))}
            </div>
          </Card>

          <Card>
            <h2 className="mb-4 text-sm font-semibold text-slate-900 dark:text-slate-100">
              {t('settings.theme')}
            </h2>
            <Select
              value={theme}
              onChange={(e) => updateMutation.mutate({ theme: e.target.value as Theme })}
              aria-label={t('settings.theme')}
            >
              <option value="system">{t('settings.theme.system')}</option>
              <option value="light">{t('settings.theme.light')}</option>
              <option value="dark">{t('settings.theme.dark')}</option>
            </Select>
          </Card>

          <Card>
            <h2 className="mb-4 text-sm font-semibold text-slate-900 dark:text-slate-100">
              {t('settings.translation')}
            </h2>
            <label className="flex cursor-pointer items-start gap-3">
              <input
                type="checkbox"
                checked={removeTashkeel}
                onChange={(e) => {
                  setRemoveTashkeel(e.target.checked)
                  updateMutation.mutate({ removeTashkeel: e.target.checked })
                }}
                className="mt-0.5 size-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
                aria-label={t('settings.removeTashkeel')}
              />
              <span className="flex flex-col gap-0.5">
                <span className="text-sm text-slate-700 dark:text-slate-200">
                  {t('settings.removeTashkeel')}
                </span>
                <span className="text-xs text-slate-500 dark:text-slate-400">
                  {t('settings.removeTashkeelHint')}
                </span>
              </span>
            </label>
            <div className="mt-4 flex items-start gap-3">
              <Gauge className="mt-0.5 size-4 shrink-0 text-indigo-500" />
              <div className="flex w-full flex-col gap-1">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-slate-700 dark:text-slate-200">
                    {t('settings.temperature')}
                  </span>
                  <span className="rounded-md bg-slate-100 px-1.5 py-0.5 text-xs font-medium tabular-nums text-slate-700 dark:bg-slate-800 dark:text-slate-200">
                    {temperature.toFixed(1)}
                  </span>
                </div>
                <input
                  type="range"
                  min={0.3}
                  max={0.8}
                  step={0.1}
                  value={temperature}
                  onChange={(e) => {
                    const v = Number(e.target.value)
                    setTemperature(v)
                    updateMutation.mutate({ temperature: v })
                  }}
                  className="w-full accent-indigo-600"
                  aria-label={t('settings.temperature')}
                />
                <div className="flex justify-between text-[11px] text-slate-400 dark:text-slate-500">
                  <span>{t('settings.temperature.literal')}</span>
                  <span>{t('settings.temperature.creative')}</span>
                </div>
                <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
                  {t('settings.temperatureHint')}
                </p>
              </div>
            </div>
          </Card>

          <Card>
            <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-slate-900 dark:text-slate-100">
              <KeyRound className="size-4 text-indigo-500" />
              {t('settings.apiKey')}
            </h2>
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-300">
                <span
                  className={`inline-block size-2 rounded-full ${
                    apiKeyStatus === 'configured' ? 'bg-emerald-500' : 'bg-slate-400'
                  }`}
                />
                {apiKeyStatus === 'configured'
                  ? t('settings.apiKeyConfigured')
                  : t('settings.apiKeyNotConfigured')}
              </div>
              <p className="text-xs text-slate-500 dark:text-slate-400">
                {t('settings.apiKeyHint')}
              </p>
              {apiKeyStatus === 'configured' ? (
                <div className="flex items-center gap-2">
                  <Button
                    variant="ghost"
                    onClick={() => clearApiKeyMutation.mutate()}
                    disabled={clearApiKeyMutation.isPending}
                  >
                    {t('settings.apiKeyClear')}
                  </Button>
                </div>
              ) : (
                <div className="flex flex-col gap-2">
                  <Input
                    type="password"
                    value={apiKeyInput}
                    onChange={(e) => setApiKeyInput(e.target.value)}
                    placeholder={t('settings.apiKeyPlaceholder')}
                    aria-label={t('settings.apiKey')}
                  />
                  <div className="flex items-center gap-2">
                    <Button
                      variant="secondary"
                      onClick={() => apiKeyMutation.mutate(apiKeyInput.trim())}
                      disabled={!apiKeyInput.trim() || apiKeyMutation.isPending}
                    >
                      <Save className="size-4" />
                      {t('settings.apiKeySave')}
                    </Button>
                    {apiKeyMutation.isSuccess ? (
                      <span className="text-xs text-emerald-600 dark:text-emerald-400">
                        {t('settings.apiKeySaved')}
                      </span>
                    ) : null}
                  </div>
                </div>
              )}
            </div>
          </Card>

          <Card>
            <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-slate-900 dark:text-slate-100">
              <RefreshCw className="size-4 text-indigo-500" />
              {t('updates.title')}
            </h2>
            <div className="flex flex-col gap-3">
              <Button
                variant="secondary"
                onClick={() => void checkForUpdates()}
                disabled={updateState.kind === 'checking' || updateState.kind === 'installing'}
              >
                <RefreshCw className="size-4" />
                {updateState.kind === 'checking' ? t('updates.checking') : t('updates.check')}
              </Button>
              {updateState.kind === 'none' ? (
                <span className="text-sm text-emerald-600 dark:text-emerald-400">
                  {t('updates.upToDate')}
                </span>
              ) : null}
              {updateState.kind === 'available' || updateState.kind === 'installing' ? (
                <div className="flex flex-col gap-2">
                  <span className="text-sm text-slate-700 dark:text-slate-300">
                    {t('updates.available', { version: updateState.version })}
                  </span>
                  <Button
                    className="self-start"
                    onClick={() => void installUpdate(updateState.version)}
                    disabled={updateState.kind === 'installing'}
                  >
                    {updateState.kind === 'installing'
                      ? t('updates.installing')
                      : t('updates.install')}
                  </Button>
                </div>
              ) : null}
              {updateState.kind === 'installed' ? (
                <span className="text-sm text-emerald-600 dark:text-emerald-400">
                  {t('updates.restart')}
                </span>
              ) : null}
              {updateState.kind === 'error' ? (
                <span className="text-sm text-red-600 dark:text-red-400">{t('updates.error')}</span>
              ) : null}
            </div>
          </Card>
        </div>
      )}
    </div>
  )
}
