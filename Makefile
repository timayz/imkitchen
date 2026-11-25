.PHONY: dev run health lint fmt-check fmt test machete check

# Optional local overrides for the api, ignored by git.
API_CONFIG := $(wildcard api/config/dev.local.toml)
API_ARGS := $(if $(API_CONFIG),--config $(API_CONFIG) )serve

# Run the api, restarting it on code changes
dev:
	cargo watch -w api -w Cargo.toml -x "run -p imkitchen-api -- $(API_ARGS)"

# Run the api once
run:
	cargo run -p imkitchen-api -- $(API_ARGS)

# Ask the running api for its health (PORT=3100 make health)
health:
	curl -fsS http://localhost:$(or $(PORT),3000)/health

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
	cargo machete --with-metadata api www admin app

# Run all checks: format, lint, machete, and test (CI-ready)
check: fmt-check lint machete test
	@echo "✓ All checks passed!"
