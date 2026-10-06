import { useCallback, useEffect, useState } from '@lynx-js/react'

import { ApiError } from '../../../lib/api/client.js'
import { type Groceries, getGroceries, toggleGrocery } from '../../../lib/api/groceries.js'
import { aisle } from '../../../lib/course.js'
import { t } from '../../../lib/i18n/index.js'
import { Button } from '../../../ui/Button.js'
import './GroceriesTab.css'

type State =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; groceries: Groceries }

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
    progress_pct:
      groceries.total_items > 0 ? Math.floor((checked_items * 100) / groceries.total_items) : 0,
  }
}

export function GroceriesTab({ refreshKey }: { refreshKey: number }) {
  const [state, setState] = useState<State>({ kind: 'loading' })

  const load = useCallback(() => {
    getGroceries()
      .then((groceries) => setState({ kind: 'ready', groceries }))
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
      setState((prev) => (prev.kind === 'ready' ? { kind: 'ready', groceries: flip(prev.groceries, key) } : prev))
      toggleGrocery(key).catch(() => load())
    },
    [load],
  )

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

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        <view className="groc__header">
          <text className="cook__eyebrow">{t('tabs.groceries')}</text>
          <text className="h1">{t('groceries.title')}</text>
          {g.aisles.length > 0 && (
            <text className="muted">
              {t('groceries.summary', {
                checked: g.checked_items,
                total: g.total_items,
                recipes: g.recipe_count,
              })}
            </text>
          )}
        </view>

        {g.aisles.length === 0 ? (
          <view className="card groc__empty">
            <text className="kitchen__hi">🛒</text>
            <text className="h2">{t('groceries.empty')}</text>
            <text className="body">{t('groceries.empty_hint')}</text>
          </view>
        ) : (
          <>
            <view className="groc__progress">
              <view className="groc__progress-fill" style={{ width: `${g.progress_pct}%` }} />
            </view>
            {g.aisles.map((a) => {
              const s = aisle(a.key)
              return (
                <view key={a.key} className={a.done ? 'aisle aisle--done' : 'aisle'}>
                  <view className="aisle__head">
                    <view className="aisle__icon" style={{ backgroundColor: s.soft }}>
                      <text className="aisle__emoji">{s.emoji}</text>
                    </view>
                    <view className="aisle__meta">
                      <text className="aisle__label">{s.label}</text>
                      <text className="muted">
                        {a.checked}/{a.total}
                      </text>
                    </view>
                    {a.done && <text className="aisle__done">✓</text>}
                  </view>
                  {a.items.map((i) => (
                    <view key={i.key} className="ing" bindtap={() => toggle(i.key)}>
                      <view className={i.checked ? 'ing__box ing__box--on' : 'ing__box'}>
                        {i.checked && <text className="ing__check">✓</text>}
                      </view>
                      <text className={i.checked ? 'ing__name ing__name--on' : 'ing__name'}>{i.name}</text>
                      <text className="ing__qty">{i.quantity_label}</text>
                    </view>
                  ))}
                </view>
              )
            })}
          </>
        )}
      </view>
    </scroll-view>
  )
}
