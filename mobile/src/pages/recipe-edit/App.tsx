import { useCallback, useEffect, useState } from '@lynx-js/react'

import '../../styles/base.css'
import './App.css'
import { ApiError } from '../../lib/api/client.js'
import type { DietaryRestriction, RecipeType } from '../../lib/api/recipe.js'
import {
  type IngredientInput,
  type RecipeInput,
  getEditable,
  updateRecipe,
  uploadThumbnail,
} from '../../lib/api/recipe-edit.js'
import { aisle, course } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { pickImage } from '../../lib/media.js'
import { back, pageParams } from '../../lib/nav.js'
import { Button } from '../../ui/Button.js'
import { Chip } from '../../ui/Chip.js'
import { Stepper } from '../../ui/Stepper.js'
import { TextField } from '../../ui/TextField.js'

const TYPES: RecipeType[] = ['Appetizer', 'MainCourse', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']
const DIETS: DietaryRestriction[] = ['Vegetarian', 'Vegan', 'GlutenFree', 'DairyFree', 'NutFree']
const UNITS: ('G' | 'ML' | null)[] = [null, 'G', 'ML']
const CATEGORIES = [
  'FruitsAndVegetables',
  'Butcher',
  'Seafood',
  'DairyAndEggs',
  'Bakery',
  'Grocery',
  'Frozen',
  'Refrigerated',
  'SnacksAndConfectionery',
]

type State = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready' }

/**
 * Recipe editor: `recipe-edit.lynx.bundle?id=…`. Dynamic ingredient and
 * instruction rows, course and diet pickers, photo upload.
 */
export function App() {
  const id = pageParams().id ?? ''
  const [state, setState] = useState<State>({ kind: 'loading' })
  const [form, setForm] = useState<RecipeInput | null>(null)
  const [busy, setBusy] = useState(false)
  const [notice, setNotice] = useState<{ kind: 'ok' | 'error'; text: string } | null>(null)

  useEffect(() => {
    getEditable(id)
      .then((input) => {
        setForm(input)
        setState({ kind: 'ready' })
      })
      .catch((err: unknown) =>
        setState({ kind: 'error', message: err instanceof ApiError ? err.message : t('error.network') }),
      )
  }, [id])

  const patch = (p: Partial<RecipeInput>) => setForm((f) => (f ? { ...f, ...p } : f))
  const patchIngredient = (index: number, p: Partial<IngredientInput>) =>
    setForm((f) =>
      f ? { ...f, ingredients: f.ingredients.map((i, k) => (k === index ? { ...i, ...p } : i)) } : f,
    )
  const patchInstruction = (index: number, p: { description?: string; time_next?: number }) =>
    setForm((f) =>
      f ? { ...f, instructions: f.instructions.map((i, k) => (k === index ? { ...i, ...p } : i)) } : f,
    )

  const save = useCallback(async () => {
    if (!form || busy) return
    setBusy(true)
    setNotice(null)
    try {
      await updateRecipe(id, {
        ...form,
        ingredients: form.ingredients.filter((i) => i.name.trim() !== ''),
        instructions: form.instructions.filter((i) => i.description.trim() !== ''),
      })
      setNotice({ kind: 'ok', text: t('edit.saved') })
    } catch (err) {
      setNotice({ kind: 'error', text: err instanceof ApiError ? err.message : t('error.network') })
    } finally {
      setBusy(false)
    }
  }, [busy, form, id])

  const photo = useCallback(async () => {
    if (busy) return
    setBusy(true)
    setNotice(null)
    try {
      const picked = await pickImage()
      if (!picked) return
      await uploadThumbnail(id, picked.mimeType, picked.base64)
      setNotice({ kind: 'ok', text: t('edit.photo_uploaded') })
    } catch (err) {
      setNotice({ kind: 'error', text: err instanceof ApiError ? err.message : String(err) })
    } finally {
      setBusy(false)
    }
  }, [busy, id])

  if (state.kind === 'loading' || !form) {
    return (
      <view className="screen cook--center">
        <text className="muted">{state.kind === 'error' ? state.message : t('common.loading')}</text>
        {state.kind === 'error' && <Button label={t('common.close')} onTap={back} variant="secondary" />}
      </view>
    )
  }

  return (
    <view className="screen">
      <scroll-view className="edit__scroll" scroll-orientation="vertical">
        <view className="content">
          <view className="row" style={{ gap: '12px' }}>
            <view className="cook__close" bindtap={back}>
              <text className="cook__close-text">←</text>
            </view>
            <text className="h2">{t('edit.title')}</text>
          </view>

          <view className="card">
            <TextField label={t('edit.name')} value={form.name} onChange={(name) => patch({ name })} maxlength={100} />
            <TextField
              label={t('edit.description')}
              value={form.description}
              onChange={(description) => patch({ description })}
              maxlength={2000}
            />
            <TextField
              label={t('edit.origin')}
              value={form.origin ?? ''}
              onChange={(origin) => patch({ origin: origin.trim() === '' ? null : origin })}
              placeholder="https://"
              maxlength={255}
            />
            <text className="field__label">{t('edit.course')}</text>
            <view className="row edit__chips">
              {TYPES.map((type) => {
                const c = course(type)
                return (
                  <Chip key={type} label={`${c.emoji} ${c.label}`} active={form.recipe_type === type} onTap={() => patch({ recipe_type: type })} />
                )
              })}
            </view>
            <Stepper label={t('edit.household')} value={form.household_size} min={1} max={20} onChange={(household_size) => patch({ household_size })} />
            <Stepper label={t('edit.prep')} value={form.prep_time} min={0} max={600} step={5} unit="min" onChange={(prep_time) => patch({ prep_time })} />
            <Stepper label={t('edit.cook')} value={form.cook_time} min={0} max={600} step={5} unit="min" onChange={(cook_time) => patch({ cook_time })} />
            <text className="field__label">{t('edit.diet')}</text>
            <view className="row edit__chips">
              {DIETS.map((d) => (
                <Chip
                  key={d}
                  label={t(`diet.${d}` as const)}
                  active={form.dietary_restrictions.includes(d)}
                  onTap={() =>
                    patch({
                      dietary_restrictions: form.dietary_restrictions.includes(d)
                        ? form.dietary_restrictions.filter((x) => x !== d)
                        : [...form.dietary_restrictions, d],
                    })
                  }
                />
              ))}
            </view>
            {form.recipe_type === 'MainCourse' && (
              <Chip
                label={form.accepts_accompaniment ? t('edit.accompaniment_on') : t('edit.accompaniment_off')}
                active={form.accepts_accompaniment}
                onTap={() => patch({ accepts_accompaniment: !form.accepts_accompaniment })}
              />
            )}
            <TextField
              label={t('edit.advance_prep')}
              value={form.advance_prep}
              onChange={(advance_prep) => patch({ advance_prep })}
              maxlength={2000}
            />
          </view>

          <view className="card">
            <text className="h2">{t('recipes.ingredients')}</text>
            {form.ingredients.map((ing, index) => (
              <view key={index} className="edit__row">
                <TextField label={t('edit.ingredient')} value={ing.name} onChange={(name) => patchIngredient(index, { name })} maxlength={100} />
                <Stepper
                  label={t('edit.quantity')}
                  value={ing.quantity}
                  min={0}
                  max={100000}
                  step={ing.unit ? 10 : 1}
                  unit={ing.unit === 'G' ? 'g' : ing.unit === 'ML' ? 'ml' : undefined}
                  onChange={(quantity) => patchIngredient(index, { quantity })}
                />
                <view className="row edit__chips">
                  {UNITS.map((u) => (
                    <Chip key={u ?? 'none'} label={u ?? t('edit.unit_none')} active={ing.unit === u} onTap={() => patchIngredient(index, { unit: u })} />
                  ))}
                </view>
                <view className="row edit__chips">
                  {CATEGORIES.map((cat) => {
                    const s = aisle(`shopping_${cat}`)
                    return (
                      <Chip
                        key={cat}
                        label={`${s.emoji} ${s.label}`}
                        active={ing.category === cat}
                        onTap={() => patchIngredient(index, { category: ing.category === cat ? null : cat })}
                      />
                    )
                  })}
                </view>
                <Button
                  label={t('edit.remove_row')}
                  onTap={() => patch({ ingredients: form.ingredients.filter((_, k) => k !== index) })}
                  variant="ghost"
                />
              </view>
            ))}
            <Button
              label={t('edit.add_ingredient')}
              onTap={() =>
                patch({ ingredients: [...form.ingredients, { name: '', quantity: 1, unit: null, category: null }] })
              }
              variant="secondary"
            />
          </view>

          <view className="card">
            <text className="h2">{t('recipes.instructions')}</text>
            {form.instructions.map((ins, index) => (
              <view key={index} className="edit__row">
                <TextField
                  label={t('edit.step_n', { n: index + 1 })}
                  value={ins.description}
                  onChange={(description) => patchInstruction(index, { description })}
                  maxlength={2000}
                />
                <Stepper
                  label={t('edit.timer')}
                  value={ins.time_next}
                  min={0}
                  max={600}
                  unit="min"
                  onChange={(time_next) => patchInstruction(index, { time_next })}
                />
                <Button
                  label={t('edit.remove_row')}
                  onTap={() => patch({ instructions: form.instructions.filter((_, k) => k !== index) })}
                  variant="ghost"
                />
              </view>
            ))}
            <Button
              label={t('edit.add_step')}
              onTap={() => patch({ instructions: [...form.instructions, { description: '', time_next: 0 }] })}
              variant="secondary"
            />
          </view>

          <view className="card">
            <text className="h2">{t('edit.photo')}</text>
            <text className="muted">{t('edit.photo_hint')}</text>
            <Button label={t('edit.pick_photo')} onTap={photo} variant="secondary" disabled={busy} />
          </view>

          {notice && <text className={notice.kind === 'ok' ? 'success' : 'error'}>{notice.text}</text>}
        </view>
      </scroll-view>
      <view className="edit__footer">
        <Button label={t('edit.save')} onTap={save} disabled={busy} block />
      </view>
    </view>
  )
}
