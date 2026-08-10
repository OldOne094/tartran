import { useEffect, useState, type FormEvent, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { CreateGlossaryInput, GlossaryEntry, UpdateGlossaryInput } from '../../../shared/types'
import { useT } from '../../i18n/I18nProvider'
import type { TKey } from '../../i18n/strings'
import { api } from '../../lib/ipcClient'
import { Button, Field, Input, Modal, Select } from '../../components/ui'

const CATEGORIES = ['character', 'location', 'item', 'technique', 'faction', 'other']

export function GlossaryDialog({
  projectId,
  entry,
  onClose
}: {
  projectId: string
  entry: GlossaryEntry | null
  onClose: () => void
}): ReactNode {
  const t = useT()
  const queryClient = useQueryClient()
  const isEdit = entry !== null

  const [zh, setZh] = useState('')
  const [en, setEn] = useState('')
  const [ar, setAr] = useState('')
  const [category, setCategory] = useState('character')
  const [notes, setNotes] = useState('')
  const [aliases, setAliases] = useState('')
  const [locked, setLocked] = useState(false)

  useEffect(() => {
    if (entry) {
      setZh(entry.zh)
      setEn(entry.en)
      setAr(entry.ar)
      setCategory(entry.category || 'character')
      setNotes(entry.notes)
      setAliases(entry.aliases.join(', '))
      setLocked(entry.locked)
    }
  }, [entry])

  const mutation = useMutation({
    mutationFn: (input: CreateGlossaryInput | UpdateGlossaryInput) =>
      isEdit
        ? api.glossary.update(projectId, entry.id, input)
        : api.glossary.create(projectId, input as CreateGlossaryInput),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['glossary', projectId] })
      onClose()
    }
  })

  const submit = (e: FormEvent): void => {
    e.preventDefault()
    if (!zh.trim()) return
    const base = {
      zh: zh.trim(),
      en: en.trim() || undefined,
      ar: ar.trim() || undefined,
      category: category || undefined,
      notes: notes.trim() || undefined,
      aliases: aliases
        .split(',')
        .map((a) => a.trim())
        .filter(Boolean)
    }
    if (isEdit) {
      mutation.mutate({ ...base, locked })
    } else {
      mutation.mutate(base as CreateGlossaryInput)
    }
  }

  return (
    <Modal title={isEdit ? t('glossary.edit') : t('glossary.add')} onClose={onClose}>
      <form onSubmit={submit} className="flex flex-col gap-4">
        <Field label={t('glossary.zh')}>
          <Input
            autoFocus
            dir="ltr"
            value={zh}
            onChange={(e) => setZh(e.target.value)}
            aria-label={t('glossary.zh')}
          />
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label={t('glossary.en')}>
            <Input
              dir="ltr"
              value={en}
              onChange={(e) => setEn(e.target.value)}
              aria-label={t('glossary.en')}
            />
          </Field>
          <Field label={t('glossary.ar')}>
            <Input
              dir="rtl"
              value={ar}
              onChange={(e) => setAr(e.target.value)}
              aria-label={t('glossary.ar')}
            />
          </Field>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Field label={t('glossary.category')}>
            <Select
              value={category}
              onChange={(e) => setCategory(e.target.value)}
              aria-label={t('glossary.category')}
            >
              {CATEGORIES.map((c) => (
                <option key={c} value={c}>
                  {t(`glossary.category.${c}` as TKey)}
                </option>
              ))}
            </Select>
          </Field>
          <Field label={t('glossary.aliases')}>
            <Input
              dir="ltr"
              value={aliases}
              onChange={(e) => setAliases(e.target.value)}
              placeholder="a, b, c"
              aria-label={t('glossary.aliases')}
            />
          </Field>
        </div>
        <Field label={t('glossary.notes')}>
          <Input value={notes} onChange={(e) => setNotes(e.target.value)} aria-label={t('glossary.notes')} />
        </Field>
        {isEdit ? (
          <label className="flex items-center gap-2 text-sm text-slate-700 dark:text-slate-300">
            <input
              type="checkbox"
              checked={locked}
              onChange={(e) => setLocked(e.target.checked)}
              className="size-4 rounded border-slate-300 text-indigo-600 focus:ring-indigo-500"
            />
            {t('glossary.locked')}
          </label>
        ) : null}
        {mutation.isError ? (
          <p className="text-sm text-red-600 dark:text-red-400">{t('common.error')}</p>
        ) : null}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose} disabled={mutation.isPending}>
            {t('common.cancel')}
          </Button>
          <Button type="submit" disabled={!zh.trim() || mutation.isPending}>
            {t('glossary.save')}
          </Button>
        </div>
      </form>
    </Modal>
  )
}
