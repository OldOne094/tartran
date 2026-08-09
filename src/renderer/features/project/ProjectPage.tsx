import { useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, BookMarked, FileText, Trash2 } from 'lucide-react'
import { useI18n } from '../../i18n/I18nProvider'
import { useProject } from '../../lib/queries'
import { Button, ConfirmDialog, EmptyState, ErrorBlock, LoadingBlock } from '../../components/ui'

type ProjectTab = 'chapters' | 'glossary'

export function ProjectPage({
  projectId,
  onBack,
  onDeleted
}: {
  projectId: string
  onBack: () => void
  onDeleted: () => void
}): ReactNode {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const { data, isLoading, isError } = useProject(projectId)
  const [tab, setTab] = useState<ProjectTab>('chapters')
  const [confirmDelete, setConfirmDelete] = useState(false)

  const deleteMutation = useMutation({
    mutationFn: (id: string) => window.api.projects.delete(id),
    onSuccess: () => {
      setConfirmDelete(false)
      void queryClient.invalidateQueries({ queryKey: ['projects'] })
      onDeleted()
    }
  })

  if (isLoading) return <LoadingBlock />

  if (isError || !data) {
    return (
      <div className="mx-auto max-w-3xl px-6 py-8">
        <ErrorBlock message={t('project.notFound')} onRetry={onBack} />
      </div>
    )
  }

  const langName = data.targetLang === 'ar' ? t('create.arabic') : t('create.english')

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <button
        className="mb-4 inline-flex items-center gap-1.5 text-sm text-slate-500 transition-colors hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
        onClick={onBack}
      >
        <ArrowLeft className="size-4 rtl:rotate-180" />
        {t('project.back')}
      </button>

      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold text-slate-900 dark:text-slate-100">
            {data.title}
          </h1>
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-slate-500 dark:text-slate-400">
            {data.author ? <span>{data.author}</span> : null}
            <span>
              {t('project.stats', {
                chapters: data.chapterCount,
                translated: data.translatedCount,
                reviewed: data.reviewedCount
              })}
            </span>
            <span className="rounded-full bg-indigo-50 px-2.5 py-0.5 text-xs font-medium text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-300">
              {t('project.targetTo', { lang: langName })}
            </span>
          </div>
        </div>
        <Button variant="danger" className="px-3 py-2 text-xs" onClick={() => setConfirmDelete(true)}>
          <Trash2 className="size-4" />
          {t('project.deleteTitle')}
        </Button>
      </div>

      <div className="mb-4 flex gap-1 border-b border-slate-200 dark:border-slate-800">
        <TabButton
          active={tab === 'chapters'}
          icon={<FileText className="size-4" />}
          label={t('project.chaptersTab')}
          onClick={() => setTab('chapters')}
        />
        <TabButton
          active={tab === 'glossary'}
          icon={<BookMarked className="size-4" />}
          label={t('project.glossaryTab')}
          onClick={() => setTab('glossary')}
        />
      </div>

      {tab === 'chapters' ? (
        <EmptyState title={t('project.chaptersSoon')} />
      ) : (
        <EmptyState title={t('project.glossarySoon')} />
      )}

      {confirmDelete ? (
        <ConfirmDialog
          title={t('projects.deleteConfirmTitle')}
          body={t('projects.deleteConfirmBody')}
          busy={deleteMutation.isPending}
          onClose={() => setConfirmDelete(false)}
          onConfirm={() => deleteMutation.mutate(projectId)}
        />
      ) : null}
    </div>
  )
}

function TabButton({
  active,
  icon,
  label,
  onClick
}: {
  active: boolean
  icon: ReactNode
  label: string
  onClick: () => void
}): ReactNode {
  return (
    <button
      className={`inline-flex items-center gap-2 border-b-2 px-3 py-2 text-sm font-medium transition-colors ${
        active
          ? 'border-indigo-600 text-indigo-700 dark:border-indigo-400 dark:text-indigo-300'
          : 'border-transparent text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-200'
      }`}
      onClick={onClick}
    >
      {icon}
      {label}
    </button>
  )
}
