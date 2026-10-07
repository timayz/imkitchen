import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { OptionRow } from '../../../../ui/OptionRow.js'
import { Sheet } from '../../../../ui/Sheet.js'
import './AddSheet.css'

export interface AddSheetProps {
  open: boolean
  busy: boolean
  onClose: () => void
  onNew: () => void
  onImport: () => void
}

/** "Add a recipe" picker behind the header's + button: write one, or import from a link. */
export function AddSheet({ open, busy, onClose, onNew, onImport }: AddSheetProps) {
  return (
    <Sheet open={open} onClose={onClose}>
      <text className="h2">{t('recipes.add')}</text>
      <view className="add__options">
        <OptionRow
          emoji="✍️"
          tint="#fde3cf"
          tone="cream"
          title={t('recipes.new')}
          hint={t('recipes.new_hint')}
          onTap={busy ? undefined : onNew}
        />
        <OptionRow
          emoji="🔗"
          tint="#dde9f5"
          tone="cream"
          title={t('recipes.import_link')}
          hint={t('recipes.import_hint')}
          onTap={busy ? undefined : onImport}
        />
      </view>
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" />
    </Sheet>
  )
}
