import { setEnabled } from 'sparkling-keep-awake'

/**
 * Keeps the screen on while the cooking screen is up (host method in
 * `methods/keep-awake`). Failures are ignored: it is a comfort feature.
 */
export function setKeepAwake(enabled: boolean): void {
  try {
    setEnabled({ enabled }, () => {})
  } catch {
    // Host without the method (e.g. tests): nothing to do.
  }
}
