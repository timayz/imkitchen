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
 * Space, in px, to leave free at the bottom of the container: the home
 * indicator on iOS. Both hosts already start the container below the status
 * bar.
 */
export function bottomInset(): number {
  return hostProps().bottomHeight ?? 0;
}
