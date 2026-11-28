# imkitchen mobile

imkitchen on Android and iOS, one code base for both: built with
[Sparkling](https://tiktok.github.io/sparkling/) on top of
[Lynx](https://lynxjs.org/) (ReactLynx, TSX, native rendering). The same
features as `app`, which serves the browser and the desktop, with the
imkitchen api as backend over its JSON API.

This is the project the features are built in (#812): navigation, theme and
translations are in place, the tabs are empty and nothing talks to the api
yet.

## Layout

| Path | What |
|---|---|
| `app.config.ts` | Rspeedy entries (one bundle per native container), package ids, splash |
| `src/pages/<name>/` | One Lynx entry per native container; `main` hosts the tab bar |
| `src/ui/` | Shared components: `Screen` (page root, theme), `BottomTabs` |
| `src/lib/nav.ts` | Navigation between containers: `push`, `replace`, `back`, `pageParams` |
| `src/lib/i18n/` | Translations (`en.ts` is the reference, `fr.ts` must have every key), `t()`; the language is the device's |
| `src/lib/theme.ts`, `src/styles/tokens.css` | Light and dark themes following the system; a screen only uses the tokens |
| `src/lib/device.ts` | What the host says about the device (`lynx.__globalProps`), null-safe |
| `src/styles/fonts.css`, `src/assets/fonts/` | Fraunces SemiBold (Latin subset, SIL OFL) for headings, inlined as base64 into every bundle |
| `methods/keep-awake/`, `methods/timer-alarm/` | `sparkling-keep-awake` (screen stays on) and `sparkling-timer-alarm` (a timer rings through AlarmManager / UNUserNotificationCenter, so it survives sleep and process death), linked into both shells for cooking mode (#824) |
| `scripts/icons.mjs` | Renders launcher, splash and store icons from `resource/icon.svg` (`npm run icons`) |
| `scripts/nav-icons.mjs` | Renders the tab bar icons into `src/assets/icons/*.png` (`npm run icons`) |
| `scripts/timer-sound.mjs` | Renders the timer's bell into `methods/timer-alarm` for both platforms (`npm run sound`) |
| `android/` | Android shell (`app.imkitchen.android`), Kotlin, Gradle 9 / AGP 9, compileSdk 37 |
| `android/methods/` | Build scripts for the published Sparkling method packages (sources stay in `node_modules`) |
| `ios/` | iOS shell (`app.imkitchen.ios`). No Mac in the dev environment: it is only ever built and started by the `Mobile` GitHub workflow (see below) |

### Adding a screen

A tab is a component rendered by `src/pages/main/App.tsx`. A screen opened on
top of the tabs is its own bundle, so it keeps the native back gesture: add
`src/pages/<name>/index.tsx`, declare it in `source.entry` of `app.config.ts`
and in `Bundle` of `src/lib/nav.ts`, then open it with `push('<name>', params)`
and read the parameters with `pageParams()`. Each bundle is its own JS
runtime: nothing in memory survives a push.

Wrap every page in `Screen`, take every colour from `src/styles/tokens.css`
and every text from `t()`.

Template: `sparkling-app-template` 2.0.1 via `create-sparkling-app` 2.0.1
(`sparkling-app-cli` 2.0.1, Android SDK `com.tiktok.sparkling:sparkling:2.0.1`).
Sparkling is in public beta, so every JS dependency is pinned exactly; bump them
deliberately and re-run `npm run autolink` afterwards.

## CI (and the only iOS build)

`.github/workflows/mobile.yaml` runs typecheck + vitest + bundles on every
change under `mobile/`. The two native builds only run when the workflow is
started by hand (`gh workflow run mobile.yaml --ref <branch>`), since they
take up to 20 minutes: an Android debug APK on Ubuntu, and the iOS simulator
build on a GitHub macOS runner (`pod install`, then `xcodebuild` with code
signing off). The iOS job uploads the `Podfile.lock` it resolved
and the full `xcodebuild` log as the `ios-build` artifact; after a Podfile
change, commit the uploaded lock.

The same job then boots a simulator (an iPhone on the newest iOS runtime the
runner's Xcode ships), installs the app and launches it once in light and
once in dark mode. The step fails if the process is gone 30 s after a launch,
which is how a missing pod, a broken bundle or a QuickJS compile error
surfaces, and if a page reported an error (an exception in `main-thread.js`,
an element the host does not provide): such a page keeps the process alive
and renders nothing. It uploads the `ios-smoke` artifact: `main-light.png`
and `main-dark.png`, `app-light.log` and `app-dark.log` (stdout/stderr),
`system.log` (unified log filtered to the app) and `lynx-errors.log`.

The iOS job caches `ios/Pods` and the derived data under `ios/build`, both
keyed on `Podfile.lock`, and builds arm64 only (the runner's native simulator
architecture). Cold: about 14 min; warm: about 2 min, the app target alone
recompiles. GitHub scopes caches per ref, so a run on a branch only sees the caches saved
on that branch or on the default one.

iOS stays on Lynx 3.6.0: the `Sparkling` 2.0.1 pod depends on
`Lynx/Framework (= 3.6.0)` exactly, which CocoaPods cannot override the way
Gradle lets Android pin a newer engine (`android/gradle/libs.versions.toml`).
Both engines move together again once Sparkling publishes a 2.1 release.

## Develop (Android)

Everything below runs inside the repo's `nix develop` shell, which provides
Node 24, JDK 17, the Android SDK (platforms 34 and 37), the emulator and `adb`.

```bash
# 1. Emulator (first time: create the AVD)
android-avd-create
android-emulator &

# 2. Build + install the debug app, start the dev server with HMR
cd mobile
npm install
npm run run:android
```

Other commands:

```bash
npm run dev          # Rspeedy dev server only (port 5969)
npm run build        # bundles -> dist/, copied into the native asset folders
npm run typecheck    # tsc --noEmit
npm test             # vitest
npm run format       # prettier
npm run doctor       # environment check (see note below)
npm run icons        # regenerate android/…/mipmap-*, resource/ and the iOS icon (ImageMagick)
cd android && ./gradlew assembleDebug   # debug APK without the CLI
```

## Screens (bundles)

| Bundle | Params | What |
|---|---|---|
| `main` | `tab?` | Tab shell: Recipes, Dishes, List, Groceries, Settings |
| `about` | — | Version of the app and what the host reports (platform, language, theme) |

### Known quirks

- `@rsbuild/core` is a direct devDependency only because `@lynx-js/qrcode-rsbuild-plugin`
  imports it while declaring it an optional peer; without a hoisted copy
  `npm run build` fails with `ERR_MODULE_NOT_FOUND`.
- `sparkling doctor` reports "Java is not installed" even though JDK 17 is on
  the PATH: it only reads `java -version` from stdout, while the JDK prints to
  stderr. Gradle builds work; ignore that line.
- The `--namespace` flag of `create-sparkling-app` 2.0.1 is ignored; the
  package was renamed by hand to `app.imkitchen.android` (Kotlin package,
  Gradle `namespace`/`applicationId`, manifest, `app.config.ts`); the iOS
  shell uses `app.imkitchen.ios` (`app.config.ts`, Xcode project, autolink).
- Lynx `fetch` has no cookie jar, does not follow redirects and has no
  `FormData`/`Blob`: the app can only talk to the api with a token and JSON
  bodies.
- The bundled Lynx has no `<input>`: both hosts provide it
  (`LynxInputComponent.kt`, `LynxInput.m`). It defaults to `maxlength=140`;
  always set it explicitly, and keep it uncontrolled (initial value only):
  echoing every keystroke back through `value` races the native cursor.
- Sparkling 2.0.1 (iOS) builds the global props but never hands them to the
  Lynx view, so a page finds `lynx.__globalProps` null and one that reads a
  field of it throws before its first render. The host injects what the pages
  read (`HostGlobalProps` in `SparklingSwiftVC.swift`, fixed upstream in
  2.1); pages go through `hostProps()` (`src/lib/device.ts`) all the same.
  A new field of `GlobalProps` (`src/typing.d.ts`) must be added there too.
- A page takes the theme when its container opens. Android recreates the
  container when the system setting changes; on iOS an open page keeps its
  theme until it is opened again.
- Sparkling 2.0.1 (Android) never delivers custom scheme query params to the
  page: `lynx.__globalProps.queryItems` only holds `containerInitTime`. The
  host router (`SparklingHostRouterDepend.kt`) injects them as
  `__globalProps.pageQuery` through the container's `RuntimeInfo`.
- `sparkling-storage` declares `project(":sparkling-method")`; the shim module
  `android/sparkling-method` re-exports the Maven artifact. Keep it outside
  the autolink block of `settings.gradle.kts`.
- The published method packages (`sparkling-navigation`, `-storage`) ship AGP 7 / Kotlin 1.8 build scripts (`kotlinOptions`, compileSdk 34, the
  Kotlin Gradle plugin AGP 9 rejects). `settings.gradle.kts` points those
  projects at `android/methods/<name>/build.gradle.kts` instead, which compile
  the sources from `node_modules`; keep their dependency lists in sync when
  bumping Sparkling.
- Every autolinked method package must also be registered in
  `SparklingApplication.kt` (`registerIDLMethod`); autolink only writes the
  registry file.
- `sparkling-method-cli codegen` 2.0.1 emits broken TypeScript and Swift;
  `methods/keep-awake` keeps only its Kotlin abstract class and hand-writes
  the rest.
- `methods/timer-alarm` rings through the OS: Android's exact-alarm
  permission is `USE_EXACT_ALARM` (auto-granted, Play reserves it for
  alarm/timer features; swap for `SCHEDULE_EXACT_ALARM` + a settings prompt if
  review objects) and both platforms ask for notification permission on the
  first start of a timer.
- `SplashActivity` is `singleTask` and finishes itself, so an icon or
  notification tap while the app runs recreates it on top of the live
  containers: it leaves at once when the task already exists. A
  `SparklingActivity` recreated after the process was killed has lost its
  context (the transfer station is in memory) and would show an empty
  "Sparkling Page"; `SparklingApplication` restarts from the splash instead.

## Release

Publication on Google Play and the App Store is #779. Until then signing is
manual on Android: keep the keystore outside the repo and point Gradle at it
from `~/.gradle/gradle.properties`. Build with
`npm run build && (cd android && SPARKLING_USE_NATIVE_ASSETS=true ./gradlew assembleRelease)`.

`app/build.gradle.kts` reads four properties, each with an env-var fallback for
CI secrets: `imkitchen.keystore` / `IMKITCHEN_KEYSTORE` (path),
`imkitchen.keystore.password`, `imkitchen.key.alias`, `imkitchen.key.password`.
Set none of them and the release build stays unsigned
(`app-release-unsigned.apk`) rather than falling back to a key nobody chose, so
nothing can accidentally ship under the wrong identity. With them set the
output is a signed `app-release.apk`.

To get an installable APK without a release key, pass the debug keystore on the
command line -- handy for testing a release build, never for distribution
(Play rejects the debug key, which is identical on every Android dev machine):

```
SPARKLING_USE_NATIVE_ASSETS=true ./gradlew assembleRelease \
  -Pimkitchen.keystore="$HOME/.android/debug.keystore" \
  -Pimkitchen.keystore.password=android \
  -Pimkitchen.key.alias=androiddebugkey -Pimkitchen.key.password=android
```

`versionCode` is still the `1` placeholder and neither it nor `versionName` is
wired to the repo version.
