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
  /** A growing text area (the host's `multiline` input mode). */
  multiline?: boolean
  /** Caption under the field. */
  hint?: string
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
  multiline,
  hint,
}: TextFieldProps) {
  const initial = useRef(value)
  return (
    <view className="field">
      <text className="field__label">{label}</text>
      <input
        className={multiline ? 'field__input field__input--multi' : 'field__input'}
        value={initial.current}
        placeholder={placeholder}
        type={type}
        maxlength={maxlength}
        multiline={multiline}
        confirm-type={multiline ? undefined : confirmType}
        text-color="#1b140c"
        bindinput={(e) => onChange(e.detail.value)}
        bindconfirm={onConfirm && !multiline ? () => onConfirm() : undefined}
      />
      {hint && <text className="field__hint">{hint}</text>}
    </view>
  )
}
