import { useCallback, useEffect, useState } from '@lynx-js/react'

import { ApiError } from '../../../../lib/api/client.js'
import {
  type Entry,
  type KitchenList,
  type Overview,
  generate,
  getDish,
  getKitchen,
  removeFromList,
} from '../../../../lib/api/kitchen.js'
import type { Recipe, Status } from '../../../../lib/api/recipe.js'
import { course, minutes } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { openExternal, push } from '../../../../lib/nav.js'
import { Button } from '../../../../ui/Button.js'
import { GenerateSheet } from './GenerateSheet.js'
import './KitchenTab.css'

type State =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; overview: Overview }

export interface KitchenTabProps {
  /** Bumped by the shell when the tab is (re)selected or the app resumes. */
  refreshKey: number
}

export function KitchenTab({ refreshKey }: KitchenTabProps) {
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [sheet, setSheet] = useState(false)
  const [busy, setBusy] = useState(false)

  const fail = (err: unknown) =>
    setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') })

  const load = useCallback(() => {
    getKitchen()
      .then((overview) => setState({ kind: 'ready', overview }))
      .catch(fail)
  }, [])

  useEffect(() => {
    load()
  }, [load, refreshKey])

  const onGenerate = useCallback(async (count: number) => {
    setBusy(true)
    try {
      const overview = await generate(count)
      setState({ kind: 'ready', overview })
      setSheet(false)
    } catch (err) {
      fail(err)
    } finally {
      setBusy(false)
    }
  }, [])

  const onRemove = useCallback(
    async (id: string) => {
      setBusy(true)
      try {
        await removeFromList(id)
        load()
      } catch (err) {
        fail(err)
      } finally {
        setBusy(false)
      }
    },
    [load],
  )

  const onFocus = useCallback(async (id: string) => {
    setBusy(true)
    try {
      const dish = await getDish(id)
      setState((prev) => {
        if (prev.kind !== 'ready' || prev.overview.kind !== 'list') return prev
        const overview: KitchenList = {
          ...prev.overview,
          entries: dish.entries,
          focused: dish.recipe,
          focused_status: dish.status,
          steps: dish.steps,
          cook_external: dish.cook_external,
          external_url: dish.external_url,
        }
        return { kind: 'ready', overview }
      })
    } catch (err) {
      fail(err)
    } finally {
      setBusy(false)
    }
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

  const { overview } = state

  if (overview.kind === 'onboarding_recipe') {
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          <text className="kitchen__hi">👨‍🍳</text>
          <text className="h1">{t('kitchen.onboarding_recipe_title')}</text>
          <text className="body">{t('kitchen.onboarding_recipe_hint')}</text>
          <Button label={t('kitchen.add_recipes')} onTap={() => {}} />
        </view>
      </scroll-view>
    )
  }

  if (overview.kind === 'onboarding_menu') {
    const parts = [t('kitchen.main_courses', { n: overview.main_count })]
    if (overview.accompaniment_count > 0) parts.push(t('kitchen.sides', { n: overview.accompaniment_count }))
    if (overview.dessert_count > 0) parts.push(t('kitchen.desserts', { n: overview.dessert_count }))
    if (overview.appetizer_count > 0) parts.push(t('kitchen.starters', { n: overview.appetizer_count }))
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          <text className="h1">{t('kitchen.onboarding_menu_title')}</text>
          <text className="body">{t('kitchen.onboarding_menu_hint')}</text>
          <view className="card">
            <text className="h2">{t('kitchen.collection')}</text>
            <text className="muted">{parts.join(' · ')}</text>
            <view className="kitchen__samples">
              {overview.recipes.slice(0, 5).map((r) => (
                <text key={r.id} className="kitchen__sample">
                  {r.name}
                </text>
              ))}
            </view>
            <Button label={t('kitchen.generate')} onTap={() => setSheet(true)} block />
          </view>
        </view>
        <GenerateSheet open={sheet} busy={busy} onClose={() => setSheet(false)} onGenerate={onGenerate} />
      </scroll-view>
    )
  }

  const list = overview
  const focused = list.focused

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        {focused === null ? (
          <view className="card kitchen__empty">
            <text className="kitchen__hi">👨‍🍳</text>
            <text className="h2">{t('kitchen.nothing')}</text>
            <text className="body">{t('kitchen.nothing_hint')}</text>
            <Button label={t('kitchen.generate')} onTap={() => setSheet(true)} block />
          </view>
        ) : (
          <>
            <view className="kitchen__header">
              <text className="h1">{t('kitchen.up_next')}</text>
              <text className="muted">
                {list.completed_count === list.total_count
                  ? t('kitchen.all_cooked')
                  : t('kitchen.cooked', { done: list.completed_count, total: list.total_count })}
              </text>
            </view>
            <view className="kitchen__regen" bindtap={() => setSheet(true)}>
              <text className="kitchen__regen-text">✨ {t('kitchen.regenerate')}</text>
            </view>
            <Hero
              recipe={focused}
              status={list.focused_status}
              cookExternal={list.cook_external}
              externalUrl={list.external_url}
              busy={busy}
              onRemove={() => onRemove(focused.id)}
            />
            <view className="kitchen__section">
              <text className="kitchen__section-title">
                {t('kitchen.your_list')} · {list.entries.length}
              </text>
              <view className="kitchen__line" />
            </view>
            {[...list.entries.filter((e) => e.status.status !== 'completed'), ...list.entries.filter((e) => e.status.status === 'completed')]
              .filter((e) => e.id !== focused.id)
              .map((entry) => (
                <Row
                  key={entry.id}
                  entry={entry}
                  busy={busy}
                  onFocus={() => onFocus(entry.id)}
                  onRemove={() => onRemove(entry.id)}
                />
              ))}
            {list.prep_ahead.length > 0 && (
              <view className="card">
                <text className="h2">{t('kitchen.prep_ahead')}</text>
                <text className="muted">{t('kitchen.prep_ahead_hint')}</text>
                {list.prep_ahead.map((e) => (
                  <view key={e.id} className="kitchen__prep">
                    <text className="kitchen__prep-text">
                      ⏰ {e.name}: {e.advance_prep}
                    </text>
                  </view>
                ))}
              </view>
            )}
          </>
        )}
      </view>
      <GenerateSheet open={sheet} busy={busy} onClose={() => setSheet(false)} onGenerate={onGenerate} />
    </scroll-view>
  )
}

interface HeroProps {
  recipe: Recipe
  status: Status
  cookExternal: boolean
  externalUrl: string | null
  busy: boolean
  onRemove: () => void
}

function Hero({ recipe, status, cookExternal, externalUrl, busy, onRemove }: HeroProps) {
  const c = course(recipe.recipe_type)
  const cta =
    status.status === 'cooking'
      ? t('kitchen.continue')
      : status.status === 'completed'
        ? t('kitchen.again')
        : t('kitchen.start')

  const startCooking = () => {
    if (cookExternal && externalUrl) openExternal(externalUrl)
    else void push('cooking', { id: recipe.id })
  }

  return (
    <view className="hero" style={{ backgroundColor: c.soft }}>
      <view className="hero__pills">
        <view className="pill" style={{ backgroundColor: c.soft }}>
          <text className="pill__text" style={{ color: c.ink }}>
            {c.emoji} {c.label}
          </text>
        </view>
        <view className="pill pill--paper">
          <text className="pill__text" style={{ color: c.ink }}>
            ⏱ {minutes(recipe.total_time)}
          </text>
        </view>
        <view className="pill pill--paper">
          <text className="pill__text" style={{ color: c.ink }}>
            👥 {recipe.household_size}
          </text>
        </view>
        {status.status === 'completed' && (
          <view className="pill pill--herb">
            <text className="pill__text pill__text--light">✓ {t('kitchen.cooked_badge')}</text>
          </view>
        )}
        {status.status === 'cooking' && (
          <view className="pill pill--ink">
            <text className="pill__text pill__text--light">{t('kitchen.cooking')}</text>
          </view>
        )}
      </view>
      <text className="hero__title">{recipe.name}</text>
      {recipe.description !== '' && <text className="hero__desc">{recipe.description}</text>}
      <view className="hero__actions">
        <view className="hero__cta" bindtap={busy ? undefined : startCooking}>
          <text className="hero__cta-text">{cta}</text>
        </view>
        <view className="hero__remove" bindtap={busy ? undefined : onRemove}>
          <text className="hero__remove-text">✕</text>
        </view>
      </view>
    </view>
  )
}

interface RowProps {
  entry: Entry
  busy: boolean
  onFocus: () => void
  onRemove: () => void
}

function Row({ entry, busy, onFocus, onRemove }: RowProps) {
  const c = course(entry.recipe_type)
  const done = entry.status.status === 'completed'
  return (
    <view className="row-wrap">
      <view className={done ? 'entry entry--done' : 'entry'} bindtap={busy ? undefined : onFocus}>
        <view className="entry__icon" style={{ backgroundColor: c.soft }}>
          <text className="entry__emoji">{c.emoji}</text>
        </view>
        <view className="entry__body">
          <text className="entry__course" style={{ color: c.ink }}>
            {c.label.toUpperCase()}
          </text>
          <text className={done ? 'entry__name entry__name--done' : 'entry__name'}>{entry.name}</text>
          <text className="entry__time">⏱ {minutes(entry.total_time)}</text>
        </view>
        <text className="entry__status" style={{ color: c.ink }}>
          {done
            ? t('kitchen.cooked_badge')
            : entry.status.status === 'cooking'
              ? t('kitchen.cooking')
              : t('kitchen.focus')}
        </text>
      </view>
      <view className="entry__remove" bindtap={busy ? undefined : onRemove}>
        <text className="entry__remove-text">✕</text>
      </view>
    </view>
  )
}
