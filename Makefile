.PHONY: start dev run migrate cluster.all www.dev www.run www.migrate admin.dev admin.run admin.migrate app.dev app.run app.migrate cert cert.install up stop down health ready lint fmt-check fmt test test.api.unit test.www test.admin test.app machete l10n check

# Traefik and the mail catcher, then the api: what a bare `make` runs.
start: up dev

# Optional local overrides for the api, ignored by git.
API_CONFIG := $(wildcard api/config/dev.local.toml)
API_ARGS := $(if $(API_CONFIG),--config $(API_CONFIG) )serve

API_MIGRATE_ARGS := $(if $(API_CONFIG),--config $(API_CONFIG) )migrate

# What www, admin and app open their connections to the local api with: for
# local use only.
DEV_WWW_TOKEN := local-dev-only-not-a-secret-www-0000
DEV_ADMIN_TOKEN := local-dev-only-not-a-secret-admin-0000
DEV_APP_TOKEN := local-dev-only-not-a-secret-app-0000

# The local api serves the sidecars (www, admin, app) on 4000, the node N of
# the local cluster on 400N. Set here rather than in a file so that
# `make dev`, `make www.dev`, `make admin.dev` and `make app.dev` agree
# without one.
API_SIDECARS_ENV = IMKITCHEN_API__SIDECARS__LISTEN=127.0.0.1:$(1) \
	IMKITCHEN_API__SIDECARS__TOKENS__WWW=$(DEV_WWW_TOKEN) \
	IMKITCHEN_API__SIDECARS__TOKENS__ADMIN=$(DEV_ADMIN_TOKEN) \
	IMKITCHEN_API__SIDECARS__TOKENS__APP=$(DEV_APP_TOKEN)

# Optional local overrides for www, ignored by git. `topcoat dev` passes no
# arguments to the binary, so the config path travels through the environment
# and `serve` is the default subcommand. www follows the api on 4000:
# IMKITCHEN_WWW__API__EVENTS=127.0.0.1:4001 make www.dev follows a node of
# the local cluster.
WWW_CONFIG := $(wildcard www/config/dev.local.toml)
WWW_ENV := $(if $(WWW_CONFIG),IMKITCHEN_WWW_CONFIG=$(WWW_CONFIG) )IMKITCHEN_WWW__API__TOKEN=$(DEV_WWW_TOKEN)

# The same for admin: admin/config/dev.local.toml, IMKITCHEN_ADMIN_CONFIG and
# IMKITCHEN_ADMIN__API__EVENTS.
ADMIN_CONFIG := $(wildcard admin/config/dev.local.toml)
ADMIN_ENV := $(if $(ADMIN_CONFIG),IMKITCHEN_ADMIN_CONFIG=$(ADMIN_CONFIG) )IMKITCHEN_ADMIN__API__TOKEN=$(DEV_ADMIN_TOKEN)

# And for app: app/config/dev.local.toml, IMKITCHEN_APP_CONFIG and
# IMKITCHEN_APP__API__EVENTS.
APP_CONFIG := $(wildcard app/config/dev.local.toml)
APP_ENV := $(if $(APP_CONFIG),IMKITCHEN_APP_CONFIG=$(APP_CONFIG) )IMKITCHEN_APP__API__TOKEN=$(DEV_APP_TOKEN)

# One node of the local three-node cluster (api/config/cluster.toml): node N
# serves HTTP on 300N, talks to the others on 700N (the store) and 710N (the
# pictures) and keeps its store and its pictures in data/nodeN.
CLUSTER_NODE_ENV = IMKITCHEN_API_CONFIG=api/config/cluster.toml \
	IMKITCHEN_API__CLUSTER__NODE_ID=$* \
	IMKITCHEN_API__SERVER__PORT=300$* \
	IMKITCHEN_API__DATABASE__PATH=data/node$*/imkitchen.db \
	IMKITCHEN_API__IMAGES__DIR=data/node$*/images \
	$(call API_SIDECARS_ENV,400$*)

# Run the api, restarting it on code changes
dev: migrate
	$(call API_SIDECARS_ENV,4000) cargo watch -w api -w aggregates -w Cargo.toml -x "run -p imkitchen-api -- $(API_ARGS)"

# Run the api once
run: migrate
	$(call API_SIDECARS_ENV,4000) cargo run -p imkitchen-api -- $(API_ARGS)

# Create the api's store if needed and apply its pending migrations
migrate:
	cargo run -p imkitchen-api -- $(API_MIGRATE_ARGS)

# Run one node of the local cluster: make cluster.node0, cluster.node1,
# cluster.node2, each in its own terminal. Writes need two of the three up.
cluster.node%:
	$(CLUSTER_NODE_ENV) cargo run -p imkitchen-api -- migrate
	$(CLUSTER_NODE_ENV) cargo run -p imkitchen-api -- serve

# Run the three nodes of the local cluster at once
cluster.all:
	cargo build -p imkitchen-api
	$(MAKE) -j3 cluster.node0 cluster.node1 cluster.node2

# Run www at http://localhost:3010 against the local api (`make dev` in
# another terminal; www starts without it and catches up when it is there):
# builds, serves, rebuilds on change and reloads the page. `serve` does not
# migrate: the store is migrated once here.
www.dev: www.migrate
	$(WWW_ENV) topcoat dev -p imkitchen-www

# Run www once, with the assets it serves bundled next to its binary
www.run: www.migrate
	topcoat asset bundle -p imkitchen-www
	$(WWW_ENV) cargo run -p imkitchen-www

# Create www's store if needed and apply its pending migrations
www.migrate:
	$(WWW_ENV) cargo run -p imkitchen-www -- migrate

# Run admin at http://localhost:3020 against the local api, the way
# `make www.dev` runs www.
admin.dev: admin.migrate
	$(ADMIN_ENV) topcoat dev -p imkitchen-admin

# Run admin once, with the assets it serves bundled next to its binary
admin.run: admin.migrate
	topcoat asset bundle -p imkitchen-admin
	$(ADMIN_ENV) cargo run -p imkitchen-admin

# Create admin's store if needed and apply its pending migrations
admin.migrate:
	$(ADMIN_ENV) cargo run -p imkitchen-admin -- migrate

# Run app at http://localhost:3030 against the local api, the way
# `make www.dev` runs www.
app.dev: app.migrate
	$(APP_ENV) topcoat dev -p imkitchen-app

# Run app once, with the assets it serves bundled next to its binary
app.run: app.migrate
	topcoat asset bundle -p imkitchen-app
	$(APP_ENV) cargo run -p imkitchen-app

# Create app's store if needed and apply its pending migrations
app.migrate:
	$(APP_ENV) cargo run -p imkitchen-app -- migrate

# The certificate Traefik serves the local addresses with (.docker/traefik).
# `make cert.install` once makes the browser trust it.
cert:
	mkdir -p .docker/traefik/certs
	sudo mkcert -key-file .docker/traefik/certs/imkitchen.key -cert-file .docker/traefik/certs/imkitchen.crt traefik.localhost imkitchen.localhost '*.imkitchen.localhost'

cert.install:
	sudo mkcert -install

# Traefik in front of what runs on the host, and the mail catcher:
# https://imkitchen.localhost is www (`make www.dev`),
# https://admin.imkitchen.localhost admin (`make admin.dev`),
# https://app.imkitchen.localhost app (`make app.dev`), and the api
# (`make dev`) both https://api.imkitchen.localhost and
# https://app.imkitchen.localhost/api.
up:
	DOCKER_GID=$$(getent group docker | cut -d: -f3) docker compose up -d --remove-orphans

stop:
	DOCKER_GID=$$(getent group docker | cut -d: -f3) docker compose stop

down:
	DOCKER_GID=$$(getent group docker | cut -d: -f3) docker compose down -v --rmi local --remove-orphans

# Ask the running api for its health (PORT=3100 make health)
health:
	curl -fsS http://localhost:$(or $(PORT),3000)/health

# Ask the running api whether its node takes writes (PORT=3001 make ready)
ready:
	curl -fsS http://localhost:$(or $(PORT),3000)/ready

# Run Clippy linter for code quality
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check code formatting (fails if not formatted)
fmt-check:
	cargo fmt --all -- --check

# Auto-fix code formatting
fmt:
	cargo fmt --all

test:
	cargo test --workspace

# The same tests cut into the jobs CI runs side by side (.github/workflows/ci.yaml),
# one crate each, the api in several: its integration tests are most of the
# build. Every job compiles the api's library once, hence no more shards.
API_SHARDS := 3
API_TESTS := $(sort $(basename $(notdir $(wildcard api/tests/*.rs))))

# The unit tests and the doctests of the api and of the aggregates.
test.api.unit:
	cargo test -p imkitchen-aggregates -p imkitchen-api --lib --bins
	cargo test -p imkitchen-aggregates -p imkitchen-api --doc

# One shard of the api's integration tests: make test.api.0, test.api.1,
# test.api.2. A file of api/tests lands in a shard by its rank, so a new one
# is never left out.
test.api.%:
	cargo test -p imkitchen-api $(shell printf '%s\n' $(API_TESTS) | awk 'NR % $(API_SHARDS) == $* { printf "--test %s ", $$1 }')

test.www:
	cargo test -p imkitchen-www

test.admin:
	cargo test -p imkitchen-admin

test.app:
	cargo test -p imkitchen-app

# Check for unused dependencies. Scoped to the workspace members: the previous
# code base is still on disk until #780 and no longer resolves.
machete:
	cargo machete --with-metadata api aggregates www admin app

# The sentences of www and app: a key a language lacks, one no page uses, an
# argument a page does not give.
l10n:
	topcoat l10n check -p imkitchen-www
	topcoat l10n check -p imkitchen-app

# Run all checks: format, lint, machete, the sentences and test (CI-ready)
check: fmt-check lint machete l10n test
	@echo "✓ All checks passed!"
