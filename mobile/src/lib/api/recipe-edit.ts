import { request } from './client.js'
import type { DietaryRestriction, Ingredient as IngredientDto, Instruction, RecipeType } from './recipe.js'

/** An ingredient as the editor sends it (no derived fields). */
export interface IngredientInput {
  name: string
  quantity: number
  unit: 'G' | 'ML' | null
  category: string | null
}

/** The editable fields, what `GET /recipes/{id}/edit` returns and `PUT` takes. */
export interface RecipeInput {
  recipe_type: RecipeType
  name: string
  origin: string | null
  description: string
  household_size: number
  prep_time: number
  cook_time: number
  ingredients: IngredientInput[]
  instructions: Instruction[]
  dietary_restrictions: DietaryRestriction[]
  accepts_accompaniment: boolean
  advance_prep: string
}

export interface Imported {
  last_id: string | null
  errors: { name: string; error: string }[]
}

export function getEditable(id: string): Promise<RecipeInput> {
  return request<RecipeInput>(`/api/v1/recipes/${encodeURIComponent(id)}/edit`)
}

export function updateRecipe(id: string, input: RecipeInput): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}`, { method: 'PUT', body: input })
}

export function importRecipes(recipes: RecipeInput[]): Promise<Imported> {
  return request<Imported>('/api/v1/recipes/import', { method: 'POST', body: recipes })
}

/** Base64 upload (the Lynx `fetch` has no FormData). */
export function uploadThumbnail(id: string, contentType: string, dataBase64: string): Promise<void> {
  return request<void>(`/api/v1/recipes/${encodeURIComponent(id)}/thumbnail`, {
    method: 'POST',
    body: { content_type: contentType, data_base64: dataBase64 },
  })
}

export type { IngredientDto }
