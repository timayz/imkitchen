import type { Groceries } from './api/groceries.js'

/**
 * The groceries view with one item set to `checked`, counters recomputed
 * the way the server computes them. Used for optimistic updates and to
 * replay queued changes on top of a fresh server response.
 */
export function setChecked(groceries: Groceries, key: string, checked: boolean): Groceries {
  let delta = 0
  const aisles = groceries.aisles.map((a) => {
    const items = a.items.map((i) => {
      if (i.key !== key || i.checked === checked) return i
      delta = checked ? 1 : -1
      return { ...i, checked }
    })
    const count = items.filter((i) => i.checked).length
    return {
      ...a,
      items,
      checked: count,
      done: a.total > 0 && count === a.total,
      pct: a.total > 0 ? Math.floor((count * 100) / a.total) : 0,
    }
  })
  if (delta === 0) return groceries
  const checked_items = groceries.checked_items + delta
  return {
    ...groceries,
    aisles,
    checked_items,
    progress_pct: groceries.total_items > 0 ? Math.floor((checked_items * 100) / groceries.total_items) : 0,
  }
}
