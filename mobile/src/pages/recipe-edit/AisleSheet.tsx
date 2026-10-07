import { AISLE_CATEGORIES, aisle } from '../../lib/course.js'
import { t } from '../../lib/i18n/index.js'
import { Button } from '../../ui/Button.js'
import { Sheet } from '../../ui/Sheet.js'
import './AisleSheet.css'

export interface AisleSheetProps {
  open: boolean
  /** The ingredient being classified, for the subtitle. */
  name: string
  value: string | null
  onPick: (category: string | null) => void
  onClose: () => void
}

/** Aisle picker behind an ingredient row's tinted square: two columns of aisle tiles plus "No aisle". */
export function AisleSheet({ open, name, value, onPick, onClose }: AisleSheetProps) {
  return (
    <Sheet open={open} onClose={onClose}>
      <view className="aisle__head">
        <text className="h2">{t('edit.aisle')}</text>
        <text className="muted">
          {t('edit.aisle_hint', { name: name.trim() === '' ? t('edit.ingredient') : name })}
        </text>
      </view>
      <view className="aisle__grid">
        {AISLE_CATEGORIES.map((cat) => {
          const s = aisle(`shopping_${cat}`)
          const on = value === cat
          return (
            <view key={cat} className={on ? 'aisle__tile aisle__tile--on' : 'aisle__tile'} bindtap={() => onPick(cat)}>
              <view className="aisle__icon" style={{ backgroundColor: s.soft }}>
                <text className="aisle__emoji">{s.emoji}</text>
              </view>
              <text className={on ? 'aisle__label aisle__label--on' : 'aisle__label'}>{s.label}</text>
              {on && <text className="aisle__check">✓</text>}
            </view>
          )
        })}
        <view
          className={
            value === null
              ? 'aisle__tile aisle__tile--none aisle__tile--on aisle__tile--none-on'
              : 'aisle__tile aisle__tile--none'
          }
          bindtap={() => onPick(null)}
        >
          <view className="aisle__icon aisle__icon--none">
            <text className="aisle__emoji">🛒</text>
          </view>
          <text className={value === null ? 'aisle__label aisle__label--on' : 'aisle__label'}>
            {t('edit.aisle_none')}
          </text>
          {value === null && <text className="aisle__check">✓</text>}
        </view>
      </view>
      <Button label={t('common.cancel')} onTap={onClose} variant="secondary" block />
    </Sheet>
  )
}
