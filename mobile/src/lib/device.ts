/**
 * Human-readable identity for a login's `User-Agent`.
 *
 * The app's own installs send `imkitchen-<os> (<OS>; <model>; <install id>)`
 * (see client-identity.ts); everything else is a browser on the website.
 */
export type Device =
  | { kind: 'app'; os: 'iOS' | 'Android'; model: string }
  | { kind: 'browser'; browser: string; os: string; mobile: boolean }
  | { kind: 'unknown' }

const APP = /^imkitchen-(android|ios)\s*\(\s*([^;]*);\s*([^;]*);/i

export function parseDevice(ua: string): Device {
  const app = APP.exec(ua)
  if (app) {
    const os = app[1].toLowerCase() === 'ios' ? 'iOS' : 'Android'
    const model = app[3].trim()
    return { kind: 'app', os, model: model && model !== 'unknown' ? model : os === 'iOS' ? 'iPhone' : 'Android' }
  }

  const os = browserOs(ua)
  const browser = browserName(ua)
  if (!os && !browser) return { kind: 'unknown' }
  return {
    kind: 'browser',
    browser: browser ?? 'Browser',
    os: os ?? '',
    mobile: /iPhone|iPad|Android|Mobile/i.test(ua),
  }
}

function browserOs(ua: string): string | null {
  if (/iPhone/.test(ua)) return 'iPhone'
  if (/iPad/.test(ua)) return 'iPad'
  if (/Android/.test(ua)) return 'Android'
  if (/CrOS/.test(ua)) return 'ChromeOS'
  if (/Windows/.test(ua)) return 'Windows'
  if (/Mac OS X|Macintosh/.test(ua)) return 'macOS'
  if (/Linux|X11/.test(ua)) return 'Linux'
  return null
}

function browserName(ua: string): string | null {
  if (/Edg\//.test(ua)) return 'Edge'
  if (/OPR\/|Opera/.test(ua)) return 'Opera'
  if (/SamsungBrowser/.test(ua)) return 'Samsung Internet'
  if (/Firefox\//.test(ua)) return 'Firefox'
  if (/Chrome\/|CriOS\//.test(ua)) return 'Chrome'
  if (/Safari\//.test(ua)) return 'Safari'
  return null
}
