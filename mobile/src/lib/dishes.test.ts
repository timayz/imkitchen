import { describe, expect, it } from 'vitest'

import type { Candidate } from './api/dishes.js'
import { MEAL_ORDER, isMultiple, problem, toggle } from './dishes.js'

const c = (id: string, recipe_type: Candidate['recipe_type']): Candidate => ({
  id,
  name: id,
  recipe_type,
  accepts_accompaniment: false,
})

const starter = c('starter', 'Appetizer')
const starter2 = c('starter-2', 'Appetizer')
const main = c('main', 'MainCourse')
const main2 = c('main-2', 'MainCourse')
const side1 = c('side-1', 'Accompaniment')
const side2 = c('side-2', 'Accompaniment')
const sauce = c('sauce', 'Condiment')

const ids = (list: Candidate[]) => list.map((x) => x.id)

describe('courses', () => {
  it('lists every course in meal order, mains first, sides and sauces repeatable', () => {
    expect(MEAL_ORDER).toEqual(['MainCourse', 'Appetizer', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment'])
    expect(MEAL_ORDER.filter(isMultiple)).toEqual(['Accompaniment', 'Condiment'])
  })
})

describe('toggle', () => {
  it('accumulates sides and sauces', () => {
    let selected = toggle([], side1)
    selected = toggle(selected, side2)
    selected = toggle(selected, sauce)
    expect(ids(selected)).toEqual(['side-1', 'side-2', 'sauce'])
  })

  it('replaces the previous pick of a single-choice course', () => {
    let selected = toggle([side1], main)
    expect(ids(selected)).toEqual(['side-1', 'main'])
    selected = toggle(selected, main2)
    expect(ids(selected)).toEqual(['side-1', 'main-2'])
    selected = toggle(selected, starter)
    selected = toggle(selected, starter2)
    expect(ids(selected)).toEqual(['side-1', 'main-2', 'starter-2'])
  })

  it('deselects on a second tap', () => {
    expect(ids(toggle([main, side1], main))).toEqual(['side-1'])
  })
})

describe('problem', () => {
  it('needs exactly one main course', () => {
    expect(problem([])).toBe('no_main')
    expect(problem([side1])).toBe('no_main')
    expect(problem([main, side1])).toBeNull()
    // Never reachable through `toggle`, but the rule still holds.
    expect(problem([main, main2])).toBe('no_main')
  })
})
