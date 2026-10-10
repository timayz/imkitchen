import { describe, expect, it } from 'vitest'

import { formatClock, remainingSeconds } from './countdown.js'

describe('remainingSeconds', () => {
  it('rounds a partial second up so the display never skips the first tick', () => {
    expect(remainingSeconds(10_000, 0)).toBe(10)
    expect(remainingSeconds(10_000, 1)).toBe(10)
    expect(remainingSeconds(10_000, 999)).toBe(10)
    expect(remainingSeconds(10_000, 1000)).toBe(9)
  })

  it('clamps at zero once the deadline has passed', () => {
    expect(remainingSeconds(10_000, 10_000)).toBe(0)
    expect(remainingSeconds(10_000, 60_000)).toBe(0)
  })
})

describe('formatClock', () => {
  it('pads minutes and seconds', () => {
    expect(formatClock(0)).toBe('00:00')
    expect(formatClock(5)).toBe('00:05')
    expect(formatClock(65)).toBe('01:05')
  })

  it('lets minutes grow past an hour and ignores fractions and negatives', () => {
    expect(formatClock(90 * 60)).toBe('90:00')
    expect(formatClock(59.9)).toBe('00:59')
    expect(formatClock(-3)).toBe('00:00')
  })
})
