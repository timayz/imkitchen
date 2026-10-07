/**
 * A cook's avatar, mirroring the web's `initials` and `bg_color` template
 * filters so the same username gets the same letters and color everywhere.
 */
export interface Avatar {
  initials: string
  bg: string
  fg: string
}

const WHITE = '#ffffff'
const BLACK = '#000000'

/** Tailwind's 500 shades, in the web filter's order. */
const COLORS: { bg: string; fg: string }[] = [
  { bg: '#ef4444', fg: WHITE }, // red
  { bg: '#f97316', fg: WHITE }, // orange
  { bg: '#f59e0b', fg: BLACK }, // amber
  { bg: '#eab308', fg: BLACK }, // yellow
  { bg: '#84cc16', fg: BLACK }, // lime
  { bg: '#22c55e', fg: WHITE }, // green
  { bg: '#10b981', fg: WHITE }, // emerald
  { bg: '#14b8a6', fg: WHITE }, // teal
  { bg: '#06b6d4', fg: BLACK }, // cyan
  { bg: '#0ea5e9', fg: WHITE }, // sky
  { bg: '#3b82f6', fg: WHITE }, // blue
  { bg: '#6366f1', fg: WHITE }, // indigo
  { bg: '#8b5cf6', fg: WHITE }, // violet
  { bg: '#a855f7', fg: WHITE }, // purple
  { bg: '#d946ef', fg: WHITE }, // fuchsia
  { bg: '#ec4899', fg: WHITE }, // pink
  { bg: '#f43f5e', fg: WHITE }, // rose
]

export function initials(username: string): string {
  const parts = username.split(/[_ ]/)
  const first = parts[0]?.charAt(0) ?? ''
  const last = parts.length > 1 ? (parts[parts.length - 1]?.charAt(0) ?? '') : ''
  return (first + last).toUpperCase()
}

export function avatar(username: string): Avatar {
  let hash = 0
  for (const c of username) hash = (hash + (c.codePointAt(0) ?? 0)) >>> 0
  const color = COLORS[hash % COLORS.length] ?? COLORS[0]
  return { initials: initials(username), ...color }
}
