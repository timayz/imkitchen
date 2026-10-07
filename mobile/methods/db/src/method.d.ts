/** SQLite document store: `(collection, key)` -> JSON string. */
export interface GetRequest {
  collection: string
  key: string
}

export interface GetResponse {
  value: string | null
}

export interface PutRequest {
  collection: string
  key: string
  value: string
}

export interface OkResponse {
  ok: boolean
}

export interface ListRequest {
  collection: string
  prefix?: string
}

export interface ListResponse {
  /** JSON array of `{key, value, updated_at}` ordered by key. */
  json: string
}

export interface ClearRequest {
  collection?: string
}

declare function get(params: GetRequest, callback: (result: GetResponse) => void): void
declare function put(params: PutRequest, callback: (result: OkResponse) => void): void
declare function remove(params: GetRequest, callback: (result: OkResponse) => void): void
declare function list(params: ListRequest, callback: (result: ListResponse) => void): void
declare function clear(params: ClearRequest, callback: (result: OkResponse) => void): void
