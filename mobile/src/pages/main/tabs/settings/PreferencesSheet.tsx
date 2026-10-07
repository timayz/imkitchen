import { useEffect, useState } from '@lynx-js/react'

import type { DietaryRestriction, RecipeType } from '../../../../lib/api/recipe.js'
import type { Preferences } from '../../../../lib/api/settings.js'
import { course } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Chip } from '../../../../ui/Chip.js'
import { Sheet } from '../../../../ui/Sheet.js'
import { Stepper } from '../../../../ui/Stepper.js'

export const DIETS: DietaryRestriction[] = ['Vegetarian', 'Vegan', 'GlutenFree', 'DairyFree', 'NutFree']
export const OPTIONAL_COURSES: RecipeType[] = ['Appetizer', 'Accompaniment', 'Dessert', 'Beverage', 'Condiment']

const METER_STEPS = 10

export interface PreferencesSheetProps {
  open: boolean
  busy: boolean
  /** Saved values; the sheet edits a draft and hands it back on save. */
  value: Preferences
  onClose: () => void
  onSave: (draft: Preferences) => void
}

function toggle<T>(list: T[], item: T): T[] {
  return list.includes(item) ? list.filter((x) => x !== item) : [...list, item]
}

export function PreferencesSheet({ open, busy, value, onClose, onSave }: PreferencesSheetProps) {
  const [draft, setDraft] = useState<Preferences>(value)

  // Seed from the saved values each time the sheet opens (and only then, so a
  // background refresh of the tab never wipes edits); cancelling drops the draft.
  useEffect(() => {
    if (open) setDraft(value)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  const patch = (p: Partial<Preferences>) => setDraft((prev) => ({ ...prev, ...p }))
  const pct = Math.round(draft.cuisine_variety_weight * 100)
  const lit = Math.round((pct / 100) * METER_STEPS)

  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <text className="h2">{t('settings.preferences')}</text>
      <Stepper
        label={t('settings.household')}
        value={draft.household_size}
        min={1}
        max={20}
        onChange={(household_size) => patch({ household_size })}
      />
      <view className="field">
        <text className="field__label">{t('settings.diet')}</text>
        <view className="set-chips">
          {DIETS.map((d) => (
            <Chip
              key={d}
              label={t(`diet.${d}` as const)}
              active={draft.dietary_restrictions.includes(d)}
              onTap={() => patch({ dietary_restrictions: toggle(draft.dietary_restrictions, d) })}
            />
          ))}
        </view>
      </view>
      <view className="field">
        <text className="field__label">{t('settings.courses')}</text>
        <text className="muted">{t('settings.courses_hint')}</text>
        <view className="set-chips">
          {OPTIONAL_COURSES.map((type) => {
            const c = course(type)
            return (
              <Chip
                key={type}
                label={`${c.emoji} ${c.label}`}
                active={draft.recipe_types.includes(type)}
                onTap={() => patch({ recipe_types: toggle(draft.recipe_types, type) })}
              />
            )
          })}
        </view>
      </view>
      <view className="field">
        <Stepper
          label={t('settings.variety')}
          value={pct}
          min={10}
          max={100}
          step={10}
          unit="%"
          onChange={(next) => patch({ cuisine_variety_weight: next / 100 })}
        />
        <view className="set-meter">
          {Array.from({ length: METER_STEPS }, (_, i) => (
            <view key={i} className={i < lit ? 'set-meter__seg set-meter__seg--on' : 'set-meter__seg'} />
          ))}
        </view>
      </view>
      <Button label={t('settings.save_preferences')} onTap={() => onSave(draft)} disabled={busy} block />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
