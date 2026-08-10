import { useEffect, useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Check, Download, FileText, Languages, Sparkles, X } from 'lucide-react'
import type { ChapterStatus, Suggestion, TranslationProgress } from '../../../shared/types'
import { useI18n, useT } from '../../i18n/I18nProvider'
import { api } from '../../lib/ipcClient'
import { useChapter, useModels, useSuggestions } from '../../lib/queries'
import { downloadExportFile } from '../../lib/download'
import { Button, ErrorBlock, LoadingBlock } from '../../components/ui'
import { ProgressBar } from '../../components/ProgressBar'
import { StatusBadge } from './StatusBadge'

export function ChapterEditor({
  projectId,
  chapterId,
  onDeleted
}: {
  projectId: string
  chapterId: string
  onDeleted: () => void
}): ReactNode {
  const t = useT()
  const { locale } = useI18n()
  const queryClient = useQueryClient()
  const { data, isLoading, isError, refetch } = useChapter(projectId, chapterId)
  const { data: storedSuggestions } = useSuggestions(projectId, chapterId)
  const { data: models } = useModels()
  const [source, setSource] = useState('')
  const [translation, setTranslation] = useState('')
  const [status, setStatus] = useState<ChapterStatus>('imported')
  const [apiKeyStatus, setApiKeyStatus] = useState<'loading' | 'configured' | 'none'>('loading')
  const [suggestions, setSuggestions] = useState<Suggestion[]>([])
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [progress, setProgress] = useState<TranslationProgress | null>(null)
  const [tokensUsed, setTokensUsed] = useState<number | null>(null)
  const [translateError, setTranslateError] = useState<string | null>(null)
  const [chunkCount, setChunkCount] = useState<number | null>(null)
  const [model, setModel] = useState<string>('')

  useEffect(() => {
    if (data) {
      setSource(data.sourceText)
      setTranslation(data.translation)
      setStatus(data.status)
    }
  }, [data])

  useEffect(() => {
    if (storedSuggestions) setSuggestions(storedSuggestions)
  }, [storedSuggestions])

  useEffect(() => {
    api.settings
      .apiKeyStatus()
      .then((s) => setApiKeyStatus(s.configured ? 'configured' : 'none'))
      .catch(() => setApiKeyStatus('none'))
  }, [])

  useEffect(() => {
    let unlisten: UnlistenFn | undefined
    let cancelled = false
    listen<TranslationProgress>('translation:progress', (event) => {
      if (!cancelled && event.payload.chapterId === chapterId) {
        setProgress(event.payload)
      }
    })
      .then((fn) => {
        if (cancelled) fn()
        else unlisten = fn
      })
      .catch(() => {})
    return () => {
      cancelled = true
      if (unlisten) unlisten()
    }
  }, [chapterId])

  const saveMutation = useMutation({
    mutationFn: (patch: { sourceText?: string; translation?: string; status?: ChapterStatus }) =>
      api.chapters.update(projectId, chapterId, patch),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['chapters', projectId] })
      void queryClient.invalidateQueries({ queryKey: ['chapter', projectId, chapterId] })
      void queryClient.invalidateQueries({ queryKey: ['projects'] })
    }
  })

  const translateMutation = useMutation({
    mutationFn: () =>
      api.translation.translateChapter(projectId, { chapterId, model: model || undefined }),
    onMutate: () => {
      setProgress({ chapterId, current: 0, total: 1, percent: 0 })
      setTokensUsed(null)
      setTranslateError(null)
      setChunkCount(null)
    },
    onSuccess: (result) => {
      setProgress(null)
      setTokensUsed(result.tokensUsed)
      setChunkCount(result.chunkCount)
      setTranslation(result.translation)
      setStatus('translated')
      setSuggestions(result.suggestions)
      saveMutation.mutate({
        translation: result.translation,
        status: 'translated'
      })
      void queryClient.invalidateQueries({ queryKey: ['suggestions', projectId, chapterId] })
    },
    onError: (error) => {
      setProgress(null)
      setTranslateError(error instanceof Error ? error.message : t('common.error'))
    }
  })

  const deleteMutation = useMutation({
    mutationFn: () => api.chapters.delete(projectId, chapterId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['chapters', projectId] })
      void queryClient.invalidateQueries({ queryKey: ['projects'] })
      onDeleted()
    }
  })

  const exportTxt = useMutation({
    mutationFn: () => api.export.chapterText(projectId, chapterId),
    onSuccess: (file) => downloadExportFile(file)
  })

  const exportDocx = useMutation({
    mutationFn: () => api.export.chapterDocx(projectId, chapterId, 'ar'),
    onSuccess: (file) => downloadExportFile(file)
  })

  const pendingSuggestions = suggestions.filter((s) => s.status === 'pending')

  const approve = (suggestion: Suggestion): void => {
    api.suggestions
      .approve(projectId, suggestion.id)
      .then(() => {
        setSuggestions((prev) =>
          prev.map((s) => (s.id === suggestion.id ? { ...s, status: 'approved' } : s))
        )
        void queryClient.invalidateQueries({ queryKey: ['glossary', projectId] })
      })
      .catch(() => {})
  }

  const reject = (suggestion: Suggestion): void => {
    api.suggestions
      .reject(projectId, suggestion.id)
      .then(() => {
        setSuggestions((prev) =>
          prev.map((s) => (s.id === suggestion.id ? { ...s, status: 'rejected' } : s))
        )
      })
      .catch(() => {})
  }

  if (isLoading) return <LoadingBlock />
  if (isError || !data) {
    return <ErrorBlock message={t('project.notFound')} onRetry={() => void refetch()} />
  }

  const dir = locale === 'ar' ? 'rtl' : 'ltr'

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
            {data.number}. {data.title || t('chapters.emptyTitle')}
          </h2>
          <StatusBadge status={status} />
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            className="px-3 py-1.5 text-xs"
            onClick={() => exportTxt.mutate()}
            disabled={exportTxt.isPending}
            title={t('editor.export.txt')}
          >
            <FileText className="size-4" />
            {t('editor.export.txt')}
          </Button>
          <Button
            variant="secondary"
            className="px-3 py-1.5 text-xs"
            onClick={() => exportDocx.mutate()}
            disabled={exportDocx.isPending}
            title={t('editor.export.docx')}
          >
            <Download className="size-4" />
            {t('editor.export.docx')}
          </Button>
          <Button
            variant="danger"
            className="px-3 py-1.5 text-xs"
            onClick={() => setConfirmDelete(true)}
          >
            <X className="size-4" />
            {t('chapters.delete')}
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div className="flex flex-col gap-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
              {t('chapters.sourceText')}
            </span>
            <span className="text-xs text-slate-400 dark:text-slate-500">
              {t('chapters.wordCount', { count: data.wordCount })}
            </span>
          </div>
          <textarea
            dir="ltr"
            className="h-80 w-full resize-none rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm leading-relaxed text-slate-900 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            value={source}
            onChange={(e) => setSource(e.target.value)}
          />
          <div>
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => saveMutation.mutate({ sourceText: source, status })}
              disabled={saveMutation.isPending}
            >
              {t('chapters.save')}
            </Button>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
              {t('editor.translation')}
            </span>
            <div className="flex items-center gap-2">
              {models && models.length > 0 ? (
                <select
                  className="rounded-lg border border-slate-300 bg-white px-2 py-1.5 text-xs text-slate-700 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200"
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  disabled={translateMutation.isPending}
                  aria-label={t('editor.model')}
                  data-model-select="true"
                >
                  <option value="">{t('editor.modelDefault')}</option>
                  {models.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.label}
                    </option>
                  ))}
                </select>
              ) : null}
              <Button
                onClick={() => translateMutation.mutate()}
                disabled={translateMutation.isPending || apiKeyStatus !== 'configured'}
                className="px-3 py-1.5 text-xs"
                data-translate="true"
              >
                <Languages className="size-4" />
                {translateMutation.isPending ? t('editor.translating') : t('editor.translate')}
              </Button>
            </div>
          </div>
          <textarea
            dir={dir}
            className="h-80 w-full resize-none rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm leading-relaxed text-slate-900 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            value={translation}
            onChange={(e) => setTranslation(e.target.value)}
            placeholder={translation ? undefined : t('editor.translation')}
          />
          <div className="flex gap-2">
            <Button
              variant="secondary"
              className="flex-1"
              onClick={() =>
                saveMutation.mutate({ translation, status: status === 'imported' ? 'translated' : status })
              }
              disabled={saveMutation.isPending}
            >
              {t('chapters.save')}
            </Button>
            <Button
              variant="secondary"
              className="flex-1"
              onClick={() => {
                setStatus('reviewed')
                saveMutation.mutate({ translation, status: 'reviewed' })
              }}
              disabled={saveMutation.isPending || !translation.trim()}
            >
              <Check className="size-4" />
              {t('status.reviewed')}
            </Button>
          </div>
        </div>
      </div>

      {apiKeyStatus === 'none' ? (
        <p className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-700 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300">
          {t('editor.noKey')}
        </p>
      ) : null}

      {progress ? (
        <div className="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900">
          <ProgressBar
            percent={progress.percent}
            current={progress.current}
            total={progress.total}
            label={t('editor.translating')}
          />
        </div>
      ) : null}

      {translateError ? (
        <p className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-300">
          {translateError}
        </p>
      ) : null}

      {tokensUsed !== null && !progress ? (
        <p className="text-xs text-slate-500 dark:text-slate-400">
          {chunkCount !== null && chunkCount > 1
            ? `${t('editor.chunked', { n: chunkCount })} · `
            : ''}
          {t('editor.tokensUsed', { count: tokensUsed })}
        </p>
      ) : null}

      <div className="rounded-xl border border-slate-200 bg-white p-4 dark:border-slate-800 dark:bg-slate-900">
        <div className="mb-3 flex items-center gap-2">
          <Sparkles className="size-4 text-indigo-500" />
          <h3 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
            {t('editor.suggestionsTitle')}
          </h3>
          <span className="rounded-full bg-indigo-50 px-2 py-0.5 text-xs font-medium text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-300">
            {pendingSuggestions.length}
          </span>
        </div>
        {pendingSuggestions.length === 0 ? (
          <p className="text-sm text-slate-500 dark:text-slate-400">
            {t('editor.suggestionsEmpty')}
          </p>
        ) : (
          <ul className="flex flex-col gap-2">
            {pendingSuggestions.map((s) => (
              <li
                key={s.id}
                className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 dark:border-slate-800 dark:bg-slate-800/50"
              >
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-slate-900 dark:text-slate-100" dir="ltr">
                      {s.zh}
                    </span>
                    <span className="text-xs text-slate-400">→</span>
                    <span className="text-sm text-slate-700 dark:text-slate-300">{s.ar || s.en}</span>
                  </div>
                  <p className="mt-0.5 truncate text-xs text-slate-500 dark:text-slate-400">
                    {s.context}
                  </p>
                </div>
                <div className="flex shrink-0 gap-2">
                  <Button
                    variant="secondary"
                    className="px-2.5 py-1 text-xs text-emerald-600 dark:text-emerald-400"
                    onClick={() => approve(s)}
                  >
                    <Check className="size-3.5" />
                    {t('editor.approve')}
                  </Button>
                  <Button
                    variant="ghost"
                    className="px-2.5 py-1 text-xs"
                    onClick={() => reject(s)}
                  >
                    <X className="size-3.5" />
                    {t('editor.reject')}
                  </Button>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {confirmDelete ? (
        <ModalConfirm
          title={t('chapters.deleteConfirmTitle')}
          body={t('chapters.deleteConfirmBody')}
          busy={deleteMutation.isPending}
          onClose={() => setConfirmDelete(false)}
          onConfirm={() => deleteMutation.mutate()}
        />
      ) : null}
    </div>
  )
}

function ModalConfirm({
  title,
  body,
  busy,
  onClose,
  onConfirm
}: {
  title: string
  body: string
  busy: boolean
  onClose: () => void
  onConfirm: () => void
}): ReactNode {
  const t = useT()
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div className="absolute inset-0 bg-slate-900/50" onClick={onClose} />
      <div className="relative w-full max-w-md rounded-xl border border-slate-200 bg-white p-5 shadow-xl dark:border-slate-800 dark:bg-slate-900">
        <h2 className="mb-2 text-base font-semibold text-slate-900 dark:text-slate-100">{title}</h2>
        <p className="text-sm text-slate-600 dark:text-slate-300">{body}</p>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" onClick={onClose} disabled={busy}>
            {t('common.cancel')}
          </Button>
          <Button variant="danger" onClick={onConfirm} disabled={busy}>
            {t('common.delete')}
          </Button>
        </div>
      </div>
    </div>
  )
}
