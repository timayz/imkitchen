import { describe, expect, it } from 'vitest'

import type { Entry } from './api/kitchen.js'
import type { Status } from './api/recipe.js'
import { cookLabel, dishTime, doneCount, isCooking, isDone, items } from './kitchen-items.js'

const idle: Status = { status: 'idle', step: null }
const cooking: Status = { status: 'cooking', step: 1 }
const done: Status = { status: 'completed', step: null }

const entry = (id: string, status: Status, extra: Partial<Entry> = {}): Entry => ({
  id,
  name: id,
  slug: id,
  recipe_type: 'MainCourse',
  status,
  advance_prep: '',
  prep_time: 5,
  cook_time: 10,
  total_time: 15,
  thumbnail_url: null,
  blur_placeholder: null,
  ...extra,
})

describe('items', () => {
  it('treats a document without dish ids as plain recipe rows', () => {
    const rows = items({ entries: [entry('a', idle), entry('b', done)] })
    expect(rows.map((r) => r.kind)).toEqual(['recipe', 'recipe'])
  })

  it('collapses a dish into one row at its first recipe, named from the groups', () => {
    const rows = items({
      entries: [
        entry('starter', idle),
        entry('main', idle, { dish_id: 'd1' }),
        entry('side', idle, { dish_id: 'd1', recipe_type: 'Accompaniment', total_time: 40 }),
        entry('dessert', idle),
        entry('other', idle, { dish_id: 'd2', recipe_type: 'Accompaniment' }),
        entry('other-main', idle, { dish_id: 'd2' }),
      ],
      dishes: [{ id: 'd1', name: 'Sunday roast' }],
    })
    expect(rows.map((r) => (r.kind === 'dish' ? `dish:${r.id}` : r.entry.id))).toEqual([
      'starter',
      'dish:d1',
      'dessert',
      'dish:d2',
    ])
    const roast = rows[1]!
    if (roast.kind !== 'dish') throw new Error('expected a dish row')
    expect(roast.name).toBe('Sunday roast')
    expect(roast.entries.map((e) => e.id)).toEqual(['main', 'side'])
    expect(dishTime(roast.entries)).toBe(40)
    // No name in the groups: the main course names the row.
    const other = rows[3]!
    if (other.kind !== 'dish') throw new Error('expected a dish row')
    expect(other.name).toBe('other-main')
  })
})

describe('status helpers', () => {
  it('a dish is done when every recipe is, cooking when any is', () => {
    const half = items({ entries: [entry('a', done, { dish_id: 'd' }), entry('b', cooking, { dish_id: 'd' })] })[0]!
    expect(isDone(half)).toBe(false)
    expect(isCooking(half)).toBe(true)
    if (half.kind === 'dish') expect(doneCount(half.entries)).toBe(1)
    const all = items({ entries: [entry('a', done, { dish_id: 'd' }), entry('b', done, { dish_id: 'd' })] })[0]!
    expect(isDone(all)).toBe(true)
    expect(isCooking(all)).toBe(false)
  })

  it('labels the cook button by status', () => {
    expect(cookLabel(idle)).toBe('kitchen.start')
    expect(cookLabel(cooking)).toBe('kitchen.continue')
    expect(cookLabel(done)).toBe('kitchen.again')
  })
})
