import { useCallback, useEffect, useState } from '@lynx-js/react'

import { ApiError } from '../../../lib/api/client.js'
import { type Groceries, type GroceryAisle, getGroceries, toggleGrocery } from '../../../lib/api/groceries.js'
import { aisle } from '../../../lib/course.js'
import { t } from '../../../lib/i18n/index.js'
import { Button } from '../../../ui/Button.js'
import './GroceriesTab.css'

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; groceries: Groceries }

type Filter = 'all' | 'todo' | 'done'

const SCROLL_ID = 'groc-scroll'

/** Recomputes the counters after flipping one item locally. */
function flip(groceries: Groceries, key: string): Groceries {
  let delta = 0
  const aisles = groceries.aisles.map((a) => {
    const items = a.items.map((i) => {
      if (i.key !== key) return i
      delta = i.checked ? -1 : 1
      return { ...i, checked: !i.checked }
    })
    const checked = items.filter((i) => i.checked).length
    return {
      ...a,
      items,
      checked,
      done: a.total > 0 && checked === a.total,
      pct: a.total > 0 ? Math.floor((checked * 100) / a.total) : 0,
    }
  })
  const checked_items = groceries.checked_items + delta
  return {
    ...groceries,
    aisles,
    checked_items,
    progress_pct: groceries.total_items > 0 ? Math.floor((checked_items * 100) / groceries.total_items) : 0,
  }
}

/** Aisles start folded once every item is in the cart. */
function foldedAisles(groceries: Groceries): Record<string, boolean> {
  const closed: Record<string, boolean> = {}
  for (const a of groceries.aisles) if (a.done) closed[a.key] = true
  return closed
}

export interface GroceriesTabProps {
  /** Bumped by the shell when the tab is (re)selected or the app resumes. */
  refreshKey: number
  /** Switches the shell to the Recipes tab (empty state). */
  onAddRecipes: () => void
}

export function GroceriesTab({ refreshKey, onAddRecipes }: GroceriesTabProps) {
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [filter, setFilter] = useState<Filter>('all')
  const [closed, setClosed] = useState<Record<string, boolean>>({})

  const load = useCallback(() => {
    getGroceries()
      .then((groceries) => {
        setClosed(foldedAisles(groceries))
        setState({ kind: 'ready', groceries })
      })
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

  const toggle = useCallback(
    (key: string) => {
      // Optimistic: flip locally, reconcile with the server afterwards.
      setState((prev) => {
        if (prev.kind !== 'ready') return prev
        const next = flip(prev.groceries, key)
        // An aisle that just became complete folds up; one that reopens unfolds.
        setClosed((c) => {
          const out = { ...c }
          next.aisles.forEach((a, i) => {
            const was = prev.groceries.aisles[i]?.done ?? false
            if (a.done && !was) out[a.key] = true
            if (!a.done && was) delete out[a.key]
          })
          return out
        })
        return { kind: 'ready', groceries: next }
      })
      toggleGrocery(key).catch(() => load())
    },
    [load],
  )

  const toggleOpen = useCallback((key: string) => {
    setClosed((c) => {
      const out = { ...c }
      if (out[key]) delete out[key]
      else out[key] = true
      return out
    })
  }, [])

  const jumpTo = useCallback((index: number) => {
    // Aisles are direct children of the scroll view, after the top block.
    lynx
      .createSelectorQuery()
      .select(`#${SCROLL_ID}`)
      .invoke({ method: 'scrollTo', params: { index: index + 1, offset: -12, smooth: true } })
      .exec()
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

  const g = state.groceries

  if (g.aisles.length === 0) {
    return (
      <scroll-view className="tab-scroll" scroll-orientation="vertical">
        <view className="content">
          <Header />
          <view className="card groc__empty">
            <view className="groc__empty-art">
              <text className="groc__empty-emoji">🛒</text>
            </view>
            <text className="groc__empty-title">{t('groceries.empty')}</text>
            <text className="body groc__center">{t('groceries.empty_hint')}</text>
            <Button label={t('groceries.go_recipes')} onTap={onAddRecipes} variant="ink" block />
          </view>
          <view className="groc__features">
            <Feature glyph="≡" tone="entree" text={t('groceries.feature_aisles')} />
            <Feature glyph="✓" tone="herb" text={t('groceries.feature_merged')} />
          </view>
        </view>
      </scroll-view>
    )
  }

  const left = g.total_items - g.checked_items
  const allDone = g.total_items > 0 && left === 0
  const shown = g.aisles
    .map((a) => ({
      ...a,
      items:
        filter === 'all'
          ? a.items
          : filter === 'todo'
            ? a.items.filter((i) => !i.checked)
            : a.items.filter((i) => i.checked),
    }))
    .filter((a) => a.items.length > 0)

  return (
    <scroll-view id={SCROLL_ID} className="tab-scroll groc__scroll" scroll-orientation="vertical">
      <view className="groc__top">
        <Header caption={t('groceries.count', { total: g.total_items, recipes: g.recipe_count })} />

        <view className="groc__progress">
          <text className="groc__progress-title">
            {t('groceries.progress', { checked: g.checked_items, total: g.total_items })}
          </text>
          <text className="muted">
            {left === 0
              ? t('groceries.left_none')
              : left === 1
                ? t('groceries.left_one')
                : t('groceries.left', { n: left })}
          </text>
          <view className="groc__bar">
            <view className="groc__bar-fill" style={{ width: `${g.progress_pct}%` }} />
          </view>
          <view className="groc__seg">
            <Seg label={t('groceries.filter_all')} active={filter === 'all'} onTap={() => setFilter('all')} />
            <Seg label={t('groceries.filter_todo')} active={filter === 'todo'} onTap={() => setFilter('todo')} />
            <Seg label={t('groceries.filter_done')} active={filter === 'done'} onTap={() => setFilter('done')} />
          </view>
        </view>

        {allDone && (
          <view className="groc__done">
            <text className="groc__done-text">✓ {t('groceries.all_done')}</text>
          </view>
        )}

        <scroll-view className="groc__jumps" scroll-orientation="horizontal">
          {g.aisles.map((a) => {
            const s = aisle(a.key)
            const index = shown.findIndex((x) => x.key === a.key)
            return (
              <view key={a.key} className="groc__jump" bindtap={() => index >= 0 && jumpTo(index)}>
                <text className="groc__jump-emoji">{s.emoji}</text>
                <text className="groc__jump-label">{s.label}</text>
                <view className="groc__jump-count" style={a.done ? undefined : { backgroundColor: s.soft }}>
                  <text className="groc__jump-count-text" style={a.done ? undefined : { color: s.hex }}>
                    {a.done ? '✓' : a.total - a.checked}
                  </text>
                </view>
              </view>
            )
          })}
        </scroll-view>

        {shown.length === 0 && (
          <text className="muted groc__center groc__none">
            {filter === 'todo' ? t('groceries.none_todo') : t('groceries.none_done')}
          </text>
        )}
      </view>

      {shown.map((a) => (
        <Aisle key={a.key} aisle={a} open={!closed[a.key]} onToggleOpen={() => toggleOpen(a.key)} onToggle={toggle} />
      ))}
      <view className="groc__tail" />
    </scroll-view>
  )
}

function Header({ caption }: { caption?: string }) {
  return (
    <view className="groc__header">
      <text className="groc__eyebrow">{t('tabs.groceries').toUpperCase()}</text>
      <text className="h1">{t('groceries.title')}</text>
      {caption && <text className="muted">{caption}</text>}
    </view>
  )
}

function Seg({ label, active, onTap }: { label: string; active: boolean; onTap: () => void }) {
  return (
    <view className={active ? 'groc__seg-item groc__seg-item--on' : 'groc__seg-item'} bindtap={onTap}>
      <text className={active ? 'groc__seg-text groc__seg-text--on' : 'groc__seg-text'}>{label}</text>
    </view>
  )
}

function Feature({ glyph, tone, text }: { glyph: string; tone: 'entree' | 'herb'; text: string }) {
  return (
    <view className="groc__feature">
      <view className="groc__feature-icon">
        <text className={`groc__feature-glyph groc__feature-glyph--${tone}`}>{glyph}</text>
      </view>
      <text className="groc__feature-text">{text}</text>
    </view>
  )
}

function Aisle({
  aisle: a,
  open,
  onToggleOpen,
  onToggle,
}: {
  aisle: GroceryAisle
  open: boolean
  onToggleOpen: () => void
  onToggle: (key: string) => void
}) {
  const s = aisle(a.key)
  return (
    <view className={a.done ? 'aisle groc__aisle aisle--done' : 'aisle groc__aisle'}>
      <view className={open ? 'aisle__head' : 'aisle__head aisle__head--closed'} bindtap={onToggleOpen}>
        <view className="aisle__icon" style={{ backgroundColor: s.soft }}>
          <text className="aisle__emoji">{s.emoji}</text>
        </view>
        <view className="aisle__meta">
          <text className="aisle__label">{s.label}</text>
          <text className="aisle__sub">
            {a.done
              ? t('groceries.aisle_done_meta', { total: a.total })
              : t('groceries.aisle_meta', { left: a.total - a.checked, checked: a.checked, total: a.total })}
          </text>
        </view>
        {a.done && (
          <view className="aisle__done-chip">
            <text className="aisle__done-chip-text">{t('groceries.done')}</text>
          </view>
        )}
        <text className={open ? 'aisle__chevron' : 'aisle__chevron aisle__chevron--closed'}>›</text>
      </view>
      {open &&
        a.items.map((i) => (
          <view key={i.key} className="ing" bindtap={() => onToggle(i.key)}>
            <view className={i.checked ? 'ing__box ing__box--on' : 'ing__box'}>
              {i.checked && <text className="ing__check">✓</text>}
            </view>
            <text className={i.checked ? 'ing__name ing__name--on' : 'ing__name'}>{i.name}</text>
            <text className="ing__qty">{i.quantity_label}</text>
          </view>
        ))}
    </view>
  )
}
