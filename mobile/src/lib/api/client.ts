import { API_BASE_URL, APP_VERSION } from '../config.js'
import { clientIdentity } from '../client-identity.js'
import { clearSession, loadSession } from '../auth/session.js'
import { t } from '../i18n/index.js'
import { setOffline } from '../offline/status.js'

/** The JSON error envelope every `/api/v1` route answers with. */
export interface ApiErrorBody {
  error: { code: string; message: string }
}

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

/** The server could not be reached (no network, DNS, timeout). */
export class NetworkError extends Error {
  constructor(
    message: string,
    public readonly cause?: unknown
  ) {
    super(message)
    this.name = 'NetworkError'
  }
}

export function isOffline(err: unknown): err is NetworkError {
  return err instanceof NetworkError
}

/** What a screen shows for a failed request. */
export function errorMessage(err: unknown): string {
  if (err instanceof ApiError) return err.message
  if (isOffline(err)) return t('offline.action')
  return t('error.network')
}

export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  body?: unknown
  headers?: Record<string, string>
  /** Send the request without the session token (login, password reset). */
  anonymous?: boolean
  /** Give up after this long (the Lynx `fetch` has no abort signal). */
  timeoutMs?: number
}

const GET_TIMEOUT_MS = 15_000
const MUTATION_TIMEOUT_MS = 30_000

type UnauthorizedHandler = () => void
let onUnauthorized: UnauthorizedHandler | null = null

/** Called once when a signed-in request comes back 401 (session ended). */
export function setUnauthorizedHandler(handler: UnauthorizedHandler | null): void {
  onUnauthorized = handler
}

function timezone(): string | null {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone ?? null
  } catch {
    return null
  }
}

/** Rejects with a `NetworkError` after `ms`; the underlying call keeps running. */
function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new NetworkError(`timeout after ${ms} ms`)), ms)
    promise.then(
      (value) => {
        clearTimeout(timer)
        resolve(value)
      },
      (err: unknown) => {
        clearTimeout(timer)
        reject(err)
      }
    )
  })
}

/**
 * Minimal JSON client over the Lynx `fetch`. No cookies, no redirects: the
 * server answers 401 with a JSON envelope, never a redirect. A request that
 * never reaches the server rejects with a `NetworkError` and flips the
 * app-wide offline flag; any answer clears it.
 */
export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = {
    Accept: 'application/json',
    'User-Agent': await clientIdentity(),
    'Accept-Language': lynx.__globalProps.language || 'en',
    'X-Client-Version': APP_VERSION,
    ...options.headers,
  }
  const tz = timezone()
  if (tz) headers['X-Timezone'] = tz

  // Every bundle is its own JS runtime: read the persisted session on first use.
  const session = options.anonymous ? null : await loadSession()
  if (session) headers.Authorization = `Bearer ${session.token}`

  const method = options.method ?? 'GET'
  let body: string | undefined
  if (options.body !== undefined) {
    headers['Content-Type'] = 'application/json'
    body = JSON.stringify(options.body)
  }

  const timeoutMs = options.timeoutMs ?? (method === 'GET' ? GET_TIMEOUT_MS : MUTATION_TIMEOUT_MS)
  let response: Response
  let text: string
  try {
    response = await withTimeout(fetch(`${API_BASE_URL}${path}`, { method, headers, body }), timeoutMs)
    text = response.status === 204 ? '' : await withTimeout(response.text(), timeoutMs)
  } catch (err) {
    setOffline(true)
    throw err instanceof NetworkError ? err : new NetworkError('fetch failed', err)
  }
  // The Lynx `fetch` does not reject when the server cannot be reached: it
  // resolves with a synthetic 499 (or 0) status and an empty body.
  if (response.status === 499 || response.status === 0) {
    setOffline(true)
    throw new NetworkError(`unreachable (HTTP ${response.status})`)
  }
  setOffline(false)

  if (response.status === 204) {
    return undefined as T
  }

  const json: unknown = text ? JSON.parse(text) : null

  if (!response.ok) {
    if (response.status === 401 && session) {
      await clearSession()
      onUnauthorized?.()
    }
    const envelope = json as Partial<ApiErrorBody> | null
    throw new ApiError(
      response.status,
      envelope?.error?.code ?? 'server',
      envelope?.error?.message ?? `HTTP ${response.status}`
    )
  }

  return json as T
}
