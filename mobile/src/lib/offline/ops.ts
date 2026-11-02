import type { Groceries } from '../api/groceries.js'
import type { CookingScreen, Dish, Entry, Overview } from '../api/kitchen.js'
import type { Status } from '../api/recipe.js'
import type { Detail } from '../api/recipes.js'
import { applyStatus } from '../cooking.js'
import { setChecked } from '../groceries.js'

/**
 * The changes a user can make while offline, each absolute so that sending
 * it twice is harmless and so that it can be re-applied on top of whatever
 * the server returns later.
 */
export type Op =
  | { kind: 'grocery'; key: string; checked: boolean }
  | { kind: 'status'; id: string; status: Status }
  | { kind: 'saved'; id: string; saved: boolean }
  /** `household_size` is the people count chosen when adding; absent = settings value. */
  | { kind: 'list'; id: string; in_list: boolean; household_size?: number }
  /** Takes a whole composed dish (every entry with that `dish_id`) off the list. */
  | { kind: 'dish_remove'; id: string }

/** Two ops with the same logical key replace each other (last one wins). */
export function logicalKey(op: Op): string {
  switch (op.kind) {
    case 'grocery':
      return `grocery:${op.key}`
    case 'status':
      return `status:${op.id}`
    case 'saved':
      return `saved:${op.id}`
    case 'list':
      return `list:${op.id}`
    case 'dish_remove':
      return `dish_remove:${op.id}`
  }
}

/** The cached documents an op changes. */
export function affects(op: Op): string[] {
  switch (op.kind) {
    case 'grocery':
      return ['groceries']
    case 'status':
      return [`cooking:${op.id}`, 'kitchen', `dish:${op.id}`]
    case 'saved':
      return [`recipe:${op.id}`]
    case 'list':
      return [`recipe:${op.id}`, 'kitchen', 'groceries']
    case 'dish_remove':
      return ['kitchen', 'groceries']
  }
}

/** Pure: `doc` (stored under `docKey`) after `op`. Unknown shapes pass through. */
export function apply<T>(op: Op, docKey: string, doc: T): T {
  switch (op.kind) {
    case 'grocery':
      if (docKey === 'groceries') return setChecked(doc as Groceries, op.key, op.checked) as T
      return doc
    case 'status': {
      if (docKey === `cooking:${op.id}`) return applyStatus(doc as CookingScreen, op.status) as T
      if (docKey === 'kitchen') return kitchenStatus(doc as Overview, op.id, op.status) as T
      if (docKey === `dish:${op.id}`) return { ...(doc as Dish), status: op.status } as T
      return doc
    }
    case 'saved':
      if (docKey === `recipe:${op.id}`) return { ...(doc as Detail), saved: op.saved } as T
      return doc
    case 'list': {
      if (docKey === `recipe:${op.id}`) return { ...(doc as Detail), in_shopping: op.in_list } as T
      // Adding shows up on the next refresh (the server builds the entry);
      // removing can be reflected at once.
      if (docKey === 'kitchen' && !op.in_list) return kitchenRemove(doc as Overview, op.id) as T
      return doc
    }
    case 'dish_remove':
      if (docKey === 'kitchen') return kitchenRemoveDish(doc as Overview, op.id) as T
      return doc
  }
}

function kitchenStatus(overview: Overview, id: string, status: Status): Overview {
  if (overview.kind !== 'list') return overview
  const entries = overview.entries.map((e) => (e.id === id ? { ...e, status } : e))
  return {
    ...overview,
    entries,
    focused_status: overview.focused?.id === id ? status : overview.focused_status,
    completed_count: entries.filter((e) => e.status.status === 'completed').length,
  }
}

function kitchenRemove(overview: Overview, id: string): Overview {
  return kitchenWithout(overview, (e) => e.id === id)
}

function kitchenRemoveDish(overview: Overview, dishId: string): Overview {
  if (overview.kind !== 'list') return overview
  const next = kitchenWithout(overview, (e) => e.dish_id === dishId)
  return next.kind === 'list' ? { ...next, dishes: (next.dishes ?? []).filter((d) => d.id !== dishId) } : next
}

/** The list without the entries matching `gone`; the focus is reset when it was one of them. */
function kitchenWithout(overview: Overview, gone: (e: Entry) => boolean): Overview {
  if (overview.kind !== 'list') return overview
  const removed = new Set(overview.entries.filter(gone).map((e) => e.id))
  const entries = overview.entries.filter((e) => !removed.has(e.id))
  const focusedGone = overview.focused !== null && removed.has(overview.focused.id)
  return {
    ...overview,
    entries,
    prep_ahead: overview.prep_ahead.filter((e) => !removed.has(e.id)),
    total_count: entries.length,
    completed_count: entries.filter((e) => e.status.status === 'completed').length,
    // The next focus is the server's call; until then show the list without one.
    focused: focusedGone ? null : overview.focused,
    focused_status: focusedGone ? { status: 'idle', step: null } : overview.focused_status,
    steps: focusedGone ? { completed: [], coming: [], current: null } : overview.steps,
  }
}
