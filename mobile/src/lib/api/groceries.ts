import { request } from './client.js'
import type { Ingredient } from './recipe.js'

export interface GroceryItem extends Ingredient {
  checked: boolean
}

export interface GroceryAisle {
  /** `shopping_<Category>` */
  key: string
  items: GroceryItem[]
  checked: number
  total: number
  done: boolean
  pct: number
}

export interface Groceries {
  recipe_count: number
  total_items: number
  checked_items: number
  progress_pct: number
  aisles: GroceryAisle[]
}

export function getGroceries(): Promise<Groceries> {
  return request<Groceries>('/api/v1/groceries')
}

export function toggleGrocery(key: string): Promise<void> {
  return request<void>('/api/v1/groceries/toggle', { method: 'POST', body: { key } })
}
