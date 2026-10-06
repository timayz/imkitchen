import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import { ApiError } from '../../../../lib/api/client.js'
import { type Me, me } from '../../../../lib/api/auth.js'
import type { RecipeType } from '../../../../lib/api/recipe.js'
import {
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
import { Button } from '../../../../ui/Button.js'
import { Chip } from '../../../../ui/Chip.js'
import { RecipeCard } from '../../../../ui/RecipeCard.js'
import { TextField } from '../../../../ui/TextField.js'
import './RecipesTab.css'

const TYPES: RecipeType[] = ['Appetizer', 'MainCourse', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']
const SORTS: SortBy[] = ['RecentlyAdded', 'Easiest', 'Hardest', 'Random']

interface Filters {
  search: string
  recipe_type?: RecipeType
  sort_by: SortBy
  mine: boolean
  in_meal_plan: boolean
  no_image: boolean
}

export function RecipesTab({ refreshKey }: { refreshKey: number }) {
  const [filters, setFilters] = useState<Filters>({
    search: '',
    sort_by: 'RecentlyAdded',
    mine: false,
    in_meal_plan: false,
    no_image: false,
  })
  const [items, setItems] = useState<Summary[]>([])
  const [cursor, setCursor] = useState<string | null>(null)
  const [hasShared, setHasShared] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [user, setUser] = useState<Me | null>(null)
  const [busy, setBusy] = useState(false)
  const generation = useRef(0)

  const params = useCallback(
    (f: Filters, after?: string): BrowseParams => ({
      after,
      recipe_type: f.recipe_type,
      search: f.search.trim() || undefined,
      sort_by: f.sort_by,
      mine: f.mine,
      in_meal_plan: f.in_meal_plan,
      no_image: f.no_image,
    }),
    [],
  )

  const load = useCallback(
    (f: Filters) => {
      const gen = ++generation.current
      setLoading(true)
      setError(null)
      browse(params(f))
        .then((result) => {
          if (gen !== generation.current) return
          setItems(result.page.edges.map((e) => e.node))
          setCursor(result.page.page_info.has_next_page ? result.page.page_info.end_cursor : null)
          setHasShared(result.has_shared)
        })
        .catch((err: unknown) => {
          if (gen !== generation.current) return
          setError(err instanceof ApiError ? err.message : t('error.network'))
        })
        .finally(() => {
          if (gen === generation.current) setLoading(false)
        })
    },
    [params],
  )

  useEffect(() => {
    me().then(setUser).catch(() => {})
  }, [])

  // Debounced on the search text, immediate on everything else.
  useEffect(() => {
    const handle = setTimeout(() => load(filters), filters.search ? 300 : 0)
    return () => clearTimeout(handle)
  }, [filters, load, refreshKey])

  const loadMore = useCallback(async () => {
    if (!cursor || busy) return
    setBusy(true)
    try {
      const result = await browse(params(filters, cursor))
      setItems((prev) => [...prev, ...result.page.edges.map((e) => e.node)])
      setCursor(result.page.page_info.has_next_page ? result.page.page_info.end_cursor : null)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, cursor, filters, params])

  const newRecipe = useCallback(async () => {
    if (busy) return
    setBusy(true)
    try {
      const { id } = await createDraft()
      await push('recipe-edit', { id })
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy])

  const toggleShareAll = useCallback(async () => {
    if (busy) return
    setBusy(true)
    try {
      if (hasShared) await unshareAll()
      else await shareAll()
      setHasShared(!hasShared)
      load(filters)
    } catch (err) {
      setError(err instanceof ApiError ? err.message : t('error.network'))
    } finally {
      setBusy(false)
    }
  }, [busy, filters, hasShared, load])

  const set = (patch: Partial<Filters>) => setFilters((f) => ({ ...f, ...patch }))

  return (
    <scroll-view className="tab-scroll" scroll-orientation="vertical">
      <view className="content">
        <view>
          <text className="cook__eyebrow">{t('recipes.library')}</text>
          <text className="h1">{t('recipes.title')}</text>
        </view>

        <TextField
          label={t('recipes.search')}
          value=""
          onChange={(search) => set({ search })}
          placeholder={t('recipes.search_placeholder')}
          confirmType="search"
        />

        <view className="row rec__actions">
          <Button label={t('recipes.new')} onTap={newRecipe} disabled={busy} />
          <Button label={t('recipes.import')} onTap={() => void push('recipe-import')} variant="secondary" />
        </view>

        <scroll-view className="rec__chips" scroll-orientation="horizontal">
          <Chip label={`🍴 ${t('recipes.all')}`} active={!filters.recipe_type} onTap={() => set({ recipe_type: undefined })} />
          {TYPES.map((type) => {
            const c = course(type)
            return (
              <Chip
                key={type}
                label={`${c.emoji} ${c.label}`}
                active={filters.recipe_type === type}
                onTap={() => set({ recipe_type: type })}
              />
            )
          })}
        </scroll-view>

        <view className="row rec__toggles">
          <Chip label={t('recipes.mine')} active={filters.mine} onTap={() => set({ mine: !filters.mine })} />
          <Chip label={t('recipes.saved')} active={filters.in_meal_plan} onTap={() => set({ in_meal_plan: !filters.in_meal_plan })} />
          <Chip label={t('recipes.no_image')} active={filters.no_image} onTap={() => set({ no_image: !filters.no_image })} />
        </view>

        <view className="row rec__toggles">
          <text className="muted">{t('recipes.sort')}</text>
          {SORTS.map((sort) => (
            <Chip key={sort} label={t(`sort.${sort}` as const)} active={filters.sort_by === sort} onTap={() => set({ sort_by: sort })} />
          ))}
        </view>

        {user?.is_chef && filters.mine && (
          <Button
            label={hasShared ? t('recipes.make_all_private') : t('recipes.share_all')}
            onTap={toggleShareAll}
            variant="secondary"
            disabled={busy}
          />
        )}

        {error && <text className="error">{error}</text>}
        {loading && items.length === 0 && <text className="muted">{t('common.loading')}</text>}
        {!loading && items.length === 0 && !error && (
          <view className="card">
            <text className="body">{t('recipes.empty')}</text>
          </view>
        )}

        {items.map((recipe) => (
          <RecipeCard key={recipe.id} recipe={recipe} showShared={filters.mine} onTap={() => void push('recipe', { id: recipe.id })} />
        ))}

        {cursor && <Button label={t('recipes.load_more')} onTap={loadMore} variant="secondary" disabled={busy} block />}
      </view>
    </scroll-view>
  )
}
