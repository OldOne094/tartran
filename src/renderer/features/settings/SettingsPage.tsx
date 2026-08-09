import { useEffect, useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { KeyRound, Languages, Save } from 'lucide-react'
import type { Theme } from '../../../shared/types'
import { useI18n, useT } from '../../i18n/I18nProvider'
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
  const [apiKeyStatus, setApiKeyStatus] = useState<'loading' | 'configured' | 'none'>('loading')
  const systemTheme = useSystemTheme()

  useEffect(() => {
    if (data) {
      setWorkspace(data.workspacePath)
      setTheme(data.theme)
    }
  }, [data])

  useEffect(() => {
    window.api.settings
      .apiKeyStatus()
      .then((s) => setApiKeyStatus(s.configured ? 'configured' : 'none'))
      .catch(() => setApiKeyStatus('none'))
  }, [])

  useEffect(() => {
    const resolved = theme === 'system' ? systemTheme : theme
    document.documentElement.classList.toggle('dark', resolved === 'dark')
  }, [theme, systemTheme])

  const updateMutation = useMutation({
    mutationFn: (patch: { workspacePath?: string; theme?: Theme }) =>
      window.api.settings.update(patch),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['settings'] })
      window.setTimeout(() => updateMutation.reset(), 2000)
    }
  })

  const saveWorkspace = (): void => {
    updateMutation.mutate({ workspacePath: workspace.trim() || undefined })
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
                <Input
                  value={workspace}
                  onChange={(e) => setWorkspace(e.target.value)}
                  aria-label={t('settings.workspace')}
                />
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
            <h2 className="mb-4 flex items-center gap-2 text-sm font-semibold text-slate-900 dark:text-slate-100">
              <KeyRound className="size-4 text-indigo-500" />
              {t('settings.apiKey')}
            </h2>
            <p className="text-sm text-slate-600 dark:text-slate-300">
              {apiKeyStatus === 'configured'
                ? t('settings.apiKeyConfigured')
                : t('settings.apiKeyNotConfigured')}
            </p>
            <p className="mt-1 text-xs text-slate-500 dark:text-slate-400">
              {t('settings.apiKeyHint')}
            </p>
          </Card>
        </div>
      )}
    </div>
  )
}
