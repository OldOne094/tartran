import { appendFileSync, mkdirSync } from 'node:fs'
import { join } from 'node:path'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

const LEVEL_ORDER: Record<LogLevel, number> = { debug: 10, info: 20, warn: 30, error: 40 }
const MAX_STRING_LEN = 300
const SECRET_PATTERN = /(AIza[0-9A-Za-z_-]{20,}|api[_-]?key[\s"':=]+[^\s"'&,}]{8,})/gi

export interface Logger {
  debug(msg: string, ctx?: Record<string, unknown>): void
  info(msg: string, ctx?: Record<string, unknown>): void
  warn(msg: string, ctx?: Record<string, unknown>): void
  error(msg: string, ctx?: Record<string, unknown>): void
}

function redact(value: string): string {
  return value.replace(SECRET_PATTERN, '[REDACTED]')
}

function truncate(value: unknown): unknown {
  if (typeof value === 'string') {
    if (value.length > MAX_STRING_LEN) {
      return `[string ${value.length} chars (truncated)]`
    }
    return redact(value)
  }
  if (Array.isArray(value)) return value.map(truncate)
  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(value)) out[k] = truncate(v)
    return out
  }
  return value
}

export function createLogger(dir: string, minLevel: LogLevel = 'info'): Logger {
  mkdirSync(dir, { recursive: true })
  let queue: string[] = []
  let flushing = false

  const flush = (): void => {
    if (flushing) return
    flushing = true
    const batch = queue.splice(0, queue.length)
    if (batch.length === 0) {
      flushing = false
      return
    }
    try {
      const day = new Date().toISOString().slice(0, 10)
      appendFileSync(join(dir, `app-${day}.log`), batch.join('\n') + '\n')
    } catch {
      /* never let logging break the app */
    }
    flushing = false
    if (queue.length > 0) queueMicrotask(flush)
  }

  const write = (level: LogLevel, msg: string, ctx?: Record<string, unknown>): void => {
    if (LEVEL_ORDER[level] < LEVEL_ORDER[minLevel]) return
    const entry = {
      t: new Date().toISOString(),
      level,
      msg: redact(msg),
      ...(ctx ? { ctx: truncate(ctx) } : {})
    }
    queue.push(JSON.stringify(entry))
    queueMicrotask(flush)
  }

  return {
    debug: (m, c) => write('debug', m, c),
    info: (m, c) => write('info', m, c),
    warn: (m, c) => write('warn', m, c),
    error: (m, c) => write('error', m, c)
  }
}

export const nullLogger: Logger = {
  debug: () => {},
  info: () => {},
  warn: () => {},
  error: () => {}
}
