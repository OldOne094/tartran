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

export function useApiKeyStatus() {
  return useQuery({ queryKey: ['apiKey'], queryFn: () => api.settings.apiKeyStatus() })
}

export function useChapters(projectId: string) {
  return useQuery({
    queryKey: ['chapters', projectId],
    queryFn: () => api.chapters.list(projectId),
    enabled: projectId.length > 0
  })
}

export function useChapter(projectId: string, chapterId: string) {
  return useQuery({
    queryKey: ['chapter', projectId, chapterId],
    queryFn: () => api.chapters.get(projectId, chapterId),
    enabled: projectId.length > 0 && chapterId.length > 0
  })
}

export function useChapterSearch(projectId: string, query: string) {
  return useQuery({
    queryKey: ['chapterSearch', projectId, query],
    queryFn: () => api.chapters.search(projectId, query),
    enabled: projectId.length > 0 && query.trim().length > 0
  })
}

export function useGlossary(projectId: string) {
  return useQuery({
    queryKey: ['glossary', projectId],
    queryFn: () => api.glossary.list(projectId),
    enabled: projectId.length > 0
  })
}

export function useSuggestions(projectId: string, chapterId: string) {
  return useQuery({
    queryKey: ['suggestions', projectId, chapterId],
    queryFn: () => api.suggestions.list(projectId, chapterId),
    enabled: projectId.length > 0 && chapterId.length > 0
  })
}

export function useModels() {
  return useQuery({ queryKey: ['models'], queryFn: () => api.translation.models() })
}
