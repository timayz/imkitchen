# imkitchen — Android (Google Play, Trusted Web Activity)

The Play Store app is a [Trusted Web Activity](https://developer.chrome.com/docs/android/trusted-web-activity/)
wrapping https://imkitchen.app, generated with [Bubblewrap](https://github.com/GoogleChromeLabs/bubblewrap).
`twa-manifest.json` in this directory is the source of truth; the Gradle
project is generated from it and can be committed alongside once generated.

## Prerequisites

- Node 18+ (`npx @bubblewrap/cli`)
- JDK 17 and the Android SDK build tools. On first run Bubblewrap offers to
  download both and records their paths in `~/.bubblewrap/config.json`.
  **NixOS note**: the auto-downloaded prebuilt binaries usually fail on NixOS
  (dynamic linker). Either add `jdk17` + an Android SDK to the dev shell, or
  run the build in a container / GitHub Actions runner and point
  `~/.bubblewrap/config.json` at the container paths.

## Generate / update the Android project

```sh
cd android
npx @bubblewrap/cli update   # regenerates the Gradle project from twa-manifest.json
```

(`bubblewrap init --manifest https://imkitchen.app/manifest.json` was used to
seed `twa-manifest.json`; you only need `update` from now on.)

## Signing

Two keys are involved; **neither keystore is ever committed**:

1. **Upload key** (local): create once and back it up in a password manager.

   ```sh
   keytool -genkeypair -keystore upload.keystore -alias upload \
     -keyalg RSA -keysize 4096 -validity 10000
   ```

2. **Play App Signing key** (Google-held): enrolled automatically on the
   first upload in Play Console.

Both SHA-256 certificate fingerprints must be served by the site in
`/.well-known/assetlinks.json` (config `[android].sha256_cert_fingerprints`
in `helm/values.yaml` — redeploy after changing):

- Upload key (needed so locally installed builds open without a URL bar):
  `keytool -list -v -keystore upload.keystore -alias upload` → SHA256.
- Play signing key (needed for store-installed builds): Play Console →
  Test and release → Setup → App signing → "App signing key certificate".

## Build

```sh
cd android
npx @bubblewrap/cli build
```

Outputs `app-release-bundle.aab` (upload this to Play Console) and
`app-release-signed.apk` (for local testing via `adb install`).

## Releasing a new version

The wrapper only needs a release when the wrapper itself changes (icons,
colors, shortcuts, TWA config) — web deploys reach users instantly. To
release: bump `appVersionCode` (monotonic integer, Play rejects reuse) and
`appVersionName` in `twa-manifest.json`, then `update` + `build`.

## Verifying

- PWA quality: `npx @bubblewrap/cli validate --url https://imkitchen.app`
- Asset links served: `curl https://imkitchen.app/.well-known/assetlinks.json`
- Google's view of the statement list:
  `https://digitalassetlinks.googleapis.com/v1/statements:list?source.web.site=https://imkitchen.app&relation=delegate_permission/common.handle_all_urls`
- On device: `adb install app-release-signed.apk`, launch — **no URL bar**
  means asset-link verification passed. Inspect with
  `adb shell pm get-app-links app.imkitchen`. Debug the TWA via Chrome
  `chrome://inspect`.

## Play Console checklist (first submission)

1. Developer account ($25 one-time). New **personal** accounts must run a
   closed test with ≥12 testers for 14 days before production access.
2. Upload the `.aab`, enroll Play App Signing, copy the app-signing SHA-256
   into `helm/values.yaml` and redeploy the site.
3. Store listing: reuse `static/screenshots/*.png` (750x1334), icon
   `static/icons/icon-512.png`; a **1024x500 feature graphic** must be
   created; en + fr descriptions; privacy policy `https://imkitchen.app/policy`.
4. Declare "Contains ads" (free tier). Data safety form: account email,
   deletion available in-app (Settings → Account).
5. IARC content rating questionnaire.
