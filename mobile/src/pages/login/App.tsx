import { useCallback, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import { login, requestPasswordReset } from '../../lib/api/auth.js'
import { saveSession } from '../../lib/auth/session.js'
import { t } from '../../lib/i18n/index.js'
import { replace } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { TextField } from '../../ui/TextField.js'

export function App() {
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  const submit = useCallback(async () => {
    if (busy) return
    setBusy(true)
    setError(null)
    setNotice(null)
    try {
      const session = await login(email.trim(), password)
      await saveSession({ token: session.token, expiresAt: session.expires_at })
      await replace('main')
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, email, password])

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

  return (
    <scroll-view className="screen" scroll-orientation="vertical">
      <view className="content login">
        <view className="login__hero">
          <text className="login__brand">imkitchen</text>
          <text className="h1">{t('login.title')}</text>
          <text className="body">{t('login.subtitle')}</text>
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
            confirmType="go"
            onConfirm={submit}
          />
          {error && <text className="error">{error}</text>}
          {notice && <text className="success">{notice}</text>}
          <Button label={t('login.submit')} onTap={submit} disabled={busy} block />
          <Button label={t('login.forgot')} onTap={forgot} variant="ghost" disabled={busy} />
        </view>

        <text className="muted">{t('login.no_account')}</text>
      </view>
    </scroll-view>
  )
}
