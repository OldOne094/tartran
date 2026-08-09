import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { dictionaries, type Locale, type TKey } from './strings'

interface I18nContextValue {
  locale: Locale
  dir: 'ltr' | 'rtl'
  setLocale: (locale: Locale) => void
  t: (key: TKey, vars?: Record<string, string | number>) => string
}

const I18nContext = createContext<I18nContextValue | null>(null)

export function I18nProvider({ children }: { children: ReactNode }): ReactNode {
  const [locale, setLocaleState] = useState<Locale>('en')

  useEffect(() => {
    let cancelled = false
    window.api.settings
      .get()
      .then((s) => {
        if (!cancelled && (s.uiLanguage === 'ar' || s.uiLanguage === 'en')) {
          setLocaleState(s.uiLanguage)
        }
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    document.documentElement.lang = locale
    document.documentElement.dir = locale === 'ar' ? 'rtl' : 'ltr'
  }, [locale])

  const setLocale = (next: Locale): void => {
    setLocaleState(next)
    window.api.settings.update({ uiLanguage: next }).catch(() => {})
  }

  const t = (key: TKey, vars?: Record<string, string | number>): string => {
    let s = dictionaries[locale][key] ?? dictionaries.en[key] ?? key
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        s = s.replace(`{${k}}`, String(v))
      }
    }
    return s
  }

  return (
    <I18nContext.Provider
      value={{ locale, dir: locale === 'ar' ? 'rtl' : 'ltr', setLocale, t }}
    >
      {children}
    </I18nContext.Provider>
  )
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used within I18nProvider')
  return ctx
}

export function useT(): I18nContextValue['t'] {
  return useI18n().t
}
