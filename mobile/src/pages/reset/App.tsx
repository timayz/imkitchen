import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import '../../styles/auth.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import { checkPasswordReset, confirmPasswordReset, requestPasswordReset } from '../../lib/api/auth.js'
import { t } from '../../lib/i18n/index.js'
import { back, pageParams, replace } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { TextField } from '../../ui/TextField.js'

/**
 * `request` → `sent` when opened from the login screen; `checking` → `new`
 * → `done` (or `expired`) when opened from the emailed link, which carries
 * the reset id as `id`.
 */
type Step = 'request' | 'sent' | 'checking' | 'new' | 'done' | 'expired'

/** An unknown, expired or used link: the API answers 404 or 400. */
function linkIsDead(err: unknown): boolean {
  return err instanceof ApiError && (err.status === 404 || err.status === 400)
}

export function App() {
  const params = pageParams()
  const linkId = params.id ?? ''
  const fromLink = linkId !== ''

  const [step, setStep] = useState<Step>(fromLink ? 'checking' : 'request')
  const [email, setEmail] = useState(params.email ?? '')
  const [password, setPassword] = useState('')
  const [confirm, setConfirm] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)

  useEffect(() => {
    if (!fromLink) return
    checkPasswordReset(linkId)
      .then(() => setStep('new'))
      .catch((err: unknown) => {
        if (linkIsDead(err)) {
          setStep('expired')
          return
        }
        // Offline: show the form anyway, the submit will say so again.
        setError(err instanceof ApiError ? err.message : t('error.network'))
        setStep('new')
      })
  }, [fromLink, linkId])

  const go = useCallback((next: Step) => {
    setStep(next)
    setError(null)
    setNotice(null)
  }, [])

  const send = useCallback(
    async (again: boolean) => {
      const address = email.trim()
      if (busy || !address) return
      setBusy(true)
      setError(null)
      setNotice(null)
      try {
        await requestPasswordReset(address)
        if (again) setNotice(t('reset.sent_again'))
        else go('sent')
      } catch (err) {
        setError(err instanceof ApiError ? err.message : t('error.network'))
      } finally {
        setBusy(false)
      }
    },
    [busy, email, go],
  )

  const submitNew = useCallback(async () => {
    if (busy) return
    if (password !== confirm) {
      setError(t('login.mismatch'))
      return
    }
    setBusy(true)
    setError(null)
    try {
      await confirmPasswordReset(linkId, password)
      go('done')
    } catch (err) {
      if (linkIsDead(err)) go('expired')
      else setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, password, confirm, linkId, go])

  // Pushed from the login screen: pop back to it. Cold-started from the
  // emailed link: this is the only container, so open the login in place.
  const toSignIn = useCallback(() => {
    if (fromLink) void replace('login')
    else back()
  }, [fromLink])

  const herb = step === 'sent' || step === 'done'
  const toggle = {
    label: showPassword ? t('login.hide_password') : t('login.show_password'),
    onTap: () => setShowPassword((on) => !on),
  }
  const passwordType = showPassword ? 'text' : 'password'
  // Below the status bar, with room for the decorative circle behind the mark.
  const topInset = (lynx.__globalProps.statusBarHeight ?? 24) + 48

  const brand = (
    <view className="row auth__brand">
      <view className="auth__mark">
        <text className="auth__mark-glyph">🍳</text>
      </view>
      <text className="auth__wordmark">imkitchen</text>
    </view>
  )

  const banners = (
    <>
      {error && (
        <view className="auth__banner auth__banner--error">
          <text className="auth__banner-glyph">!</text>
          <text className="auth__banner-text auth__banner-text--error">{error}</text>
        </view>
      )}
      {notice && (
        <view className="auth__banner auth__banner--notice">
          <text className="auth__banner-glyph auth__banner-glyph--notice">✓</text>
          <text className="auth__banner-text auth__banner-text--notice">{notice}</text>
        </view>
      )}
    </>
  )

  const signInRow = (label: string) => (
    <view className="row auth__switch">
      <text className="muted">{label}</text>
      <Button label={t('reset.back_to_sign_in')} onTap={toSignIn} variant="ghost" disabled={busy} />
    </view>
  )

  return (
    <view className="screen">
      <scroll-view className="auth__scroll" scroll-orientation="vertical">
        <view className="auth">
          <view className={herb ? 'auth__blob auth__blob--herb' : 'auth__blob'} />
          <view className={herb ? 'auth__dot auth__dot--herb' : 'auth__dot'} />

          {step === 'request' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
                <view className="auth__titles">
                  <text className="auth__title">{t('reset.title')}</text>
                  <text className="body">{t('reset.subtitle')}</text>
                </view>
              </view>
              <view className="auth__card">
                <TextField
                  label={t('login.email')}
                  value={email}
                  onChange={setEmail}
                  type="email"
                  confirmType="send"
                  onConfirm={() => void send(false)}
                  placeholder="chef@example.com"
                  hint={t('reset.email_hint')}
                  invalid={error !== null}
                  surface="cream"
                />
                {banners}
                <Button label={t('reset.submit')} onTap={() => void send(false)} disabled={busy} block size="lg" />
              </view>
              {signInRow(t('reset.remembered'))}
            </>
          )}

          {step === 'sent' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
                <view className="reset__status">
                  <view className="reset__mark reset__mark--herb">
                    <text className="reset__mark-glyph">✉</text>
                  </view>
                  <view className="auth__titles">
                    <text className="auth__title">{t('reset.sent_title')}</text>
                    <text className="body">{t('reset.sent_body', { email: email.trim() })}</text>
                  </view>
                </view>
              </view>
              <view className="auth__card">
                <view className="reset__tips">
                  <text className="reset__tips-title">{t('reset.not_received')}</text>
                  {[t('reset.tip_spam'), t('reset.tip_address'), t('reset.tip_wait')].map((tip) => (
                    <view className="reset__tip" key={tip}>
                      <view className="reset__tip-dot" />
                      <text className="reset__tip-text">{tip}</text>
                    </view>
                  ))}
                </view>
                {banners}
                <Button
                  label={t('reset.resend')}
                  onTap={() => void send(true)}
                  variant="secondary"
                  disabled={busy}
                  block
                  size="lg"
                />
                <Button
                  label={t('reset.other_email')}
                  onTap={() => go('request')}
                  variant="ghost"
                  disabled={busy}
                  block
                />
              </view>
              {signInRow(t('reset.remembered'))}
            </>
          )}

          {step === 'checking' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
              </view>
              <view className="reset__checking">
                <text className="body">{t('reset.checking')}</text>
              </view>
            </>
          )}

          {step === 'new' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
                <view className="auth__titles">
                  <text className="auth__title">{t('reset.new_title')}</text>
                  <text className="body">{t('reset.new_subtitle')}</text>
                </view>
              </view>
              <view className="auth__card">
                <TextField
                  label={t('reset.new_password')}
                  value={password}
                  onChange={setPassword}
                  type={passwordType}
                  confirmType="next"
                  trailing={toggle}
                  hint={t('login.password_hint')}
                  invalid={error !== null}
                  surface="cream"
                />
                <TextField
                  label={t('login.confirm_password')}
                  value={confirm}
                  onChange={setConfirm}
                  type={passwordType}
                  confirmType="go"
                  onConfirm={submitNew}
                  invalid={error !== null}
                  surface="cream"
                />
                {banners}
                <Button label={t('reset.new_submit')} onTap={submitNew} disabled={busy} block size="lg" />
              </view>
              {signInRow(t('reset.changed_mind'))}
            </>
          )}

          {step === 'done' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
              </view>
              <view className="reset__center">
                <view className="reset__mark reset__mark--herb reset__mark--big">
                  <text className="reset__mark-glyph reset__mark-glyph--big">✓</text>
                </view>
                <view className="auth__titles">
                  <text className="auth__title reset__center-text">{t('reset.done_title')}</text>
                  <text className="body reset__center-text">{t('reset.done_body')}</text>
                </view>
              </view>
              <view className="reset__footer">
                <Button label={t('reset.done_sign_in')} onTap={() => void replace('login')} block size="lg" />
              </view>
            </>
          )}

          {step === 'expired' && (
            <>
              <view className="auth__hero" style={{ paddingTop: `${topInset}px` }}>
                {brand}
                <view className="reset__status">
                  <view className="reset__mark">
                    <text className="reset__mark-glyph">!</text>
                  </view>
                  <view className="auth__titles">
                    <text className="auth__title">{t('reset.expired_title')}</text>
                    <text className="body">{t('reset.expired_body')}</text>
                  </view>
                </view>
              </view>
              <view className="auth__card">
                <view className="auth__banner auth__banner--error">
                  <text className="auth__banner-glyph">!</text>
                  <text className="auth__banner-text auth__banner-text--error">{t('reset.expired_note')}</text>
                </view>
                <Button label={t('reset.request_again')} onTap={() => go('request')} block size="lg" />
              </view>
              {signInRow(t('reset.remembered'))}
            </>
          )}
        </view>
      </scroll-view>
    </view>
  )
}
