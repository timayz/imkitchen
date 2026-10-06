import { useEffect, useState } from '@lynx-js/react'

import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'
import { TextField } from '../../../../ui/TextField.js'

export interface DeleteSheetProps {
  open: boolean
  busy: boolean
  error: string | null
  onClose: () => void
  onConfirm: (password: string) => void
}

export function DeleteSheet({ open, busy, error, onClose, onConfirm }: DeleteSheetProps) {
  const [password, setPassword] = useState('')

  useEffect(() => {
    if (open) setPassword('')
  }, [open])

  const confirm = () => {
    if (password) onConfirm(password)
  }
  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <view className="set-delete__icon">
        <text className="set-delete__emoji">🗑️</text>
      </view>
      <text className="h2">{t('settings.delete_confirm_title')}</text>
      <text className="body">
        {t('settings.delete_hint')} {t('settings.delete_confirm_hint')}
      </text>
      {/* Remount per opening so a cancelled attempt leaves no password behind. */}
      {open && (
        <TextField
          label={t('login.password')}
          value=""
          onChange={setPassword}
          type="password"
          confirmType="go"
          onConfirm={confirm}
        />
      )}
      {error && <text className="error">{error}</text>}
      <Button
        label={t('settings.delete_confirm')}
        onTap={confirm}
        variant="danger"
        disabled={busy || !password}
        block
      />
      <Button label={t('common.cancel')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
