import { describe, expect, it } from 'vitest'

import { parseDevice } from './device.js'

describe('parseDevice', () => {
  it('recognises the app’s own identity string', () => {
    expect(parseDevice('imkitchen-android (Android; Pixel 8; 01J9ABCDEFGHJKMNPQRSTVWXYZ)')).toEqual({
      kind: 'app',
      os: 'Android',
      model: 'Pixel 8',
    })
    expect(parseDevice('imkitchen-ios (iOS; unknown; 01J9ABCDEFGHJKMNPQRSTVWXYZ)')).toEqual({
      kind: 'app',
      os: 'iOS',
      model: 'iPhone',
    })
  })

  it('names desktop browsers', () => {
    expect(
      parseDevice(
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36'
      )
    ).toEqual({ kind: 'browser', browser: 'Chrome', os: 'macOS', mobile: false })
    expect(parseDevice('Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:130.0) Gecko/20100101 Firefox/130.0')).toEqual({
      kind: 'browser',
      browser: 'Firefox',
      os: 'Windows',
      mobile: false,
    })
  })

  it('names mobile browsers', () => {
    expect(
      parseDevice(
        'Mozilla/5.0 (iPhone; CPU iPhone OS 17_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.6 Mobile/15E148 Safari/604.1'
      )
    ).toEqual({ kind: 'browser', browser: 'Safari', os: 'iPhone', mobile: true })
  })

  it('gives up on gibberish', () => {
    expect(parseDevice('curl/8.4.0')).toEqual({ kind: 'unknown' })
  })
})
