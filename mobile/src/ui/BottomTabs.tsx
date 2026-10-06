import './BottomTabs.css'

export interface Tab<K extends string> {
  key: K
  label: string
  glyph: string
}

export interface BottomTabsProps<K extends string> {
  tabs: Tab<K>[]
  active: K
  onSelect: (key: K) => void
}

/**
 * Sparkling has no native tab bar; this view-based one lives inside the
 * `main` bundle so switching tabs never leaves the container.
 */
export function BottomTabs<K extends string>({ tabs, active, onSelect }: BottomTabsProps<K>) {
  const inset = lynx.__globalProps.bottomHeight ?? 0
  return (
    <view className="tabs" style={{ paddingBottom: `${inset}px` }}>
      {tabs.map((tab) => (
        <view
          key={tab.key}
          className={tab.key === active ? 'tab tab--active' : 'tab'}
          bindtap={() => onSelect(tab.key)}
        >
          <text className="tab__glyph">{tab.glyph}</text>
          <text className={tab.key === active ? 'tab__label tab__label--active' : 'tab__label'}>
            {tab.label}
          </text>
        </view>
      ))}
    </view>
  )
}
