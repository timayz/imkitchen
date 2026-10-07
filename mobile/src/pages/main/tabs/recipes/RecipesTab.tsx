import { useCallback, useEffect, useState } from '@lynx-js/react'

import { errorMessage, isOffline } from '../../../../lib/api/client.js'
import { type Me, me } from '../../../../lib/api/auth.js'
import type { RecipeType } from '../../../../lib/api/recipe.js'
import {
  type Browse,
  type BrowseParams,
  type SortBy,
  type Summary,
  browse,
  createDraft,
  shareAll,
  unshareAll,
} from '../../../../lib/api/recipes.js'
import { course } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { push } from '../../../../lib/nav.js'
import { useResource } from '../../../../lib/use-resource.js'
import { Button } from '../../../../ui/Button.js'
import { Chip } from '../../../../ui/Chip.js'
import { RecipeCard } from '../../../../ui/RecipeCard.js'
import { SortSheet } from '../../../../ui/SortSheet.js'
import { Spinner } from '../../../../ui/Spinner.js'
import { AddSheet } from './AddSheet.js'
import './RecipesTab.css'

const TYPES: RecipeType[] = ['Appetizer', 'MainCourse', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']

interface Filters {
  search: string
  recipe_type?: RecipeType
  sort_by: SortBy
  mine: boolean
  in_meal_plan: boolean
  no_image: boolean
}

const DEFAULT_FILTERS: Filters = {
  search: '',
  sort_by: 'RecentlyAdded',
  mine: false,
  in_meal_plan: false,
  no_image: false,
}

function params(f: Filters, after?: string): BrowseParams {
  return {
    after,
    recipe_type: f.recipe_type,
    search: f.search.trim() || undefined,
    sort_by: f.sort_by,
    mine: f.mine,
    in_meal_plan: f.in_meal_plan,
    no_image: f.no_image,
  }
}

/** The first page of a filter set is cached; searches are not. */
function cacheKey(f: Filters): string | null {
  if (f.search.trim() !== '') return null
  return `recipes:browse:${JSON.stringify(params(f))}`
}

export function RecipesTab({ refreshKey }: { refreshKey: number }) {
  const [filters, setFilters] = useState<Filters>(DEFAULT_FILTERS)
  // Debounced on the search text, immediate on everything else.
  const [applied, setApplied] = useState<Filters>(DEFAULT_FILTERS)
  // Bumped to remount the (uncontrolled) search input when filters are cleared.
  const [searchKey, setSearchKey] = useState(0)
  const first = useResource<Browse>(cacheKey(applied), () => browse(params(applied)), [applied, refreshKey])
  const { data: user } = useResource<Me>('me', me, [])
  // Pages after the first are network-only and reset with the first page.
  const [extra, setExtra] = useState<Summary[]>([])
  const [cursor, setCursor] = useState<string | null>(null)
  const [notice, setNotice] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)
  const [loadingMore, setLoadingMore] = useState(false)
  const [addOpen, setAddOpen] = useState(false)
  const [sortOpen, setSortOpen] = useState(false)

  useEffect(() => {
    const handle = setTimeout(() => setApplied(filters), filters.search ? 300 : 0)
    return () => clearTimeout(handle)
  }, [filters])

  const page = first.data
  useEffect(() => {
    setExtra([])
    setCursor(page?.page.page_info.has_next_page ? page.page.page_info.end_cursor : null)
  }, [page])

  const items = [...(page?.page.edges.map((e) => e.node) ?? []), ...extra]
  const hasShared = page?.has_shared ?? false
  const loading = first.loading
  const error = first.error ?? notice

  // Infinite scroll: fired by the scroll-view when the bottom comes within
  // `lower-threshold` px, like the web's "visible" sentinel. A page fetched
  // for a filter set that has since changed is dropped so it never gets
  // appended to the wrong list.
  const loadMore = useCallback(async () => {
    if (!cursor || loading || loadingMore) return
    const before = page
    setLoadingMore(true)
    try {
      const result = await browse(params(applied, cursor))
      if (before !== page) return
      setExtra((prev) => [...prev, ...result.page.edges.map((e) => e.node)])
      setCursor(result.page.page_info.has_next_page ? result.page.page_info.end_cursor : null)
    } catch (err) {
      // Offline: keep the cursor, the next scroll tries again.
      if (!isOffline(err)) setNotice(errorMessage(err))
    } finally {
      setLoadingMore(false)
    }
  }, [cursor, loading, loadingMore, page, applied])

  const newRecipe = useCallback(async () => {
    if (busy) return
    setBusy(true)
    try {
      const { id } = await createDraft()
      setAddOpen(false)
      await push('recipe-edit', { id })
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [busy])

  const importRecipe = useCallback(() => {
    setAddOpen(false)
    void push('recipe-import')
  }, [])

  const toggleShareAll = useCallback(async () => {
    if (busy) return
    setBusy(true)
    try {
      if (hasShared) await unshareAll()
      else await shareAll()
      first.refresh()
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [busy, hasShared, first.refresh])

  const set = (patch: Partial<Filters>) => setFilters((f) => ({ ...f, ...patch }))
  const clear = () => {
    setFilters(DEFAULT_FILTERS)
    setSearchKey((k) => k + 1)
  }

  const selected = filters.recipe_type ? course(filters.recipe_type) : null
  const sectionLabel = filters.mine ? t('recipes.my_recipes') : selected ? selected.label : t('recipes.all_recipes')
  const filtered =
    filters.search.trim() !== '' ||
    filters.recipe_type !== undefined ||
    filters.mine ||
    filters.in_meal_plan ||
    filters.no_image

  return (
    <scroll-view
      className="tab-scroll"
      scroll-orientation="vertical"
      lower-threshold={400}
      bindscrolltolower={() => void loadMore()}
    >
      <view className="content">
        <view className="rec__header">
          <view className="rec__header-text">
            <text className="cook__eyebrow">{t('recipes.library')}</text>
            <text className="h1">{t('tabs.recipes')}</text>
          </view>
          <view className="rec__add" bindtap={() => setAddOpen(true)}>
            <text className="rec__add-glyph">+</text>
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

        <view className="rec__toggles">
          <Chip label={t('recipes.mine')} active={filters.mine} onTap={() => set({ mine: !filters.mine })} />
          <Chip
            label={t('recipes.saved')}
            active={filters.in_meal_plan}
            onTap={() => set({ in_meal_plan: !filters.in_meal_plan })}
          />
          <Chip
            label={t('recipes.no_image')}
            active={filters.no_image}
            onTap={() => set({ no_image: !filters.no_image })}
          />
          <view className="rec__spacer" />
          <view className="rec__sort" bindtap={() => setSortOpen(true)}>
            <text className="rec__sort-text">{t(`sort.${filters.sort_by}` as const)} ▾</text>
          </view>
        </view>

        {user?.is_chef && filters.mine && (
          <view className={hasShared ? 'rec__note' : 'rec__note rec__note--private'}>
            <view className="rec__note-icon">
              <text className="rec__note-glyph">{hasShared ? '📤' : '🔒'}</text>
            </view>
            <text className={hasShared ? 'rec__note-text' : 'rec__note-text rec__note-text--private'}>
              {hasShared ? t('recipes.shared_note') : t('recipes.private_note')}
            </text>
            <view className="rec__note-btn" bindtap={busy ? undefined : toggleShareAll}>
              <text className="rec__note-btn-text">
                {hasShared ? t('recipes.make_all_private') : t('recipes.share_all')}
              </text>
            </view>
          </view>
        )}

        <view className="rec__section">
          <text className="rec__eyebrow">{sectionLabel.toUpperCase()}</text>
          <view className="rec__line" />
        </view>

        {error && <text className="error">{error}</text>}
        {loading && items.length === 0 && <Spinner />}
        {!loading && items.length === 0 && !error && (
          <view className="card rec__empty">
            <view className="rec__empty-art" style={{ backgroundColor: selected ? selected.soft : '#fde3cf' }}>
              <text className="rec__empty-emoji">{selected ? selected.emoji : '🍴'}</text>
            </view>
            <text className="rec__empty-title">{t('recipes.empty_title')}</text>
            <text className="body rec__center">{t('recipes.empty')}</text>
            {filtered && <Button label={t('recipes.clear_filters')} onTap={clear} variant="ink" block />}
            <Button label={t('recipes.add')} onTap={() => setAddOpen(true)} variant="ghost" />
          </view>
        )}

        <view className="rec__list">
          {items.map((recipe) => (
            <RecipeCard
              key={recipe.id}
              recipe={recipe}
              showShared={filters.mine}
              onTap={() => void push('recipe', { id: recipe.id })}
            />
          ))}
        </view>

        {cursor && loadingMore && (
          <view className="rec__more">
            <Spinner />
          </view>
        )}
      </view>

      <AddSheet
        open={addOpen}
        busy={busy}
        onClose={() => setAddOpen(false)}
        onNew={newRecipe}
        onImport={importRecipe}
      />
      <SortSheet
        open={sortOpen}
        value={filters.sort_by}
        onClose={() => setSortOpen(false)}
        onPick={(sort_by) => set({ sort_by })}
      />
    </scroll-view>
  )
}
