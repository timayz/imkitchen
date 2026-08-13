# imkitchen — iOS (App Store, WKWebView shell)

A minimal UIKit app wrapping https://imkitchen.app in a full-screen
`WKWebView`. Everything ships from the web — the shell only adds what the
web can't do on iOS: persistent cookies across launches, external links in
Safari, a native offline fallback, and a native screen wake-lock for cooking
mode.

Building and submitting requires **macOS + Xcode** and an Apple Developer
Program membership ($99/year, individual is fine, no D-U-N-S needed).

## ⚠️ The user-agent token is load-bearing

`WebViewController.swift` appends ` imkitchen-ios` to the WKWebView
user agent. Two server behaviors depend on it:

1. **Store compliance** — the server hides every upgrade/billing surface,
   ad slot and analytics script when it sees the token
   (`web/shared/src/template.rs`, guard `DenyIosApp`). Apple forbids
   external payment flows for digital goods in-app (Guideline 3.1.1);
   users subscribe on the web instead, and existing Premium users keep
   their features.
2. **Sessions** — session validation exact-matches the full UA string
   (`web/shared/src/auth.rs`). The token must be **stable and
   version-less**: never append an app version to it, or every signed-in
   user is logged out by the next app update.

## Project generation

The Xcode project is generated from `project.yml` with
[XcodeGen](https://github.com/yonaskolb/XcodeGen) (the `.xcodeproj` is not
committed):

```sh
brew install xcodegen
cd ios
xcodegen generate
open imkitchen.xcodeproj
```

Set your signing team in Xcode (Signing & Capabilities → automatic signing).
Bundle id: `app.imkitchen`.

## What to verify in the simulator

- Login persists across app relaunch (persistent `WKWebsiteDataStore`).
- No upgrade/billing UI anywhere; `https://imkitchen.app/upgrade` and
  `/settings/billing` show the 404 page.
- External links (legal, mailto) open in Safari view / Mail, not in the shell.
- Cold-start offline (simulator: Settings → toggle network off before launch)
  shows the native retry screen; retry works when back online.
- Cooking mode keeps the screen awake (device only): start cooking a recipe,
  leave the device idle — the screen must not lock while the cook page is
  open, and must lock normally after leaving it.
- Swipe-back gesture navigates history.
- Session survives an app **rebuild** (proves the UA stayed stable).

## App Store submission notes

- **Guideline 4.2 (minimum functionality) risk is real** for webview apps.
  In App Review notes, point out the native integrations (persistent
  sessions, offline fallback, Safari hand-off, native wake-lock during
  cooking) and the full offline-capable PWA experience. If rejected,
  iterate with the reviewer; push notifications (meal reminders) are the
  strongest next native feature to add.
- **Screenshots**: App Store Connect requires 6.9"/6.7" sizes
  (1290x2796 or 1320x2868) — capture from the simulator in en and fr. The
  repo's existing 750x1334 screenshots fit no current slot.
- **App Privacy label**: with ads and analytics suppressed for the iOS shell
  server-side, the app itself collects nothing beyond the account data users
  provide (email); no tracking, so no App Tracking Transparency prompt.
- Privacy policy URL: https://imkitchen.app/policy · Support URL:
  https://imkitchen.app/contact
- Account deletion is available in-app (Settings → Account), which Apple
  requires for apps with account creation.
- Age rating questionnaire: likely 4+.
- Distribute a build via TestFlight first; then submit for review (typically
  1–3 days per iteration).
