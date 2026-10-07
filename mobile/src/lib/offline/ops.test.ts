import { describe, expect, it } from 'vitest'

import type { Groceries } from '../api/groceries.js'
import type { CookingScreen, Dish, KitchenList, Overview } from '../api/kitchen.js'
import type { Recipe, Status } from '../api/recipe.js'
import type { Detail } from '../api/recipes.js'
import { affects, apply, logicalKey } from './ops.js'

const recipe: Recipe = {
  id: 'r1',
  owner_id: 'u1',
  owner_name: null,
  recipe_type: 'MainCourse',
  name: 'Shakshuka',
  slug: 'shakshuka',
  origin: null,
  description: '',
  household_size: 2,
  prep_time: 10,
  cook_time: 25,
  total_time: 35,
  ingredients: [
    {
      name: 'Tomatoes',
      quantity: 400,
      unit: 'G',
      category: 'FruitsAndVegetables',
      quantity_label: '400 g',
      key: 'Tomatoes-G',
    },
    { name: 'Eggs', quantity: 4, unit: null, category: 'DairyAndEggs', quantity_label: '4', key: 'Eggs-' },
  ],
  instructions: [
    { description: 'Simmer.', time_next: 10 },
    { description: 'Crack.', time_next: 0 },
    { description: 'Cover.', time_next: 5 },
  ],
  dietary_restrictions: [],
  accepts_accompaniment: true,
  advance_prep: '',
  is_shared: false,
  difficulty_score: 3,
  created_at: 0,
  thumbnail_url: null,
  blur_placeholder: null,
}

const groceries: Groceries = {
  recipe_count: 1,
  total_items: 2,
  checked_items: 0,
  progress_pct: 0,
  aisles: [
    {
      key: 'shopping_DairyAndEggs',
      items: [{ ...recipe.ingredients[1]!, checked: false }],
      checked: 0,
      total: 1,
      done: false,
      pct: 0,
    },
    {
      key: 'shopping_FruitsAndVegetables',
      items: [{ ...recipe.ingredients[0]!, checked: false }],
      checked: 0,
      total: 1,
      done: false,
      pct: 0,
    },
  ],
}

const entry = (id: string, status: Status) => ({
  id,
  name: id,
  slug: id,
  recipe_type: 'MainCourse' as const,
  status,
  advance_prep: '',
  prep_time: 1,
  cook_time: 1,
  total_time: 2,
  thumbnail_url: null,
  blur_placeholder: null,
})

const kitchen: KitchenList = {
  kind: 'list',
  entries: [entry('r1', { status: 'idle', step: null }), entry('r2', { status: 'completed', step: null })],
  focused: recipe,
  focused_status: { status: 'idle', step: null },
  completed_count: 1,
  total_count: 2,
  prep_ahead: [],
  steps: { completed: [], coming: [], current: null },
  cook_external: false,
  external_url: null,
}

const screen: CookingScreen = {
  recipe,
  status: { status: 'idle', step: null },
  steps: { completed: [], coming: [], current: null },
  origin_embeddable: false,
  show_ingredients: true,
  ingredient_aisles: [{ key: 'shopping_DairyAndEggs', items: [recipe.ingredients[1]!] }],
  external_url: null,
}

const detail: Detail = {
  ...recipe,
  is_owner: false,
  saved: false,
  in_shopping: false,
  owner_description: '',
  owner_stat: { total: 1, favorite: 0, shared: 1, from_community: 0 },
}

describe('grocery op', () => {
  it('checks one item and recounts like the server', () => {
    const op = { kind: 'grocery', key: 'Eggs-', checked: true } as const
    expect(logicalKey(op)).toBe('grocery:Eggs-')
    expect(affects(op)).toEqual(['groceries'])
    const next = apply(op, 'groceries', groceries)
    expect(next.checked_items).toBe(1)
    expect(next.progress_pct).toBe(50)
    expect(next.aisles[0]).toMatchObject({ checked: 1, done: true, pct: 100 })
    expect(next.aisles[0]!.items[0]!.checked).toBe(true)
    // Already in that state: unchanged.
    expect(apply(op, 'groceries', next)).toBe(next)
    // Other documents pass through.
    expect(apply(op, 'kitchen', kitchen)).toBe(kitchen)
  })
})

describe('status op', () => {
  const op = { kind: 'status', id: 'r1', status: { status: 'cooking', step: 1 } } as const

  it('moves the cooking screen to the step', () => {
    const next = apply(op, 'cooking:r1', screen)
    expect(next.status).toEqual({ status: 'cooking', step: 1 })
    expect(next.show_ingredients).toBe(false)
    expect(next.ingredient_aisles).toEqual([])
    expect(next.steps.current).toEqual({ index: 1, description: 'Crack.', time_next: 0 })
    expect(next.steps.completed).toHaveLength(1)
    expect(next.steps.coming).toHaveLength(1)
  })

  it('rebuilds the ingredient aisles when going back to idle on a cached screen', () => {
    const cookingScreen = apply(op, 'cooking:r1', screen)
    const back = apply(
      { kind: 'status', id: 'r1', status: { status: 'idle', step: null } },
      'cooking:r1',
      cookingScreen
    )
    expect(back.show_ingredients).toBe(true)
    expect(back.ingredient_aisles.map((a) => a.key)).toEqual(['shopping_DairyAndEggs', 'shopping_FruitsAndVegetables'])
  })

  it('updates the kitchen entry, focus and completed count', () => {
    const done = { kind: 'status', id: 'r1', status: { status: 'completed', step: null } } as const
    const next = apply(done, 'kitchen', kitchen as Overview) as KitchenList
    expect(next.entries[0]!.status.status).toBe('completed')
    expect(next.focused_status.status).toBe('completed')
    expect(next.completed_count).toBe(2)
    expect(apply(done, 'kitchen', { kind: 'onboarding_recipe' } as Overview)).toEqual({ kind: 'onboarding_recipe' })
    expect(apply(done, 'dish:r1', { status: screen.status } as Dish).status.status).toBe('completed')
  })
})

describe('saved and list ops', () => {
  it('flags the recipe detail', () => {
    expect(apply({ kind: 'saved', id: 'r1', saved: true }, 'recipe:r1', detail).saved).toBe(true)
    expect(apply({ kind: 'list', id: 'r1', in_list: true }, 'recipe:r1', detail).in_shopping).toBe(true)
  })

  it('removes the entry from the kitchen at once, adding waits for the server', () => {
    const removed = apply({ kind: 'list', id: 'r1', in_list: false }, 'kitchen', kitchen as Overview) as KitchenList
    expect(removed.entries.map((e) => e.id)).toEqual(['r2'])
    expect(removed.total_count).toBe(1)
    expect(removed.focused).toBeNull()
    expect(apply({ kind: 'list', id: 'r3', in_list: true }, 'kitchen', kitchen as Overview)).toBe(kitchen)
    expect(affects({ kind: 'list', id: 'r1', in_list: false })).toEqual(['recipe:r1', 'kitchen', 'groceries'])
  })
})
