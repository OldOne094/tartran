import { useQuery } from '@tanstack/react-query'
import { api } from './ipcClient'

export function useProjects() {
  return useQuery({ queryKey: ['projects'], queryFn: () => api.projects.list() })
}

export function useProject(projectId: string) {
  return useQuery({
    queryKey: ['project', projectId],
    queryFn: () => api.projects.get(projectId),
    enabled: projectId.length > 0
  })
}

export function useSettings() {
  return useQuery({ queryKey: ['settings'], queryFn: () => api.settings.get() })
}
