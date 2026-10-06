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
  const [showPassword, setShowPassword] = useState(false)
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
      const session = mode === 'register' ? await register(email.trim(), password) : await login(email.trim(), password)
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
  const toggle = {
    label: showPassword ? t('login.hide_password') : t('login.show_password'),
    onTap: () => setShowPassword((on) => !on),
  }
  const passwordType = showPassword ? 'text' : 'password'
  // Below the status bar, with room for the decorative circle behind the mark.
  const topInset = (lynx.__globalProps.statusBarHeight ?? 24) + 48

  return (
    <view className="screen">
      <scroll-view className="login__scroll" scroll-orientation="vertical">
        <view className="login">
          <view className={registering ? 'login__blob login__blob--herb' : 'login__blob'} />
          <view className={registering ? 'login__dot' : 'login__dot login__dot--herb'} />

          <view className="login__hero" style={{ paddingTop: `${topInset}px` }}>
            <view className="row login__brand">
              <view className="login__mark">
                <text className="login__mark-glyph">🍳</text>
              </view>
              <text className="login__wordmark">imkitchen</text>
            </view>
            <view className="login__titles">
              <text className="login__title">{registering ? t('login.register_title') : t('login.title')}</text>
              <text className="body">{registering ? t('login.register_subtitle') : t('login.subtitle')}</text>
            </view>
          </view>

          <view className="login__card">
            <TextField
              label={t('login.email')}
              value={email}
              onChange={setEmail}
              type="email"
              confirmType="next"
              placeholder="chef@example.com"
              invalid={error !== null}
              surface="cream"
            />
            <TextField
              label={t('login.password')}
              value={password}
              onChange={setPassword}
              type={passwordType}
              confirmType={registering ? 'next' : 'go'}
              onConfirm={registering ? undefined : submit}
              labelAction={registering ? undefined : { label: t('login.forgot'), onTap: forgot }}
              trailing={toggle}
              hint={registering ? t('login.password_hint') : undefined}
              invalid={error !== null}
              surface="cream"
            />
            {registering && (
              <TextField
                label={t('login.confirm_password')}
                value={confirm}
                onChange={setConfirm}
                type={passwordType}
                confirmType="go"
                onConfirm={submit}
                invalid={error !== null}
                surface="cream"
              />
            )}
            {error && (
              <view className="login__banner login__banner--error">
                <text className="login__banner-glyph">!</text>
                <text className="login__banner-text login__banner-text--error">{error}</text>
              </view>
            )}
            {notice && (
              <view className="login__banner login__banner--notice">
                <text className="login__banner-glyph login__banner-glyph--notice">✓</text>
                <text className="login__banner-text login__banner-text--notice">{notice}</text>
              </view>
            )}
            <Button
              label={registering ? t('login.register_submit') : t('login.submit')}
              onTap={submit}
              disabled={busy}
              block
              size="lg"
            />
          </view>

          <view className="row login__switch">
            <text className="muted">{registering ? t('login.have_account') : t('login.no_account')}</text>
            <Button
              label={registering ? t('login.sign_in') : t('login.create_one')}
              onTap={() => switchMode(registering ? 'login' : 'register')}
              variant="ghost"
              disabled={busy}
            />
          </view>
        </view>
      </scroll-view>
    </view>
  )
}
