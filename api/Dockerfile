# syntax=docker/dockerfile:1
#
# The image of the api, built from the root of the repository:
#
#   docker build -f api/Dockerfile -t timayz/imkitchen:api-local .
#
# It starts from the builder image (`api/builder.Dockerfile`), which holds
# the toolchain and a build of the api: only what changed since is compiled.
FROM timayz/imkitchen:api-builder AS builder

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
