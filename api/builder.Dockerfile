# syntax=docker/dockerfile:1
# The image the release build starts from (`api/Dockerfile`): the toolchain
# and a first build of the api, so a release only recompiles what changed
# since. Rebuilt by hand (the Builder workflow) when the toolchain or many
# dependencies move. Built from the root of the repository:
#
#   docker build -f api/builder.Dockerfile -t timayz/imkitchen:api-builder .
FROM rust:1.100-alpine

# build-base, cmake and perl build the C the crypto and WebP crates carry;
# clang-libs is what the store's bindings generator looks for.
RUN apk add --no-cache musl-dev build-base cmake perl clang-libs ca-certificates

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

RUN cargo build --release --locked -p imkitchen-api
