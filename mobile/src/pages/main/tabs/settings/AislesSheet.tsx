import { useEffect, useState } from '@lynx-js/react'

import { completeAisleOrder, moveAisle } from '../../../../lib/aisles.js'
import { AISLE_CATEGORIES, aisle } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'

export interface AislesSheetProps {
  open: boolean
  busy: boolean
  /** Saved order; the sheet edits a draft and hands it back on save. */
  value: string[]
  onClose: () => void
  onSave: (order: string[]) => void
}

/** Up/down reordering of the grocery aisles, plus a reset to the store-walk default. */
export function AislesSheet({ open, busy, value, onClose, onSave }: AislesSheetProps) {
  const [draft, setDraft] = useState<string[]>(() => completeAisleOrder(value))

  // Seed from the saved order each time the sheet opens (and only then, so a
  // background refresh of the tab never wipes edits); cancelling drops the draft.
  useEffect(() => {
    if (open) setDraft(completeAisleOrder(value))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  const last = draft.length - 1

  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <text className="h2">{t('settings.aisles')}</text>
      <text className="muted">{t('settings.aisles_hint')}</text>
      <view className="set-aisles">
        {draft.map((cat, i) => {
          const s = aisle(`shopping_${cat}`)
          return (
            <view key={cat} className={i === last ? 'set-aisle set-aisle--last' : 'set-aisle'}>
              <text className="set-aisle__index">{i + 1}</text>
              <view className="set-aisle__tile" style={{ backgroundColor: s.soft }}>
                <text className="set-aisle__emoji">{s.emoji}</text>
              </view>
              <text className="set-aisle__label">{s.label}</text>
              <view
                className={i === 0 ? 'set-aisle__btn set-aisle__btn--off' : 'set-aisle__btn'}
                bindtap={() => setDraft((d) => moveAisle(d, i, -1))}
              >
                <text className="set-aisle__glyph">↑</text>
              </view>
              <view
                className={i === last ? 'set-aisle__btn set-aisle__btn--off' : 'set-aisle__btn'}
                bindtap={() => setDraft((d) => moveAisle(d, i, 1))}
              >
                <text className="set-aisle__glyph">↓</text>
              </view>
            </view>
          )
        })}
      </view>
      <Button
        label={t('settings.aisles_reset')}
        onTap={() => setDraft([...AISLE_CATEGORIES])}
        variant="secondary"
        disabled={busy}
        block
      />
      <Button label={t('settings.save_aisles')} onTap={() => onSave(draft)} disabled={busy} block />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
