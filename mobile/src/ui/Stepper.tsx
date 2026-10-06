import './Stepper.css'

export interface StepperProps {
  label: string
  value: number
  min: number
  max: number
  step?: number
  unit?: string
  onChange: (value: number) => void
}

/** Numeric picker without a keyboard: −/+ around the value. */
export function Stepper({ label, value, min, max, step = 1, unit, onChange }: StepperProps) {
  const clamp = (n: number) => Math.min(max, Math.max(min, n))
  return (
    <view className="stepper">
      <text className="field__label">{label}</text>
      <view className="stepper__row">
        <view className="stepper__btn" bindtap={() => onChange(clamp(value - step))}>
          <text className="stepper__glyph">−</text>
        </view>
        <text className="stepper__value">
          {value}
          {unit ? ` ${unit}` : ''}
        </text>
        <view className="stepper__btn" bindtap={() => onChange(clamp(value + step))}>
          <text className="stepper__glyph">+</text>
        </view>
      </view>
    </view>
  )
}
