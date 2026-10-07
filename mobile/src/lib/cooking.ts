import type { Aisle, CookingScreen, Steps } from './api/kitchen.js'
import type { Ingredient, Instruction, Status } from './api/recipe.js'

/**
 * Client-side copy of the server's cooking cursor logic
 * (`web/shared/src/services/kitchen.rs`), so the cooking screen can move
 * between steps without a round trip and while offline.
 */

/** `next_status`: one step forward or back. */
export function nextStatus(direction: 'next' | 'prev', current: Status, len: number): Status {
  if (direction === 'prev') {
    if (current.status === 'idle') return { status: 'idle', step: null }
    if (current.status === 'cooking') {
      const pos = current.step ?? 0
      return pos === 0 ? { status: 'idle', step: null } : { status: 'cooking', step: pos - 1 }
    }
    return len <= 1 ? { status: 'idle', step: null } : { status: 'cooking', step: len - 2 }
  }
  if (current.status === 'idle') {
    return len <= 1 ? { status: 'completed', step: null } : { status: 'cooking', step: 0 }
  }
  if (current.status === 'cooking') {
    const pos = current.step ?? 0
    return pos + 1 < len - 1 ? { status: 'cooking', step: pos + 1 } : { status: 'completed', step: null }
  }
  return { status: 'completed', step: null }
}

/** `split_instructions` with `idle_shows_first_step = false` (the cooking screen). */
export function splitInstructions(instructions: Instruction[], status: Status): Steps {
  const text = (i: Instruction, index: number) => ({ index, text: i.description })
  const current = (index: number) => {
    const i = instructions[index]
    return i ? { index, description: i.description, time_next: i.time_next } : null
  }
  if (status.status === 'idle') return { completed: [], coming: [], current: null }
  if (status.status === 'cooking') {
    const pos = status.step ?? 0
    return {
      completed: instructions.slice(0, pos).map(text),
      coming: instructions.slice(pos + 1).map((i, k) => text(i, pos + 1 + k)),
      current: current(pos),
    }
  }
  const len = instructions.length
  return {
    completed: instructions.slice(0, Math.max(0, len - 1)).map(text),
    coming: [],
    current: len > 0 ? current(len - 1) : null,
  }
}

/** `group_ingredients_by_aisle`: `shopping_<Category>` sections, sorted by key. */
export function groupByAisle(ingredients: Ingredient[]): Aisle[] {
  const groups = new Map<string, Ingredient[]>()
  ingredients.forEach((i) => {
    const key = `shopping_${i.category ?? 'Unknown'}`
    const items = groups.get(key)
    if (items) items.push(i)
    else groups.set(key, [i])
  })
  return Array.from(groups.entries())
    .map(([key, items]) => ({ key, items }))
    .sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0))
}

/** The cooking screen as the server would render it for `status`. */
export function applyStatus(screen: CookingScreen, status: Status): CookingScreen {
  const instructions = screen.recipe.instructions
  const show_ingredients = status.status === 'idle' && instructions.length > 0
  return {
    ...screen,
    status,
    steps: splitInstructions(instructions, status),
    show_ingredients,
    // The server sends the aisles only on the ingredient screen; rebuild them
    // when a cached screen goes back to it.
    ingredient_aisles: show_ingredients
      ? screen.ingredient_aisles.length > 0
        ? screen.ingredient_aisles
        : groupByAisle(screen.recipe.ingredients)
      : [],
  }
}
