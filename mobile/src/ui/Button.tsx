import './Button.css'

export interface ButtonProps {
  label: string
  onTap: () => void
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'ink'
  disabled?: boolean
  block?: boolean
  /** 52px tall primary action (sign-in forms). */
  size?: 'md' | 'lg'
}

export function Button({ label, onTap, variant = 'primary', disabled, block, size = 'md' }: ButtonProps) {
  const classes = ['btn', `btn--${variant}`]
  if (disabled) classes.push('btn--disabled')
  if (block) classes.push('btn--block')
  if (size === 'lg') classes.push('btn--lg')
  return (
    <view className={classes.join(' ')} bindtap={disabled ? undefined : onTap}>
      <text className={`btn__text btn__text--${variant}`}>{label}</text>
    </view>
  )
}
