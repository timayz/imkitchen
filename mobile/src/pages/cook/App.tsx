import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import '../../styles/base.css'
import { ApiError } from '../../lib/api/client.js'
import type { RecipeType } from '../../lib/api/recipe.js'
import { type Cook, type CookParams, type SortBy, type Summary, getCook } from '../../lib/api/recipes.js'
import { avatar } from '../../lib/avatar.js'
import { course } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { back, pageParams, push } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { Chip } from '../../ui/Chip.js'
import { RecipeCard } from '../../ui/RecipeCard.js'
import { SortSheet } from '../../ui/SortSheet.js'
import './App.css'

const TYPES: RecipeType[] = ['Appetizer', 'MainCourse', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']
/** The orders the web's cook page offers. */
const SORTS: SortBy[] = ['RecentlyAdded', 'Easiest', 'Hardest']

interface Filters {
  search: string
  recipe_type?: RecipeType
  sort_by: SortBy
}

const DEFAULT_FILTERS: Filters = { search: '', sort_by: 'RecentlyAdded' }

type Profile = Pick<Cook, 'username' | 'description' | 'stat'>
type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; profile: Profile }

/** A chef's public recipes: `cook.lynx.bundle?username=…`. */
export function App() {
  const username = pageParams().username ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [filters, setFilters] = useState<Filters>(DEFAULT_FILTERS)
  // Bumped to remount the (uncontrolled) search input when filters are cleared.
  const [searchKey, setSearchKey] = useState(0)
  const [items, setItems] = useState<Summary[]>([])
  const [cursor, setCursor] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [loadingMore, setLoadingMore] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [sortOpen, setSortOpen] = useState(false)
  const generation = useRef(0)

  const params = useCallback(
    (f: Filters, after?: string): CookParams => ({
      after,
      recipe_type: f.recipe_type,
      search: f.search.trim() || undefined,
      sort_by: f.sort_by,
    }),
    [],
  )

  // Every filter change refetches the profile too; it is one request either way.
  const load = useCallback(
    (f: Filters) => {
      const gen = ++generation.current
      setLoading(true)
      setLoadingMore(false)
      setError(null)
      getCook(username, params(f))
        .then((cook) => {
          if (gen !== generation.current) return
          setState({ kind: 'ready', profile: cook })
          setItems(cook.recipes.edges.map((e) => e.node))
          setCursor(cook.recipes.page_info.has_next_page ? cook.recipes.page_info.end_cursor : null)
        })
        .catch((err: unknown) => {
          if (gen !== generation.current) return
          const message = err instanceof ApiError ? err.message : t('error.network')
          // Without a profile the whole screen is the error; afterwards it shows inline.
          setState((prev) => (prev.kind === 'ready' ? prev : { kind: 'error', message }))
          setError(message)
        })
        .finally(() => {
          if (gen === generation.current) setLoading(false)
        })
    },
    [params, username],
  )

  // Debounced on the search text, immediate on everything else.
  useEffect(() => {
    const handle = setTimeout(() => load(filters), filters.search ? 300 : 0)
    return () => clearTimeout(handle)
  }, [filters, load])

  // Infinite scroll: fired by the scroll-view when the bottom comes within
  // `lower-threshold` px, like the web's "visible" sentinel. A page fetched for
  // a filter set that has since changed is dropped (same generation guard as
  // `load`) so it never gets appended to the wrong list.
  const loadMore = useCallback(async () => {
    if (!cursor || loading || loadingMore) return
    const gen = generation.current
    setLoadingMore(true)
    try {
      const cook = await getCook(username, params(filters, cursor))
      if (gen !== generation.current) return
      setItems((prev) => [...prev, ...cook.recipes.edges.map((e) => e.node)])
      setCursor(cook.recipes.page_info.has_next_page ? cook.recipes.page_info.end_cursor : null)
    } catch (err) {
      if (gen !== generation.current) return
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      if (gen === generation.current) setLoadingMore(false)
    }
  }, [cursor, filters, loading, loadingMore, params, username])

  const set = (patch: Partial<Filters>) => setFilters((f) => ({ ...f, ...patch }))
  const clear = () => {
    setFilters(DEFAULT_FILTERS)
    setSearchKey((k) => k + 1)
  }

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

  const { profile } = state
  const a = avatar(profile.username)
  const selected = filters.recipe_type ? course(filters.recipe_type) : null
  const sectionLabel = selected ? selected.label : t('recipes.all_recipes')
  const filtered = filters.search.trim() !== '' || filters.recipe_type !== undefined

  return (
    <view className="screen">
      <scroll-view
        className="cook__scroll"
        scroll-orientation="vertical"
        lower-threshold={400}
        bindscrolltolower={() => void loadMore()}
      >
        <view className="content">
          <view className="row" style={{ gap: '12px' }}>
            <view className="cook__close" bindtap={back}>
              <text className="cook__close-text">←</text>
            </view>
            <text className="cook__eyebrow">{t('cook.eyebrow')}</text>
          </view>

          <view className="card cook__profile">
            <view className="row" style={{ gap: '14px' }}>
              <view className="cook__avatar" style={{ backgroundColor: a.bg }}>
                <text className="cook__avatar-text" style={{ color: a.fg }}>
                  {a.initials}
                </text>
              </view>
              <view className="cook__identity">
                <text className="cook__name" text-maxline="1">
                  @{profile.username}
                </text>
                <text className="badge badge--chef">{t('cook.eyebrow')}</text>
              </view>
            </view>
            {profile.description !== '' && <text className="body">{profile.description}</text>}
            <view className="cook__stats">
              <view className="cook__stat cook__stat--divider">
                <text className="cook__stat-value">{profile.stat.shared}</text>
                <text className="cook__stat-label">{t('recipes.shared')}</text>
              </view>
              <view className="cook__stat">
                <text className="cook__stat-value">{profile.stat.total}</text>
                <text className="cook__stat-label">{t('tabs.recipes')}</text>
              </view>
            </view>
          </view>

          <view className="rec__search">
            <text className="rec__search-icon">🔍</text>
            <input
              key={searchKey}
              className="rec__search-input"
              value=""
              placeholder={t('recipes.search_placeholder')}
              maxlength={255}
              confirm-type="search"
              text-color="#1b140c"
              bindinput={(e) => set({ search: e.detail.value })}
            />
          </view>

          <scroll-view className="rec__chips" scroll-orientation="horizontal">
            <Chip
              label={`🍴 ${t('recipes.all')}`}
              active={!filters.recipe_type}
              onTap={() => set({ recipe_type: undefined })}
            />
            {TYPES.map((type) => {
              const c = course(type)
              return (
                <Chip
                  key={type}
                  label={`${c.emoji} ${c.label}`}
                  active={filters.recipe_type === type}
                  tint={{ bg: c.soft, fg: c.ink }}
                  onTap={() => set({ recipe_type: filters.recipe_type === type ? undefined : type })}
                />
              )
            })}
          </scroll-view>

          <view className="rec__section">
            <text className="rec__eyebrow">{sectionLabel.toUpperCase()}</text>
            <view className="rec__line" />
            <view className="rec__sort" bindtap={() => setSortOpen(true)}>
              <text className="rec__sort-text">{t(`sort.${filters.sort_by}` as const)} ▾</text>
            </view>
          </view>

          {error && <text className="error">{error}</text>}
          {loading && items.length === 0 && <text className="muted">{t('common.loading')}</text>}
          {!loading && items.length === 0 && !error && (
            <view className="card rec__empty">
              <view className="rec__empty-art" style={{ backgroundColor: selected ? selected.soft : '#fde3cf' }}>
                <text className="rec__empty-emoji">{selected ? selected.emoji : '🍴'}</text>
              </view>
              <text className="rec__empty-title">{filtered ? t('recipes.empty_title') : t('cook.empty_title')}</text>
              <text className="body rec__center">
                {filtered ? t('cook.empty_filtered') : t('cook.empty', { username: profile.username })}
              </text>
              {filtered && <Button label={t('recipes.clear_filters')} onTap={clear} variant="ink" block />}
            </view>
          )}

          <view className="rec__list">
            {items.map((recipe) => (
              <RecipeCard key={recipe.id} recipe={recipe} onTap={() => void push('recipe', { id: recipe.id })} />
            ))}
          </view>

          {cursor && loadingMore && (
            <view className="rec__more">
              <text className="muted">{t('common.loading')}</text>
            </view>
          )}
        </view>
      </scroll-view>

      <SortSheet
        open={sortOpen}
        value={filters.sort_by}
        sorts={SORTS}
        onClose={() => setSortOpen(false)}
        onPick={(sort_by) => set({ sort_by })}
      />
    </view>
  )
}
