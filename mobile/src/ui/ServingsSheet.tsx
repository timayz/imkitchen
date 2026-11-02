import { useEffect, useState } from '@lynx-js/react'

import { defaultHouseholdSize } from '../lib/api/settings.js'
import { t } from '../lib/i18n/index.js'
import { Button } from './Button.js'
import { Sheet } from './Sheet.js'
import { Stepper } from './Stepper.js'
import './ServingsSheet.css'

export const MAX_SERVINGS = 20

export interface ServingsSheetProps {
  open: boolean
  busy: boolean
  title: string
  confirmLabel: string
  onClose: () => void
  onConfirm: (household_size: number) => void
}

/**
 * "For how many people?" picker shown before a recipe or dish is added to
 * the list. Opens on the settings household size; the choice is one-off.
 */
export function ServingsSheet({ open, busy, title, confirmLabel, onClose, onConfirm }: ServingsSheetProps) {
  const [people, setPeople] = useState<number | null>(null)

  useEffect(() => {
    if (!open) return
    setPeople(null)
    void defaultHouseholdSize().then((n) => setPeople(Math.min(MAX_SERVINGS, Math.max(1, n))))
  }, [open])

  const value = people ?? 1
  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <text className="servings__title">{title}</text>
      <text className="body">{t('list.people_hint')}</text>
      <Stepper label={t('list.people')} value={value} min={1} max={MAX_SERVINGS} onChange={setPeople} />
      <Button
        label={confirmLabel}
        onTap={() => onConfirm(value)}
        disabled={busy || people === null}
        variant="ink"
        block
      />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
