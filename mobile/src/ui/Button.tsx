import './Button.css'

export interface ButtonProps {
  label: string
  onTap: () => void
  variant?: 'primary' | 'secondary' | 'ghost'
  disabled?: boolean
  block?: boolean
}

export function Button({ label, onTap, variant = 'primary', disabled, block }: ButtonProps) {
  const classes = ['btn', `btn--${variant}`]
  if (disabled) classes.push('btn--disabled')
  if (block) classes.push('btn--block')
  return (
    <view className={classes.join(' ')} bindtap={disabled ? undefined : onTap}>
      <text className={`btn__text btn__text--${variant}`}>{label}</text>
    </view>
  )
}
