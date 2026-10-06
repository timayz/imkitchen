import { useRef } from '@lynx-js/react'

import './TextField.css'

export interface FieldAction {
  label: string
  onTap: () => void
}

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
  /** Small link at the right of the label ("Forgot your password?"). */
  labelAction?: FieldAction
  /** Tappable text inside the field, at its right edge ("Show" / "Hide"). */
  trailing?: FieldAction
  /** Outlines the field in the error color. */
  invalid?: boolean
  /** Field fill; `cream` reads better on a paper card. */
  surface?: 'paper' | 'cream'
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
  labelAction,
  trailing,
  invalid,
  surface = 'paper',
}: TextFieldProps) {
  const initial = useRef(value)
  const inputClasses = ['field__input']
  if (multiline) inputClasses.push('field__input--multi')
  if (trailing) inputClasses.push('field__input--trailing')
  if (surface === 'cream') inputClasses.push('field__input--cream')
  if (invalid) inputClasses.push('field__input--invalid')
  return (
    <view className="field">
      <view className="field__head">
        <text className="field__label">{label}</text>
        {labelAction && (
          <text className="field__action" bindtap={labelAction.onTap}>
            {labelAction.label}
          </text>
        )}
      </view>
      <view className="field__box">
        <input
          className={inputClasses.join(' ')}
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
        {trailing && (
          <view className="field__trailing" bindtap={trailing.onTap}>
            <text className="field__trailing-text">{trailing.label}</text>
          </view>
        )}
      </view>
      {hint && <text className="field__hint">{hint}</text>}
    </view>
  )
}
