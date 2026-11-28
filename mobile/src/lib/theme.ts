import { hostProps } from './device.js';

export type Theme = 'light' | 'dark';

/**
 * The system setting as the host reports it; light when it says nothing.
 * Read it while rendering: ReactLynx renders the page again when the host
 * updates its global props.
 */
export function theme(): Theme {
  return hostProps().theme === 'dark' ? 'dark' : 'light';
}

/**
 * Colours for what takes a literal instead of a CSS variable: `tint-color` on
 * `<image>`, `text-color` on `<input>` and the container colours of a scheme
 * (src/lib/nav.ts). They mirror `--color-paper`, `--color-accent`,
 * `--color-ink` and `--color-ink-muted` in styles/tokens.css.
 */
export const palette: Record<
  Theme,
  { paper: string; accent: string; ink: string; inkMuted: string }
> = {
  light: { paper: '#fbf5e9', accent: '#b03d14', ink: '#1b140c', inkMuted: '#6f6354' },
  dark: { paper: '#1b140c', accent: '#f58a47', ink: '#fbf5e9', inkMuted: '#b5a894' },
};
