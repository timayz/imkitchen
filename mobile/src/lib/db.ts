import * as native from 'sparkling-db'

/** One row of a collection, as `Db.list` returns it. */
export interface DbRow {
  key: string
  value: string
  /** Milliseconds since the epoch, set by the store on every write. */
  updated_at: number
}

/**
 * The local document store (`methods/db`, SQLite). Values are JSON strings;
 * the cache and the offline queue decide what goes in them.
 */
export interface DbBackend {
  get(collection: string, key: string): Promise<string | null>
  put(collection: string, key: string, value: string): Promise<void>
  remove(collection: string, key: string): Promise<void>
  list(collection: string, prefix?: string): Promise<DbRow[]>
  clear(collection?: string): Promise<void>
}

/** Never fails: a host without the method (iOS, tests) behaves like an empty store. */
class NativeDb implements DbBackend {
  private available = true

  private call<T>(run: (cb: (r: native.DbResponse<T>) => void) => void, method: string): Promise<T | null> {
    if (!this.available) return Promise.resolve(null)
    return new Promise((resolve) => {
      try {
        run((res) => {
          if (res.code === 1) {
            resolve(res.data ?? null)
            return
          }
          if (native.UNAVAILABLE_CODES.includes(res.code)) {
            // No bridge or no method: stop asking for the rest of this runtime.
            this.available = false
          } else {
            console.warn(`db.${method}: ${res.msg}`)
          }
          resolve(null)
        })
      } catch (err) {
        this.available = false
        console.warn(`db.${method}: ${String(err)}`)
        resolve(null)
      }
    })
  }

  async get(collection: string, key: string): Promise<string | null> {
    const res = await this.call<{ value?: string | null }>((cb) => native.get({ collection, key }, cb), 'get')
    return res?.value ?? null
  }

  async put(collection: string, key: string, value: string): Promise<void> {
    await this.call((cb) => native.put({ collection, key, value }, cb), 'put')
  }

  async remove(collection: string, key: string): Promise<void> {
    await this.call((cb) => native.remove({ collection, key }, cb), 'remove')
  }

  async list(collection: string, prefix?: string): Promise<DbRow[]> {
    const res = await this.call<{ json?: string }>((cb) => native.list({ collection, prefix }, cb), 'list')
    if (!res?.json) return []
    try {
      const rows = JSON.parse(res.json) as unknown
      return Array.isArray(rows) ? (rows as DbRow[]) : []
    } catch {
      return []
    }
  }

  async clear(collection?: string): Promise<void> {
    await this.call((cb) => native.clear({ collection }, cb), 'clear')
  }
}

/** In-memory store for tests and for hosts without the native method. */
export class MemoryDb implements DbBackend {
  readonly rows = new Map<string, DbRow>()

  private id(collection: string, key: string): string {
    return `${collection}::${key}`
  }

  async get(collection: string, key: string): Promise<string | null> {
    return this.rows.get(this.id(collection, key))?.value ?? null
  }

  async put(collection: string, key: string, value: string): Promise<void> {
    this.rows.set(this.id(collection, key), { key, value, updated_at: Date.now() })
  }

  async remove(collection: string, key: string): Promise<void> {
    this.rows.delete(this.id(collection, key))
  }

  async list(collection: string, prefix = ''): Promise<DbRow[]> {
    const out: DbRow[] = []
    this.rows.forEach((row, id) => {
      if (id.startsWith(`${collection}::`) && row.key.startsWith(prefix)) out.push(row)
    })
    return out.sort((a, b) => (a.key < b.key ? -1 : a.key > b.key ? 1 : 0))
  }

  async clear(collection?: string): Promise<void> {
    if (collection === undefined) {
      this.rows.clear()
      return
    }
    const gone: string[] = []
    this.rows.forEach((_, id) => {
      if (id.startsWith(`${collection}::`)) gone.push(id)
    })
    gone.forEach((id) => this.rows.delete(id))
  }
}

let backend: DbBackend = new NativeDb()

/** Swaps the store (tests). `null` restores the native one. */
export function setDbBackend(next: DbBackend | null): void {
  backend = next ?? new NativeDb()
}

export const db: DbBackend = {
  get: (c, k) => backend.get(c, k),
  put: (c, k, v) => backend.put(c, k, v),
  remove: (c, k) => backend.remove(c, k),
  list: (c, p) => backend.list(c, p),
  clear: (c) => backend.clear(c),
}
