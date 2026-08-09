import { useState, type FormEvent, type ReactNode } from 'react'
import { useMutation } from '@tanstack/react-query'
import { FolderPlus } from 'lucide-react'
import type { CreateProjectInput, ProjectSummary, TargetLang } from '../../../shared/types'
import { useI18n } from '../../i18n/I18nProvider'
import { Button, Field, Input, Modal, Select } from '../../components/ui'

export function CreateProjectDialog({
  onCreated,
  onClose
}: {
  onCreated: (project: ProjectSummary) => void
  onClose: () => void
}): ReactNode {
  const { t } = useI18n()
  const [title, setTitle] = useState('')
  const [author, setAuthor] = useState('')
  const [targetLang, setTargetLang] = useState<TargetLang>('ar')

  const mutation = useMutation({
    mutationFn: (input: CreateProjectInput) => window.api.projects.create(input),
    onSuccess: (project) => {
      onClose()
      onCreated(project)
    }
  })

  const submit = (e: FormEvent): void => {
    e.preventDefault()
    if (!title.trim()) return
    mutation.mutate({ title: title.trim(), author: author.trim(), targetLang })
  }

  return (
    <Modal title={t('create.title')} onClose={onClose}>
      <form onSubmit={submit} className="flex flex-col gap-4">
        <Field label={t('create.name')}>
          <Input
            autoFocus
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder={t('create.namePlaceholder')}
            aria-label={t('create.name')}
          />
        </Field>
        <Field label={t('create.author')}>
          <Input value={author} onChange={(e) => setAuthor(e.target.value)} aria-label={t('create.author')} />
        </Field>
        <Field label={t('create.target')}>
          <Select
            value={targetLang}
            onChange={(e) => setTargetLang(e.target.value as TargetLang)}
            aria-label={t('create.target')}
          >
            <option value="ar">{t('create.arabic')}</option>
            <option value="en">{t('create.english')}</option>
          </Select>
        </Field>
        {mutation.isError ? (
          <p className="text-sm text-red-600 dark:text-red-400">{t('create.error')}</p>
        ) : null}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose} disabled={mutation.isPending}>
            {t('create.cancel')}
          </Button>
          <Button type="submit" disabled={!title.trim() || mutation.isPending}>
            <FolderPlus className="size-4" />
            {t('create.submit')}
          </Button>
        </div>
      </form>
    </Modal>
  )
}
