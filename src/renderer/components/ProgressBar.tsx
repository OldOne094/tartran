import type { ReactNode } from 'react'

export function ProgressBar({
  percent,
  current,
  total,
  label
}: {
  percent: number
  current?: number
  total?: number
  label?: string
}): ReactNode {
  const clamped = Math.max(0, Math.min(100, percent))
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-center justify-between text-xs text-slate-600 dark:text-slate-300">
        <span>{label}</span>
        <span>
          {current != null && total != null ? `${current} / ${total} · ` : ''}
          {Math.round(clamped)}%
        </span>
      </div>
      <div
        className="h-2 w-full overflow-hidden rounded-full bg-slate-200 dark:bg-slate-800"
        role="progressbar"
        aria-valuenow={Math.round(clamped)}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          className="h-full rounded-full bg-indigo-600 transition-all duration-300"
          style={{ width: `${clamped}%` }}
        />
      </div>
    </div>
  )
}
