import type { ReactNode } from '@lynx-js/react'

import './Sheet.css'

export interface SheetProps {
  open: boolean
  onClose: () => void
  children: ReactNode
}

/** Bottom sheet over the current screen (the web's mobile modal). */
export function Sheet({ open, onClose, children }: SheetProps) {
  if (!open) return null
  return (
    <view className="sheet__backdrop" bindtap={onClose}>
      <view className="sheet" catchtap={() => {}}>
        <view className="sheet__handle" />
        {children}
      </view>
    </view>
  )
}
