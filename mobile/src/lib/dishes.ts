import type { Candidate } from './api/dishes.js'
import type { RecipeType } from './api/recipe.js'

/** Courses in meal order, as the composer lists them. */
export const MEAL_ORDER: readonly RecipeType[] = [
  'MainCourse',
  'Appetizer',
  'Accompaniment',
  'Dessert',
  'Beverage',
  'Condiment',
]

/** Courses a dish may hold several of; every other course is one at most. */
export function isMultiple(course: RecipeType): boolean {
  return course === 'Accompaniment' || course === 'Condiment'
}

/**
 * Select or deselect a candidate. Picking a single-choice course (a main,
 * starter, dessert or drink) replaces the one already picked for it, so the
 * selection always satisfies the composition rule for those courses. The
 * selection carries whole candidates: the composer only ever holds a page of
 * the pool, so a pick must survive the next search.
 */
export function toggle(selected: Candidate[], candidate: Candidate): Candidate[] {
  if (selected.some((c) => c.id === candidate.id)) return selected.filter((c) => c.id !== candidate.id)
  if (isMultiple(candidate.recipe_type)) return [...selected, candidate]
  return [...selected.filter((c) => c.recipe_type !== candidate.recipe_type), candidate]
}

/** Why the selection cannot be saved yet, or `null` when it can. */
export function problem(selected: Candidate[]): 'no_main' | null {
  return selected.filter((c) => c.recipe_type === 'MainCourse').length === 1 ? null : 'no_main'
}
