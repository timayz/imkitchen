import type { Entry } from '../../../../lib/api/kitchen.js'
import { course, minutes } from '../../../../lib/course.js'
import { t } from '../../../../lib/i18n/index.js'
import { type Item, cookLabel, doneCount } from '../../../../lib/kitchen-items.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'
import './PlatSheet.css'

export interface PlatSheetProps {
  /** The dish whose recipes are listed; `null` keeps the sheet closed. */
  plat: (Item & { kind: 'dish' }) | null
  busy: boolean
  onClose: () => void
  /** Opens the cooking screen of one recipe of the dish. */
  onCook: (id: string) => void
  /** Takes the whole dish off the list. */
  onRemove: () => void
}

/** The recipes of a composed dish, each with its cook button (the web's dish modal). */
export function PlatSheet({ plat, busy, onClose, onCook, onRemove }: PlatSheetProps) {
  return (
    <Sheet open={plat !== null} onClose={onClose}>
      {plat && (
        <>
          <text className="plat__eyebrow">
            {t('kitchen.dish')} · {t('kitchen.dish_progress', { done: doneCount(plat.entries), total: plat.entries.length })}
          </text>
          <text className="plat__title">{plat.name}</text>
          <text className="body">{t('kitchen.dish_hint')}</text>
          <view className="plat__list">
            {plat.entries.map((entry) => (
              <PlatRow key={entry.id} entry={entry} busy={busy} onCook={() => onCook(entry.id)} />
            ))}
          </view>
          <Button label={t('kitchen.remove_dish')} onTap={onRemove} variant="danger" disabled={busy} block />
          <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
        </>
      )}
    </Sheet>
  )
}

function PlatRow({ entry, busy, onCook }: { entry: Entry; busy: boolean; onCook: () => void }) {
  const c = course(entry.recipe_type)
  const done = entry.status.status === 'completed'
  const cooking = entry.status.status === 'cooking'
  return (
    <view className={done ? 'plat__row plat__row--done' : 'plat__row'}>
      <view className="plat__tile" style={{ backgroundColor: c.soft }}>
        <text className="plat__emoji">{c.emoji}</text>
      </view>
      <view className="plat__body">
        <text className="plat__course" style={{ color: c.ink }}>
          {c.label}
        </text>
        <text className={done ? 'plat__name plat__name--done' : 'plat__name'} text-maxline="1">
          {entry.name}
        </text>
        <text className="plat__meta">
          {minutes(entry.total_time)}
          {done ? ` · ✓ ${t('kitchen.cooked_badge')}` : cooking ? ` · 🔥 ${t('kitchen.cooking')}` : ''}
        </text>
      </view>
      <view className={done ? 'plat__cook plat__cook--again' : 'plat__cook'} bindtap={busy ? undefined : onCook}>
        <text className={done ? 'plat__cook-text plat__cook-text--again' : 'plat__cook-text'}>
          ▶ {t(cookLabel(entry.status))}
        </text>
      </view>
    </view>
  )
}
