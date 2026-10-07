import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

vi.hoisted(() => {
  Object.assign(globalThis, {
    __API_BASE_URL__: 'http://test',
    __APP_VERSION__: 'test',
    lynx: { __globalProps: { language: 'en' } },
  })
})

vi.mock('../client-identity.js', () => ({ clientIdentity: async () => 'imkitchen-test (Test; model; id)' }))
vi.mock('../auth/session.js', () => ({
  loadSession: async () => ({ token: 'tok', expiresAt: '2099-01-01T00:00:00Z' }),
  clearSession: vi.fn(async () => {}),
}))

import { isOfflineNow, setOffline } from '../offline/status.js'
import { ApiError, NetworkError, errorMessage, isOffline, request } from './client.js'

function response(status: number, body: string): Response {
  return { status, ok: status >= 200 && status < 300, text: async () => body } as unknown as Response
}

describe('request', () => {
  const fetchMock = vi.fn<typeof fetch>()

  beforeEach(() => {
    vi.stubGlobal('fetch', fetchMock)
    fetchMock.mockReset()
    setOffline(false)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
    vi.useRealTimers()
  })

  it('parses JSON and clears the offline flag', async () => {
    setOffline(true)
    fetchMock.mockResolvedValue(response(200, '{"kind":"list"}'))
    expect(await request('/api/v1/kitchen')).toEqual({ kind: 'list' })
    expect(isOfflineNow()).toBe(false)
    const [, init] = fetchMock.mock.calls[0]!
    expect((init!.headers as Record<string, string>).Authorization).toBe('Bearer tok')
  })

  it('returns undefined on 204', async () => {
    fetchMock.mockResolvedValue(response(204, ''))
    expect(await request('/api/v1/groceries/check', { method: 'PUT', body: {} })).toBeUndefined()
  })

  it('maps the error envelope to ApiError', async () => {
    fetchMock.mockResolvedValue(response(400, '{"error":{"code":"user","message":"ingredient not found"}}'))
    const err = await request('/api/v1/groceries/check', { method: 'PUT', body: {} }).catch((e: unknown) => e)
    expect(err).toBeInstanceOf(ApiError)
    expect(err).toMatchObject({ status: 400, code: 'user', message: 'ingredient not found' })
    expect(isOffline(err)).toBe(false)
    expect(errorMessage(err)).toBe('ingredient not found')
  })

  it('turns a failed fetch into NetworkError and flags the app offline', async () => {
    fetchMock.mockRejectedValue(new TypeError('Failed to fetch'))
    const err = await request('/api/v1/kitchen').catch((e: unknown) => e)
    expect(err).toBeInstanceOf(NetworkError)
    expect(isOffline(err)).toBe(true)
    expect(isOfflineNow()).toBe(true)
    expect(errorMessage(err)).toMatch(/back online/)
  })

  it('treats the Lynx synthetic 499 as unreachable', async () => {
    fetchMock.mockResolvedValue(response(499, ''))
    const err = await request('/api/v1/kitchen').catch((e: unknown) => e)
    expect(err).toBeInstanceOf(NetworkError)
    expect(isOfflineNow()).toBe(true)
  })

  it('gives up after the timeout', async () => {
    vi.useFakeTimers()
    fetchMock.mockReturnValue(new Promise(() => {}))
    const pending = request('/api/v1/kitchen', { timeoutMs: 1000 }).catch((e: unknown) => e)
    await vi.advanceTimersByTimeAsync(1001)
    const err = await pending
    expect(err).toBeInstanceOf(NetworkError)
    expect(isOfflineNow()).toBe(true)
  })
})
