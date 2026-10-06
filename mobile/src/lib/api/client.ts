import { API_BASE_URL, APP_VERSION } from '../config.js'
import { clientIdentity } from '../client-identity.js'
import { clearSession, loadSession } from '../auth/session.js'

/** The JSON error envelope every `/api/v1` route answers with. */
export interface ApiErrorBody {
  error: { code: string; message: string }
}

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export interface RequestOptions {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  body?: unknown
  headers?: Record<string, string>
  /** Send the request without the session token (login, password reset). */
  anonymous?: boolean
}

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

/**
 * Minimal JSON client over the Lynx `fetch`. No cookies, no redirects: the
 * server answers 401 with a JSON envelope, never a redirect.
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

  let body: string | undefined
  if (options.body !== undefined) {
    headers['Content-Type'] = 'application/json'
    body = JSON.stringify(options.body)
  }

  const response = await fetch(`${API_BASE_URL}${path}`, {
    method: options.method ?? 'GET',
    headers,
    body,
  })

  if (response.status === 204) {
    return undefined as T
  }

  const text = await response.text()
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
      envelope?.error?.message ?? `HTTP ${response.status}`,
    )
  }

  return json as T
}
