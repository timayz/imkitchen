import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { errorMessage } from '../../lib/api/client.js'
import {
  type Candidate,
  type CandidatePage,
  createDish,
  deleteDish,
  getCandidates,
  getDish,
  updateDish,
} from '../../lib/api/dishes.js'
import type { RecipeType } from '../../lib/api/recipe.js'
import { course } from '../../lib/course.js'
import { MEAL_ORDER, isMultiple, problem, toggle } from '../../lib/dishes.js'
import { t } from '../../lib/i18n/index.js'
import { back, pageParams } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { Chip } from '../../ui/Chip.js'
import { Sheet } from '../../ui/Sheet.js'
import { Spinner } from '../../ui/Spinner.js'
import { TextField } from '../../ui/TextField.js'

/** Matches shown per course; the rest is reached by typing a name. */
const PAGE = 20
/** Keystrokes settle this long before a search goes out. */
const DEBOUNCE_MS = 250

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; name: string }

/**
 * Compose a dish (`dish-edit.lynx.bundle`) or edit one (`?id=…`): a name,
 * one main course, and the courses to go with it. Each course is searched
 * by name, a page at a time, so a library of hundreds of recipes stays
 * usable; what is picked is kept apart from the matches.
 */
export function App() {
  const id = pageParams().id
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [name, setName] = useState('')
  const [selected, setSelected] = useState<Candidate[]>([])
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)

  useEffect(() => {
    let cancelled = false
    ;(id ? getDish(id) : Promise.resolve(null))
      .then((dish) => {
        if (cancelled) return
        const initial = dish?.name ?? ''
        setName(initial)
        setSelected(
          dish
            ? dish.recipes.map((r) => ({
                id: r.id,
                name: r.name,
                recipe_type: r.recipe_type,
                accepts_accompaniment: false,
              }))
            : []
        )
        setState({ kind: 'ready', name: initial })
      })
      .catch((err: unknown) => {
        if (!cancelled) setState({ kind: 'error', message: errorMessage(err) })
      })
    return () => {
      cancelled = true
    }
  }, [id])

  const save = useCallback(async () => {
    if (busy || state.kind !== 'ready') return
    setBusy(true)
    setNotice(null)
    try {
      const input = { name: name.trim(), recipe_ids: selected.map((c) => c.id) }
      if (id) await updateDish(id, input)
      else await createDish(input)
      back()
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [busy, id, name, selected, state.kind])

  const remove = useCallback(async () => {
    if (!id || busy) return
    setConfirmDelete(false)
    setBusy(true)
    setNotice(null)
    try {
      await deleteDish(id)
      back()
    } catch (err) {
      setNotice(errorMessage(err))
    } finally {
      setBusy(false)
    }
  }, [busy, id])

  if (state.kind !== 'ready') {
    return (
      <view className="screen cook--center">
        {state.kind === 'error' ? <text className="muted">{state.message}</text> : <Spinner size="lg" />}
        {state.kind === 'error' && <Button label={t('common.close')} onTap={back} variant="secondary" />}
      </view>
    )
  }

  const issue = name.trim() === '' ? 'no_name' : problem(selected)

  return (
    <view className="screen">
      <view className="dish__bar">
        <view className="dish__back" bindtap={back}>
          <text className="dish__back-glyph">←</text>
        </view>
        <view className="dish__bar-text">
          <text className="h2">{id ? t('dish.edit_title') : t('dish.new_title')}</text>
          <text className="muted">{t('dishes.title')}</text>
        </view>
      </view>

      <scroll-view className="dish__scroll" scroll-orientation="vertical">
        <view className="content">
          {notice && <text className="error">{notice}</text>}
          <view className="card">
            <TextField
              label={t('dish.name')}
              value={state.name}
              onChange={setName}
              placeholder={t('dish.name_placeholder')}
              maxlength={100}
              surface="cream"
            />
            <text className="muted">{t('dish.rule')}</text>
          </view>

          {MEAL_ORDER.map((type) => (
            <CourseSection
              key={type}
              type={type}
              selected={selected.filter((c) => c.recipe_type === type)}
              onToggle={(candidate) => setSelected((s) => toggle(s, candidate))}
            />
          ))}

          {id && (
            <view className="dish__danger">
              <Button label={t('dish.delete')} onTap={() => setConfirmDelete(true)} variant="danger" disabled={busy} />
            </view>
          )}
        </view>
      </scroll-view>

      <view className="dish__footer">
        <view className="dish__footer-text">
          <text className="dish__footer-title">
            {issue === 'no_name' ? t('dish.needs_name') : issue === 'no_main' ? t('dish.needs_main') : t('dish.ready')}
          </text>
          <text className="muted">{t('dish.selected', { n: selected.length })}</text>
        </view>
        <Button label={t('dish.save')} onTap={() => void save()} disabled={busy || issue !== null} />
      </view>

      <Sheet open={confirmDelete} onClose={() => setConfirmDelete(false)}>
        <text className="h2">{t('dish.delete_title')}</text>
        <text className="body">{t('dish.delete_body')}</text>
        <Button label={t('dish.delete')} onTap={() => void remove()} variant="danger" block />
        <Button label={t('common.cancel')} onTap={() => setConfirmDelete(false)} variant="ghost" />
      </Sheet>
    </view>
  )
}

interface CourseSectionProps {
  type: RecipeType
  /** The picks of this course, shown first and kept whatever the search. */
  selected: Candidate[]
  onToggle: (candidate: Candidate) => void
}

/** One course: its picks, a name search, and a page of matches. */
function CourseSection({ type, selected, onToggle }: CourseSectionProps) {
  const c = course(type)
  const [search, setSearch] = useState('')
  const [page, setPage] = useState<CandidatePage | null>(null)
  const [failed, setFailed] = useState(false)
  const generation = useRef(0)

  useEffect(() => {
    const gen = ++generation.current
    const handle = setTimeout(
      () => {
        getCandidates({ recipe_type: type, search: search.trim() || undefined, limit: PAGE })
          .then((result) => {
            if (gen !== generation.current) return
            setPage(result)
            setFailed(false)
          })
          .catch(() => {
            if (gen === generation.current) setFailed(true)
          })
      },
      search === '' ? 0 : DEBOUNCE_MS
    )
    return () => clearTimeout(handle)
  }, [type, search])

  const hint = type === 'MainCourse' ? t('dish.required') : isMultiple(type) ? t('dish.pick_any') : t('dish.pick_one')
  const picked = new Set(selected.map((s) => s.id))
  const matches = page?.candidates.filter((m) => !picked.has(m.id)) ?? []
  const more = page ? Math.max(0, page.total - page.candidates.length) : 0
  const emptyPool = page !== null && page.total === 0 && search.trim() === '' && selected.length === 0
  // A pick that matches the search is pinned above and is answer enough.
  const needle = search.trim().toLowerCase()
  const noMatch = needle !== '' && !selected.some((s) => s.name.toLowerCase().includes(needle))

  return (
    <view className="dish__section">
      <view className="dish__section-head">
        <text className="dish__section-title">
          {c.emoji} {c.label}
        </text>
        <text className="muted">{hint}</text>
      </view>
      {selected.length > 0 && (
        <view className="dish__chips">
          {selected.map((item) => (
            <Chip
              key={item.id}
              label={`✓ ${item.name}`}
              active
              tint={{ bg: c.soft, fg: c.ink }}
              onTap={() => onToggle(item)}
            />
          ))}
        </view>
      )}
      {emptyPool ? (
        <text className="muted">{t('dish.none_of_kind')}</text>
      ) : (
        <>
          <view className="rec__search">
            <text className="rec__search-icon">🔍</text>
            <input
              className="rec__search-input"
              value=""
              placeholder={t('dish.search')}
              maxlength={100}
              confirm-type="search"
              text-color="#1b140c"
              bindinput={(e) => setSearch(e.detail.value)}
            />
          </view>
          {page === null && !failed && <Spinner />}
          {failed && <text className="error">{t('error.network')}</text>}
          {matches.length > 0 && (
            <view className="dish__chips">
              {matches.map((item) => (
                <Chip key={item.id} label={item.name} active={false} onTap={() => onToggle(item)} />
              ))}
            </view>
          )}
          {page !== null && matches.length === 0 && noMatch && (
            <text className="muted">{t('dish.no_match', { q: search.trim() })}</text>
          )}
          {more > 0 && <text className="muted">{t('dish.more', { n: more })}</text>}
        </>
      )}
    </view>
  )
}
