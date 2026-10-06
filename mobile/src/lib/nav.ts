import { close, open } from 'sparkling-navigation'

/**
 * Query parameters the current container was opened with. The host router
 * injects them as `pageQuery`; `queryItems` is the documented field but is
 * empty on Android 2.0.1.
 */
export function pageParams(): Record<string, string> {
  const props = lynx.__globalProps
  return { ...(props.queryItems ?? {}), ...(props.pageQuery ?? {}) }
}

export type Bundle = 'main' | 'login' | 'cooking' | 'recipe' | 'recipe-edit' | 'recipe-import' | 'cook'

function scheme(bundle: Bundle, params: Record<string, string> = {}): string {
  const query = new URLSearchParams({
    bundle: `${bundle}.lynx.bundle`,
    hide_nav_bar: '1',
    // Screens render their own loading states.
    hide_loading: '1',
    screen_orientation: 'portrait',
    ...params,
  })
  return `hybrid://lynxview_page?${query.toString()}`
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
        if (res.code === 1) resolve()
        else reject(new Error(`navigation failed: ${res.msg}`))
      },
    )
  })
}

/** Pushes a new native container on top of the current one. */
export function push(bundle: Bundle, params: Record<string, string> = {}): Promise<void> {
  return navigate(bundle, params, false)
}

/** Opens `bundle` in place of the current container (login <-> main). */
export function replace(bundle: Bundle, params: Record<string, string> = {}): Promise<void> {
  return navigate(bundle, params, true)
}

export function back(): void {
  close()
}

/** Opens an http(s) URL in the system browser (recipe origins). */
export function openExternal(url: string): void {
  open({ scheme: url, options: { useSysBrowser: true } }, () => {})
}
