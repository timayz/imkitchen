import { useCallback, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import { login, register, requestPasswordReset } from '../../lib/api/auth.js'
import { saveSession } from '../../lib/auth/session.js'
import { t } from '../../lib/i18n/index.js'
import { replace } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { TextField } from '../../ui/TextField.js'

type Mode = 'login' | 'register'

export function App() {
  const [mode, setMode] = useState<Mode>('login')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const switchMode = useCallback((next: Mode) => {
    setMode(next)
    setError(null)
    setNotice(null)
  }, [])

  const submit = useCallback(async () => {
    if (busy) return
    if (mode === 'register' && password !== confirm) {
      setError(t('login.mismatch'))
      return
    }
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      const session =
        mode === 'register'
          ? await register(email.trim(), password)
          : await login(email.trim(), password)
      await saveSession({ token: session.token, expiresAt: session.expires_at })
      await replace('main')
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, mode, email, password, confirm])

  const forgot = useCallback(async () => {
    if (busy || !email.trim()) return
    setBusy(true)
    setError(null)
    try {
      await requestPasswordReset(email.trim())
      setNotice(t('login.reset_sent'))
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, email])

  const registering = mode === 'register'

  return (
    <scroll-view className="screen" scroll-orientation="vertical">
      <view className="content login">
        <view className="login__hero">
          <text className="login__brand">imkitchen</text>
          <text className="h1">{registering ? t('login.register_title') : t('login.title')}</text>
          <text className="body">
            {registering ? t('login.register_subtitle') : t('login.subtitle')}
          </text>
        </view>

        <view className="card">
          <TextField
            label={t('login.email')}
            value={email}
            onChange={setEmail}
            type="email"
            confirmType="next"
            placeholder="chef@example.com"
          />
          <TextField
            label={t('login.password')}
            value={password}
            onChange={setPassword}
            type="password"
            confirmType={registering ? 'next' : 'go'}
            onConfirm={registering ? undefined : submit}
          />
          {registering && (
            <>
              <text className="muted">{t('login.password_hint')}</text>
              <TextField
                label={t('login.confirm_password')}
                value={confirm}
                onChange={setConfirm}
                type="password"
                confirmType="go"
                onConfirm={submit}
              />
            </>
          )}
          {error && <text className="error">{error}</text>}
          {notice && <text className="success">{notice}</text>}
          <Button
            label={registering ? t('login.register_submit') : t('login.submit')}
            onTap={submit}
            disabled={busy}
            block
          />
          {!registering && (
            <Button label={t('login.forgot')} onTap={forgot} variant="ghost" disabled={busy} />
          )}
        </view>

        <view className="row login__switch">
          <text className="muted">
            {registering ? t('login.have_account') : t('login.no_account')}
          </text>
          <Button
            label={registering ? t('login.sign_in') : t('login.create_one')}
            onTap={() => switchMode(registering ? 'login' : 'register')}
            variant="ghost"
            disabled={busy}
          />
        </view>
      </view>
    </scroll-view>
  )
}
