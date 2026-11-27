# imkitchen app

The application of imkitchen, built with
[Slint](https://github.com/slint-ui/slint): one code base for the desktops
(Linux, macOS, Windows), Android and iOS.

This is its skeleton: the window, the navigation between its screens, a
theme and the translations. The screens are empty until their issues fill
them (#772 to #778), and the theme holds the values of the skeleton until the
design gives its tokens (#782, #783).

## Layout

```
ui/app.slint          the window: the navigation and the current screen
ui/navigation.slint   the destinations, and the bar that shows them
ui/theme.slint        colours, sizes, spacing: light and dark, after the system
ui/screens/           one file a screen, all built on page.slint
ui/icons/             Material Symbols (Apache 2.0)
lang/<language>/      the sentences of the app in that language
src/lib.rs            the whole app: `run`, and `android_main` for Android
src/main.rs           the binary of the desktops and of iOS
ios/                  the Xcode project of iOS, as XcodeGen generates it
tests/app.rs          the app driven without a screen
```

The window lays itself out for the room it has (`Form` in `theme.slint`, the
window size classes of Material 3): under 600px wide the navigation is a bar
along the bottom, from 600px a rail down the side, from 840px a rail with its
labels beside the icons. On a phone the screens stay clear of what the system
keeps of the edges (status bar, notch, gesture bar) and the bar paints under
them.

One look everywhere: the style of Slint's widgets is set in `build.rs`
instead of following the platform, and the screens take their colours from
`Theme`, never from the style.

## Running it

```sh
make app.run        # on this desktop
make app.android    # on the Android emulator, or on the phone adb sees
make app.ios        # builds for the iOS simulator, on a Mac
```

### Desktop

`cargo run -p imkitchen-app`. The build asks nothing of the system. On Linux
the app loads Wayland or X11, OpenGL and fontconfig when it starts: any
desktop has them, and the Nix shell names them for NixOS.

### Android

The Nix shell has what it takes: the SDK, the NDK (r30 or newer: Slint draws
with Skia on Android, from binaries built against that libc++), the Rust
targets and [cargo-apk2](https://crates.io/crates/cargo-apk2), which packages
the library as an APK from `[package.metadata.android]` in `Cargo.toml`.

```sh
android-avd-create     # once
android-emulator       # in another terminal
make app.android
```

A phone, plugged in with USB debugging on:
`make app.android APP_ANDROID_TARGET=aarch64-linux-android`.

`adb logcat -s slint` shows what the app prints.

### iOS

On a Mac with Xcode, `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`
and [XcodeGen](https://github.com/yonaskolb/XcodeGen):

```sh
make app.ios
xcrun simctl install booted app/ios/build/Build/Products/Debug-iphonesimulator/imkitchen.app
xcrun simctl launch booted app.imkitchen.ios
```

`make app.ios` generates `ios/imkitchen.xcodeproj` from `ios/project.yml`:
open it in Xcode to run the app on a device, with your own team for the
signature. Xcode compiles nothing itself: `ios/cargo-build.sh` has cargo
build the binary.

No Mac builds the app but the CI's (`.github/workflows/app.yaml`): it builds
it for the simulator, starts it there and uploads what it looks like.

## Translations

A sentence the user reads is written in English in the Slint sources, as
`@tr("…")`, and translated in `lang/<language>/LC_MESSAGES/imkitchen-app.po`.
The translations are compiled into the binary, and the app takes the language
of the system when it has it.

```sh
make app.l10n    # adds the new sentences to each language, to be translated
make l10n        # fails while a sentence is not translated
```

A new language is a new directory of `lang/` with a copy of that file, and a
line of `CFBundleLocalizations` in `ios/project.yml`.

## Tests

`cargo test -p imkitchen-app` drives the window without a screen, through
Slint's testing backend: where the navigation is for the width of the window,
what a tap on a destination shows, and the translations.

## Licence of Slint

Slint is used under its
[royalty-free licence](https://github.com/slint-ui/slint/blob/master/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md),
which asks that the app credits it: the settings screen shows the
`AboutSlint` widget.
