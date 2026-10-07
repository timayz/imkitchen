import type { Session } from '../../../../lib/api/settings.js'
import { parseDevice } from '../../../../lib/device.js'
import { t } from '../../../../lib/i18n/index.js'
import { Button } from '../../../../ui/Button.js'
import { Sheet } from '../../../../ui/Sheet.js'

export interface DevicesSheetProps {
  open: boolean
  busy: boolean
  sessions: Session[]
  onClose: () => void
  onRevoke: (id: string) => void
  onRevokeOthers: () => void
}

/** `User-Agent` → "Pixel 8 · imkitchen app" / "Chrome · macOS" plus a tile emoji. */
export function describeDevice(userAgent: string): { name: string; emoji: string } {
  const d = parseDevice(userAgent)
  switch (d.kind) {
    case 'app':
      return { name: t('settings.app_device', { model: d.model }), emoji: '📱' }
    case 'browser':
      return { name: d.os ? `${d.browser} · ${d.os}` : d.browser, emoji: d.mobile ? '📱' : '💻' }
    default:
      return { name: t('settings.unknown_device'), emoji: '💻' }
  }
}

export function DevicesSheet({ open, busy, sessions, onClose, onRevoke, onRevokeOthers }: DevicesSheetProps) {
  const others = sessions.filter((s) => !s.current)
  return (
    <Sheet open={open} onClose={() => !busy && onClose()}>
      <text className="h2">{t('settings.sessions')}</text>
      <text className="body">{t('settings.sessions_hint')}</text>
      <view className="set-group">
        {sessions.map((s, i) => {
          const d = describeDevice(s.user_agent)
          return (
            <view key={s.id} className={i === sessions.length - 1 ? 'set-row set-row--last' : 'set-row'}>
              <view className={s.current ? 'set-row__tile set-row__tile--herb' : 'set-row__tile set-row__tile--cream'}>
                <text className="set-row__emoji">{d.emoji}</text>
              </view>
              <view className="set-row__body">
                <text className="set-row__title">{d.name}</text>
                <text className="set-row__meta">{s.tz}</text>
              </view>
              {s.current ? (
                <view className="set-chip">
                  <text className="set-chip__text">{t('settings.this_device')}</text>
                </view>
              ) : (
                <view
                  className={busy ? 'set-smallbtn btn--disabled' : 'set-smallbtn'}
                  bindtap={busy ? undefined : () => onRevoke(s.id)}
                >
                  <text className="set-smallbtn__text">{t('settings.revoke')}</text>
                </view>
              )}
            </view>
          )
        })}
      </view>
      {others.length > 0 && (
        <Button label={t('settings.revoke_others')} onTap={onRevokeOthers} variant="secondary" disabled={busy} block />
      )}
      <Button label={t('common.close')} onTap={onClose} variant="ghost" disabled={busy} />
    </Sheet>
  )
}
