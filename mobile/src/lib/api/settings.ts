import { peekDoc, readDoc, writeDoc } from '../cache.js'
import { request } from './client.js'
import type { DietaryRestriction, RecipeType } from './recipe.js'

/** Cache key of the general settings document (shared across bundles). */
export const GENERAL_KEY = 'settings:general'

const FALLBACK_HOUSEHOLD_SIZE = 4

export interface General {
  email: string
  description: string
  household_size: number
  dietary_restrictions: DietaryRestriction[]
  recipe_types: RecipeType[]
  /** 0.1 – 1.0 */
  cuisine_variety_weight: number
  /** Every aisle (`IngredientCategory` names), in grocery-list order. */
  aisle_order: string[]
}

export interface Preferences {
  household_size: number
  dietary_restrictions: DietaryRestriction[]
  recipe_types: RecipeType[]
  cuisine_variety_weight: number
}

export interface Session {
  id: string
  user_agent: string
  tz: string
  current: boolean
}

export function getGeneral(): Promise<General> {
  return request<General>('/api/v1/settings/general')
}

/**
 * The household size from settings, used as the default of the "how many
 * people?" pickers. Served from the cached settings document when there is
 * one (so it works offline and across bundles), fetched otherwise.
 */
export async function defaultHouseholdSize(): Promise<number> {
  const cached = peekDoc<General>(GENERAL_KEY) ?? (await readDoc<General>(GENERAL_KEY))
  if (cached) return cached.household_size
  try {
    const general = await getGeneral()
    await writeDoc(GENERAL_KEY, general)
    return general.household_size
  } catch {
    return FALLBACK_HOUSEHOLD_SIZE
  }
}

export function updatePreferences(p: Preferences): Promise<void> {
  return request<void>('/api/v1/settings/preferences', { method: 'PUT', body: p })
}

/** A partial or repeated list is fine: the server completes it in default order. */
export function updateAisleOrder(aisles: string[]): Promise<void> {
  return request<void>('/api/v1/settings/aisles', { method: 'PUT', body: { aisles } })
}

export function updateProfile(description: string): Promise<void> {
  return request<void>('/api/v1/settings/profile', { method: 'PUT', body: { description } })
}

export function setUsername(username: string): Promise<void> {
  return request<void>('/api/v1/settings/username', { method: 'POST', body: { username } })
}

export function requestAccountPasswordReset(): Promise<void> {
  return request<void>('/api/v1/settings/account/password-reset', { method: 'POST' })
}

export function getSessions(): Promise<Session[]> {
  return request<Session[]>('/api/v1/settings/sessions')
}

export function revokeSession(id: string): Promise<void> {
  return request<void>(`/api/v1/settings/sessions/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

/** Irreversible: on success every session is gone, including this one. */
export function deleteAccount(password: string): Promise<void> {
  return request<void>('/api/v1/settings/account', { method: 'DELETE', body: { password } })
}
