import { ipcMain } from 'electron'
import { z } from 'zod'
import { IPC } from '../../shared/ipc'
import { err, ok } from '../../shared/result'
import type { ProjectsManager } from '../storage/projects'
import type { Logger } from '../logger'

const createProjectSchema = z.object({
  title: z.string().min(1).max(200),
  author: z.string().max(200).default(''),
  targetLang: z.enum(['ar', 'en']),
  sourceLang: z.enum(['zh']).optional()
})

const updateProjectSchema = z.object({
  projectId: z.string().min(1),
  patch: z
    .object({
      title: z.string().min(1).max(200).optional(),
      author: z.string().max(200).optional(),
      targetLang: z.enum(['ar', 'en']).optional()
    })
    .strict()
})

export function registerProjects(deps: { projects: ProjectsManager; logger: Logger }): void {
  ipcMain.handle(IPC.projectsList, async () => {
    try {
      return ok(deps.projects.list())
    } catch (e) {
      deps.logger.error('ipc:projects:list', { err: String(e) })
      return err('LIST_FAILED', 'Could not list projects')
    }
  })

  ipcMain.handle(IPC.projectsCreate, async (_event, input: unknown) => {
    const parsed = createProjectSchema.safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid project input')
    try {
      return ok(deps.projects.create(parsed.data))
    } catch (e) {
      deps.logger.error('ipc:projects:create', { err: String(e) })
      return err('CREATE_FAILED', 'Could not create project')
    }
  })

  ipcMain.handle(IPC.projectsGet, async (_event, input: unknown) => {
    const parsed = z.string().min(1).safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid project id')
    try {
      return ok(deps.projects.get(parsed.data))
    } catch {
      return err('NOT_FOUND', 'Project not found')
    }
  })

  ipcMain.handle(IPC.projectsUpdate, async (_event, input: unknown) => {
    const parsed = updateProjectSchema.safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid project update')
    try {
      return ok(deps.projects.update(parsed.data.projectId, parsed.data.patch))
    } catch (e) {
      deps.logger.error('ipc:projects:update', { err: String(e) })
      return err('UPDATE_FAILED', 'Could not update project')
    }
  })

  ipcMain.handle(IPC.projectsDelete, async (_event, input: unknown) => {
    const parsed = z.string().min(1).safeParse(input)
    if (!parsed.success) return err('INVALID_INPUT', 'Invalid project id')
    try {
      deps.projects.delete(parsed.data)
      return ok(null)
    } catch (e) {
      deps.logger.error('ipc:projects:delete', { err: String(e) })
      return err('DELETE_FAILED', 'Could not delete project')
    }
  })
}
