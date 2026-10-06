import { request } from './client.js'
import type { DietaryRestriction, Recipe, RecipeType } from './recipe.js'

export type SortBy = 'RecentlyAdded' | 'Easiest' | 'Hardest' | 'Random'

export interface Summary {
  id: string
  owner_id: string
  owner_name: string | null
  recipe_type: RecipeType
  name: string
  slug: string
  description: string
  prep_time: number
  cook_time: number
  total_time: number
  dietary_restrictions: DietaryRestriction[]
  accepts_accompaniment: boolean
  is_shared: boolean
  difficulty_score: number
  created_at: number
  thumbnail_url: string | null
  blur_placeholder: string | null
}

export interface Page<T> {
  edges: { cursor: string; node: T }[]
  page_info: {
    has_previous_page: boolean
    has_next_page: boolean
    start_cursor: string | null
    end_cursor: string | null
  }
}

export interface BrowseParams {
  after?: string
  recipe_type?: RecipeType
  search?: string
  sort_by?: SortBy
  in_meal_plan?: boolean
  mine?: boolean
  no_image?: boolean
}

export interface Browse {
  page: Page<Summary>
  has_shared: boolean
}

export interface Stat {
  total: number
  favorite: number
  shared: number
  from_community: number
}

export interface Detail extends Recipe {
  is_owner: boolean
  saved: boolean
  in_shopping: boolean
  owner_description: string
  owner_stat: Stat
}

export interface Cook {
  username: string
  description: string
  stat: Stat
  recipes: Page<Summary>
}

/** The web cook page's filters: `GET /cooks/{username}`. */
export interface CookParams {
  after?: string
  recipe_type?: RecipeType
  search?: string
  sort_by?: SortBy
}

function query(params: Record<string, string | boolean | undefined>): string {
  const q = new URLSearchParams()
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === '' || v === false) continue
    q.set(k, String(v))
  }
  const s = q.toString()
  return s ? `?${s}` : ''
}

export function browse(params: BrowseParams): Promise<Browse> {
  return request<Browse>(`/api/v1/recipes${query({ ...params })}`)
}

export function createDraft(): Promise<{ id: string }> {
  return request<{ id: string }>('/api/v1/recipes', { method: 'POST' })
}

export function getRecipe(idOrSlug: string): Promise<Detail> {
  return request<Detail>(`/api/v1/recipes/${encodeURIComponent(idOrSlug)}`)
}

export function getSimilar(idOrSlug: string): Promise<Page<Summary>> {
  return request<Page<Summary>>(`/api/v1/recipes/${encodeURIComponent(idOrSlug)}/similar`)
}

export function saveRecipe(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/save`, { method: 'POST' })
}

export function unsaveRecipe(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/save`, { method: 'DELETE' })
}

export function addRecipeToList(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/shopping`, { method: 'POST' })
}

export function shareRecipe(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/share`, { method: 'POST' })
}

export function unshareRecipe(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/unshare`, { method: 'POST' })
}

export function shareAll(): Promise<void> {
  return request<void>('/api/v1/recipes/share-all', { method: 'POST' })
}

export function unshareAll(): Promise<void> {
  return request<void>('/api/v1/recipes/unshare-all', { method: 'POST' })
}

export function deleteRecipe(id: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

export function recipeExists(id: string): Promise<boolean> {
  return request<{ exists: boolean }>(`/api/v1/recipes/${encodeURIComponent(id)}/exists`).then(
    (r) => r.exists,
  )
}

export function getCook(username: string, params: CookParams = {}): Promise<Cook> {
  return request<Cook>(`/api/v1/cooks/${encodeURIComponent(username)}${query({ ...params })}`)
}

/** Polls `exists` until it reports `expected`, at most ~10 s. */
export async function waitForRecipe(id: string, expected: boolean): Promise<boolean> {
  for (let i = 0; i < 20; i++) {
    try {
      if ((await recipeExists(id)) === expected) return true
    } catch {
      // keep polling
    }
    await new Promise((r) => setTimeout(r, 500))
  }
  return false
}
