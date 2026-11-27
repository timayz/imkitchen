# syntax=docker/dockerfile:1
#
# The image of admin, built from the root of the repository:
#
#   docker build -f admin/Dockerfile -t timayz/imkitchen:admin-local .
#
# It starts from the builder image (`admin/builder.Dockerfile`), which holds
# the toolchain and a build of admin: only what changed since is compiled.
FROM timayz/imkitchen:admin-builder AS builder

COPY . .

# The binary, then the assets it serves (the stylesheet), bundled from that
# very build: a bundle is only good for the binary it was made from.
RUN cargo build --release --locked -p imkitchen-admin \
 && topcoat asset bundle --release -p imkitchen-admin

# What the image holds, laid out as it will sit in it: copying the parent of
# a directory keeps its owner and mode, which copying the directory itself
# does not. admin looks for its assets next to its binary.
RUN mkdir -p /out/app /out/tmp /out/var/lib/imkitchen \
 && cp target/release/imkitchen-admin /out/app/ \
 && cp -r target/release/assets /out/app/assets \
 && chown 10001:10001 /out/var/lib/imkitchen \
 && chmod 1777 /out/tmp

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
# /app: the binary and its assets. /tmp, and /var/lib/imkitchen: where the
# built-in configuration keeps the store (`data/admin.db` under the working
# directory); mount a volume here, or name another path in the
# configuration. The store holds nothing that is not the api's: lost, it is
# built again from its events.
COPY --from=builder /out/ /

USER 10001:10001
WORKDIR /var/lib/imkitchen

EXPOSE 3020

ENTRYPOINT [ "/app/imkitchen-admin" ]
# `migrate` first: `serve` refuses to start without its store.
CMD ["serve"]
