import { t } from '../lib/i18n/index.js'
import './OfflineBanner.css'

export interface OfflineBannerProps {
  /** Changes waiting in the offline queue. */
  pending?: number
  onRetry?: () => void
}

/** One-line strip shown while the app runs on what it saved locally. */
export function OfflineBanner({ pending = 0, onRetry }: OfflineBannerProps) {
  return (
    <view className="offline">
      <text className="offline__text">{pending > 0 ? t('offline.pending', { n: pending }) : t('offline.banner')}</text>
      {onRetry && (
        <view className="offline__retry" bindtap={onRetry}>
          <text className="offline__retry-text">{t('common.retry')}</text>
        </view>
      )}
    </view>
  )
}
