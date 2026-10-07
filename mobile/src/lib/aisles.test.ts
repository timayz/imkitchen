import { describe, expect, it } from 'vitest'

import { completeAisleOrder, moveAisle } from './aisles.js'
import { AISLE_CATEGORIES } from './course.js'

describe('moveAisle', () => {
  const order = ['a', 'b', 'c']

  it('moves an entry up', () => {
    expect(moveAisle(order, 1, -1)).toEqual(['b', 'a', 'c'])
  })

  it('moves an entry down', () => {
    expect(moveAisle(order, 1, 1)).toEqual(['a', 'c', 'b'])
  })

  it('leaves the edges alone', () => {
    expect(moveAisle(order, 0, -1)).toBe(order)
    expect(moveAisle(order, 2, 1)).toBe(order)
    expect(moveAisle(order, 5, 1)).toBe(order)
  })

  it('does not mutate the input', () => {
    moveAisle(order, 0, 1)
    expect(order).toEqual(['a', 'b', 'c'])
  })
})

describe('completeAisleOrder', () => {
  it('keeps the given order first and appends the missing aisles in default order', () => {
    const out = completeAisleOrder(['Bakery', 'Frozen', 'Bakery'])
    expect(out.slice(0, 2)).toEqual(['Bakery', 'Frozen'])
    expect(out).toHaveLength(AISLE_CATEGORIES.length)
    expect(out.slice(2)).toEqual(AISLE_CATEGORIES.filter((c) => c !== 'Bakery' && c !== 'Frozen'))
  })

  it('is the default for an empty order', () => {
    expect(completeAisleOrder([])).toEqual([...AISLE_CATEGORIES])
  })

  it('drops names the app does not know', () => {
    expect(completeAisleOrder(['Nope', 'Seafood'])[0]).toBe('Seafood')
    expect(completeAisleOrder(['Nope'])).not.toContain('Nope')
  })
})
