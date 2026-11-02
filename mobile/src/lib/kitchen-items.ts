import type { DishGroup, Entry, KitchenList } from './api/kitchen.js'
import type { Status } from './api/recipe.js'

/**
 * One row of the kitchen list: a loose recipe, or a composed dish ("plat")
 * holding every entry that carries its `dish_id`.
 */
export type Item = { kind: 'recipe'; entry: Entry } | { kind: 'dish'; id: string; name: string; entries: Entry[] }

/**
 * Groups the list's entries into rows, each placed where its first recipe
 * sits. Documents cached before dishes were grouped have no `dish_id`, so
 * every entry is a recipe row until the next refresh.
 */
export function items(list: Pick<KitchenList, 'entries' | 'dishes'>): Item[] {
  const names = new Map<string, string>((list.dishes ?? []).map((d: DishGroup) => [d.id, d.name]))
  const rows: Item[] = []
  const placed = new Set<string>()
  list.entries.forEach((entry) => {
    const dishId = entry.dish_id ?? null
    if (dishId === null) {
      rows.push({ kind: 'recipe', entry })
      return
    }
    if (placed.has(dishId)) return
    placed.add(dishId)
    const entries = list.entries.filter((e) => e.dish_id === dishId)
    const main = entries.find((e) => e.recipe_type === 'MainCourse') ?? entries[0]!
    rows.push({ kind: 'dish', id: dishId, name: names.get(dishId) ?? main.name, entries })
  })
  return rows
}

export function isDone(item: Item): boolean {
  if (item.kind === 'recipe') return item.entry.status.status === 'completed'
  return item.entries.every((e) => e.status.status === 'completed')
}

export function isCooking(item: Item): boolean {
  if (item.kind === 'recipe') return item.entry.status.status === 'cooking'
  return item.entries.some((e) => e.status.status === 'cooking')
}

export function doneCount(entries: Entry[]): number {
  return entries.filter((e) => e.status.status === 'completed').length
}

/** Total time of the longest recipe: the courses cook in parallel. */
export function dishTime(entries: Entry[]): number {
  return entries.reduce((max, e) => Math.max(max, e.total_time), 0)
}

/** The label of the button that opens the cooking screen for `status`. */
export function cookLabel(status: Status): 'kitchen.start' | 'kitchen.continue' | 'kitchen.again' {
  if (status.status === 'cooking') return 'kitchen.continue'
  if (status.status === 'completed') return 'kitchen.again'
  return 'kitchen.start'
}
