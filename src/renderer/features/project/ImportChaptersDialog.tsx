import { useState, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Download } from 'lucide-react'
import type { ImportChaptersInput } from '../../../shared/types'
import { useI18n } from '../../i18n/I18nProvider'
import { api } from '../../lib/ipcClient'
import { Button, Field, Modal, Select } from '../../components/ui'

export function ImportChaptersDialog({
  projectId,
  onClose
}: {
  projectId: string
  onClose: () => void
}): ReactNode {
  const { t } = useI18n()
  const queryClient = useQueryClient()
  const [text, setText] = useState('')
  const [splitBy, setSplitBy] = useState<ImportChaptersInput['splitBy']>('auto')
  const [result, setResult] = useState<{ imported: number; skipped: number } | null>(null)

  const mutation = useMutation({
    mutationFn: (input: ImportChaptersInput) => api.chapters.import(projectId, input),
    onSuccess: (r) => {
      setResult({ imported: r.imported, skipped: r.skipped })
      void queryClient.invalidateQueries({ queryKey: ['chapters', projectId] })
      void queryClient.invalidateQueries({ queryKey: ['projects'] })
    }
  })

  const submit = (): void => {
    if (!text.trim()) return
    mutation.mutate({ text, splitBy })
  }

  return (
    <Modal title={t('chapters.importTitle')} onClose={onClose}>
      <div className="flex flex-col gap-4">
        <Field label={t('chapters.splitBy')}>
          <Select
            value={splitBy}
            onChange={(e) => setSplitBy(e.target.value as ImportChaptersInput['splitBy'])}
            aria-label={t('chapters.splitBy')}
          >
            <option value="auto">{t('chapters.split.auto')}</option>
            <option value="marker">{t('chapters.split.marker')}</option>
            <option value="paragraphs">{t('chapters.split.paragraphs')}</option>
          </Select>
        </Field>
        <Field label={t('chapters.sourceText')} hint={t('chapters.importHint')}>
          <textarea
            className="h-56 w-full resize-none rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 placeholder:text-slate-400 focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            value={text}
            onChange={(e) => setText(e.target.value)}
            placeholder={t('chapters.importPlaceholder')}
            aria-label={t('chapters.sourceText')}
          />
        </Field>
        {mutation.isError ? (
          <p className="text-sm text-red-600 dark:text-red-400">{t('common.error')}</p>
        ) : null}
        {result ? (
          <p className="text-sm text-emerald-600 dark:text-emerald-400">
            {t('chapters.imported', { count: result.imported })}
            {result.skipped > 0 ? ` · ${t('chapters.skipped', { count: result.skipped })}` : ''}
          </p>
        ) : null}
        <div className="flex justify-end gap-2">
          <Button variant="secondary" onClick={onClose} disabled={mutation.isPending}>
            {t('common.close')}
          </Button>
          <Button
            onClick={submit}
            disabled={!text.trim() || mutation.isPending}
          >
            <Download className="size-4" />
            {t('chapters.import')}
          </Button>
        </div>
      </div>
    </Modal>
  )
}
