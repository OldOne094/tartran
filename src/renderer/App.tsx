import { useState, type ReactNode } from 'react'
import { BookOpenText, Library, Settings } from 'lucide-react'
import { useT } from './i18n/I18nProvider'
import { ProjectsPage } from './features/projects/ProjectsPage'
import { ProjectPage } from './features/project/ProjectPage'
import { SettingsPage } from './features/settings/SettingsPage'

type Route = { name: 'projects' } | { name: 'project'; projectId: string } | { name: 'settings' }

function NavButton({
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
      className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
        active
          ? 'bg-indigo-50 text-indigo-700 dark:bg-indigo-950/50 dark:text-indigo-300'
          : 'text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
      }`}
      onClick={onClick}
    >
      {icon}
      {label}
    </button>
  )
}

export function App(): ReactNode {
  const t = useT()
  const [route, setRoute] = useState<Route>({ name: 'projects' })

  return (
    <div className="flex h-full bg-slate-50 text-slate-900 dark:bg-slate-950 dark:text-slate-100">
      <aside className="flex w-56 shrink-0 flex-col border-e border-slate-200 bg-white dark:border-slate-800 dark:bg-slate-900">
        <div className="flex items-center gap-2 px-4 py-4">
          <div className="rounded-lg bg-indigo-600 p-1.5 text-white">
            <BookOpenText className="size-4" />
          </div>
          <span className="text-base font-semibold">{t('app.name')}</span>
        </div>
        <nav className="flex flex-col gap-1 px-2">
          <NavButton
            active={route.name === 'projects'}
            icon={<Library className="size-4" />}
            label={t('nav.projects')}
            onClick={() => setRoute({ name: 'projects' })}
          />
          <NavButton
            active={route.name === 'settings'}
            icon={<Settings className="size-4" />}
            label={t('nav.settings')}
            onClick={() => setRoute({ name: 'settings' })}
          />
        </nav>
      </aside>
      <main className="min-w-0 flex-1 overflow-y-auto">
        {route.name === 'projects' ? (
          <ProjectsPage onOpenProject={(id) => setRoute({ name: 'project', projectId: id })} />
        ) : null}
        {route.name === 'project' ? (
          <ProjectPage
            key={route.projectId}
            projectId={route.projectId}
            onBack={() => setRoute({ name: 'projects' })}
            onDeleted={() => setRoute({ name: 'projects' })}
          />
        ) : null}
        {route.name === 'settings' ? <SettingsPage /> : null}
      </main>
    </div>
  )
}
