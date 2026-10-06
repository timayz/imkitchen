import type { SortBy } from '../../../../lib/api/recipes.js'
import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'
import './SortSheet.css'

export const SORTS: SortBy[] = ['RecentlyAdded', 'Easiest', 'Hardest', 'Random']

export interface SortSheetProps {
  open: boolean
  value: SortBy
  onClose: () => void
  /** Applied immediately; the list behind the sheet reloads. */
  onPick: (sort: SortBy) => void
}

/** Sort order picker behind the filter row's sort chip. */
export function SortSheet({ open, value, onClose, onPick }: SortSheetProps) {
  return (
    <Sheet open={open} onClose={onClose}>
      <text className="h2">{t('recipes.sort')}</text>
      <view className="sort__list">
        {SORTS.map((sort, i) => {
          const on = sort === value
          const classes = ['sort__row']
          if (i < SORTS.length - 1) classes.push('sort__row--divided')
          return (
            <view key={sort} className={classes.join(' ')} bindtap={() => onPick(sort)}>
              <view className="sort__text">
                <text className="sort__label">{t(`sort.${sort}` as const)}</text>
                <text className="sort__hint">{t(`sort.${sort}_hint` as const)}</text>
              </view>
              <view className={on ? 'sort__dot sort__dot--on' : 'sort__dot'}>
                {on && <text className="sort__check">✓</text>}
              </view>
            </view>
          )
        })}
      </view>
      <Button label={t('common.done')} onTap={onClose} variant="ink" block />
    </Sheet>
  )
}
