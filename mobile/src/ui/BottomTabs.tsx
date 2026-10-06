import './BottomTabs.css'

export interface Tab<K extends string> {
  key: K
  label: string
  /** Transparent PNG (see scripts/nav-icons.mjs), tinted to the tab's state. */
  icon: string
}

export interface BottomTabsProps<K extends string> {
  tabs: Tab<K>[]
  active: K
  onSelect: (key: K) => void
}

// `tint-color` takes a literal colour, not a CSS variable: these mirror
// `--color-primary-500` / `--color-ink-3` in styles/tokens.css, which are the
// web nav's `text-primary-500` / `text-ink-3`.
const TINT_ACTIVE = '#ef6c1e'
const TINT_IDLE = '#6f6354'

/**
 * Sparkling has no native tab bar; this view-based one lives inside the
 * `main` bundle so switching tabs never leaves the container.
 */
export function BottomTabs<K extends string>({ tabs, active, onSelect }: BottomTabsProps<K>) {
  const inset = lynx.__globalProps.bottomHeight ?? 0
  return (
    <view className="tabs" style={{ paddingBottom: `${inset}px` }}>
      {tabs.map((tab) => {
        const isActive = tab.key === active
        return (
          <view key={tab.key} className="tab" bindtap={() => onSelect(tab.key)}>
            <image className="tab__icon" src={tab.icon} tint-color={isActive ? TINT_ACTIVE : TINT_IDLE} />
            <text className={isActive ? 'tab__label tab__label--active' : 'tab__label'}>{tab.label}</text>
          </view>
        )
      })}
    </view>
  )
}
