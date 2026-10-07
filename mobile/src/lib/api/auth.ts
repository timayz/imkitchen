import { request } from './client.js'

export interface Me {
  id: string
  email: string
  username: string | null
  role: 'User' | 'Chef' | 'Admin'
  is_chef: boolean
  is_admin: boolean
  is_premium: boolean
  premium_enabled: boolean
  tz: string
}

export interface Token {
  token: string
  expires_at: string
}

export interface SessionResponse extends Token {
  user: Me
}

export function login(email: string, password: string): Promise<SessionResponse> {
  return request<SessionResponse>('/api/v1/auth/login', {
    method: 'POST',
    body: { email, password },
    anonymous: true,
  })
}

/** Creates the account and signs this device in (201 with a session). */
export function register(email: string, password: string): Promise<SessionResponse> {
  return request<SessionResponse>('/api/v1/auth/register', {
    method: 'POST',
    body: { email, password },
    anonymous: true,
  })
}

export function logout(): Promise<void> {
  return request<void>('/api/v1/auth/logout', { method: 'POST' })
}

export function refresh(): Promise<Token> {
  return request<Token>('/api/v1/auth/refresh', { method: 'POST' })
}

export function requestPasswordReset(email: string): Promise<void> {
  return request<void>('/api/v1/auth/password-reset', {
    method: 'POST',
    body: { email },
    anonymous: true,
  })
}

/**
 * Whether the id from the emailed link can still be used. Rejects with a
 * 404 for an unknown link and a 400 once it has expired (15 minutes) or
 * been used.
 */
export function checkPasswordReset(id: string): Promise<void> {
  return request<void>(`/api/v1/auth/password-reset/${encodeURIComponent(id)}`, { anonymous: true })
}

/** Sets the new password; 422 when it is outside 8 to 20 characters. */
export function confirmPasswordReset(id: string, password: string): Promise<void> {
  return request<void>(`/api/v1/auth/password-reset/${encodeURIComponent(id)}`, {
    method: 'POST',
    body: { password },
    anonymous: true,
  })
}

export function me(): Promise<Me> {
  return request<Me>('/api/v1/me')
}
