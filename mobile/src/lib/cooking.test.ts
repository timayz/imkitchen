import { describe, expect, it } from 'vitest'

import type { Instruction, Status } from './api/recipe.js'
import { groupByAisle, nextStatus, splitInstructions } from './cooking.js'

const idle: Status = { status: 'idle', step: null }
const cooking = (step: number): Status => ({ status: 'cooking', step })
const completed: Status = { status: 'completed', step: null }

// Mirrors the `next_status` tests in web/shared/src/services/kitchen.rs.
describe('nextStatus', () => {
  it('walks forward through steps to completed', () => {
    expect(nextStatus('next', idle, 3)).toEqual(cooking(0))
    expect(nextStatus('next', cooking(0), 3)).toEqual(cooking(1))
    expect(nextStatus('next', cooking(1), 3)).toEqual(completed)
    expect(nextStatus('next', completed, 3)).toEqual(completed)
  })

  it('walks back from completed to idle', () => {
    expect(nextStatus('prev', completed, 3)).toEqual(cooking(1))
    expect(nextStatus('prev', cooking(1), 3)).toEqual(cooking(0))
    expect(nextStatus('prev', cooking(0), 3)).toEqual(idle)
    expect(nextStatus('prev', idle, 3)).toEqual(idle)
  })

  it('single-step recipes skip cooking', () => {
    expect(nextStatus('next', idle, 1)).toEqual(completed)
    expect(nextStatus('prev', completed, 1)).toEqual(idle)
    expect(nextStatus('next', idle, 0)).toEqual(completed)
  })
})

describe('splitInstructions', () => {
  const steps: Instruction[] = [
    { description: 'Simmer.', time_next: 10 },
    { description: 'Crack.', time_next: 0 },
    { description: 'Cover.', time_next: 5 },
  ]

  it('shows nothing while idle (the ingredient screen)', () => {
    expect(splitInstructions(steps, idle)).toEqual({ completed: [], coming: [], current: null })
  })

  it('splits around the cooking cursor', () => {
    expect(splitInstructions(steps, cooking(1))).toEqual({
      completed: [{ index: 0, text: 'Simmer.' }],
      coming: [{ index: 2, text: 'Cover.' }],
      current: { index: 1, description: 'Crack.', time_next: 0 },
    })
  })

  it('keeps the last step current once completed', () => {
    expect(splitInstructions(steps, completed)).toEqual({
      completed: [
        { index: 0, text: 'Simmer.' },
        { index: 1, text: 'Crack.' },
      ],
      coming: [],
      current: { index: 2, description: 'Cover.', time_next: 5 },
    })
    expect(splitInstructions([], completed).current).toBeNull()
  })
})

describe('groupByAisle', () => {
  it('groups by shopping_<Category>, sorted, unknown last', () => {
    const base = { quantity: 1, unit: null, quantity_label: '1' }
    const aisles = groupByAisle([
      { ...base, name: 'Eggs', category: 'DairyAndEggs', key: 'Eggs-' },
      { ...base, name: 'Salt', category: null, key: 'Salt-' },
      { ...base, name: 'Butter', category: 'DairyAndEggs', key: 'Butter-' },
      { ...base, name: 'Bread', category: 'Bakery', key: 'Bread-' },
    ])
    expect(aisles.map((a) => a.key)).toEqual(['shopping_Bakery', 'shopping_DairyAndEggs', 'shopping_Unknown'])
    expect(aisles[1]!.items.map((i) => i.name)).toEqual(['Eggs', 'Butter'])
  })
})
