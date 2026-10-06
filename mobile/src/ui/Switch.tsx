import './Switch.css'

export interface SwitchProps {
  on: boolean
  onToggle: () => void
}

/** On/off toggle: herb track when on, knob slides right. */
export function Switch({ on, onToggle }: SwitchProps) {
  return (
    <view className={on ? 'switch switch--on' : 'switch'} bindtap={onToggle}>
      <view className="switch__knob" />
    </view>
  )
}
