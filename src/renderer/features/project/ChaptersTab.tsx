import { useState, type ReactNode } from 'react'
import { Plus, Search } from 'lucide-react'
import { useT } from '../../i18n/I18nProvider'
import { api } from '../../lib/ipcClient'
import { useChapters } from '../../lib/queries'
import { Button, EmptyState, LoadingBlock, Spinner } from '../../components/ui'
import { ImportChaptersDialog } from './ImportChaptersDialog'
import { ChapterEditor } from './ChapterEditor'
import { StatusBadge } from './StatusBadge'

export function ChaptersTab({ projectId }: { projectId: string }): ReactNode {
  const t = useT()
  const { data, isLoading } = useChapters(projectId)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [importing, setImporting] = useState(false)
  const [query, setQuery] = useState('')
  const [searchResults, setSearchResults] = useState<Array<{ id: string; snippet: string }> | null>(null)
  const [searching, setSearching] = useState(false)
  const [searchError, setSearchError] = useState(false)

  const search = (q: string): void => {
    setQuery(q)
    if (!q.trim()) {
      setSearchResults(null)
      setSearchError(false)
      return
    }
    setSearching(true)
    setSearchError(false)
    api.chapters
      .search(projectId, q.trim())
      .then((r) => setSearchResults(r))
      .catch(() => setSearchError(true))
      .finally(() => setSearching(false))
  }

  const visibleChapters = searchResults ? searchResults.map((r) => r.id) : null
  const filtered = visibleChapters ? data?.filter((c) => visibleChapters.includes(c.id)) : data

  if (isLoading) return <LoadingBlock />

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="relative w-full max-w-xs">
          <Search className="absolute start-3 top-1/2 size-4 -translate-y-1/2 text-slate-400" />
          <input
            className="w-full rounded-lg border border-slate-300 bg-white py-2 pe-3 ps-9 text-sm text-slate-900 placeholder:text-slate-400 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            value={query}
            onChange={(e) => search(e.target.value)}
            placeholder={t('chapters.searchPlaceholder')}
            aria-label={t('chapters.searchPlaceholder')}
          />
        </div>
        <Button onClick={() => setImporting(true)}>
          <Plus className="size-4" />
          {t('chapters.import')}
        </Button>
      </div>

      {searching ? (
        <div className="flex justify-center py-6">
          <Spinner />
        </div>
      ) : null}
      {searchError ? (
        <p className="text-sm text-red-600 dark:text-red-400">{t('common.error')}</p>
      ) : null}
      {query.trim() && searchResults && searchResults.length === 0 ? (
        <EmptyState title={t('chapters.noSearchResults')} />
      ) : null}

      {filtered && filtered.length === 0 && !query.trim() ? (
        <EmptyState title={t('chapters.empty')} />
      ) : null}

      {filtered && filtered.length > 0 ? (
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start">
          <div className="w-full shrink-0 lg:w-64">
            <ul className="flex max-h-[30rem] flex-col gap-1 overflow-y-auto rounded-xl border border-slate-200 bg-white p-2 dark:border-slate-800 dark:bg-slate-900">
              {filtered.map((c) => {
                const snippet = searchResults?.find((r) => r.id === c.id)?.snippet
                return (
                  <li key={c.id}>
                    <button
                      className={`w-full rounded-lg px-3 py-2 text-start transition-colors ${
                        selectedId === c.id
                          ? 'bg-indigo-50 dark:bg-indigo-950/50'
                          : 'hover:bg-slate-50 dark:hover:bg-slate-800/60'
                      }`}
                      onClick={() => setSelectedId(c.id)}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
                          {c.number}. {c.title || t('chapters.emptyTitle')}
                        </span>
                        <StatusBadge status={c.status} />
                      </div>
                      <div className="mt-0.5 flex items-center justify-between gap-2">
                        <span className="text-xs text-slate-400 dark:text-slate-500">
                          {t('chapters.wordCount', { count: c.wordCount })}
                        </span>
                      </div>
                      {snippet ? (
                        <p className="mt-1 line-clamp-2 text-xs text-slate-500 dark:text-slate-400">
                          {snippet}
                        </p>
                      ) : null}
                    </button>
                  </li>
                )
              })}
            </ul>
          </div>

          <div className="min-w-0 flex-1">
            {selectedId ? (
              <ChapterEditor
                key={selectedId}
                projectId={projectId}
                chapterId={selectedId}
                onDeleted={() => {
                  setSelectedId(null)
                  setQuery('')
                  setSearchResults(null)
                }}
              />
            ) : (
              <EmptyState title={t('chapters.selectHint')} />
            )}
          </div>
        </div>
      ) : null}

      {importing ? (
        <ImportChaptersDialog projectId={projectId} onClose={() => setImporting(false)} />
      ) : null}
    </div>
  )
}
