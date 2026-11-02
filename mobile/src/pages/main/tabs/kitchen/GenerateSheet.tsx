import { useEffect, useState } from '@lynx-js/react'

import { defaultHouseholdSize } from '../../../../lib/api/settings.js'
import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { MAX_SERVINGS } from '../../../../ui/ServingsSheet.js'
import { Sheet } from '../../../../ui/Sheet.js'
import { Stepper } from '../../../../ui/Stepper.js'
import './GenerateSheet.css'

const PRESETS = [3, 7, 14, 30]

export interface GenerateSheetProps {
  open: boolean
  busy: boolean
  /** Meals still to cook on the current list; generating replaces them. */
  remaining: number
  onClose: () => void
  /** `household_size` is the people count every generated recipe is scaled to. */
  onGenerate: (count: number, household_size: number) => void
}

/** "How many meals?" and "how many people?" picker, the web's generate modal. */
export function GenerateSheet({ open, busy, remaining, onClose, onGenerate }: GenerateSheetProps) {
  const [count, setCount] = useState(7)
  const [people, setPeople] = useState<number | null>(null)
  const clamp = (n: number) => Math.min(30, Math.max(1, n))

  // Defaults to the settings household size each time the sheet opens.
  useEffect(() => {
    if (!open) return
    setPeople(null)
    void defaultHouseholdSize().then((n) => setPeople(Math.min(MAX_SERVINGS, Math.max(1, n))))
  }, [open])

  return (
    <Sheet open={open} onClose={onClose}>
      <text className="gen__eyebrow">{t('kitchen.generate')}</text>
      <text className="body">{t('kitchen.generate_hint')}</text>
      {remaining > 0 && (
        <view className="gen__warn">
          <text className="gen__warn-text">
            ⚠️ {remaining === 1 ? t('kitchen.replaces_one') : t('kitchen.replaces', { n: remaining })}
          </text>
        </view>
      )}
      <text className="gen__label">{t('kitchen.how_many')}</text>
      <view className="gen__presets">
        {PRESETS.map((n) => (
          <view
            key={n}
            className={n === count ? 'gen__preset gen__preset--active' : 'gen__preset'}
            bindtap={() => setCount(n)}
          >
            <text className={n === count ? 'gen__preset-text gen__preset-text--active' : 'gen__preset-text'}>{n}</text>
          </view>
        ))}
      </view>
      <view className="gen__stepper">
        <view className="gen__step-btn" bindtap={() => setCount(clamp(count - 1))}>
          <text className="gen__step-glyph">−</text>
        </view>
        <view className="gen__num-wrap">
          <text className="gen__num">{count}</text>
          <text className="gen__unit">{count === 1 ? t('kitchen.meal') : t('kitchen.meals')}</text>
        </view>
        <view className="gen__step-btn" bindtap={() => setCount(clamp(count + 1))}>
          <text className="gen__step-glyph">+</text>
        </view>
      </view>
      <Stepper label={t('list.people')} value={people ?? 1} min={1} max={MAX_SERVINGS} onChange={setPeople} />
      <Button
        label={`✨ ${t('kitchen.generate_cta', { n: count })}`}
        onTap={() => onGenerate(count, people ?? 1)}
        disabled={busy || people === null}
        variant="ink"
        block
      />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
