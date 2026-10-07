import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import { errorMessage } from '../../../../lib/api/client.js'
import {
  type Dish,
  type Entry,
  type KitchenList,
  type Overview,
  generate,
  getDish,
  getKitchen,
} from '../../../../lib/api/kitchen.js'
import type { Recipe, RecipeType, Status } from '../../../../lib/api/recipe.js'
import { createDraft } from '../../../../lib/api/recipes.js'
import { readDoc, revalidate, writeDoc } from '../../../../lib/cache.js'
import { course, minutes } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { openExternal, push } from '../../../../lib/nav.js'
import { enqueue } from '../../../../lib/offline/queue.js'
import { useResource } from '../../../../lib/use-resource.js'
import { Button } from '../../../../ui/Button.js'
import { OptionRow } from '../../../../ui/OptionRow.js'
import { Sheet } from '../../../../ui/Sheet.js'
import { Spinner } from '../../../../ui/Spinner.js'
import { GenerateSheet } from './GenerateSheet.js'
import './KitchenTab.css'

export interface KitchenTabProps {
  /** Bumped by the shell when the tab is (re)selected or the app resumes. */
  refreshKey: number
  /** Switches the shell to the Recipes tab (onboarding "Add recipes"). */
  onAddRecipes: () => void
}

export function KitchenTab({ refreshKey, onAddRecipes }: KitchenTabProps) {
  // Cached locally: renders at once and refreshes in the background.
  const { data, error, refresh } = useResource<Overview>('kitchen', getKitchen, [refreshKey])
  // What the tab shows: the cached/fresh overview, or it with another recipe
  // of the list in focus (a local choice the server does not keep).
  const [overview, setOverview] = useState<Overview | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [sheet, setSheet] = useState(false)
  const [menu, setMenu] = useState(false)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (data) setOverview(data)
  }, [data])

  const onGenerate = useCallback(async (count: number) => {
    setBusy(true)
    setNotice(null)
    try {
      const next = await generate(count)
      await writeDoc('kitchen', next)
      setSheet(false)
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [])

  // Removing goes through the offline queue: the cached list drops the entry
  // at once and the server learns about it when reachable.
  const onRemove = useCallback((id: string) => {
    setNotice(null)
    void enqueue({ kind: 'list', id, in_list: false })
  }, [])

  // Onboarding shortcuts: the same actions as the Recipes tab's + button.
  const onWrite = useCallback(async () => {
    if (busy) return
    setBusy(true)
    setNotice(null)
    try {
      const { id } = await createDraft()
      await push('recipe-edit', { id })
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [busy])

  const onImport = useCallback(() => {
    void push('recipe-import')
  }, [])

  const onFocus = useCallback(async (id: string) => {
    setBusy(true)
    setNotice(null)
    try {
      let dish: Dish | null
      try {
        dish = await revalidate<Dish>(`dish:${id}`, () => getDish(id))
      } catch (err) {
        // Offline: the dish seen before is good enough.
        dish = await readDoc<Dish>(`dish:${id}`)
        if (!dish) {
          setNotice(errorMessage(err))
          return
        }
      }
      const focused = dish
      setOverview((prev) => {
        if (!prev || prev.kind !== 'list') return prev
        const next: KitchenList = {
          ...prev,
          entries: focused.entries,
          focused: focused.recipe,
          focused_status: focused.status,
          steps: focused.steps,
          cook_external: focused.cook_external,
          external_url: focused.external_url,
        }
        return next
      })
    } finally {
      setBusy(false)
    }
  }, [])

  if (!overview) {
    if (error) {
      return (
        <view className="content">
          <view className="card">
            <text className="error">{error}</text>
            <Button label={t('common.retry')} onTap={refresh} variant="secondary" />
          </view>
        </view>
      )
    }
    return (
      <view className="content content--center">
        <Spinner size="lg" />
      </view>
    )
  }

  if (overview.kind === 'onboarding_recipe') {
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          {notice && <text className="error">{notice}</text>}
          <view className="kt-header__text">
            <text className="h1">{t('kitchen.hi_chef')}</text>
            <text className="muted">{t('kitchen.step_add_recipe')}</text>
          </view>
          <view className="card kt-empty">
            <view className="kt-empty__art">
              <text className="kt-empty__emoji">👨‍🍳</text>
            </view>
            <text className="kt-empty__title">{t('kitchen.onboarding_recipe_title')}</text>
            <text className="body kt-center">{t('kitchen.onboarding_recipe_hint')}</text>
            <Button label={t('kitchen.add_first_recipe')} onTap={onAddRecipes} variant="ink" block />
            <view className="kt-dots">
              <view className="kt-dot kt-dot--on" />
              <view className="kt-dot" />
            </view>
          </view>
          <text className="kt-eyebrow">{t('kitchen.three_ways')}</text>
          <view className="kt-options">
            <OptionRow
              emoji="✍️"
              tint="#d8ebdb"
              title={t('recipes.new')}
              hint={t('recipes.new_hint')}
              onTap={busy ? undefined : onWrite}
            />
            <OptionRow
              emoji="🔗"
              tint="#dde9f5"
              title={t('recipes.import_link')}
              hint={t('recipes.import_hint')}
              onTap={onImport}
            />
            <OptionRow
              emoji="👥"
              tint="#fde3cf"
              title={t('kitchen.browse_community')}
              hint={t('kitchen.browse_community_hint')}
              onTap={onAddRecipes}
            />
          </view>
        </view>
      </scroll-view>
    )
  }

  if (overview.kind === 'onboarding_menu') {
    const samples = overview.recipes.slice(0, 5)
    const extra = Math.max(0, overview.main_count - samples.length)
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          <view className="row">
            <Chip tone="herb" text={`✓ ${t('kitchen.step_recipes')}`} />
            <Chip tone="main" text={t('kitchen.step_list')} />
          </view>
          <view className="kt-header__text">
            <text className="h1">{t('kitchen.onboarding_menu_title')}</text>
            <text className="body">{t('kitchen.onboarding_menu_hint')}</text>
          </view>
          <view className="card">
            <text className="h2">{t('kitchen.collection')}</text>
            <view className="kt-counts">
              <Count n={overview.main_count} label={t('kitchen.count_mains')} type="MainCourse" />
              {overview.appetizer_count > 0 && (
                <Count n={overview.appetizer_count} label={t('kitchen.count_starters')} type="Appetizer" />
              )}
              {overview.accompaniment_count > 0 && (
                <Count n={overview.accompaniment_count} label={t('kitchen.count_sides')} type="Accompaniment" />
              )}
              {overview.dessert_count > 0 && (
                <Count n={overview.dessert_count} label={t('kitchen.count_desserts')} type="Dessert" />
              )}
            </view>
            <text className="kt-eyebrow">{t('kitchen.sample_mains')}</text>
            <view className="kt-samples">
              {samples.map((r) => (
                <text key={r.id} className="kt-sample">
                  {r.name}
                </text>
              ))}
              {extra > 0 && <text className="kt-sample kt-sample--more">{t('kitchen.more', { n: extra })}</text>}
            </view>
            <Button label={`✨ ${t('kitchen.generate')}`} onTap={() => setSheet(true)} variant="ink" block />
            <Button label={t('kitchen.add_by_hand')} onTap={onAddRecipes} variant="ghost" block />
          </view>
          <view className="kt-note">
            <view className="kt-note__icon">
              <text className="kt-note__glyph">🔥</text>
            </view>
            <text className="kt-note__text">
              <text className="kt-note__strong">{t('kitchen.next_title')} </text>
              {t('kitchen.next_hint')}
            </text>
          </view>
        </view>
        <GenerateSheet open={sheet} busy={busy} remaining={0} onClose={() => setSheet(false)} onGenerate={onGenerate} />
      </scroll-view>
    )
  }

  const list = overview
  // The server always focuses a recipe; `null` only comes from a list
  // rewritten offline when the focused recipe was removed. The rows still
  // render, the hero returns with the next refresh (or a tap on a row).
  const focused = list.focused
  const remaining = list.entries.filter((e) => e.status.status !== 'completed').length
  const allCooked = list.total_count > 0 && list.completed_count === list.total_count
  const others = [
    ...list.entries.filter((e) => e.status.status !== 'completed'),
    ...list.entries.filter((e) => e.status.status === 'completed'),
  ].filter((e) => e.id !== focused?.id)

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        {notice && <text className="error">{notice}</text>}
        {list.entries.length === 0 ? (
          <>
            <Header caption={t('kitchen.empty_caption')} />
            <Progress done={0} total={5} />
            <view className="card kt-empty">
              <view className="kt-empty__art">
                <text className="kt-empty__emoji">👨‍🍳</text>
              </view>
              <text className="kt-empty__title">{t('kitchen.nothing')}</text>
              <text className="body kt-center">{t('kitchen.nothing_hint')}</text>
              <Button label={`✨ ${t('kitchen.generate')}`} onTap={() => setSheet(true)} variant="ink" block />
              <Button label={t('kitchen.browse_recipes')} onTap={onAddRecipes} variant="ghost" />
            </view>
          </>
        ) : (
          <>
            <Header
              caption={t('kitchen.cooked_round', {
                done: list.completed_count,
                total: list.total_count,
              })}
              onRegenerate={() => setSheet(true)}
            />
            <Progress done={list.completed_count} total={list.total_count} />
            {allCooked && (
              <view className="kt-done">
                <text className="kt-done__text">✓ {t('kitchen.all_cooked')}</text>
              </view>
            )}
            {focused !== null && (
              <Hero
                recipe={focused}
                status={list.focused_status}
                cookExternal={list.cook_external}
                externalUrl={list.external_url}
                busy={busy}
                onMore={() => setMenu(true)}
              />
            )}
            <view className="kt-section">
              <text className="kt-eyebrow">
                {t('kitchen.your_list')} · {others.length}
              </text>
              <view className="kt-section__line" />
            </view>
            {others.length > 0 && <text className="muted">{t('kitchen.swipe_hint')}</text>}
            {others.map((entry) => (
              <Row
                key={entry.id}
                entry={entry}
                busy={busy}
                onFocus={() => onFocus(entry.id)}
                onRemove={() => onRemove(entry.id)}
              />
            ))}
            {list.prep_ahead.length > 0 && (
              <view className="kt-prep">
                <text className="kt-prep__title">⏰ {t('kitchen.prep_ahead')}</text>
                {list.prep_ahead.map((e) => (
                  <text key={e.id} className="kt-prep__text">
                    <text className="kt-prep__name">{e.name}</text> · {e.advance_prep}
                  </text>
                ))}
              </view>
            )}
            {focused !== null && (
              <Sheet open={menu} onClose={() => setMenu(false)}>
                <text className="h2">{focused.name}</text>
                <Button
                  label={t('kitchen.remove')}
                  onTap={() => {
                    setMenu(false)
                    onRemove(focused.id)
                  }}
                  variant="danger"
                  disabled={busy}
                  block
                />
                <Button label={t('common.cancel')} onTap={() => setMenu(false)} variant="ghost" />
              </Sheet>
            )}
          </>
        )}
      </view>
      <GenerateSheet
        open={sheet}
        busy={busy}
        remaining={remaining}
        onClose={() => setSheet(false)}
        onGenerate={onGenerate}
      />
    </scroll-view>
  )
}

function Header({ caption, onRegenerate }: { caption: string; onRegenerate?: () => void }) {
  return (
    <view className="kt-header">
      <view className="kt-header__text">
        <text className="h1">{t('kitchen.up_next')}</text>
        <text className="muted">{caption}</text>
      </view>
      {onRegenerate && (
        <view className="kt-regen" bindtap={onRegenerate}>
          <text className="kt-regen__glyph">✨</text>
        </view>
      )}
    </view>
  )
}

/** One segment per meal in the round, filled once cooked. */
function Progress({ done, total }: { done: number; total: number }) {
  const segments = Array.from({ length: Math.max(total, 1) }, (_, i) => i < done)
  return (
    <view className="kt-progress">
      {segments.map((on, i) => (
        <view key={i} className={on ? 'kt-progress__seg kt-progress__seg--on' : 'kt-progress__seg'} />
      ))}
    </view>
  )
}

function Chip({ tone, text }: { tone: 'herb' | 'main'; text: string }) {
  return (
    <view className={`kt-chip kt-chip--${tone}`}>
      <text className={`kt-chip__text kt-chip__text--${tone}`}>{text}</text>
    </view>
  )
}

function Count({ n, label, type }: { n: number; label: string; type: RecipeType }) {
  const c = course(type)
  return (
    <view className="kt-count" style={{ backgroundColor: c.soft }}>
      <text className="kt-count__n" style={{ color: c.ink }}>
        {n}
      </text>
      <text className="kt-count__label" style={{ color: c.ink }}>
        {label}
      </text>
    </view>
  )
}

interface HeroProps {
  recipe: Recipe
  status: Status
  cookExternal: boolean
  externalUrl: string | null
  busy: boolean
  onMore: () => void
}

function Hero({ recipe, status, cookExternal, externalUrl, busy, onMore }: HeroProps) {
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
    <view className="kt-hero">
      <view className="kt-hero__band" style={{ backgroundColor: c.soft }}>
        <view className="kt-hero__tile">
          <text className="kt-hero__tile-emoji">{c.emoji}</text>
        </view>
        <view className="kt-hero__band-text">
          <text className="kt-hero__course" style={{ color: c.ink }}>
            {c.label.toUpperCase()}
          </text>
          <text className="kt-hero__meta" style={{ color: c.ink }}>
            ⏱ {minutes(recipe.total_time)} 👥 {t('kitchen.servings', { n: recipe.household_size })}
          </text>
        </view>
        {status.status === 'completed' && <Chip tone="herb" text={`✓ ${t('kitchen.cooked_badge')}`} />}
        {status.status === 'cooking' && <Chip tone="main" text={`🔥 ${t('kitchen.cooking')}`} />}
      </view>
      <view className="kt-hero__body">
        <text className="kt-hero__title">{recipe.name}</text>
        {recipe.description !== '' && <text className="kt-hero__desc">{recipe.description}</text>}
        <view className="kt-hero__actions">
          <view className="kt-cta" bindtap={busy ? undefined : startCooking}>
            <text className="kt-cta__text">🔥 {cta}</text>
          </view>
          <view className="kt-iconbtn" bindtap={() => void push('recipe', { id: recipe.id })}>
            <text className="kt-iconbtn__glyph">📖</text>
          </view>
          <view className="kt-iconbtn" bindtap={busy ? undefined : onMore}>
            <text className="kt-iconbtn__glyph">⋯</text>
          </view>
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

/** Width of the remove action revealed by swiping a row left. */
const REVEAL = 88
/** Horizontal travel before a touch counts as a swipe rather than a tap. */
const SLOP = 8

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n))

/**
 * A list entry: tap to focus it, swipe left to reveal "Remove". The offset is
 * plain state, so the row follows the finger through the background thread;
 * the snap at the end animates through CSS.
 */
function Row({ entry, busy, onFocus, onRemove }: RowProps) {
  const c = course(entry.recipe_type)
  const done = entry.status.status === 'completed'
  const cooking = entry.status.status === 'cooking'
  const [dx, setDx] = useState(0)
  const [open, setOpen] = useState(false)
  const [dragging, setDragging] = useState(false)
  const start = useRef<{ x: number; dx: number } | null>(null)
  const moved = useRef(false)

  const onTouchStart = (e: { touches: { clientX: number }[] }) => {
    const touch = e.touches[0]
    if (!touch) return
    start.current = { x: touch.clientX, dx: open ? -REVEAL : 0 }
    moved.current = false
  }
  const onTouchMove = (e: { touches: { clientX: number }[] }) => {
    const touch = e.touches[0]
    if (!touch || !start.current) return
    const delta = touch.clientX - start.current.x
    if (Math.abs(delta) > SLOP) {
      moved.current = true
      setDragging(true)
    }
    setDx(clamp(start.current.dx + delta, -REVEAL, 0))
  }
  const onTouchEnd = () => {
    if (!start.current) return
    start.current = null
    const snap = dx < -REVEAL / 2
    setOpen(snap)
    setDx(snap ? -REVEAL : 0)
    setDragging(false)
  }
  const onTap = () => {
    // A swipe ends with a tap on some hosts; never treat it as a focus.
    if (moved.current) {
      moved.current = false
      return
    }
    if (open) {
      setOpen(false)
      setDx(0)
      return
    }
    if (!busy) onFocus()
  }

  const classes = ['kt-row']
  if (done) classes.push('kt-row--done')
  if (!dragging) classes.push('kt-row--snap')

  return (
    <view className="kt-swipe">
      <view className="kt-swipe__action" bindtap={busy ? undefined : onRemove}>
        <text className="kt-swipe__glyph">🗑</text>
        <text className="kt-swipe__text">{t('kitchen.remove_short')}</text>
      </view>
      <view
        className={classes.join(' ')}
        style={{ transform: `translateX(${dx}px)` }}
        bindtap={onTap}
        bindtouchstart={onTouchStart}
        bindtouchmove={onTouchMove}
        bindtouchend={onTouchEnd}
        bindtouchcancel={onTouchEnd}
      >
        <view className="kt-row__tile" style={{ backgroundColor: c.soft }}>
          <text className="kt-row__emoji">{c.emoji}</text>
        </view>
        <view className="kt-row__body">
          <text className={done ? 'kt-row__name kt-row__name--done' : 'kt-row__name'} text-maxline="1">
            {entry.name}
          </text>
          <text className="kt-row__meta">
            {c.label} · {minutes(entry.total_time)}
          </text>
        </view>
        {done && <Chip tone="herb" text={`✓ ${t('kitchen.cooked_badge')}`} />}
        {cooking && <Chip tone="main" text={`🔥 ${t('kitchen.cooking')}`} />}
        <text className="kt-row__chevron">›</text>
      </view>
    </view>
  )
}
