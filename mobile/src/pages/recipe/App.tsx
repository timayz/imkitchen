import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import { type Me, me } from '../../lib/api/auth.js'
import {
  type Detail,
  type Summary,
  addRecipeToList,
  deleteRecipe,
  getRecipe,
  getSimilar,
  saveRecipe,
  shareRecipe,
  unsaveRecipe,
  unshareRecipe,
  waitForRecipe,
} from '../../lib/api/recipes.js'
import { course, minutes } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { back, openExternal, pageParams, push } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { RecipeCard } from '../../ui/RecipeCard.js'
import { RecipeImage } from '../../ui/RecipeImage.js'
import { Sheet } from '../../ui/Sheet.js'
import { Spinner } from '../../ui/Spinner.js'

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; recipe: Detail }

/** Recipe detail: `recipe.lynx.bundle?id=<id or slug>`. */
export function App() {
  const id = pageParams().id ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [similar, setSimilar] = useState<Summary[]>([])
  const [user, setUser] = useState<Me | null>(null)
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  const fail = (err: unknown) =>
    setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') })

  const load = useCallback(() => {
    getRecipe(id)
      .then((recipe) => {
        setState({ kind: 'ready', recipe })
        getSimilar(recipe.id)
          .then((page) => setSimilar(page.edges.map((e) => e.node)))
          .catch(() => {})
      })
      .catch(fail)
  }, [id])

  useEffect(() => {
    load()
    me()
      .then(setUser)
      .catch(() => {})
  }, [load])

  // Returning from the editor: show what was saved.
  useEffect(() => {
    const emitter = lynx.getJSModule('GlobalEventEmitter')
    emitter.addListener('onShow', load)
    return () => emitter.removeListener('onShow', load)
  }, [load])

  const run = useCallback(
    async (action: () => Promise<void>, patch?: (r: Detail) => Detail) => {
      if (busy) return
      setBusy(true)
      setNotice(null)
      try {
        await action()
        if (patch) setState((prev) => (prev.kind === 'ready' ? { kind: 'ready', recipe: patch(prev.recipe) } : prev))
      } catch (err) {
        setNotice(err instanceof ApiError ? err.message : t('error.network'))
      } finally {
        setBusy(false)
      }
    },
    [busy],
  )

  const remove = useCallback(async () => {
    if (state.kind !== 'ready') return
    setConfirmDelete(false)
    setBusy(true)
    try {
      await deleteRecipe(state.recipe.id)
      await waitForRecipe(state.recipe.id, false)
      back()
    } catch (err) {
      setNotice(err instanceof ApiError ? err.message : t('error.network'))
      setBusy(false)
    }
  }, [state])

  if (state.kind === 'loading') {
    return (
      <view className="screen rdet--center">
        <Spinner size="lg" />
      </view>
    )
  }
  if (state.kind === 'error') {
    return (
      <view className="screen rdet--center">
        <view className="card">
          <text className="error">{state.message}</text>
          <Button label={t('common.close')} onTap={back} variant="secondary" />
        </view>
      </view>
    )
  }

  const r = state.recipe
  const c = course(r.recipe_type)
  const isChef = user?.is_chef === true
  const owner = r.owner_name
  const prepNeeded = r.advance_prep.trim() !== ''

  return (
    <view className="screen">
      <scroll-view className="rdet__scroll" scroll-orientation="vertical">
        {/* Full-bleed hero: course-tinted photo, floating back button, badges at the foot */}
        <view className="rdet__hero">
          <RecipeImage
            thumbnailUrl={r.thumbnail_url}
            blurPlaceholder={r.blur_placeholder}
            recipeType={r.recipe_type}
            size="cover"
          />
          <view className="rdet__hero-bar">
            <view className="rdet__round" bindtap={back}>
              <text className="rdet__round-glyph">←</text>
            </view>
          </view>
          <view className="rdet__badges">
            <view className="rdet__badge rdet__badge--paper">
              <text className="rdet__badge-text" style={{ color: c.ink }}>
                {c.label}
              </text>
            </view>
            {r.is_owner && r.is_shared && (
              <view className="rdet__badge rdet__badge--shared">
                <text className="rdet__badge-text rdet__badge-text--shared">{t('recipes.shared')}</text>
              </view>
            )}
            {r.is_owner && !r.is_shared && (
              <view className="rdet__badge rdet__badge--private">
                <text className="rdet__badge-text rdet__badge-text--private">{t('recipes.private')}</text>
              </view>
            )}
          </view>
        </view>

        <view className="content">
          <view className="rdet__title">
            <text className="rdet__h1">{r.name}</text>
            {r.description !== '' && <text className="body">{r.description}</text>}
            {r.origin && (
              <view className="rdet__origin" bindtap={() => openExternal(r.origin!)}>
                <text className="link">↗ {t('recipes.origin')}</text>
              </view>
            )}
          </view>

          {!r.is_owner && owner && (
            <view className="rdet__author" bindtap={() => void push('cook', { username: owner })}>
              <view className="rdet__avatar">
                <text className="rdet__avatar-text">{owner.charAt(0).toUpperCase()}</text>
              </view>
              <view className="rdet__author-body">
                <view className="row">
                  <text className="rdet__author-name">@{owner}</text>
                  <text className="badge badge--chef">{t('cook.eyebrow')}</text>
                </view>
                <text className="rcard__meta">{t('recipes.shared_count', { n: r.owner_stat.shared })}</text>
              </view>
              <text className="rcard__chevron">›</text>
            </view>
          )}

          <view className="rdet__stats">
            <Stat label={t('recipes.prep')} value={minutes(r.prep_time)} divider />
            <Stat label={t('recipes.cook_time')} value={minutes(r.cook_time)} divider />
            <Stat label={t('recipes.serves')} value={String(r.household_size)} />
          </view>

          {(r.dietary_restrictions.length > 0 || r.accepts_accompaniment) && (
            <view className="hero__pills">
              {r.dietary_restrictions.map((d) => (
                <view key={d} className="pill pill--line">
                  <text className="pill__text">{t(`diet.${d}` as const)}</text>
                </view>
              ))}
              {r.accepts_accompaniment && (
                <view className="pill pill--line">
                  <text className="pill__text">{t('recipes.takes_side')}</text>
                </view>
              )}
            </view>
          )}

          {notice && <text className="error">{notice}</text>}

          <view className="rdet__actions">
            <Button
              label={r.in_shopping ? t('recipes.in_list') : t('recipes.add_to_list')}
              onTap={() =>
                run(
                  () => addRecipeToList(r.id),
                  (x) => ({ ...x, in_shopping: true }),
                )
              }
              variant={r.in_shopping ? 'secondary' : 'primary'}
              disabled={busy || r.in_shopping}
              block
            />
            {!r.is_owner && (
              <Button
                label={r.saved ? t('recipes.saved_btn') : t('recipes.save')}
                onTap={() =>
                  run(
                    () => (r.saved ? unsaveRecipe(r.id) : saveRecipe(r.id)),
                    (x) => ({ ...x, saved: !x.saved }),
                  )
                }
                variant="secondary"
                disabled={busy}
                block
              />
            )}
            {r.is_owner && (
              <view className="rdet__row">
                <view className="rdet__secondary" bindtap={() => void push('recipe-edit', { id: r.id })}>
                  <text className="rdet__secondary-text">{t('recipes.edit')}</text>
                </view>
                {isChef && (
                  <view
                    className={busy ? 'rdet__secondary rdet__secondary--disabled' : 'rdet__secondary'}
                    bindtap={
                      busy
                        ? undefined
                        : () =>
                            run(
                              () => (r.is_shared ? unshareRecipe(r.id) : shareRecipe(r.id)),
                              (x) => ({ ...x, is_shared: !x.is_shared }),
                            )
                    }
                  >
                    <text className="rdet__secondary-text">
                      {r.is_shared ? t('recipes.make_private') : t('recipes.make_public')}
                    </text>
                  </view>
                )}
                <view
                  className={busy ? 'rdet__icon-btn rdet__secondary--disabled' : 'rdet__icon-btn'}
                  bindtap={busy ? undefined : () => setConfirmDelete(true)}
                >
                  <text className="rdet__icon-btn-glyph">🗑</text>
                </view>
              </view>
            )}
          </view>

          {prepNeeded && (
            <view className="rdet__prep">
              <view className="rdet__prep-icon">
                <text className="rdet__prep-glyph">⏰</text>
              </view>
              <view className="rdet__prep-body">
                <text className="rdet__eyebrow">{t('recipes.plan_ahead').toUpperCase()}</text>
                <text className="rdet__prep-text">{r.advance_prep}</text>
              </view>
            </view>
          )}

          <view className="card rdet__card">
            <view className="rdet__card-head">
              <text className="h2">{t('recipes.ingredients')}</text>
              <text className="rdet__count">{t('recipes.items_count', { n: r.ingredients.length })}</text>
            </view>
            {r.ingredients.map((i, index) => (
              <view
                key={i.key}
                className={index === r.ingredients.length - 1 ? 'rdet__ing rdet__ing--last' : 'rdet__ing'}
              >
                <text className="rdet__ing-name">{i.name}</text>
                <text className="rdet__ing-qty">{i.quantity_label}</text>
              </view>
            ))}
          </view>

          {r.instructions.length > 0 && (
            <view className="card rdet__steps">
              <text className="h2">{t('recipes.instructions')}</text>
              {r.instructions.map((ins, index) => (
                <view key={index} className="rdet__step-group">
                  <view className="rdet__step">
                    <view className="rdet__num" style={{ backgroundColor: c.soft }}>
                      <text className="rdet__num-text" style={{ color: c.ink }}>
                        {index + 1}
                      </text>
                    </view>
                    <text className="rdet__step-text">{ins.description}</text>
                  </view>
                  {ins.time_next > 0 && (
                    <view className="rdet__wait">
                      <text className="rdet__wait-text">⏲ {t('recipes.wait', { time: minutes(ins.time_next) })}</text>
                    </view>
                  )}
                </view>
              ))}
            </view>
          )}

          {similar.length > 0 && (
            <view className="rdet__similar">
              <text className="h2">{t('recipes.similar')}</text>
              {similar.map((s) => (
                <RecipeCard key={s.id} recipe={s} onTap={() => void push('recipe', { id: s.id })} />
              ))}
            </view>
          )}
        </view>
      </scroll-view>

      <Sheet open={confirmDelete} onClose={() => setConfirmDelete(false)}>
        <text className="h2">{t('recipes.delete_title')}</text>
        <text className="body">{t('recipes.delete_hint')}</text>
        <Button label={t('recipes.delete')} onTap={remove} variant="danger" block />
        <Button label={t('common.cancel')} onTap={() => setConfirmDelete(false)} variant="ghost" />
      </Sheet>
    </view>
  )
}

function Stat({ label, value, divider }: { label: string; value: string; divider?: boolean }) {
  return (
    <view className={divider ? 'rdet__stat rdet__stat--divider' : 'rdet__stat'}>
      <text className="rdet__eyebrow">{label.toUpperCase()}</text>
      <text className="rdet__stat-value">{value}</text>
    </view>
  )
}
