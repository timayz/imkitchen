.PHONY: dev run migrate cluster.all health ready lint fmt-check fmt test machete check

# Optional local overrides for the api, ignored by git.
API_CONFIG := $(wildcard api/config/dev.local.toml)
API_ARGS := $(if $(API_CONFIG),--config $(API_CONFIG) )serve

API_MIGRATE_ARGS := $(if $(API_CONFIG),--config $(API_CONFIG) )migrate

# One node of the local three-node cluster (api/config/cluster.toml): node N
# serves HTTP on 300N, talks to the others on 700N and keeps its store in
# data/nodeN.
CLUSTER_NODE_ENV = IMKITCHEN_API_CONFIG=api/config/cluster.toml \
	IMKITCHEN_API__CLUSTER__NODE_ID=$* \
	IMKITCHEN_API__SERVER__PORT=300$* \
	IMKITCHEN_API__DATABASE__PATH=data/node$*/imkitchen.db

# Run the api, restarting it on code changes
dev: migrate
	cargo watch -w api -w Cargo.toml -x "run -p imkitchen-api -- $(API_ARGS)"

# Run the api once
run: migrate
	cargo run -p imkitchen-api -- $(API_ARGS)

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

# Check for unused dependencies. Scoped to the workspace members: the previous
# code base is still on disk until #780 and no longer resolves.
machete:
	cargo machete --with-metadata api aggregates www admin app

# Run all checks: format, lint, machete, and test (CI-ready)
check: fmt-check lint machete test
	@echo "✓ All checks passed!"
