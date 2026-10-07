import type { SortBy } from '../lib/api/recipes.js'
import { t } from '../lib/i18n/index.js'
import { Button } from './Button.js'
import { Sheet } from './Sheet.js'
import './SortSheet.css'

const ALL_SORTS: SortBy[] = ['RecentlyAdded', 'Easiest', 'Hardest', 'Random']

export interface SortSheetProps {
  open: boolean
  value: SortBy
  onClose: () => void
  /** Applied immediately; the list behind the sheet reloads. */
  onPick: (sort: SortBy) => void
  /** The orders offered; the library's four by default. */
  sorts?: SortBy[]
}

/** Sort order picker behind the filter row's sort chip. */
export function SortSheet({ open, value, onClose, onPick, sorts = ALL_SORTS }: SortSheetProps) {
  return (
    <Sheet open={open} onClose={onClose}>
      <text className="h2">{t('recipes.sort')}</text>
      <view className="sort__list">
        {sorts.map((sort, i) => {
          const on = sort === value
          const classes = ['sort__row']
          if (i < sorts.length - 1) classes.push('sort__row--divided')
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
