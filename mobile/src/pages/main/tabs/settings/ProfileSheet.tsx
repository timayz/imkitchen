import { useEffect, useState } from '@lynx-js/react'

import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'
import { TextField } from '../../../../ui/TextField.js'

const MAX = 500

export interface ProfileSheetProps {
  open: boolean
  busy: boolean
  username: string
  description: string
  onClose: () => void
  onSave: (description: string) => void
}

export function ProfileSheet({ open, busy, username, description, onClose, onSave }: ProfileSheetProps) {
  const [draft, setDraft] = useState(description)

  useEffect(() => {
    if (open) setDraft(description)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <text className="h2">{t('settings.profile')}</text>
      <text className="body">{t('settings.profile_hint')}</text>
      <view className="set-locked">
        <view className="set-locked__body">
          <text className="set-locked__label">{t('settings.username')}</text>
          <text className="set-locked__value">{username}</text>
        </view>
        <text className="set-locked__meta">{t('settings.username_locked')}</text>
        <text className="set-locked__glyph">🔒</text>
      </view>
      {/* Remount per opening so the uncontrolled field picks up the saved text. */}
      {open && (
        <view className="field">
          <TextField label={t('settings.description')} value={description} onChange={setDraft} maxlength={MAX} />
          <text className="set-counter">{`${draft.length} / ${MAX}`}</text>
        </view>
      )}
      <Button label={t('settings.save_profile')} onTap={() => onSave(draft)} disabled={busy} block />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
