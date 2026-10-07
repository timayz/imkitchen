import './Chip.css'

export interface ChipProps {
  label: string
  active: boolean
  onTap: () => void
  /** Colours used instead of ink when active (course-tinted filter chips). */
  tint?: { bg: string; fg: string }
}

export function Chip({ label, active, onTap, tint }: ChipProps) {
  const tinted = active && tint
  return (
    <view
      className={active ? 'chip chip--active' : 'chip'}
      style={tinted ? { backgroundColor: tint.bg, borderColor: tint.bg } : undefined}
      bindtap={onTap}
    >
      <text
        className={active ? 'chip__text chip__text--active' : 'chip__text'}
        style={tinted ? { color: tint.fg } : undefined}
      >
        {label}
      </text>
    </view>
  )
}
