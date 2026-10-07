import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import { type Groceries, type GroceryAisle, getGroceries } from '../../../lib/api/groceries.js'
import { aisle } from '../../../lib/course.js'
import { t } from '../../../lib/i18n/index.js'
import { enqueue } from '../../../lib/offline/queue.js'
import { useResource } from '../../../lib/use-resource.js'
import { Button } from '../../../ui/Button.js'
import { Spinner } from '../../../ui/Spinner.js'
import './GroceriesTab.css'

type Filter = 'all' | 'todo' | 'done'

const SCROLL_ID = 'groc-scroll'

export interface GroceriesTabProps {
  /** Bumped by the shell when the tab is (re)selected or the app resumes. */
  refreshKey: number
  /** Switches the shell to the Recipes tab (empty state). */
  onAddRecipes: () => void
}

export function GroceriesTab({ refreshKey, onAddRecipes }: GroceriesTabProps) {
  // Cached locally: renders at once, refreshes in the background, and keeps
  // working offline (checks go through the offline queue).
  const { data: g, error, refresh } = useResource<Groceries>('groceries', getGroceries, [refreshKey])
  const [filter, setFilter] = useState<Filter>('all')
  const [closed, setClosed] = useState<Record<string, boolean>>({})
  const wasDone = useRef<Record<string, boolean> | null>(null)

  // Aisles start folded once every item is in the cart; one that just became
  // complete folds up, one that reopens unfolds.
  useEffect(() => {
    if (!g) return
    const prev = wasDone.current
    const done: Record<string, boolean> = {}
    g.aisles.forEach((a) => {
      done[a.key] = a.done
    })
    wasDone.current = done
    setClosed((c) => {
      const out = { ...c }
      g.aisles.forEach((a) => {
        const was = prev ? (prev[a.key] ?? false) : false
        if (a.done && !was) out[a.key] = true
        if (!a.done && was) delete out[a.key]
      })
      return out
    })
  }, [g])

  const toggle = useCallback(
    (key: string) => {
      if (!g) return
      let checked: boolean | null = null
      g.aisles.forEach((a) => {
        a.items.forEach((i) => {
          if (i.key === key) checked = i.checked
        })
      })
      if (checked === null) return
      // Optimistic: the queue patches the cached list and sends the change.
      void enqueue({ kind: 'grocery', key, checked: !checked })
    },
    [g]
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

  if (!g) {
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
