# sparkling-keep-awake

Custom Sparkling Method: keep the screen on while the cooking screen is up
(see also `../db` and `../timer-alarm`). Scaffolded with `sparkling-method-cli init` 2.0.1; the generated
TypeScript/Swift stubs were unusable (invalid syntax), so `index.ts` and the
Kotlin method are hand-written against the generated `AbsSetEnabledMethodIDL`.

- Android: `KeepAwakeSetEnabledMethod` toggles `FLAG_KEEP_SCREEN_ON` on the
  top Sparkling activity. Registered in `SparklingApplication.kt`.
- iOS: `KeepAwakeSetEnabledMethod` (`ios/Sources`) toggles the process-wide
  `UIApplication.isIdleTimerDisabled`; the cooking screen turns it back off
  when it unmounts. Auto-registered (direct `PipeMethod` subclass); pod
  `ios/Sparkling-KeepAwake.podspec`. Built by the `Mobile` workflow only.

Linked into the app with `npm install ./methods/keep-awake` + `npm run autolink`.
