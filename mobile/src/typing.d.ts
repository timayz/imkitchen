declare module '@lynx-js/types' {
  /**
   * What the native host hands to every page. Sparkling fills it on Android;
   * on iOS the host does (`HostGlobalProps` in ios/SparklingGo/SparklingGo/SparklingSwiftVC.swift),
   * so only the fields both sides provide are listed. Read them through
   * `hostProps()` in src/lib/device.ts rather than `lynx.__globalProps`.
   */
  interface GlobalProps {
    /** `android` | `ios` */
    os?: string;
    osVersion?: string;
    /** OS language, e.g. `en-US`, `fr-FR`. */
    language?: string;
    /** `light` | `dark`, the system setting when the container opened. */
    theme?: string;
    /** iOS: bottom safe-area inset. */
    bottomHeight?: number;
    /** Query parameters of the container's scheme URL, all strings. On
     *  Android 2.0.1 this only holds `containerInitTime`; use `pageQuery`. */
    queryItems?: Record<string, string>;
    /** Android: scheme query parameters injected by the host router (see
     *  `SparklingHostRouterDepend.kt`). */
    pageQuery?: Record<string, string>;
  }

  /**
   * The hosts provide `<input>` themselves (`LynxInputComponent.kt`,
   * `LynxInput.m`) and read the text color from this prop.
   */
  interface InputProps {
    value?: string;
    'text-color'?: string;
    /** Android: growing text area, wrapped lines, return inserts a newline. */
    multiline?: boolean;
  }
}

export {};
