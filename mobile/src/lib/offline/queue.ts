import { ApiError, isOffline } from '../api/client.js'
import { getGroceries, setGroceryChecked } from '../api/groceries.js'
import { getCooking, getDish, getKitchen, removeFromList, setStatus } from '../api/kitchen.js'
import { addRecipeToList, getRecipe, saveRecipe, unsaveRecipe } from '../api/recipes.js'
import { patchDoc, revalidate, setRebaseHook, writeDoc } from '../cache.js'
import { db } from '../db.js'
import { setOffline } from './status.js'
import { type Op, affects, apply, logicalKey } from './ops.js'

/**
 * The offline write queue: kitchen changes are applied to the cached
 * documents at once, stored here, and sent in order as soon as the server
 * can be reached. Each bundle has its own runtime; the queue lives in the
 * local store so any of them can drain it.
 */
const COLLECTION = 'queue'
const MAX_ATTEMPTS = 5

export interface QueueEntry {
  /** `<created_at>-<logicalKey>`: sorts in arrival order. */
  key: string
  op: Op
  created_at: number
  attempts: number
}

type PendingListener = (count: number) => void
const pendingListeners = new Set<PendingListener>()
const inFlight = new Set<string>()
let draining = false
let drainAgain = false

export function subscribePending(listener: PendingListener): () => void {
  pendingListeners.add(listener)
  return () => {
    pendingListeners.delete(listener)
  }
}

async function readAll(): Promise<QueueEntry[]> {
  const rows = await db.list(COLLECTION)
  const entries: QueueEntry[] = []
  rows.forEach((row) => {
    try {
      entries.push(JSON.parse(row.value) as QueueEntry)
    } catch {
      // A corrupt row is dropped on the next clear.
    }
  })
  return entries
}

async function store(entry: QueueEntry): Promise<void> {
  await db.put(COLLECTION, entry.key, JSON.stringify(entry))
}

async function notifyPending(): Promise<void> {
  const count = await pendingCount()
  pendingListeners.forEach((listener) => listener(count))
}

export async function pendingCount(): Promise<number> {
  return (await db.list(COLLECTION)).length
}

/**
 * Applies `op` to the cached documents, stores it and tries to send it.
 * An op on the same logical key as one still waiting replaces it in place,
 * so flipping a grocery twice sends one request with the final state.
 */
export async function enqueue(op: Op): Promise<void> {
  const docKeys = affects(op)
  for (let i = 0; i < docKeys.length; i++) {
    await patchDoc(docKeys[i]!, (doc: unknown) => apply(op, docKeys[i]!, doc))
  }

  const logical = logicalKey(op)
  const entries = await readAll()
  const existing = entries.find((e) => logicalKey(e.op) === logical && !inFlight.has(e.key))
  if (existing) {
    await store({ ...existing, op, attempts: 0 })
  } else {
    const created_at = Date.now()
    await store({ key: `${String(created_at).padStart(13, '0')}-${logical}`, op, created_at, attempts: 0 })
  }
  await notifyPending()
  void drain()
}

function send(op: Op): Promise<unknown> {
  switch (op.kind) {
    case 'grocery':
      return setGroceryChecked(op.key, op.checked)
    case 'status': {
      // Through `rebase`: a later step may already be waiting behind this one.
      const key = `cooking:${op.id}`
      return setStatus(op.id, op.status).then(async (screen) => writeDoc(key, await rebase(key, screen)))
    }
    case 'saved':
      return op.saved ? saveRecipe(op.id) : unsaveRecipe(op.id)
    case 'list':
      return op.in_list ? addRecipeToList(op.id) : removeFromList(op.id)
  }
}

/** Whether a rejected op is settled on the server anyway. */
function settled(op: Op, err: ApiError): boolean {
  return op.kind === 'list' && !op.in_list && err.status === 404
}

const fetchers: Record<string, (id: string) => Promise<unknown>> = {
  kitchen: () => getKitchen(),
  groceries: () => getGroceries(),
  cooking: (id) => getCooking(id),
  recipe: (id) => getRecipe(id),
  dish: (id) => getDish(id),
}

async function refreshDocs(keys: Set<string>): Promise<void> {
  const list = Array.from(keys)
  for (let i = 0; i < list.length; i++) {
    const key = list[i]!
    const colon = key.indexOf(':')
    const fetcher = fetchers[colon === -1 ? key : key.slice(0, colon)]
    if (!fetcher) continue
    try {
      await revalidate(key, () => fetcher(key.slice(colon + 1)))
    } catch {
      // Best effort: the screen revalidates on its own next show.
    }
  }
}

/**
 * Sends the waiting ops in order. Stops at the first unreachable server
 * (they stay queued); drops an op the server rejects (4xx) and refreshes
 * what it touched so the screens show the server's truth; gives up on an
 * op after `MAX_ATTEMPTS` server errors.
 */
export async function drain(): Promise<void> {
  if (draining) {
    drainAgain = true
    return
  }
  draining = true
  const touched = new Set<string>()
  try {
    const entries = await readAll()
    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i]!
      inFlight.add(entry.key)
      try {
        await send(entry.op)
        await db.remove(COLLECTION, entry.key)
        affects(entry.op).forEach((k) => touched.add(k))
      } catch (err) {
        if (isOffline(err)) {
          setOffline(true)
          break
        }
        if (err instanceof ApiError && err.status < 500) {
          await db.remove(COLLECTION, entry.key)
          if (!settled(entry.op, err)) affects(entry.op).forEach((k) => touched.add(k))
          continue
        }
        const attempts = entry.attempts + 1
        if (attempts >= MAX_ATTEMPTS) {
          await db.remove(COLLECTION, entry.key)
          affects(entry.op).forEach((k) => touched.add(k))
        } else {
          await store({ ...entry, attempts })
        }
        break
      } finally {
        inFlight.delete(entry.key)
      }
    }
  } finally {
    draining = false
  }
  await notifyPending()
  if (touched.size > 0) await refreshDocs(touched)
  if (drainAgain) {
    drainAgain = false
    await drain()
  }
}

/**
 * Re-applies the ops still waiting on top of `fresh` server data for
 * `docKey`, so a refresh never undoes a change that has not been sent yet.
 */
export async function rebase<T>(docKey: string, fresh: T): Promise<T> {
  const entries = await readAll()
  let doc = fresh
  for (let i = 0; i < entries.length; i++) {
    const op = entries[i]!.op
    if (affects(op).includes(docKey)) doc = apply(op, docKey, doc)
  }
  return doc
}

/** Routes every cache write of server data through `rebase`. Idempotent. */
export function installQueue(): void {
  setRebaseHook(rebase)
}
