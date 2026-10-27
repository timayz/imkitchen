import { request } from './client.js'
import type { RecipeType } from './recipe.js'

/** A recipe of a dish, in meal order (starter, main, sides, dessert, drink, sauces). */
export interface DishRecipe {
  id: string
  name: string
  slug: string
  recipe_type: RecipeType
  prep_time: number
  cook_time: number
  total_time: number
  thumbnail_url: string | null
  blur_placeholder: string | null
}

export interface Dish {
  id: string
  name: string
  recipes: DishRecipe[]
  /** Every recipe of the dish is already in the kitchen list. */
  in_list: boolean
  /** Longest recipe, in minutes. */
  total_time: number
  created_at: number
  updated_at: number
}

/** A recipe the user can put in a dish. */
export interface Candidate {
  id: string
  name: string
  recipe_type: RecipeType
  accepts_accompaniment: boolean
}

export interface ComposeInput {
  name: string
  /** Any order; the server stores meal order. */
  recipe_ids: string[]
}

export function getDishes(): Promise<Dish[]> {
  return request<{ dishes: Dish[] }>('/api/v1/dishes').then((r) => r.dishes)
}

export function getDish(id: string): Promise<Dish> {
  return request<Dish>(`/api/v1/dishes/${encodeURIComponent(id)}`)
}

export interface CandidateQuery {
  recipe_type?: RecipeType
  /** Case-insensitive substring of the name. */
  search?: string
  /** Page size, 1..=100 (server default 20). */
  limit?: number
}

export interface CandidatePage {
  candidates: Candidate[]
  /** How many recipes match in all, beyond this page. */
  total: number
}

/** A page of the pool by name: a library of hundreds of mains is searched, never listed whole. */
export function getCandidates(query: CandidateQuery = {}): Promise<CandidatePage> {
  const params = new URLSearchParams()
  if (query.recipe_type) params.set('recipe_type', query.recipe_type)
  if (query.search) params.set('search', query.search)
  if (query.limit) params.set('limit', String(query.limit))
  const qs = params.toString()
  return request<CandidatePage>(`/api/v1/dishes/candidates${qs ? `?${qs}` : ''}`)
}

export function createDish(input: ComposeInput): Promise<{ id: string }> {
  return request<{ id: string }>('/api/v1/dishes', { method: 'POST', body: input })
}

export function updateDish(id: string, input: ComposeInput): Promise<void> {
  return request<void>(`/api/v1/dishes/${encodeURIComponent(id)}`, { method: 'PUT', body: input })
}

export function deleteDish(id: string): Promise<void> {
  return request<void>(`/api/v1/dishes/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

export function addDishToList(id: string): Promise<void> {
  return request<void>(`/api/v1/dishes/${encodeURIComponent(id)}/shopping`, { method: 'POST' })
}
