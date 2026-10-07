import './Segmented.css'

export interface SegmentedProps<T extends string | null> {
  options: { value: T; label: string }[]
  value: T
  onChange: (value: T) => void
}

/** Small exclusive choice (units): one ink-filled segment in a white track. */
export function Segmented<T extends string | null>({ options, value, onChange }: SegmentedProps<T>) {
  return (
    <view className="seg">
      {options.map((o) => {
        const on = o.value === value
        return (
          <view
            key={o.value ?? ''}
            className={on ? 'seg__item seg__item--on' : 'seg__item'}
            bindtap={() => onChange(o.value)}
          >
            <text className={on ? 'seg__text seg__text--on' : 'seg__text'}>{o.label}</text>
          </view>
        )
      })}
    </view>
  )
}
