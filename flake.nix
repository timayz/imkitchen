{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config = {
            # androidenv is unfree and needs the SDK license accepted here;
            # /etc/nixos settings do not apply to a flake's own nixpkgs import.
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        pkgs-unstable = import nixpkgs-unstable {
          inherit system;
        };

        # Playwright comes from nixpkgs-unstable rather than a third-party flake.
        # Note nixpkgs-unstable must stay recent enough: webkit is in the closure
        # of playwright-test either way, and older revs fail to build it with
        # `auto-patchelf could not satisfy dependency <lib>` (libhyphen, then
        # libmanette, cf. pietdevries94/playwright-web-flake#24). If that error
        # reappears, `nix flake update nixpkgs-unstable`.
        #
        # `selectBrowsers` rather than `browsers-chromium`: the latter also drops
        # chromium-headless-shell, which Playwright >= 1.49 launches for headless
        # chromium. This keeps chromium, chromium-headless-shell and ffmpeg.
        playwright-browsers = pkgs-unstable.playwright-driver.selectBrowsers {
          withFirefox = false;
          withWebkit = false;
        };

        # Android SDK for the Sparkling (Lynx) native mobile app in ./mobile
        # (Android shell in ./mobile/android).
        # Versions follow the sparkling-app-template: compileSdk/targetSdk 34
        # (also what `sparkling doctor` checks); build-tools 30.0.3 is what
        # AGP 7.4.2 insists on installing, 33.0.1 provides an aapt2 new enough
        # for AGP 7.4.2 (see GRADLE_OPTS below). No NDK/CMake: Lynx and the
        # Sparkling methods ship prebuilt AARs.
        androidComposition = pkgs.androidenv.composeAndroidPackages {
          platformVersions = [ "34" ];
          buildToolsVersions = [ "30.0.3" "33.0.1" ];
          platformToolsVersion = "latest";
          includeEmulator = true;
          includeSystemImages = true;
          systemImageTypes = [ "google_apis" ];
          abiVersions = [ "x86_64" ];
          includeNDK = false;
          includeCmake = false;
          includeSources = false;
        };
        androidSdk = androidComposition.androidsdk;
        androidHome = "${androidSdk}/libexec/android-sdk";
        androidAapt2BuildTools = "33.0.1";
        avdName = "imkitchen-api34";
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            gcc
            lsof
            glib
            openssl
            pkg-config
            cargo-watch
            cargo-machete
            cargo-tarpaulin
            cargo-edit
            tailwindcss_4
            imagemagick # mobile/scripts/icons.mjs
            pkgs-unstable.playwright-test
            (writeShellScriptBin "mcp-server-playwright" ''
              export PWMCP_PROFILES_DIR_FOR_TEST="$PWD/.pwmcp-profiles"
              exec ${pkgs-unstable.playwright-mcp}/bin/playwright-mcp "$@"
            '')
            mkcert
            sqlite
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })

            # Sparkling / Lynx toolchain. Node ^22 || ^24 is required by
            # sparkling-app-cli; JDK 11 is what the template's Gradle 8.2 +
            # AGP 7.4.2 + forced Java toolchain 11 expect. Gradle itself comes
            # from the template's wrapper (android/gradlew), not nixpkgs.
            nodejs_24
            jdk11
            androidSdk
            (writeShellScriptBin "android-avd-create" ''
              # cmdline-tools need JDK 17+; the shell's JAVA_HOME is JDK 11 for Gradle.
              export JAVA_HOME="${pkgs.jdk17.home}"
              exec avdmanager create avd -n "${avdName}" \
                -k "system-images;android-34;google_apis;x86_64" -d pixel_6 --force "$@"
            '')
            (writeShellScriptBin "android-emulator" ''
              # `-gpu host` is required here: with the default (auto) and
              # swiftshader_indirect backends the emulator segfaults a few
              # seconds after the Lynx app starts rendering. Flags passed on
              # the command line come after and override these defaults.
              exec emulator -avd "${avdName}" -gpu host -no-boot-anim "$@"
            '')
          ];
          shellHook = ''
            # timada-admin's build.rs runs Tailwind; use the packaged CLI instead of downloading it.
            export TAILWIND_CLI="$(command -v tailwindcss)"

            # Playwright browsers come from the Nix store, not `npx playwright install`.
            export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
            export PLAYWRIGHT_BROWSERS_PATH="${playwright-browsers}"

            # topcoat-cli (`topcoat dev`, `topcoat asset bundle`, `topcoat ui`) is not in
            # nixpkgs; install it with cargo, pinned to the topcoat version the workspace uses.
            export PATH="$HOME/.cargo/bin:$PATH"
            TOPCOAT_CLI_VERSION="0.9.0"
            if ! cargo install --list 2>/dev/null | grep -q "^topcoat-cli v$TOPCOAT_CLI_VERSION:"; then
              cargo install topcoat-cli --version "$TOPCOAT_CLI_VERSION" --locked
            fi

            # Android SDK for `sparkling doctor` / `sparkling run:android` / android/gradlew.
            export ANDROID_HOME="${androidHome}"
            export ANDROID_SDK_ROOT="$ANDROID_HOME"
            export JAVA_HOME="${pkgs.jdk11.home}"
            # The Nix store is read-only and Maven's prebuilt aapt2 is not patched
            # for NixOS, so point AGP at the SDK's build-tools copy instead.
            export GRADLE_OPTS="-Dorg.gradle.project.android.aapt2FromMavenOverride=$ANDROID_HOME/build-tools/${androidAapt2BuildTools}/aapt2"
          '';
        };
      }
    );
}
