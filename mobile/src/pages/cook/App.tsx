import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import { ApiError } from '../../lib/api/client.js'
import { type Cook, type Summary, getCook } from '../../lib/api/recipes.js'
import { t } from '../../lib/i18n/index.js'
import { back, pageParams, push } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { RecipeCard } from '../../ui/RecipeCard.js'
import './App.css'

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; cook: Cook }

/** A chef's public recipes: `cook.lynx.bundle?username=…`. */
export function App() {
  const username = pageParams().username ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [items, setItems] = useState<Summary[]>([])
  const [cursor, setCursor] = useState<string | null>(null)
  const [loadingMore, setLoadingMore] = useState(false)

  useEffect(() => {
    getCook(username)
      .then((cook) => {
        setState({ kind: 'ready', cook })
        setItems(cook.recipes.edges.map((e) => e.node))
        setCursor(cook.recipes.page_info.has_next_page ? cook.recipes.page_info.end_cursor : null)
      })
      .catch((err: unknown) =>
        setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') }),
      )
  }, [username])

  // Infinite scroll: fired by the scroll-view when the bottom comes within
  // `lower-threshold` px, like the web's "visible" sentinel.
  const loadMore = useCallback(async () => {
    if (!cursor || loadingMore) return
    setLoadingMore(true)
    try {
      const cook = await getCook(username, cursor)
      setItems((prev) => [...prev, ...cook.recipes.edges.map((e) => e.node)])
      setCursor(cook.recipes.page_info.has_next_page ? cook.recipes.page_info.end_cursor : null)
    } finally {
      setLoadingMore(false)
    }
  }, [cursor, loadingMore, username])

  if (state.kind === 'loading') {
    return (
      <view className="screen cook--center">
        <text className="muted">{t('common.loading')}</text>
      </view>
    )
  }
  if (state.kind === 'error') {
    return (
      <view className="screen cook--center">
        <view className="card">
          <text className="error">{state.message}</text>
          <Button label={t('common.close')} onTap={back} variant="secondary" />
        </view>
      </view>
    )
  }

  const { cook } = state
  return (
    <scroll-view
      className="screen"
      scroll-orientation="vertical"
      lower-threshold={400}
      bindscrolltolower={() => void loadMore()}
    >
      <view className="content">
        <view className="row" style={{ gap: '12px' }}>
          <view className="cook__close" bindtap={back}>
            <text className="cook__close-text">←</text>
          </view>
          <text className="muted">{t('cook.eyebrow')}</text>
        </view>
        <text className="h1">@{cook.username}</text>
        {cook.description !== '' && <text className="body">{cook.description}</text>}
        <text className="muted">{t('cook.stats', { shared: cook.stat.shared, total: cook.stat.total })}</text>
        {items.map((recipe) => (
          <RecipeCard key={recipe.id} recipe={recipe} onTap={() => void push('recipe', { id: recipe.id })} />
        ))}
        {cursor && loadingMore && (
          <view className="cook__more">
            <text className="muted">{t('common.loading')}</text>
          </view>
        )}
      </view>
    </scroll-view>
  )
}
