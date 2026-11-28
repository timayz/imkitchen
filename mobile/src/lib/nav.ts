import { close, open } from 'sparkling-navigation';

import { hostProps } from './device.js';
import { palette, theme } from './theme.js';

/** One per entry in app.config.ts. */
export type Bundle = 'main' | 'about';

/**
 * Query parameters the current container was opened with. Sparkling 2.0.1
 * only delivers them through `queryItems` on iOS; on Android the host router
 * injects them as `pageQuery`.
 */
export function pageParams(): Record<string, string> {
  const props = hostProps();
  return { ...(props.queryItems ?? {}), ...(props.pageQuery ?? {}) };
}

export function scheme(bundle: Bundle, params: Record<string, string> = {}): string {
  const dark = theme() === 'dark';
  // What the container shows before the page has rendered.
  const paper = palette[dark ? 'dark' : 'light'].paper;
  const query = new URLSearchParams({
    bundle: `${bundle}.lynx.bundle`,
    hide_nav_bar: '1',
    // Screens render their own loading states.
    hide_loading: '1',
    screen_orientation: 'portrait',
    container_bg_color: paper,
    loading_bg_color: paper,
    status_font_mode: dark ? 'light' : 'dark',
    ...params,
  });
  return `hybrid://lynxview_page?${query.toString()}`;
}

function navigate(bundle: Bundle, params: Record<string, string>, replaceCurrent: boolean) {
  return new Promise<void>((resolve, reject) => {
    open(
      {
        scheme: scheme(bundle, params),
        options: replaceCurrent
          ? { replace: true, replaceType: 'onlyCloseAfterOpenSucceed' }
          : undefined,
      },
      (res) => {
        if (res.code === 1) resolve();
        else reject(new Error(`navigation failed: ${res.msg}`));
      }
    );
  });
}

/** Pushes a new native container on top of the current one. */
export function push(bundle: Bundle, params: Record<string, string> = {}): Promise<void> {
  return navigate(bundle, params, false);
}

/** Opens `bundle` in place of the current container. */
export function replace(bundle: Bundle, params: Record<string, string> = {}): Promise<void> {
  return navigate(bundle, params, true);
}

/** Closes the current container, back to the one below. */
export function back(): void {
  close();
}
