import { afterEach, beforeEach, describe, expect, it } from 'vitest'

import { clearDocs, patchDoc, peekDoc, readDoc, revalidate, setRebaseHook, subscribeDoc, writeDoc } from './cache.js'
import { MemoryDb, setDbBackend } from './db.js'

describe('cache', () => {
  beforeEach(() => {
    clearDocs()
    setDbBackend(new MemoryDb())
  })
  afterEach(() => {
    setDbBackend(null)
    setRebaseHook(null)
  })

  it('round-trips documents as JSON and remembers them in this runtime', async () => {
    expect(await readDoc('kitchen')).toBeNull()
    expect(peekDoc('kitchen')).toBeNull()
    await writeDoc('kitchen', { kind: 'list', entries: [] })
    expect(await readDoc('kitchen')).toEqual({ kind: 'list', entries: [] })
    expect(peekDoc('kitchen')).toEqual({ kind: 'list', entries: [] })
    // Reads come from the store and refresh the memory copy.
    clearDocs()
    expect(peekDoc('kitchen')).toBeNull()
    expect(await readDoc('kitchen')).toEqual({ kind: 'list', entries: [] })
    expect(peekDoc('kitchen')).toEqual({ kind: 'list', entries: [] })
    setDbBackend(new MemoryDb())
    expect(await readDoc('kitchen')).toBeNull()
    expect(peekDoc('kitchen')).toBeNull()
  })

  it('patches a stored document and notifies subscribers', async () => {
    const seen: unknown[] = []
    const unsubscribe = subscribeDoc<{ n: number }>('doc', (doc) => seen.push(doc))
    expect(await patchDoc<{ n: number }>('doc', (d) => ({ n: d.n + 1 }))).toBeNull()
    await writeDoc('doc', { n: 1 })
    expect(await patchDoc<{ n: number }>('doc', (d) => ({ n: d.n + 1 }))).toEqual({ n: 2 })
    expect(seen).toEqual([{ n: 1 }, { n: 2 }])
    unsubscribe()
    await writeDoc('doc', { n: 3 })
    expect(seen).toHaveLength(2)
  })

  it('revalidate stores what the rebase hook returns', async () => {
    const calls: unknown[] = []
    setRebaseHook(async function <T>(key: string, fresh: T): Promise<T> {
      calls.push([key, fresh])
      return { ...(fresh as object), pending: true } as T
    })
    const result = await revalidate('groceries', async () => ({ checked_items: 0 }))
    expect(result).toEqual({ checked_items: 0, pending: true })
    expect(await readDoc('groceries')).toEqual({ checked_items: 0, pending: true })
    expect(calls).toEqual([['groceries', { checked_items: 0 }]])
  })

  it('revalidate propagates fetch failures and leaves the store alone', async () => {
    await writeDoc('kitchen', { old: true })
    await expect(revalidate('kitchen', async () => Promise.reject(new Error('boom')))).rejects.toThrow('boom')
    expect(await readDoc('kitchen')).toEqual({ old: true })
  })
})

describe('MemoryDb', () => {
  it('lists a collection by key with an optional prefix and clears by collection', async () => {
    const db = new MemoryDb()
    await db.put('queue', '2-b', 'B')
    await db.put('queue', '1-a', 'A')
    await db.put('docs', 'kitchen', 'K')
    expect((await db.list('queue')).map((r) => r.key)).toEqual(['1-a', '2-b'])
    expect((await db.list('queue', '2')).map((r) => r.value)).toEqual(['B'])
    await db.clear('queue')
    expect(await db.list('queue')).toEqual([])
    expect(await db.get('docs', 'kitchen')).toBe('K')
    await db.clear()
    expect(await db.get('docs', 'kitchen')).toBeNull()
  })
})
