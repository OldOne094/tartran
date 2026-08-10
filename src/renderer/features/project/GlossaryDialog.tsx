import { useEffect, useState, type FormEvent, type ReactNode } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import type { CreateGlossaryInput, GlossaryEntry, UpdateGlossaryInput } from '../../../shared/types'
import { useT } from '../../i18n/I18nProvider'
import type { TKey } from '../../i18n/strings'
import { api } from '../../lib/ipcClient'
import { Button, Field, Input, Modal, Select } from '../../components/ui'

const CATEGORIES = ['character', 'location', 'item', 'technique', 'faction', 'other']

type ReplaceScope = 'none' | 'all' | 'chapter'

export function GlossaryDialog({
  projectId,
  entry,
  chapterId,
  onClose
}: {
  projectId: string
  entry: GlossaryEntry | null
  chapterId?: string
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
  const [replaceScope, setReplaceScope] = useState<ReplaceScope>('none')
  const [replaceDone, setReplaceDone] = useState<number | null>(null)

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
    setReplaceScope('none')
    setReplaceDone(null)
  }, [entry])

  const mutation = useMutation({
    mutationFn: (input: CreateGlossaryInput | UpdateGlossaryInput) =>
      isEdit
        ? api.glossary.update(projectId, entry.id, input)
        : api.glossary.create(projectId, input as CreateGlossaryInput),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['glossary', projectId] })
      if (isEdit && replaceScope !== 'none') {
        void runReplacements().then((total) => {
          setReplaceDone(total)
          void queryClient.invalidateQueries({ queryKey: ['chapters', projectId] })
          window.setTimeout(onClose, 1200)
        })
      } else {
        onClose()
      }
    }
  })

  async function runReplacements(): Promise<number> {
    if (!isEdit) return 0
    const oldAr = entry.ar.trim()
    const newAr = ar.trim()
    const oldEn = entry.en.trim()
    const newEn = en.trim()
    const chapter = replaceScope === 'chapter' ? chapterId : undefined
    const calls: Array<Promise<{ changed: number }>> = []
    if (oldAr && newAr && oldAr !== newAr) {
      calls.push(api.glossary.replace(projectId, oldAr, newAr, chapter))
    }
    if (oldEn && newEn && oldEn !== newEn) {
      calls.push(api.glossary.replace(projectId, oldEn, newEn, chapter))
    }
    const results = await Promise.all(calls)
    return results.reduce((n, r) => n + r.changed, 0)
  }

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

  const valueChanged = isEdit && (en !== entry.en || ar !== entry.ar)

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
        {isEdit && valueChanged ? (
          <fieldset className="rounded-lg border border-slate-200 bg-slate-50 p-3 dark:border-slate-700 dark:bg-slate-800/50">
            <legend className="px-1 text-xs font-medium text-slate-600 dark:text-slate-300">
              {t('editor.replaceLabel')}
            </legend>
            <p className="mb-2 text-xs text-slate-500 dark:text-slate-400">{t('editor.replaceHint')}</p>
            <div className="flex flex-wrap gap-3 text-sm">
              <label className="flex cursor-pointer items-center gap-1.5">
                <input
                  type="radio"
                  name="replace-scope"
                  value="none"
                  checked={replaceScope === 'none'}
                  onChange={() => setReplaceScope('none')}
                  className="size-3.5 text-indigo-600"
                />
                {t('editor.replaceNone')}
              </label>
              <label className="flex cursor-pointer items-center gap-1.5">
                <input
                  type="radio"
                  name="replace-scope"
                  value="all"
                  checked={replaceScope === 'all'}
                  onChange={() => setReplaceScope('all')}
                  className="size-3.5 text-indigo-600"
                />
                {t('editor.replaceAll')}
              </label>
              {chapterId ? (
                <label className="flex cursor-pointer items-center gap-1.5">
                  <input
                    type="radio"
                    name="replace-scope"
                    value="chapter"
                    checked={replaceScope === 'chapter'}
                    onChange={() => setReplaceScope('chapter')}
                    className="size-3.5 text-indigo-600"
                  />
                  {t('editor.replaceChapter')}
                </label>
              ) : null}
            </div>
            {replaceDone !== null ? (
              <p className="mt-2 text-xs font-medium text-emerald-600 dark:text-emerald-400">
                {t('editor.replaceDone', { count: replaceDone })}
              </p>
            ) : null}
          </fieldset>
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
