import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
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
        <Option
          emoji="✍️"
          tint="#fde3cf"
          title={t('recipes.new')}
          hint={t('recipes.new_hint')}
          onTap={busy ? undefined : onNew}
        />
        <Option
          emoji="🔗"
          tint="#dde9f5"
          title={t('recipes.import_link')}
          hint={t('recipes.import_hint')}
          onTap={busy ? undefined : onImport}
        />
      </view>
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" />
    </Sheet>
  )
}

interface OptionProps {
  emoji: string
  tint: string
  title: string
  hint: string
  onTap?: () => void
}

function Option({ emoji, tint, title, hint, onTap }: OptionProps) {
  return (
    <view className="add__option" bindtap={onTap}>
      <view className="add__tile" style={{ backgroundColor: tint }}>
        <text className="add__emoji">{emoji}</text>
      </view>
      <view className="add__text">
        <text className="add__title">{title}</text>
        <text className="add__hint">{hint}</text>
      </view>
      <text className="add__chevron">›</text>
    </view>
  )
}
