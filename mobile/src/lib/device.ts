import type { GlobalProps } from '@lynx-js/types';

/**
 * What the host said about the device and the container. Never null: a page
 * that reads a missing `lynx.__globalProps` throws before its first render
 * and stays blank.
 */
export function hostProps(): GlobalProps {
  return lynx.__globalProps ?? {};
}

export type Os = 'android' | 'ios';

export function os(): Os {
  return hostProps().os === 'ios' ? 'ios' : 'android';
}

/**
 * Space, in px, the page leaves free at the edges of its container. On iOS
 * the container fills the screen, behind the status bar and the home
 * indicator; on Android it sits between the system bars.
 */
export function insets(): { top: number; bottom: number } {
  const props = hostProps();
  return { top: props.topHeight ?? 0, bottom: props.bottomHeight ?? 0 };
}
