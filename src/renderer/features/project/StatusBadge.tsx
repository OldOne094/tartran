import type { ReactNode } from 'react'
import type { ChapterStatus } from '../../../shared/types'
import { useT } from '../../i18n/I18nProvider'

const statusStyles: Record<ChapterStatus, string> = {
  imported: 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300',
  translating: 'bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300',
  translated: 'bg-indigo-100 text-indigo-700 dark:bg-indigo-950/60 dark:text-indigo-300',
  reviewed: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300',
  exported: 'bg-violet-100 text-violet-700 dark:bg-violet-950/60 dark:text-violet-300'
}

export function StatusBadge({ status }: { status: ChapterStatus }): ReactNode {
  const t = useT()
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${statusStyles[status]}`}
    >
      {t(`status.${status}`)}
    </span>
  )
}
