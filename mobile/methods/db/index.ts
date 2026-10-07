import * as sparklingMethod from 'sparkling-method'

interface Pipe {
  call(method: string, params: unknown, callback: (v: unknown) => void): void
}

// `sparkling-method` is typed as CommonJS, so its default export is reachable
// either as the namespace itself (bundler interop) or as `.default`.
const pipe: Pipe = (sparklingMethod as unknown as { default?: Pipe }).default ?? (sparklingMethod as unknown as Pipe)

/**
 * Raw bridge answer. `code` 1 = ok, anything else failed (`msg`); result
 * fields come back under `data`.
 */
export interface DbResponse<T> {
  code: number
  msg: string
  data?: T
}

/** The bridge codes meaning the method (or the bridge itself) is absent. */
export const UNAVAILABLE_CODES: readonly number[] = [-2, -3, -4, -5]

function call<T>(method: string, params: unknown, callback: (r: DbResponse<T>) => void): void {
  pipe.call(method, params, (v: unknown) => {
    const response = v as Partial<DbResponse<T>> | null
    const code = response?.code ?? -1
    callback({ code, msg: response?.msg ?? (code === 1 ? 'ok' : 'Unknown error'), data: response?.data })
  })
}

export interface GetRequest {
  collection: string
  key: string
}

/** The document, `null`/absent when there is none. */
export function get(params: GetRequest, callback: (r: DbResponse<{ value?: string | null }>) => void): void {
  call('Db.get', params, callback)
}

export interface PutRequest {
  collection: string
  key: string
  value: string
}

export function put(params: PutRequest, callback: (r: DbResponse<{ ok?: boolean }>) => void): void {
  call('Db.put', params, callback)
}

export function remove(params: GetRequest, callback: (r: DbResponse<{ ok?: boolean }>) => void): void {
  call('Db.remove', params, callback)
}

export interface ListRequest {
  collection: string
  /** Only keys starting with this. */
  prefix?: string
}

/** `json` is a JSON array of `{key, value, updated_at}` ordered by key. */
export function list(params: ListRequest, callback: (r: DbResponse<{ json?: string }>) => void): void {
  call('Db.list', params, callback)
}

export interface ClearRequest {
  /** Omit to delete every collection. */
  collection?: string
}

export function clear(params: ClearRequest, callback: (r: DbResponse<{ ok?: boolean }>) => void): void {
  call('Db.clear', params, callback)
}
