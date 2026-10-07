import './Counter.css'

export interface CounterProps {
  value: number
  min: number
  max: number
  step?: number
  /** Text shown for the value; defaults to the number itself. */
  format?: (value: number) => string
  /** Greys the value out (a timer at "None"). */
  muted?: boolean
  onChange: (value: number) => void
}

/**
 * Joined −/value/+ group without a keyboard (ingredient quantities, timers,
 * serves). The labelled, stand-alone picker is `Stepper`.
 */
export function Counter({ value, min, max, step = 1, format, muted, onChange }: CounterProps) {
  const clamp = (n: number) => Math.min(max, Math.max(min, n))
  const atMin = value <= min
  const atMax = value >= max
  return (
    <view className="counter">
      <view
        className={atMin ? 'counter__btn counter__btn--off' : 'counter__btn'}
        bindtap={() => onChange(clamp(value - step))}
      >
        <text className="counter__glyph">−</text>
      </view>
      <text className={muted ? 'counter__value counter__value--muted' : 'counter__value'}>
        {format ? format(value) : value}
      </text>
      <view
        className={atMax ? 'counter__btn counter__btn--off' : 'counter__btn'}
        bindtap={() => onChange(clamp(value + step))}
      >
        <text className="counter__glyph">+</text>
      </view>
    </view>
  )
}
