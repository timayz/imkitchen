import { bottomInset } from '../lib/device.js';
import { palette, theme } from '../lib/theme.js';
import './BottomTabs.css';

export interface Tab<K extends string> {
  key: K;
  label: string;
  /** Transparent PNG (see scripts/nav-icons.mjs), tinted to the tab's state. */
  icon: string;
}

export interface BottomTabsProps<K extends string> {
  tabs: Tab<K>[];
  active: K;
  onSelect: (key: K) => void;
}

/**
 * Sparkling has no native tab bar; this view-based one lives inside the
 * `main` bundle so switching tabs never leaves the container.
 */
export function BottomTabs<K extends string>({ tabs, active, onSelect }: BottomTabsProps<K>) {
  // `tint-color` takes a literal colour, not a CSS variable.
  const colors = palette[theme()];
  return (
    <view className="tabs" style={{ paddingBottom: `${bottomInset()}px` }}>
      {tabs.map((tab) => {
        const isActive = tab.key === active;
        return (
          <view
            key={tab.key}
            id={`tab-${tab.key}`}
            className="tab"
            accessibility-element={true}
            accessibility-label={tab.label}
            accessibility-traits="button"
            bindtap={() => onSelect(tab.key)}
          >
            <image
              className="tab__icon"
              src={tab.icon}
              tint-color={isActive ? colors.accent : colors.inkMuted}
            />
            <text className={isActive ? 'tab__label tab__label--active' : 'tab__label'}>
              {tab.label}
            </text>
          </view>
        );
      })}
    </view>
  );
}
