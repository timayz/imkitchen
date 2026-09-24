# imkitchen for Android

A [Trusted Web Activity](https://developer.chrome.com/docs/android/trusted-web-activity)
wrapping `https://app.imkitchen.app`. Chrome renders the site full-screen with
no browser UI; there is no app code of our own beyond this configuration.

Only `twa-manifest.json` and this file are committed. Everything Bubblewrap
generates (`app/`, `gradle/`, `*.apk`, `*.aab`, keystores) is ignored.

## Why a separate host

A TWA runs inside Chrome and shares its cookie jar and User-Agent, so there is
no custom UA token to identify it and no per-request signal on the main domain
that is present on *every* request. Any persisted "this is the app" marker
would leak into the user's own Chrome and hide billing on the website too.

Serving the shell from `app.imkitchen.app` makes the `Host` header an
unambiguous signal on navigations, TwinSpark partials and POSTs alike, and —
because the auth cookie sets no `domain` — gives the app its own cookie jar.
Server side this drives `is_native_host` (`web/shared/src/native.rs`), the
`purchase_enabled` template filter, and the `DenyNativeApp` route guard.

`additionalTrustedOrigins` is deliberately empty. Adding `imkitchen.app` would
let the app navigate to the billing host without browser UI, which is exactly
the ambiguity the split host removes. Empty means any link off the app host
visibly opens a Custom Tab.

## Prerequisites

The server must be deployed with a `[native_app]` config section and the app
hostname live **before** building, because Bubblewrap reads the live manifest:

```sh
curl https://app.imkitchen.app/manifest.json
curl https://app.imkitchen.app/.well-known/assetlinks.json   # 200 application/json
```

## First build

```sh
cd android
npx @bubblewrap/cli init --manifest https://app.imkitchen.app/manifest.json
# Answer prompts to match twa-manifest.json, or overwrite the generated file
# with it and re-run `npx @bubblewrap/cli update`.
npx @bubblewrap/cli build
```

Confirm the generated `app/build.gradle` has `targetSdkVersion 36`. Since
2026-08-31 new Play submissions must target Android 16; a lower value is
rejected at upload.

## Keystore

`bubblewrap init` offers to generate one. It is the **upload key**.

- Never commit it, never put it in the Docker image, the Helm chart or CI.
- Back up the keystore, its password, the key alias and the key password in at
  least two places.
- Enrol in **Play App Signing** (required for new apps): it is the only
  recovery path if the upload key is lost.

## The fingerprint footgun

Play App Signing means Google **re-signs the bundle with a key it holds**.
Digital Asset Links on a Play-installed build therefore checks the *app
signing* key, while a local `adb install` build carries your *upload* key.

Ship only the upload fingerprint and the local build verifies fine while the
Play build shows a URL bar. **Both belong in
`config.native_app.sha256_cert_fingerprints`.**

The app signing fingerprint only exists after the first upload, so:

1. Build the AAB and upload it to the internal testing track.
2. Play Console → Test and release → Setup → App integrity → App signing →
   copy the **app signing key** SHA-256.
3. Add it to `helm/values.yaml` alongside the upload fingerprint.
4. Redeploy. This is config only — no image rebuild, which is why the
   fingerprints live in config rather than in the template.
5. Install from the internal track and confirm **no URL bar**.

## Verifying Digital Asset Links

```sh
npx @bubblewrap/cli validate
adb install -r app-release-signed.apk
adb logcat | grep -i "origin verification"
```

Google's verifier:

```
https://digitalassetlinks.googleapis.com/v1/statements:list?source.web.site=https://app.imkitchen.app&relation=delegate_permission/common.handle_all_urls
```

The pass signal is simply that no URL bar appears at the top of the app.

## Releasing an update

Bump both `appVersionCode` (integer, must increase) and `appVersionName` in
`twa-manifest.json`, then `npx @bubblewrap/cli update && npx @bubblewrap/cli build`.

Note that a TWA update is only needed when *this configuration* changes —
shipping new web features reaches users through the normal deploy, with no
store review.
