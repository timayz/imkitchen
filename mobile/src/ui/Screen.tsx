import type { ReactNode } from '@lynx-js/react';

import '../styles/base.css';
import { insets } from '../lib/device.js';
import { theme } from '../lib/theme.js';

/**
 * The root of every page: fills the container, follows the system theme and
 * keeps the content clear of the status bar where the container runs behind
 * it. The bottom inset is left to the page, whose last element decides what
 * sits behind the home indicator (see `BottomTabs`).
 */
export function Screen({ children }: { children?: ReactNode }) {
  return (
    <view
      className={theme() === 'dark' ? 'screen theme--dark' : 'screen'}
      style={{ paddingTop: `${insets().top}px` }}
    >
      {children}
    </view>
  );
}
