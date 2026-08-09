import { useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { BookOpenText, Plus, Trash2 } from 'lucide-react'
import type { ProjectSummary } from '../../../shared/types'
import { useI18n } from '../../i18n/I18nProvider'
import { useProjects } from '../../lib/queries'
import { Button, ConfirmDialog, EmptyState, ErrorBlock, LoadingBlock } from '../../components/ui'
import { CreateProjectDialog } from './CreateProjectDialog'

function formatDate(iso: string, locale: string): string {
  return new Date(iso).toLocaleDateString(locale === 'ar' ? 'ar' : 'en', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

function ProjectCard({
  project,
  onOpen,
  onDelete
}: {
  project: ProjectSummary
  onOpen: () => void
  onDelete: () => void
}): ReactNode {
  const { t, locale } = useI18n()
  return (
    <div className="flex flex-col rounded-xl border border-slate-200 bg-white p-5 shadow-sm transition-colors hover:border-indigo-300 dark:border-slate-800 dark:bg-slate-900 dark:hover:border-indigo-800">
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-start gap-3">
          <div className="rounded-lg bg-indigo-50 p-2.5 text-indigo-600 dark:bg-indigo-950/50 dark:text-indigo-400">
            <BookOpenText className="size-5" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-slate-900 dark:text-slate-100">
              {project.title}
            </h3>
            {project.author ? (
              <p className="text-xs text-slate-500 dark:text-slate-400">{project.author}</p>
            ) : null}
          </div>
        </div>
        <button
          className="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950/40"
          onClick={onDelete}
          title={t('projects.delete')}
          aria-label={t('projects.delete')}
        >
          <Trash2 className="size-4" />
        </button>
      </div>

      {project.corrupted ? (
        <p className="mt-3 text-xs text-red-600 dark:text-red-400">{t('projects.corrupted')}</p>
      ) : (
        <div className="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs text-slate-500 dark:text-slate-400">
          <span>{t('projects.chapters', { count: project.chapterCount })}</span>
          <span>{t('projects.translated', { count: project.translatedCount })}</span>
          <span>{t('projects.reviewed', { count: project.reviewedCount })}</span>
        </div>
      )}

      <div className="mt-4 flex items-center justify-between gap-3">
        <span className="text-xs text-slate-400 dark:text-slate-500">
          {t('projects.updated', { date: formatDate(project.updatedAt, locale) })}
        </span>
        <Button variant="secondary" className="px-3 py-1.5 text-xs" onClick={onOpen}>
          {t('projects.open')}
        </Button>
      </div>
    </div>
  )
}

export function ProjectsPage({
  onOpenProject
}: {
  onOpenProject: (projectId: string) => void
}): ReactNode {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const { data, isLoading, isError, refetch } = useProjects()
  const [creating, setCreating] = useState(false)
  const [toDelete, setToDelete] = useState<ProjectSummary | null>(null)

  const deleteMutation = useMutation({
    mutationFn: (id: string) => window.api.projects.delete(id),
    onSuccess: () => {
      setToDelete(null)
      void queryClient.invalidateQueries({ queryKey: ['projects'] })
    }
  })

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <div className="mb-6 flex items-center justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-slate-900 dark:text-slate-100">
            {t('projects.title')}
          </h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{t('nav.projects')}</p>
        </div>
        <Button onClick={() => setCreating(true)}>
          <Plus className="size-4" />
          {t('projects.new')}
        </Button>
      </div>

      {isLoading ? <LoadingBlock /> : null}

      {isError ? <ErrorBlock message={t('projects.loadError')} onRetry={() => void refetch()} /> : null}

      {data && data.length === 0 ? (
        <EmptyState title={t('projects.empty')} hint={t('projects.new')} />
      ) : null}

      {data && data.length > 0 ? (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          {data.map((p) => (
            <ProjectCard
              key={p.id}
              project={p}
              onOpen={() => onOpenProject(p.id)}
              onDelete={() => setToDelete(p)}
            />
          ))}
        </div>
      ) : null}

      {creating ? (
        <CreateProjectDialog
          onClose={() => setCreating(false)}
          onCreated={(project) => {
            void queryClient.invalidateQueries({ queryKey: ['projects'] })
            onOpenProject(project.id)
          }}
        />
      ) : null}

      {toDelete ? (
        <ConfirmDialog
          title={t('projects.deleteConfirmTitle')}
          body={t('projects.deleteConfirmBody')}
          busy={deleteMutation.isPending}
          onClose={() => setToDelete(null)}
          onConfirm={() => deleteMutation.mutate(toDelete.id)}
        />
      ) : null}
    </div>
  )
}
