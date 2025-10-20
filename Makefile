.PHONY: css css-watch dev lint fmt fmt-fix test build check machete lighthouse help

# Build CSS once
css:
	tailwindcss -i static/css/tailwind.css -o static/css/main.css --minify

# Watch CSS for changes
css-watch:
	tailwindcss -i static/css/tailwind.css -o static/css/main.css --watch

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

# Run tests
test:
	cargo test --workspace

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
check: test lint fmt machete
	@echo "✓ All checks passed!"

# Show help
help:
	@echo "Available commands:"
	@echo "  make css        - Build Tailwind CSS (minified)"
	@echo "  make css-watch  - Watch and rebuild CSS on changes"
	@echo "  make dev        - Watch and run server on code changes"
	@echo "  make fmt        - Check code formatting (fails if not formatted)"
	@echo "  make fmt-fix    - Auto-fix code formatting"
	@echo "  make lint       - Run Clippy linter (deny warnings)"
	@echo "  make machete    - Check for unused dependencies"
	@echo "  make test       - Run all tests"
	@echo "  make build      - Build the project"
	@echo "  make lighthouse - Run Lighthouse performance audits (Docker required)"
	@echo "  make check      - Run fmt + lint + machete + test (CI-ready)"
