import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

vi.hoisted(() => {
  Object.assign(globalThis, {
    __API_BASE_URL__: 'http://test',
    __APP_VERSION__: 'test',
    lynx: { __globalProps: { language: 'en' } },
  })
})

vi.mock('../api/groceries.js', () => ({ getGroceries: vi.fn(), setGroceryChecked: vi.fn() }))
vi.mock('../api/kitchen.js', () => ({
  getKitchen: vi.fn(),
  getDish: vi.fn(),
  getCooking: vi.fn(),
  removeFromList: vi.fn(),
  setStatus: vi.fn(),
}))
vi.mock('../api/recipes.js', () => ({
  getRecipe: vi.fn(),
  saveRecipe: vi.fn(),
  unsaveRecipe: vi.fn(),
  addRecipeToList: vi.fn(),
}))

import { ApiError, NetworkError } from '../api/client.js'
import { getGroceries, setGroceryChecked } from '../api/groceries.js'
import { getKitchen, removeFromList } from '../api/kitchen.js'
import { clearDocs, readDoc, setRebaseHook, writeDoc } from '../cache.js'
import { MemoryDb, setDbBackend } from '../db.js'
import { drain, enqueue, installQueue, pendingCount, rebase, subscribePending } from './queue.js'
import { isOfflineNow, setOffline } from './status.js'

const groceries = {
  recipe_count: 1,
  total_items: 1,
  checked_items: 0,
  progress_pct: 0,
  aisles: [
    {
      key: 'shopping_DairyAndEggs',
      items: [
        {
          name: 'Eggs',
          quantity: 4,
          unit: null,
          category: 'DairyAndEggs',
          quantity_label: '4',
          key: 'Eggs-',
          checked: false,
        },
      ],
      checked: 0,
      total: 1,
      done: false,
      pct: 0,
    },
  ],
}

const check = (checked: boolean) => ({ kind: 'grocery', key: 'Eggs-', checked }) as const

/** A promise settled from the outside, to hold a request in flight. */
function deferred<T>() {
  let resolve!: (v: T) => void
  let reject!: (e: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

describe('offline queue', () => {
  let db: MemoryDb

  beforeEach(async () => {
    clearDocs()
    db = new MemoryDb()
    setDbBackend(db)
    setOffline(false)
    vi.mocked(setGroceryChecked).mockReset()
    vi.mocked(getGroceries).mockReset()
    vi.mocked(getKitchen).mockReset()
    vi.mocked(removeFromList).mockReset()
    vi.mocked(getGroceries).mockResolvedValue(groceries)
    await writeDoc('groceries', groceries)
  })
  afterEach(() => {
    setDbBackend(null)
    setRebaseHook(null)
  })

  it('patches the cached document and sends the change at once', async () => {
    vi.mocked(setGroceryChecked).mockResolvedValue(undefined)
    const counts: number[] = []
    const unsubscribe = subscribePending((n) => counts.push(n))
    await enqueue(check(true))
    expect((await readDoc<typeof groceries>('groceries'))!.checked_items).toBe(1)
    // Let the immediate drain finish.
    await vi.waitFor(async () => expect(await pendingCount()).toBe(0))
    expect(setGroceryChecked).toHaveBeenCalledWith('Eggs-', true)
    expect(counts).toContain(1)
    expect(counts[counts.length - 1]).toBe(0)
    unsubscribe()
  })

  it('keeps queued changes while the server is unreachable and sends them on drain', async () => {
    vi.mocked(setGroceryChecked).mockRejectedValue(new NetworkError('down'))
    await enqueue(check(true))
    await vi.waitFor(() => expect(isOfflineNow()).toBe(true))
    expect(await pendingCount()).toBe(1)

    vi.mocked(setGroceryChecked).mockResolvedValue(undefined)
    await drain()
    expect(await pendingCount()).toBe(0)
    // The touched document is refreshed from the server afterwards.
    expect(getGroceries).toHaveBeenCalled()
  })

  it('coalesces changes to the same item, keeping the first position', async () => {
    vi.mocked(setGroceryChecked).mockRejectedValue(new NetworkError('down'))
    await enqueue(check(true))
    await enqueue({ kind: 'saved', id: 'r1', saved: true })
    await enqueue(check(false))
    const rows = await db.list('queue')
    expect(rows).toHaveLength(2)
    expect(rows.map((r) => (JSON.parse(r.value) as { op: { kind: string } }).op.kind)).toEqual(['grocery', 'saved'])
    expect((JSON.parse(rows[0]!.value) as { op: { checked: boolean } }).op.checked).toBe(false)
    expect((await readDoc<typeof groceries>('groceries'))!.checked_items).toBe(0)
  })

  it('does not coalesce onto a change already in flight', async () => {
    const first = deferred<void>()
    vi.mocked(setGroceryChecked).mockReturnValueOnce(first.promise).mockResolvedValue(undefined)
    await enqueue(check(true))
    await vi.waitFor(() => expect(setGroceryChecked).toHaveBeenCalledTimes(1))
    await enqueue(check(false))
    expect(await pendingCount()).toBe(2)
    first.resolve()
    await vi.waitFor(async () => expect(await pendingCount()).toBe(0))
    expect(setGroceryChecked).toHaveBeenLastCalledWith('Eggs-', false)
  })

  it('drops a change the server rejects and refreshes what it touched', async () => {
    vi.mocked(setGroceryChecked).mockRejectedValue(new ApiError(400, 'user', 'ingredient not found'))
    await enqueue(check(true))
    await vi.waitFor(async () => expect(await pendingCount()).toBe(0))
    expect(getGroceries).toHaveBeenCalled()
    expect(isOfflineNow()).toBe(false)
  })

  it('treats a 404 on a removal as done without refreshing', async () => {
    vi.mocked(removeFromList).mockRejectedValue(new ApiError(404, 'not_found', 'gone'))
    await enqueue({ kind: 'list', id: 'r9', in_list: false })
    await vi.waitFor(async () => expect(await pendingCount()).toBe(0))
    expect(getKitchen).not.toHaveBeenCalled()
  })

  it('retries server errors a few times, then gives up', async () => {
    vi.mocked(setGroceryChecked).mockRejectedValue(new ApiError(500, 'server', 'oops'))
    await enqueue(check(true))
    await vi.waitFor(() => expect(setGroceryChecked).toHaveBeenCalledTimes(1))
    const attempts = async () => (JSON.parse((await db.list('queue'))[0]!.value) as { attempts: number }).attempts
    expect(await attempts()).toBe(1)
    await drain()
    await drain()
    await drain()
    expect(await attempts()).toBe(4)
    await drain()
    expect(await pendingCount()).toBe(0)
  })

  it('rebases pending changes on top of fresh server data', async () => {
    vi.mocked(setGroceryChecked).mockRejectedValue(new NetworkError('down'))
    await enqueue(check(true))
    expect((await rebase('groceries', groceries)).checked_items).toBe(1)
    expect((await rebase('kitchen', { kind: 'list' })).kind).toBe('list')
    installQueue()
  })
})
