import type { ReactNode } from '@lynx-js/react';

import '../styles/base.css';
import { theme } from '../lib/theme.js';

/**
 * The root of every page: fills the container and follows the system theme.
 * The bottom inset is left to the page, whose last element decides what sits
 * behind the home indicator (see `BottomTabs`).
 */
export function Screen({ children }: { children?: ReactNode }) {
  return <view className={theme() === 'dark' ? 'screen theme--dark' : 'screen'}>{children}</view>;
}
