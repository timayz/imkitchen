#!/usr/bin/env bash
# Run by Xcode, from the project `project.yml` describes: builds the binary of
# the app with cargo, for each architecture Xcode asks for, and puts it where
# Xcode expects the executable of the bundle.
set -euo pipefail

# Xcode's own toolchain comes first in the PATH it gives a script, and its
# `cc` builds for iOS: what cargo compiles for the Mac itself (build scripts,
# macros) needs the system's. Cargo is not in that PATH at all.
export PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH:$HOME/.cargo/bin"

if [ "$CONFIGURATION" = "Debug" ]; then
    profile=debug
    release=()
else
    profile=release
    release=(--release)
fi

# The target directory of the workspace: the same cache as every other build.
target_dir="${CARGO_TARGET_DIR:-$SRCROOT/../../target}"

executables=()
for arch in $ARCHS; do
    case "$arch${LLVM_TARGET_TRIPLE_SUFFIX-}" in
        arm64) target=aarch64-apple-ios ;;
        arm64-simulator) target=aarch64-apple-ios-sim ;;
        x86_64-simulator) target=x86_64-apple-ios ;;
        *)
            echo "error: no Rust target for $arch${LLVM_TARGET_TRIPLE_SUFFIX-}" >&2
            exit 1
            ;;
    esac
    cargo build -p imkitchen-app --bin imkitchen-app --target "$target" \
        --target-dir "$target_dir" ${release[@]+"${release[@]}"}
    executables+=("$target_dir/$target/$profile/imkitchen-app")
done

lipo -create -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" "${executables[@]}"
