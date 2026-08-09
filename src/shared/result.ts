import type { IpcResult } from './ipc'

export function ok<T>(data: T): IpcResult<T> {
  return { ok: true, data }
}

export function err(code: string, message: string): IpcResult<never> {
  return { ok: false, error: { code, message } }
}
