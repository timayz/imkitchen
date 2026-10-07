import { clearDocs } from '../cache.js'
import { db } from '../db.js'
import { storageGet, storageRemove, storageSet } from '../storage.js'

export interface Session {
  token: string
  /** RFC 3339 */
  expiresAt: string
}

const TOKEN_KEY = 'auth.token'
const EXPIRES_KEY = 'auth.expires_at'

let current: Session | null | undefined

/** Loads the persisted session once; later calls are in-memory. */
export async function loadSession(): Promise<Session | null> {
  if (current !== undefined) return current
  const [token, expiresAt] = await Promise.all([storageGet<string>(TOKEN_KEY), storageGet<string>(EXPIRES_KEY)])
  current = token && expiresAt ? { token, expiresAt } : null
  return current
}

export function currentSession(): Session | null {
  return current ?? null
}

export async function saveSession(session: Session): Promise<void> {
  current = session
  // Another account may sign in on this phone: never show it the previous
  // one's cached data or queued changes.
  clearDocs()
  await db.clear()
  await Promise.all([storageSet(TOKEN_KEY, session.token), storageSet(EXPIRES_KEY, session.expiresAt)])
}

/** Forgets the token and wipes the local store (cache and offline queue). */
export async function clearSession(): Promise<void> {
  current = null
  clearDocs()
  await Promise.all([storageRemove(TOKEN_KEY), storageRemove(EXPIRES_KEY), db.clear()])
}

/** True when the token expires within `days`. */
export function expiresWithin(session: Session, days: number): boolean {
  const at = Date.parse(session.expiresAt)
  return Number.isNaN(at) || at - Date.now() < days * 24 * 60 * 60 * 1000
}
