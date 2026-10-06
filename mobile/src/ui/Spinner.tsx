import './Spinner.css'

export interface SpinnerProps {
  /** 28px ring for full-screen loading states; 20px otherwise. */
  size?: 'md' | 'lg'
}

export function Spinner({ size = 'md' }: SpinnerProps) {
  return <view className={size === 'lg' ? 'spinner spinner--lg' : 'spinner'} />
}
