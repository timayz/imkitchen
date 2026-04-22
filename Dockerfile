# syntax=docker/dockerfile:1
FROM timayz/imkitchen:builder AS builder

COPY . .

RUN cargo build --release --bin imkitchen

RUN mkdir /var/lib/imkitchen

FROM scratch

COPY --from=builder /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /app/target/release/imkitchen /usr/bin/imkitchen

USER 10001:10001

EXPOSE 3000

ENTRYPOINT [ "imkitchen" ]
CMD ["serve"]

