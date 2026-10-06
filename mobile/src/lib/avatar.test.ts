import { describe, expect, it } from 'vitest'

import { avatar, initials } from './avatar.js'

describe('initials', () => {
  it('takes the first letter of the first and last word', () => {
    expect(initials('marie_lefevre')).toBe('ML')
    expect(initials('theo b')).toBe('TB')
  })
  it('uses a single letter for a one-word name', () => {
    expect(initials('pastrychef')).toBe('P')
  })
  it('is empty for an empty name', () => {
    expect(initials('')).toBe('')
  })
})

describe('avatar', () => {
  it('is stable for the same username', () => {
    expect(avatar('pastrychef')).toEqual(avatar('pastrychef'))
  })
  it('hashes like the web: the sum of the code points modulo the palette', () => {
    // 'a' = 97 → 97 % 17 = 12 → violet
    expect(avatar('a').bg).toBe('#8b5cf6')
    // 'ab' = 97 + 98 = 195 → 195 % 17 = 8 → cyan, dark text
    expect(avatar('ab')).toEqual({ initials: 'A', bg: '#06b6d4', fg: '#000000' })
  })
})
