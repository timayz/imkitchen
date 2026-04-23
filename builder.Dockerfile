# syntax=docker/dockerfile:1
FROM rust:1.95-alpine

RUN apk add --no-cache musl-dev tzdata \
        openssl-dev openssl-libs-static \
        pkgconf git libpq-dev \
        protoc protobuf-dev

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

# Set `SYSROOT` to a dummy path (default is /usr) because pkg-config-rs *always*
# links those located in that path dynamically but we want static linking, c.f.
# https://github.com/rust-lang/pkg-config-rs/blob/54325785816695df031cef3b26b6a9a203bbc01b/src/lib.rs#L613
ENV SYSROOT=/dummy

# The env var tells pkg-config-rs to statically link libpq.
ENV LIBPQ_STATIC=1

WORKDIR /app

COPY . .

RUN cargo build --release --bin imkitchen

