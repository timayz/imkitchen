import type { StandardProps } from '@lynx-js/types'

declare module '@lynx-js/types' {
  interface GlobalProps {
    /** `android` | `ios` */
    os: string
    osVersion: string
    /** Device model, lowercase (e.g. `pixel 8`). */
    deviceModel: string
    /** OS language, e.g. `en-US`, `fr-FR`. */
    language: string
    theme: string
    preferredTheme?: string
    isNotchScreen: boolean
    screenWidth: number
    screenHeight: number
    statusBarHeight: number
    /** Android: system navigation bar height in dp. */
    navigationBarHeight?: number
    /** iOS: bottom safe-area inset. */
    bottomHeight?: number
    /** Query parameters of the container's scheme URL, all strings. On
     *  Android 2.0.1 this only holds `containerInitTime`; use `pageQuery`. */
    queryItems: Record<string, string>
    /** Scheme query parameters injected by the host router (see
     *  `SparklingHostRouterDepend.kt`). */
    pageQuery?: Record<string, string>
    containerID: string
  }

  /**
   * The host app provides `<input>` itself (`LynxInputComponent.kt`, a
   * single-line EditText) and reads the text color from this prop.
   */
  interface InputProps {
    value?: string
    'text-color'?: string
    /** Growing text area: wrapped lines, return inserts a newline. */
    multiline?: boolean
  }

  /** `<webview>` is not typed by @lynx-js/types 3.6; it needs a fixed size. */
  interface WebviewProps extends StandardProps {
    src?: string
    html?: string
  }

  interface IntrinsicElements extends Lynx.IntrinsicElements {
    webview: WebviewProps
  }
}
