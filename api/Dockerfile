# syntax=docker/dockerfile:1
#
# The image of the api, built from the root of the repository:
#
#   docker build -f api/Dockerfile -t timayz/imkitchen-api .
#
# The dependencies are built in a layer of their own (cargo-chef), which
# only changes with Cargo.lock: a change to the code builds the api alone.

FROM rust:1.100-alpine AS chef

# build-base, cmake and perl build the C the crypto and WebP crates carry;
# clang-libs is what the store's bindings generator looks for.
RUN apk add --no-cache musl-dev build-base cmake perl clang-libs ca-certificates

RUN cargo install cargo-chef --locked --version 0.1.77

WORKDIR /app

FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

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

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --locked --recipe-path recipe.json -p imkitchen-api

COPY . .

RUN cargo build --release --locked -p imkitchen-api

# The directories the image needs besides the binary, laid out as they will
# sit in it: copying their parent keeps each one's owner and mode, which
# copying a directory itself does not.
RUN mkdir -p /out/tmp /out/var/lib/imkitchen \
 && chown 10001:10001 /out/var/lib/imkitchen \
 && chmod 1777 /out/tmp

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /app/target/release/imkitchen-api /usr/bin/imkitchen-api
# /tmp, and /var/lib/imkitchen: where the built-in configuration keeps the
# store and the pictures (`data/…` under the working directory); mount a
# volume here, or name other paths in the configuration.
COPY --from=builder /out/ /

USER 10001:10001
WORKDIR /var/lib/imkitchen

# HTTP, the sidecars, and the other nodes of a cluster: the store and the
# pictures.
EXPOSE 3000 4000 7000 7100

ENTRYPOINT [ "imkitchen-api" ]
# `migrate` first: `serve` refuses to start without its store.
CMD ["serve"]
