import { db } from './db.js'

/**
 * Cached API responses, one JSON document per key (`kitchen`, `groceries`,
 * `recipe:<id>`…), in the local store so they survive navigation between
 * bundles and app restarts. Screens read the document first and revalidate
 * it from the network; see `useResource`.
 */
const COLLECTION = 'docs'

// Per-runtime copy of what was last read or written: a screen that mounts
// again (tab switch) renders its first frame from it instead of waiting on
// the bridge. Reads still go to the store, which other bundles may have
// written to since (the cooking screen patches the kitchen document).
const memory = new Map<string, unknown>()

type Listener = (doc: unknown) => void
const listeners = new Map<string, Set<Listener>>()

type RebaseHook = <T>(key: string, fresh: T) => Promise<T>
// Identity until the offline queue installs itself: it re-applies the
// changes still waiting to be sent on top of what the server returned.
let rebaseHook: RebaseHook = async (_key, fresh) => fresh

export function setRebaseHook(hook: RebaseHook | null): void {
  rebaseHook = hook ?? (async (_key, fresh) => fresh)
}

/** The document already seen by this runtime, without touching the store. */
export function peekDoc<T>(key: string): T | null {
  return memory.has(key) ? (memory.get(key) as T) : null
}

export async function readDoc<T>(key: string): Promise<T | null> {
  const raw = await db.get(COLLECTION, key)
  if (raw === null) {
    memory.delete(key)
    return null
  }
  try {
    const doc = JSON.parse(raw) as T
    memory.set(key, doc)
    return doc
  } catch {
    return null
  }
}

/** Stores `value` and notifies this runtime's subscribers of `key`. */
export async function writeDoc<T>(key: string, value: T): Promise<void> {
  memory.set(key, value)
  await db.put(COLLECTION, key, JSON.stringify(value))
  emit(key, value)
}

export async function removeDoc(key: string): Promise<void> {
  memory.delete(key)
  await db.remove(COLLECTION, key)
}

/** Forgets every document in this runtime (sign-out wipes the store itself). */
export function clearDocs(): void {
  memory.clear()
}

/** Applies `fn` to the stored document, if any, and stores the result. */
export async function patchDoc<T>(key: string, fn: (doc: T) => T): Promise<T | null> {
  const doc = await readDoc<T>(key)
  if (doc === null) return null
  const next = fn(doc)
  await writeDoc(key, next)
  return next
}

export function subscribeDoc<T>(key: string, listener: (doc: T) => void): () => void {
  let set = listeners.get(key)
  if (!set) {
    set = new Set()
    listeners.set(key, set)
  }
  const wrapped: Listener = (doc) => listener(doc as T)
  set.add(wrapped)
  return () => {
    set.delete(wrapped)
    if (set.size === 0) listeners.delete(key)
  }
}

function emit(key: string, doc: unknown): void {
  listeners.get(key)?.forEach((listener) => listener(doc))
}

/**
 * Fetches, folds the pending offline changes into the result, stores it and
 * returns it. Every write of server data goes through here so a background
 * refresh never undoes a change that is still waiting to be sent.
 */
export async function revalidate<T>(key: string, fetcher: () => Promise<T>): Promise<T> {
  const fresh = await fetcher()
  const rebased = await rebaseHook(key, fresh)
  await writeDoc(key, rebased)
  return rebased
}
