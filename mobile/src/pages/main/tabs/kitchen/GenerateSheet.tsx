import { useState } from '@lynx-js/react'

import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'
import './GenerateSheet.css'

const PRESETS = [3, 7, 14, 30]

export interface GenerateSheetProps {
  open: boolean
  busy: boolean
  onClose: () => void
  onGenerate: (count: number) => void
}

/** "How many meals?" picker, the web's generate modal. */
export function GenerateSheet({ open, busy, onClose, onGenerate }: GenerateSheetProps) {
  const [count, setCount] = useState(7)
  const clamp = (n: number) => Math.min(30, Math.max(1, n))

  return (
    <Sheet open={open} onClose={onClose}>
      <text className="gen__eyebrow">{t('kitchen.generate')}</text>
      <text className="body">{t('kitchen.generate_hint')}</text>
      <text className="gen__label">{t('kitchen.how_many')}</text>
      <view className="gen__presets">
        {PRESETS.map((n) => (
          <view
            key={n}
            className={n === count ? 'gen__preset gen__preset--active' : 'gen__preset'}
            bindtap={() => setCount(n)}
          >
            <text className={n === count ? 'gen__preset-text gen__preset-text--active' : 'gen__preset-text'}>
              {n}
            </text>
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
      <Button
        label={t('kitchen.generate_cta', { n: count })}
        onTap={() => onGenerate(count)}
        disabled={busy}
        block
      />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
