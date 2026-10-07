import { AISLE_CATEGORIES } from './course.js'

/**
 * Pure helpers behind the aisle-order sheet in settings.
 */

/** The order with the entry at `index` moved one step up (-1) or down (1); unchanged at the edges. */
export function moveAisle(order: string[], index: number, delta: -1 | 1): string[] {
  const target = index + delta
  if (index < 0 || index >= order.length || target < 0 || target >= order.length) return order
  const next = [...order]
  next[index] = order[target]
  next[target] = order[index]
  return next
}

/**
 * Mirror of the server's `IngredientCategory::complete_aisle_order`: the first
 * occurrence of each entry keeps its place, then every missing aisle follows in
 * default order. Unknown names are dropped.
 */
export function completeAisleOrder(order: string[], all: readonly string[] = AISLE_CATEGORIES): string[] {
  const out: string[] = []
  for (const cat of [...order, ...all]) {
    if (all.includes(cat) && !out.includes(cat)) out.push(cat)
  }
  return out
}
