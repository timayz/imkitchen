import './Chip.css'

export interface ChipProps {
  label: string
  active: boolean
  onTap: () => void
}

export function Chip({ label, active, onTap }: ChipProps) {
  return (
    <view className={active ? 'chip chip--active' : 'chip'} bindtap={onTap}>
      <text className={active ? 'chip__text chip__text--active' : 'chip__text'}>{label}</text>
    </view>
  )
}
