# syntax=docker/dockerfile:1
# The image the release build starts from (`admin/Dockerfile`): the toolchain,
# what bundles the assets and a first build of admin, so a release only
# recompiles what changed since. Rebuilt by the Builder workflow when the
# dependencies move, or by hand. Built from the root of the repository:
#
#   docker build -f admin/builder.Dockerfile -t timayz/imkitchen:admin-builder .
FROM rust:1.100-alpine

# build-base, cmake and perl build the C the crypto crates carry; clang-libs
# is what the store's bindings generator looks for; libstdc++ and libgcc are
# what the Tailwind CLI runs on.
RUN apk add --no-cache musl-dev build-base cmake perl clang-libs ca-certificates libstdc++ libgcc

# The Tailwind CLI `admin/build.rs` compiles the stylesheet with. topcoat
# would download it by itself, but the build it picks does not run on musl.
# The version is the one topcoat pins (`topcoat::tailwind`'s default).
ARG TAILWIND_VERSION=4.3.2
RUN case "$(apk --print-arch)" in \
        x86_64) asset=x64-musl; sha256=ae828e9e989ecbddb2bef856af8b0308ba162583b4922b3a065b5e26f86b0691 ;; \
        aarch64) asset=arm64-musl; sha256=24a0dd39cbbced9d94f6313a747cc29ab2523a6a7b69204f2151e0af6aad6eef ;; \
        *) echo "no Tailwind CLI for $(apk --print-arch)" >&2; exit 1 ;; \
    esac \
 && wget -q -O /usr/local/bin/tailwindcss \
        "https://github.com/tailwindlabs/tailwindcss/releases/download/v${TAILWIND_VERSION}/tailwindcss-linux-${asset}" \
 && echo "${sha256}  /usr/local/bin/tailwindcss" | sha256sum -c - \
 && chmod +x /usr/local/bin/tailwindcss
ENV TAILWIND_CLI=/usr/local/bin/tailwindcss

# `topcoat asset bundle`, at the version of the workspace's `topcoat`, read
# from the lock file: it refuses to bundle for another.
COPY Cargo.lock /tmp/Cargo.lock
RUN version="$(awk '/^name = "topcoat"$/ { getline; gsub(/"/, "", $3); print $3 }' /tmp/Cargo.lock)" \
 && test -n "${version}" \
 && cargo install topcoat-cli --version "${version}" --locked \
 && rm /tmp/Cargo.lock

ARG USER=imkitchen
ARG UID=10001
ARG GID=10001

# Create group explicitly with fixed GID, then user with fixed UID/GID.
# This guarantees GID=10001 (adduser alone may pick a different GID).
RUN addgroup -g "${GID}" -S "${USER}" && \
    adduser \
        --disabled-password \
        --gecos "" \
        --home "/nonexistent" \
        --shell "/sbin/nologin" \
        --no-create-home \
        --uid "${UID}" \
        --ingroup "${USER}" \
        "${USER}"

WORKDIR /app

COPY . .

RUN cargo build --release --locked -p imkitchen-admin
