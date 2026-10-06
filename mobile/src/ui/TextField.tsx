import { useRef } from '@lynx-js/react'

import './TextField.css'

export interface TextFieldProps {
  label: string
  /** Initial text. Later changes are ignored: the field is uncontrolled. */
  value: string
  onChange: (value: string) => void
  placeholder?: string
  type?: 'text' | 'password' | 'email' | 'number' | 'digit' | 'tel'
  /** Lynx's platform default is 140; always explicit. */
  maxlength?: number
  confirmType?: 'done' | 'next' | 'go' | 'search' | 'send'
  onConfirm?: () => void
}

/**
 * Uncontrolled on purpose: echoing every keystroke back through the `value`
 * prop races the native EditText (the JS state lags one event behind) and
 * drops or reorders characters. The host input keeps the text; `onChange`
 * lifts it.
 */
export function TextField({
  label,
  value,
  onChange,
  placeholder,
  type = 'text',
  maxlength = 255,
  confirmType = 'done',
  onConfirm,
}: TextFieldProps) {
  const initial = useRef(value)
  return (
    <view className="field">
      <text className="field__label">{label}</text>
      <input
        className="field__input"
        value={initial.current}
        placeholder={placeholder}
        type={type}
        maxlength={maxlength}
        confirm-type={confirmType}
        text-color="#1b140c"
        bindinput={(e) => onChange(e.detail.value)}
        bindconfirm={onConfirm ? () => onConfirm() : undefined}
      />
    </view>
  )
}
