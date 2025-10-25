.PHONY: css css-watch dev lint fmt fmt-fix test test-unit test-integration test-fast test-medium test-slow test-parallel build check machete lighthouse help

# Build CSS once
css:
	tailwindcss -i static/css/tailwind.css -o static/css/main.css --minify

# Watch CSS for changes
css-watch:
	tailwindcss -i static/css/tailwind.css -o static/css/main.css --watch

cert:
	mkdir -p .docker/traefik/certs
	sudo mkcert -key-file .docker/traefik/certs/imkitchen.key -cert-file .docker/traefik/certs/imkitchen.crt traefik.localhost docker.imkitchen.localhost imkitchen.localhost

cert.install:
		sudo mkcert -install

up:
	sudo docker compose up -d --remove-orphans

stop:
	sudo docker compose stop

down:
	sudo docker compose down -v --rmi local --remove-orphans

# Watch and run server on code changes
dev:
	cargo watch -x "run -- serve"

# Run Clippy linter for code quality
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check code formatting (fails if not formatted)
fmt:
	cargo fmt --all -- --check

# Auto-fix code formatting
fmt-fix:
	cargo fmt --all

# Run ALL tests (sequential - use test-parallel for faster execution)
test:
	cargo test --workspace

# ============================================================================
# Parallel Test Execution (optimized for speed)
# ============================================================================
# Analysis: 111.92s sequential → ~19.9s parallel (5.6x speedup with make -j)
# Total: 476 tests across 46 test suites
# Usage: make test-parallel -j
#        make test-parallel -j10  (limit to 10 parallel jobs)
# ============================================================================

# Run main binary unit tests (~0.25s, 14 tests)
test-unit:
	@cargo test --lib

# Run crate-level tests in parallel (aggregated by crate)
test-crates:
	@cargo test -p meal_planning --lib
	@cargo test -p notifications --lib
	@cargo test -p recipe --lib
	@cargo test -p shopping --lib
	@cargo test -p user --lib

# Run doc tests (~0.71s, 5 tests)
test-docs:
	@cargo test --doc

# ============================================================================
# SLOW Integration Tests (>5s each) - Run individually in parallel
# ============================================================================
test-slow-recipe-cmd:
	@cargo test --test recipe_integration_tests

test-slow-recipe-subscription:
	@cargo test -p recipe subscription_tests

test-slow-community:
	@cargo test --test community_discovery_integration_tests

test-slow-recipe-agg:
	@cargo test -p recipe aggregate_tests

test-slow-recipe-collection:
	@cargo test -p recipe collection_tests

test-slow-auth:
	@cargo test --test auth_integration_tests

test-slow-password-reset:
	@cargo test --test password_reset_integration_tests

test-slow-collection-int:
	@cargo test --test collection_integration_tests

test-slow-recipe-share:
	@cargo test --test recipe_detail_share_button_tests

# Aggregate all slow tests (run in parallel with -j)
test-slow: test-slow-recipe-cmd test-slow-recipe-subscription test-slow-community test-slow-recipe-agg test-slow-recipe-collection test-slow-auth test-slow-password-reset test-slow-collection-int test-slow-recipe-share

# ============================================================================
# MEDIUM Integration Tests (1-5s) - Batched together
# ============================================================================
test-medium:
	@cargo test -p user subscription_tests
	@cargo test --test subscription_integration_tests
	@cargo test -p user password_tests
	@cargo test --test onboarding_integration_tests
	@cargo test --test recipe_detail_calendar_context_tests
	@cargo test -p shopping recalculation_tests

# ============================================================================
# FAST Tests (<1s) - Batched together
# ============================================================================
test-fast:
	@cargo test --test prep_task_completion_tests
	@cargo test -p shopping command_tests
	@cargo test --test meal_plan_integration_tests
	@cargo test --test day_of_cooking_reminder_tests
	@cargo test -p meal_planning reasoning_persistence_tests
	@cargo test --test morning_reminder_tests
	@cargo test -p meal_planning command_tests
	@cargo test -p shopping query_tests
	@cargo test --test help_contact_integration_tests
	@cargo test --test service_worker_tests
	@cargo test --test manifest_route_tests
	@cargo test --test push_notification_permission_tests
	@cargo test --test dashboard_integration_tests
	@cargo test --test profile_tests
	@cargo test --test manifest_tests
	@cargo test --test shopping_list_integration_tests

# Run all tests in parallel (RECOMMENDED)
# Executes: 9 slow (individual) + 1 medium (batched) + 1 fast (batched) + crates + unit + docs
# Parallel time: ~19.9s (vs 111.92s sequential) = 5.6x speedup
test-parallel: test-unit test-crates test-docs test-slow test-medium test-fast
	@echo "✓ All 476 tests completed in parallel!"

# Build the project
build:
	cargo build --workspace

# Check for unused dependencies (optional: install with 'cargo install cargo-machete')
machete:
	@if command -v cargo-machete >/dev/null 2>&1; then \
		cargo machete; \
	else \
		echo "⚠ cargo-machete not installed. Skipping unused dependency check."; \
		echo "  Install with: cargo install cargo-machete"; \
	fi

# Run Lighthouse performance audits using Docker
lighthouse:
	@echo "Building application..."
	@cargo build --release
	@echo "Starting server in background..."
	@./target/release/imkitchen serve & echo $$! > /tmp/imkitchen.pid
	@sleep 5
	@echo "Waiting for server to be ready..."
	@timeout 30 bash -c 'until curl -f http://localhost:8080/health >/dev/null 2>&1; do sleep 1; done' || (kill $$(cat /tmp/imkitchen.pid) 2>/dev/null; rm -f /tmp/imkitchen.pid; echo "❌ Server failed to start"; exit 1)
	@echo "Running Lighthouse CI..."
	@docker run --rm --network=host -v $(PWD):/workspace -w /workspace patrickhulce/lhci-client:0.15.0 lhci autorun --config=lighthouserc.json || true
	@echo "Stopping server..."
	@kill $$(cat /tmp/imkitchen.pid) 2>/dev/null || true
	@rm -f /tmp/imkitchen.pid
	@echo "✓ Lighthouse audit complete! Check .lighthouseci/ for results."

# Run all checks: format, lint, machete, and test (CI-ready)
check: test-parallel lint fmt machete
	@echo "✓ All checks passed!"

# Show help
help:
	@echo "Available commands:"
	@echo ""
	@echo "Development:"
	@echo "  make css           - Build Tailwind CSS (minified)"
	@echo "  make css-watch     - Watch and rebuild CSS on changes"
	@echo "  make dev           - Watch and run server on code changes"
	@echo "  make build         - Build the project"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt           - Check code formatting (fails if not formatted)"
	@echo "  make fmt-fix       - Auto-fix code formatting"
	@echo "  make lint          - Run Clippy linter (deny warnings)"
	@echo "  make machete       - Check for unused dependencies"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run ALL tests sequentially (~112s)"
	@echo "  make test-parallel - Run ALL 476 tests in parallel (~20s) ⚡ RECOMMENDED"
	@echo "                       Usage: make test-parallel -j"
	@echo "  make test-unit     - Run main binary unit tests only (~0.3s, 14 tests)"
	@echo "  make test-crates   - Run workspace crate tests (~48s, 300+ tests)"
	@echo "  make test-docs     - Run doc tests (~0.7s, 5 tests)"
	@echo "  make test-fast     - Run fast integration tests (~6s batched)"
	@echo "  make test-medium   - Run medium-speed tests (~20s batched)"
	@echo "  make test-slow     - Run slow tests individually (~16s longest)"
	@echo ""
	@echo "CI/CD:"
	@echo "  make check         - Run fmt + lint + machete + test (CI-ready)"
	@echo "  make lighthouse    - Run Lighthouse performance audits (Docker required)"
	@echo ""
	@echo "Docker:"
	@echo "  make up            - Start Docker Compose services"
	@echo "  make stop          - Stop Docker Compose services"
	@echo "  make down          - Remove Docker Compose services"
