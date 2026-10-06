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
        <text className="muted">{t('common.loading')}</text>
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

  return (
    <view className="screen">
      <scroll-view className="rdet__scroll" scroll-orientation="vertical">
        <view className="content">
          <view className="row rdet__bar">
            <view className="cook__close" bindtap={back}>
              <text className="cook__close-text">←</text>
            </view>
            <text className="muted">
              {t('tabs.recipes')} / {c.label}
            </text>
          </view>

          <RecipeImage
            thumbnailUrl={r.thumbnail_url}
            blurPlaceholder={r.blur_placeholder}
            recipeType={r.recipe_type}
            size="hero"
          />

          <view className="hero__pills">
            <view className="pill" style={{ backgroundColor: c.soft }}>
              <text className="pill__text" style={{ color: c.ink }}>
                {c.emoji} {c.label}
              </text>
            </view>
            <view className="pill pill--line">
              <text className="pill__text">⏱ {minutes(r.total_time)}</text>
            </view>
            <view className="pill pill--line">
              <text className="pill__text">👥 {r.household_size}</text>
            </view>
            {r.is_shared && !r.is_owner && (
              <view className="pill pill--line">
                <text className="pill__text">{t('recipes.shared')}</text>
              </view>
            )}
            {r.dietary_restrictions.map((d) => (
              <view key={d} className="pill pill--line">
                <text className="pill__text">{t(`diet.${d}` as const)}</text>
              </view>
            ))}
          </view>

          <text className="h1">{r.name}</text>
          {r.description !== '' && <text className="body">{r.description}</text>}
          {r.owner_name && (
            <text className="link" bindtap={() => void push('cook', { username: r.owner_name! })}>
              @{r.owner_name}
            </text>
          )}
          {r.origin && (
            <text className="link" bindtap={() => openExternal(r.origin!)}>
              {t('recipes.origin')}
            </text>
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
              />
            )}
            {r.is_owner && (
              <Button
                label={t('recipes.edit')}
                onTap={() => void push('recipe-edit', { id: r.id })}
                variant="secondary"
              />
            )}
            {r.is_owner && isChef && (
              <Button
                label={r.is_shared ? t('recipes.make_private') : t('recipes.make_public')}
                onTap={() =>
                  run(
                    () => (r.is_shared ? unshareRecipe(r.id) : shareRecipe(r.id)),
                    (x) => ({ ...x, is_shared: !x.is_shared }),
                  )
                }
                variant="secondary"
                disabled={busy}
              />
            )}
            {r.is_owner && (
              <Button
                label={t('recipes.delete')}
                onTap={() => setConfirmDelete(true)}
                variant="ghost"
                disabled={busy}
              />
            )}
          </view>

          {r.advance_prep.trim() !== '' && (
            <view className="kitchen__prep">
              <text className="kitchen__prep-text">⏰ {r.advance_prep}</text>
            </view>
          )}

          <view className="card">
            <text className="h2">
              {t('recipes.ingredients')} · {r.ingredients.length}
            </text>
            {r.ingredients.map((i) => (
              <view key={i.key} className="row rdet__ing">
                <text className="rdet__ing-name">{i.name}</text>
                <text className="ing__qty">{i.quantity_label}</text>
              </view>
            ))}
          </view>

          {r.instructions.length > 0 && (
            <view className="card">
              <text className="h2">{t('recipes.instructions')}</text>
              {r.instructions.map((ins, index) => (
                <view key={index} className="row rdet__step">
                  <view className="rdet__num" style={{ backgroundColor: c.soft }}>
                    <text className="rdet__num-text" style={{ color: c.ink }}>
                      {index + 1}
                    </text>
                  </view>
                  <view className="rdet__step-body">
                    <text className="body">{ins.description}</text>
                    {ins.time_next > 0 && <text className="muted">⏲ {minutes(ins.time_next)}</text>}
                  </view>
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
        <Button label={t('recipes.delete')} onTap={remove} block />
        <Button label={t('common.cancel')} onTap={() => setConfirmDelete(false)} variant="ghost" />
      </Sheet>
    </view>
  )
}
