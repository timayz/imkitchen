import { useCallback, useEffect, useRef, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import type { DietaryRestriction, Instruction, RecipeType } from '../../lib/api/recipe.js'
import {
  type IngredientInput,
  type RecipeInput,
  getEditable,
  updateRecipe,
  uploadThumbnail,
} from '../../lib/api/recipe-edit.js'
import { getRecipe } from '../../lib/api/recipes.js'
import { aisle, course, minutes } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { pickImage } from '../../lib/media.js'
import { back, pageParams } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { Chip } from '../../ui/Chip.js'
import { Counter } from '../../ui/Counter.js'
import { RecipeImage } from '../../ui/RecipeImage.js'
import { Segmented } from '../../ui/Segmented.js'
import { Sheet } from '../../ui/Sheet.js'
import { Spinner } from '../../ui/Spinner.js'
import { Switch } from '../../ui/Switch.js'
import { TextField } from '../../ui/TextField.js'
import { AisleSheet } from './AisleSheet.js'

const TYPES: RecipeType[] = ['Appetizer', 'MainCourse', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']
const DIETS: DietaryRestriction[] = ['Vegetarian', 'Vegan', 'GlutenFree', 'DairyFree', 'NutFree']

/** Minutes between thumbnail upload and the first look for the resized file. */
const PHOTO_POLL_MS = [3000, 8000]

type Keyed<T> = T & { key: number }

/**
 * The form as edited: ingredient and instruction rows carry a local key so
 * their uncontrolled inputs survive a row being removed above them.
 */
interface Draft extends Omit<RecipeInput, 'ingredients' | 'instructions'> {
  ingredients: Keyed<IngredientInput>[]
  instructions: Keyed<Instruction>[]
}

interface Photo {
  thumbnail_url: string | null
  blur_placeholder: string | null
}

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready' }

/** What `PUT` receives: keys stripped, empty rows dropped. */
function toInput(draft: Draft): RecipeInput {
  const { ingredients, instructions, ...rest } = draft
  return {
    ...rest,
    ingredients: ingredients.filter((i) => i.name.trim() !== '').map(({ key: _key, ...i }) => i),
    instructions: instructions.filter((i) => i.description.trim() !== '').map(({ key: _key, ...i }) => i),
  }
}

/**
 * Text input without a label, uncontrolled for the same reason as
 * `TextField`: the host EditText keeps the text, `onChange` lifts it.
 */
function BareInput({
  className,
  value,
  placeholder,
  maxlength,
  multiline,
  onChange,
}: {
  className: string
  value: string
  placeholder: string
  maxlength: number
  multiline?: boolean
  onChange: (value: string) => void
}) {
  const initial = useRef(value)
  return (
    <input
      className={className}
      value={initial.current}
      placeholder={placeholder}
      maxlength={maxlength}
      multiline={multiline}
      text-color="#1b140c"
      bindinput={(e) => onChange(e.detail.value)}
    />
  )
}

/**
 * Recipe editor: `recipe-edit.lynx.bundle?id=…`. Photo, basics, dietary and
 * pairing, details, then ingredient and instruction rows; Save sits in a
 * pinned footer and backing out with unsaved edits asks first.
 */
export function App() {
  const id = pageParams().id ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [draft, setDraft] = useState<Draft | null>(null)
  // Serialised payload as last saved; the footer compares against it.
  const [saved, setSaved] = useState('')
  const [photo, setPhoto] = useState<Photo | null>(null)
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<{ kind: 'ok' | 'error'; text: string } | null>(null)
  const [aisleFor, setAisleFor] = useState<number | null>(null)
  const [discardOpen, setDiscardOpen] = useState(false)
  const nextKey = useRef(0)
  const key = () => nextKey.current++

  const loadPhoto = useCallback(
    () =>
      getRecipe(id)
        .then((r) => setPhoto({ thumbnail_url: r.thumbnail_url, blur_placeholder: r.blur_placeholder }))
        .catch(() => {}),
    [id],
  )

  useEffect(() => {
    getEditable(id)
      .then((input) => {
        const loaded: Draft = {
          ...input,
          ingredients: input.ingredients.map((i) => ({ ...i, key: key() })),
          instructions: input.instructions.map((i) => ({ ...i, key: key() })),
        }
        setDraft(loaded)
        // Serialise through `toInput` so key order matches the dirty check.
        setSaved(JSON.stringify(toInput(loaded)))
        setState({ kind: 'ready' })
      })
      .catch((err: unknown) =>
        setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') }),
      )
    void loadPhoto()
  }, [id, loadPhoto])

  const patch = (p: Partial<Draft>) => setDraft((d) => (d ? { ...d, ...p } : d))
  const patchIngredient = (index: number, p: Partial<IngredientInput>) =>
    setDraft((d) => (d ? { ...d, ingredients: d.ingredients.map((i, k) => (k === index ? { ...i, ...p } : i)) } : d))
  const patchInstruction = (index: number, p: Partial<Instruction>) =>
    setDraft((d) => (d ? { ...d, instructions: d.instructions.map((i, k) => (k === index ? { ...i, ...p } : i)) } : d))

  const save = useCallback(async (): Promise<boolean> => {
    if (!draft || busy) return false
    setBusy(true)
    setNotice(null)
    try {
      const input = toInput(draft)
      await updateRecipe(id, input)
      setSaved(JSON.stringify(input))
      setNotice({ kind: 'ok', text: t('edit.saved') })
      return true
    } catch (err) {
      setNotice({ kind: 'error', text: err instanceof ApiError ? err.message : t('error.network') })
      return false
    } finally {
      setBusy(false)
    }
  }, [busy, draft, id])

  const changePhoto = useCallback(async () => {
    if (busy) return
    setBusy(true)
    setNotice(null)
    try {
      const picked = await pickImage()
      if (!picked) return
      await uploadThumbnail(id, picked.mimeType, picked.base64)
      setNotice({ kind: 'ok', text: t('edit.photo_uploaded') })
      // The server resizes in the background; look for the result a bit later.
      PHOTO_POLL_MS.forEach((ms) => setTimeout(() => void loadPhoto(), ms))
    } catch (err) {
      setNotice({ kind: 'error', text: err instanceof ApiError ? err.message : String(err) })
    } finally {
      setBusy(false)
    }
  }, [busy, id, loadPhoto])

  if (state.kind === 'loading' || !draft) {
    return (
      <view className="screen cook--center">
        {state.kind === 'error' ? <text className="muted">{state.message}</text> : <Spinner size="lg" />}
        {state.kind === 'error' && <Button label={t('common.close')} onTap={back} variant="secondary" />}
      </view>
    )
  }

  const c = course(draft.recipe_type)
  const dirty = JSON.stringify(toInput(draft)) !== saved
  const leave = () => (dirty ? setDiscardOpen(true) : back())
  const units: { value: 'G' | 'ML' | null; label: string }[] = [
    { value: null, label: t('edit.unit_none') },
    { value: 'G', label: 'g' },
    { value: 'ML', label: 'ml' },
  ]
  const timer = (v: number) => (v === 0 ? t('edit.timer_none') : minutes(v))

  return (
    <view className="screen">
      <view className="edit__bar">
        <view className="edit__back" bindtap={leave}>
          <text className="edit__back-glyph">←</text>
        </view>
        <view className="edit__bar-text">
          <text className="h2">{t('edit.title')}</text>
          <text className="muted">
            {t('tabs.recipes')} / {c.label}
          </text>
        </view>
      </view>

      <scroll-view className="edit__scroll" scroll-orientation="vertical">
        <view className="content">
          <view className="edit__photo">
            <view className="edit__hero">
              <RecipeImage
                thumbnailUrl={photo?.thumbnail_url ?? null}
                blurPlaceholder={photo?.blur_placeholder ?? null}
                recipeType={draft.recipe_type}
                size="hero"
              />
              <view className={busy ? 'edit__photo-btn edit__photo-btn--off' : 'edit__photo-btn'} bindtap={changePhoto}>
                <text className="edit__photo-text">📷 {t('edit.change_photo')}</text>
              </view>
            </view>
            <text className="field__hint">{t('edit.photo_hint')}</text>
          </view>

          <view className="card">
            <TextField label={t('edit.name')} value={draft.name} onChange={(name) => patch({ name })} maxlength={100} />
            <TextField
              label={t('edit.description')}
              value={draft.description}
              onChange={(description) => patch({ description })}
              maxlength={2000}
              multiline
            />
            <view className="field">
              <text className="field__label">{t('edit.course')}</text>
              <view className="edit__chips">
                {TYPES.map((type) => {
                  const tc = course(type)
                  return (
                    <Chip
                      key={type}
                      label={`${tc.emoji} ${tc.label}`}
                      active={draft.recipe_type === type}
                      tint={{ bg: tc.soft, fg: tc.ink }}
                      onTap={() => patch({ recipe_type: type })}
                    />
                  )
                })}
              </view>
            </view>
            <view className="edit__group">
              <view className="edit__grow">
                <text className="edit__grow-label">{t('edit.household')}</text>
                <Counter
                  value={draft.household_size}
                  min={1}
                  max={20}
                  onChange={(household_size) => patch({ household_size })}
                />
              </view>
              <view className="edit__grow">
                <text className="edit__grow-label">{t('edit.prep')}</text>
                <Counter
                  value={draft.prep_time}
                  min={0}
                  max={600}
                  step={5}
                  format={minutes}
                  onChange={(prep_time) => patch({ prep_time })}
                />
              </view>
              <view className="edit__grow edit__grow--last">
                <text className="edit__grow-label">{t('edit.cook')}</text>
                <Counter
                  value={draft.cook_time}
                  min={0}
                  max={600}
                  step={5}
                  format={minutes}
                  onChange={(cook_time) => patch({ cook_time })}
                />
              </view>
            </view>
          </view>

          <view className="card">
            <view className="field">
              <text className="field__label">{t('edit.diet')}</text>
              <view className="edit__chips">
                {DIETS.map((d) => (
                  <Chip
                    key={d}
                    label={t(`diet.${d}` as const)}
                    active={draft.dietary_restrictions.includes(d)}
                    onTap={() =>
                      patch({
                        dietary_restrictions: draft.dietary_restrictions.includes(d)
                          ? draft.dietary_restrictions.filter((x) => x !== d)
                          : [...draft.dietary_restrictions, d],
                      })
                    }
                  />
                ))}
              </view>
            </view>
            {draft.recipe_type === 'MainCourse' && (
              <view className="edit__toggle">
                <view className="edit__toggle-text">
                  <text className="edit__toggle-label">{t('edit.accompaniment_on')}</text>
                  <text className="field__hint">{t('edit.side_hint')}</text>
                </view>
                <Switch
                  on={draft.accepts_accompaniment}
                  onToggle={() => patch({ accepts_accompaniment: !draft.accepts_accompaniment })}
                />
              </view>
            )}
          </view>

          <view className="card">
            <TextField
              label={t('edit.origin')}
              value={draft.origin ?? ''}
              onChange={(origin) => patch({ origin: origin.trim() === '' ? null : origin })}
              placeholder="https://"
              maxlength={255}
            />
            <TextField
              label={t('edit.advance_prep')}
              value={draft.advance_prep}
              onChange={(advance_prep) => patch({ advance_prep })}
              maxlength={2000}
              multiline
              hint={t('edit.advance_hint')}
            />
          </view>

          <view className="edit__section">
            <text className="h2">{t('recipes.ingredients')}</text>
            <text className="muted">{draft.ingredients.length}</text>
          </view>
          <view className="edit__list">
            {draft.ingredients.map((ing, index) => {
              const a = ing.category ? aisle(`shopping_${ing.category}`) : null
              return (
                <view key={ing.key} className="edit__row">
                  <view className="edit__row-top">
                    <view
                      className={a ? 'edit__aisle' : 'edit__aisle edit__aisle--unset'}
                      style={a ? { backgroundColor: a.soft } : undefined}
                      bindtap={() => setAisleFor(index)}
                    >
                      <text className="edit__aisle-emoji">{a ? a.emoji : '🛒'}</text>
                    </view>
                    <BareInput
                      className="edit__name"
                      value={ing.name}
                      placeholder={t('edit.ingredient_placeholder')}
                      maxlength={100}
                      onChange={(name) => patchIngredient(index, { name })}
                    />
                    <view
                      className="edit__remove"
                      bindtap={() => patch({ ingredients: draft.ingredients.filter((_, k) => k !== index) })}
                    >
                      <text className="edit__remove-glyph">✕</text>
                    </view>
                  </view>
                  <view className="edit__row-bottom">
                    <Counter
                      value={ing.quantity}
                      min={0}
                      max={100000}
                      step={ing.unit ? 10 : 1}
                      onChange={(quantity) => patchIngredient(index, { quantity })}
                    />
                    <Segmented options={units} value={ing.unit} onChange={(unit) => patchIngredient(index, { unit })} />
                  </view>
                </view>
              )
            })}
            <view className="edit__add">
              <Button
                label={t('edit.add_ingredient')}
                onTap={() =>
                  patch({
                    ingredients: [
                      ...draft.ingredients,
                      { key: key(), name: '', quantity: 1, unit: null, category: null },
                    ],
                  })
                }
                variant="secondary"
                block
              />
            </view>
          </view>

          <view className="edit__section">
            <text className="h2">{t('recipes.instructions')}</text>
            <text className="muted">{t('edit.steps_n', { n: draft.instructions.length })}</text>
          </view>
          <view className="edit__list">
            {draft.instructions.map((ins, index) => (
              <view key={ins.key} className="edit__step">
                <view className="edit__num" style={{ backgroundColor: c.soft }}>
                  <text className="edit__num-text" style={{ color: c.ink }}>
                    {index + 1}
                  </text>
                </view>
                <view className="edit__step-body">
                  <BareInput
                    className="edit__step-input"
                    value={ins.description}
                    placeholder={t('edit.step_placeholder')}
                    maxlength={2000}
                    multiline
                    onChange={(description) => patchInstruction(index, { description })}
                  />
                  <view className="edit__step-meta">
                    <text className="edit__timer">⏱ {t('edit.timer')}</text>
                    <view className="edit__spacer" />
                    <Counter
                      value={ins.time_next}
                      min={0}
                      max={600}
                      format={timer}
                      muted={ins.time_next === 0}
                      onChange={(time_next) => patchInstruction(index, { time_next })}
                    />
                    <view
                      className="edit__remove edit__remove--sm"
                      bindtap={() => patch({ instructions: draft.instructions.filter((_, k) => k !== index) })}
                    >
                      <text className="edit__remove-glyph">✕</text>
                    </view>
                  </view>
                </view>
              </view>
            ))}
            <view className="edit__add">
              <Button
                label={t('edit.add_step')}
                onTap={() =>
                  patch({ instructions: [...draft.instructions, { key: key(), description: '', time_next: 0 }] })
                }
                variant="secondary"
                block
              />
            </view>
          </view>

          {notice && (
            <view
              className={notice.kind === 'ok' ? 'edit__notice edit__notice--ok' : 'edit__notice edit__notice--error'}
            >
              <text className={notice.kind === 'ok' ? 'edit__notice-text edit__notice-text--ok' : 'edit__notice-text'}>
                {notice.kind === 'ok' ? '✓ ' : ''}
                {notice.text}
              </text>
            </view>
          )}
        </view>
      </scroll-view>

      <view className="edit__footer">
        <view className="edit__footer-text">
          <text className="edit__footer-title">{dirty ? t('edit.unsaved') : t('edit.clean')}</text>
          {dirty && <text className="muted">{t('edit.unsaved_hint')}</text>}
        </view>
        <Button label={t('edit.save')} onTap={() => void save()} disabled={busy || !dirty} />
      </view>

      <AisleSheet
        open={aisleFor !== null}
        name={aisleFor !== null ? (draft.ingredients[aisleFor]?.name ?? '') : ''}
        value={aisleFor !== null ? (draft.ingredients[aisleFor]?.category ?? null) : null}
        onPick={(category) => {
          if (aisleFor !== null) patchIngredient(aisleFor, { category })
          setAisleFor(null)
        }}
        onClose={() => setAisleFor(null)}
      />

      <Sheet open={discardOpen} onClose={() => setDiscardOpen(false)}>
        <view className="edit__discard">
          <view className="edit__discard-icon">
            <text className="edit__discard-glyph">⚠</text>
          </view>
          <view className="edit__discard-text">
            <text className="h2">{t('edit.discard_title')}</text>
            <text className="body">{t('edit.discard_hint')}</text>
          </view>
        </view>
        <Button
          label={t('edit.save_leave')}
          onTap={() => {
            setDiscardOpen(false)
            void save().then((ok) => ok && back())
          }}
          disabled={busy}
          block
        />
        <Button label={t('edit.discard')} onTap={back} variant="secondary" block />
        <Button label={t('edit.keep_editing')} onTap={() => setDiscardOpen(false)} variant="ghost" />
      </Sheet>
    </view>
  )
}
