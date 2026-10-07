import './OptionRow.css'

export interface OptionRowProps {
  emoji: string
  /** Background of the emoji tile (a literal colour: Lynx styles take no CSS variables here). */
  tint: string
  title: string
  hint: string
  /** `paper` on the cream page ground, `cream` inside a white sheet or card. */
  tone?: 'paper' | 'cream'
  onTap?: () => void
}

/** A tappable "do this" row: emoji tile, title, hint, chevron. */
export function OptionRow({ emoji, tint, title, hint, tone = 'paper', onTap }: OptionRowProps) {
  return (
    <view className={`option-row option-row--${tone}`} bindtap={onTap}>
      <view className="option-row__tile" style={{ backgroundColor: tint }}>
        <text className="option-row__emoji">{emoji}</text>
      </view>
      <view className="option-row__text">
        <text className="option-row__title">{title}</text>
        <text className="option-row__hint">{hint}</text>
      </view>
      <text className="option-row__chevron">›</text>
    </view>
  )
}
