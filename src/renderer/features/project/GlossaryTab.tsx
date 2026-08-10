import { useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Download, Lock, Pencil, Plus, Search, Trash2 } from 'lucide-react'
import type { GlossaryEntry } from '../../../shared/types'
import { useI18n, useT } from '../../i18n/I18nProvider'
import type { TKey } from '../../i18n/strings'
import { api } from '../../lib/ipcClient'
import { useGlossary } from '../../lib/queries'
import { downloadExportFile } from '../../lib/download'
import { Button, ConfirmDialog, EmptyState, LoadingBlock, Spinner } from '../../components/ui'
import { GlossaryDialog } from './GlossaryDialog'

export function GlossaryTab({ projectId }: { projectId: string }): ReactNode {
  const t = useT()
  const { locale } = useI18n()
  const queryClient = useQueryClient()
  const { data, isLoading } = useGlossary(projectId)
  const [editing, setEditing] = useState<GlossaryEntry | 'new' | null>(null)
  const [toDelete, setToDelete] = useState<GlossaryEntry | null>(null)
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<Array<{ id: string; snippet: string }> | null>(null)
  const [searching, setSearching] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.glossary.delete(projectId, id),
    onSuccess: () => {
      setToDelete(null)
      void queryClient.invalidateQueries({ queryKey: ['glossary', projectId] })
    }
  })

  const exportXlsx = useMutation({
    mutationFn: () => api.export.glossaryXlsx(projectId),
    onSuccess: (file) => downloadExportFile(file)
  })

  const search = (q: string): void => {
    setQuery(q)
    if (!q.trim()) {
      setResults(null)
      return
    }
    setSearching(true)
    api.glossary
      .search(projectId, q.trim())
      .then((r) => setResults(r))
      .catch(() => setResults([]))
      .finally(() => setSearching(false))
  }

  if (isLoading) return <LoadingBlock />

  const matched = results ? new Set(results.map((r) => r.id)) : null
  const visible = results ? data?.filter((e) => matched?.has(e.id)) : data

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="relative w-full max-w-xs">
          <Search className="absolute start-3 top-1/2 size-4 -translate-y-1/2 text-slate-400" />
          <input
            className="w-full rounded-lg border border-slate-300 bg-white py-2 pe-3 ps-9 text-sm text-slate-900 placeholder:text-slate-400 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            value={query}
            onChange={(e) => search(e.target.value)}
            placeholder={t('glossary.searchPlaceholder')}
            aria-label={t('glossary.searchPlaceholder')}
          />
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            onClick={() => exportXlsx.mutate()}
            disabled={exportXlsx.isPending}
          >
            <Download className="size-4" />
            {t('glossary.exportXlsx')}
          </Button>
          <Button onClick={() => setEditing('new')}>
            <Plus className="size-4" />
            {t('glossary.add')}
          </Button>
        </div>
      </div>

      {searching ? (
        <div className="flex justify-center py-6">
          <Spinner />
        </div>
      ) : null}

      {visible && visible.length === 0 && !query.trim() ? (
        <EmptyState title={t('glossary.empty')} />
      ) : null}

      {visible && visible.length === 0 && query.trim() ? (
        <EmptyState title={t('chapters.noSearchResults')} />
      ) : null}

      {visible && visible.length > 0 ? (
        <div className="overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900">
          <table className="w-full text-start text-sm">
            <thead>
              <tr className="border-b border-slate-200 text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
                <th className="px-4 py-2.5 text-start font-medium">{t('glossary.zh')}</th>
                <th className="px-4 py-2.5 text-start font-medium">{t('glossary.en')}</th>
                <th className="px-4 py-2.5 text-start font-medium">{t('glossary.ar')}</th>
                <th className="px-4 py-2.5 text-start font-medium">{t('glossary.category')}</th>
                <th className="px-4 py-2.5 text-end font-medium"></th>
              </tr>
            </thead>
            <tbody>
              {visible.map((e) => (
                <tr
                  key={e.id}
                  className="border-b border-slate-100 last:border-0 hover:bg-slate-50 dark:border-slate-800/60 dark:hover:bg-slate-800/40"
                >
                  <td className="px-4 py-2.5" dir="ltr">
                    <div className="font-medium text-slate-900 dark:text-slate-100">{e.zh}</div>
                    {e.locked ? (
                      <span className="mt-0.5 inline-flex items-center gap-1 text-xs text-slate-400">
                        <Lock className="size-3" />
                        {t('glossary.locked')}
                      </span>
                    ) : null}
                  </td>
                  <td className="px-4 py-2.5 text-slate-700 dark:text-slate-300">{e.en || '—'}</td>
                  <td className="px-4 py-2.5 text-slate-700 dark:text-slate-300" dir={locale === 'ar' ? 'rtl' : 'ltr'}>
                    {e.ar || '—'}
                  </td>
                  <td className="px-4 py-2.5 text-slate-500 dark:text-slate-400">
                    {e.category ? t(`glossary.category.${e.category}` as TKey) : '—'}
                  </td>
                  <td className="px-4 py-2.5">
                    <div className="flex justify-end gap-1">
                      <button
                        className="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-slate-800"
                        onClick={() => setEditing(e)}
                        title={t('common.edit')}
                        aria-label={t('common.edit')}
                      >
                        <Pencil className="size-3.5" />
                      </button>
                      <button
                        className="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/40"
                        onClick={() => setToDelete(e)}
                        title={t('common.delete')}
                        aria-label={t('common.delete')}
                      >
                        <Trash2 className="size-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

      {editing ? (
        <GlossaryDialog
          projectId={projectId}
          entry={editing === 'new' ? null : editing}
          onClose={() => setEditing(null)}
        />
      ) : null}

      {toDelete ? (
        <ConfirmDialog
          title={t('glossary.deleteConfirmTitle')}
          body={t('glossary.deleteConfirmBody')}
          busy={deleteMutation.isPending}
          onClose={() => setToDelete(null)}
          onConfirm={() => deleteMutation.mutate(toDelete.id)}
        />
      ) : null}
    </div>
  )
}
