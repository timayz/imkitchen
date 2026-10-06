import { useCallback, useEffect, useState } from '@lynx-js/react'

import { ApiError } from '../../../lib/api/client.js'
import { logout, me, type Me } from '../../../lib/api/auth.js'
import type { DietaryRestriction, RecipeType } from '../../../lib/api/recipe.js'
import {
  type General,
  type Session,
  getGeneral,
  getSessions,
  requestAccountPasswordReset,
  revokeSession,
  setUsername,
  updatePreferences,
  updateProfile,
} from '../../../lib/api/settings.js'
import { clearSession } from '../../../lib/auth/session.js'
import { course } from '../../../lib/course.js'
import { t } from '../../../lib/i18n/index.js'
import { replace } from '../../../lib/nav.js'
import { Button } from '../../../ui/Button.js'
import { Chip } from '../../../ui/Chip.js'
import { Stepper } from '../../../ui/Stepper.js'
import { TextField } from '../../../ui/TextField.js'
import './SettingsTab.css'

const DIETS: DietaryRestriction[] = ['Vegetarian', 'Vegan', 'GlutenFree', 'DairyFree', 'NutFree']
const OPTIONAL_COURSES: RecipeType[] = ['Appetizer', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']

type State =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; user: Me; general: General; sessions: Session[] }

export function SettingsTab({ refreshKey }: { refreshKey: number }) {
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<{ kind: 'ok' | 'error'; text: string } | null>(null)
  const [username, setUsernameInput] = useState('')
  const [description, setDescription] = useState<string | null>(null)

  const load = useCallback(() => {
    Promise.all([me(), getGeneral(), getSessions()])
      .then(([user, general, sessions]) => setState({ kind: 'ready', user, general, sessions }))
      .catch((err: unknown) =>
        setState({
          kind: 'error',
          message: err instanceof ApiError ? err.message : t('error.network'),
        }),
      )
  }, [])

  useEffect(() => {
    load()
  }, [load, refreshKey])

  const run = useCallback(
    async (action: () => Promise<void>, ok: string, reload = false) => {
      if (busy) return
      setBusy(true)
      setNotice(null)
      try {
        await action()
        setNotice({ kind: 'ok', text: ok })
        if (reload) load()
      } catch (err) {
        setNotice({ kind: 'error', text: err instanceof ApiError ? err.message : t('error.network') })
      } finally {
        setBusy(false)
      }
    },
    [busy, load],
  )

  const patchGeneral = (p: Partial<General>) =>
    setState((prev) => (prev.kind === 'ready' ? { ...prev, general: { ...prev.general, ...p } } : prev))

  const signOut = useCallback(async () => {
    try {
      await logout()
    } catch {
      // The token is gone client-side either way.
    }
    await clearSession()
    await replace('login')
  }, [])

  if (state.kind === 'loading') {
    return (
      <view className="content">
        <text className="muted">{t('common.loading')}</text>
      </view>
    )
  }
  if (state.kind === 'error') {
    return (
      <view className="content">
        <view className="card">
          <text className="error">{state.message}</text>
          <Button label={t('common.retry')} onTap={load} variant="secondary" />
        </view>
      </view>
    )
  }

  const { user, general, sessions } = state

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        <text className="h1">{t('settings.title')}</text>

        <view className="card">
          <text className="muted">{t('settings.signed_in_as')}</text>
          <text className="h2">{user.username ?? user.email}</text>
          <text className="body">{user.email}</text>
          <view className="row">
            {user.premium_enabled && user.is_premium && (
              <text className="badge badge--premium">{t('settings.premium')}</text>
            )}
            {user.is_chef && <text className="badge badge--chef">{t('settings.chef')}</text>}
          </view>
          {!user.username && (
            <>
              <TextField
                label={t('settings.username')}
                value=""
                onChange={setUsernameInput}
                placeholder="chef_jane"
                maxlength={25}
              />
              <text className="muted">{t('settings.username_hint')}</text>
              <Button
                label={t('settings.username_set')}
                onTap={() => run(() => setUsername(username.trim()), t('settings.username_done'), true)}
                variant="secondary"
                disabled={busy || username.trim().length < 3}
              />
            </>
          )}
        </view>

        {notice && <text className={notice.kind === 'ok' ? 'success' : 'error'}>{notice.text}</text>}

        <view className="card">
          <text className="h2">{t('settings.preferences')}</text>
          <Stepper
            label={t('settings.household')}
            value={general.household_size}
            min={1}
            max={20}
            onChange={(household_size) => patchGeneral({ household_size })}
          />
          <text className="field__label">{t('settings.diet')}</text>
          <view className="row set__chips">
            {DIETS.map((d) => (
              <Chip
                key={d}
                label={t(`diet.${d}` as const)}
                active={general.dietary_restrictions.includes(d)}
                onTap={() =>
                  patchGeneral({
                    dietary_restrictions: general.dietary_restrictions.includes(d)
                      ? general.dietary_restrictions.filter((x) => x !== d)
                      : [...general.dietary_restrictions, d],
                  })
                }
              />
            ))}
          </view>
          <text className="field__label">{t('settings.courses')}</text>
          <text className="muted">{t('settings.courses_hint')}</text>
          <view className="row set__chips">
            {OPTIONAL_COURSES.map((type) => {
              const c = course(type)
              return (
                <Chip
                  key={type}
                  label={`${c.emoji} ${c.label}`}
                  active={general.recipe_types.includes(type)}
                  onTap={() =>
                    patchGeneral({
                      recipe_types: general.recipe_types.includes(type)
                        ? general.recipe_types.filter((x) => x !== type)
                        : [...general.recipe_types, type],
                    })
                  }
                />
              )
            })}
          </view>
          <Stepper
            label={t('settings.variety')}
            value={Math.round(general.cuisine_variety_weight * 100)}
            min={10}
            max={100}
            step={10}
            unit="%"
            onChange={(pct) => patchGeneral({ cuisine_variety_weight: pct / 100 })}
          />
          <Button
            label={t('settings.save_preferences')}
            onTap={() =>
              run(
                () =>
                  updatePreferences({
                    household_size: general.household_size,
                    dietary_restrictions: general.dietary_restrictions,
                    recipe_types: general.recipe_types,
                    cuisine_variety_weight: general.cuisine_variety_weight,
                  }),
                t('settings.preferences_saved'),
              )
            }
            disabled={busy}
          />
        </view>

        <view className="card">
          <text className="h2">{t('settings.profile')}</text>
          <TextField
            label={t('settings.description')}
            value={general.description}
            onChange={setDescription}
            maxlength={500}
          />
          <Button
            label={t('settings.save_profile')}
            onTap={() => run(() => updateProfile(description ?? general.description), t('settings.profile_saved'))}
            variant="secondary"
            disabled={busy}
          />
        </view>

        <view className="card">
          <text className="h2">{t('settings.account')}</text>
          <text className="body">{t('settings.password_hint')}</text>
          <Button
            label={t('settings.password_reset')}
            onTap={() => run(requestAccountPasswordReset, t('settings.password_sent'))}
            variant="secondary"
            disabled={busy}
          />
        </view>

        <view className="card">
          <text className="h2">{t('settings.sessions')}</text>
          {sessions.map((s) => (
            <view key={s.id} className="row set__session">
              <view className="set__session-body">
                <text className="body">{s.user_agent}</text>
                <text className="muted">
                  {s.tz}
                  {s.current ? ` · ${t('settings.this_device')}` : ''}
                </text>
              </view>
              {!s.current && (
                <Button
                  label={t('settings.revoke')}
                  onTap={() => run(() => revokeSession(s.id), t('settings.revoked'), true)}
                  variant="ghost"
                  disabled={busy}
                />
              )}
            </view>
          ))}
          <Button label={t('settings.logout')} onTap={signOut} variant="secondary" disabled={busy} />
        </view>
      </view>
    </scroll-view>
  )
}
