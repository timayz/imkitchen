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

export function me(): Promise<Me> {
  return request<Me>('/api/v1/me')
}
