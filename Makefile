.PHONY: css css-watch dev lint fmt fmt-fix test check machete help

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

test:
	cargo test --workspace

# Check for unused dependencies (optional: install with 'cargo install cargo-machete')
machete:
	@if command -v cargo-machete >/dev/null 2>&1; then \
		cargo machete; \
	else \
		echo "⚠ cargo-machete not installed. Skipping unused dependency check."; \
		echo "  Install with: cargo install cargo-machete"; \
	fi

# Run all checks: format, lint, machete, and test (CI-ready)
check: test lint fmt-fix machete
	@echo "✓ All checks passed!"

# Show help
help:
	@echo "Available commands:"
	@echo ""
	@echo "Development:"
	@echo "  make css           - Build Tailwind CSS (minified)"
	@echo "  make css-watch     - Watch and rebuild CSS on changes"
	@echo "  make dev           - Watch and run server on code changes"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt           - Check code formatting (fails if not formatted)"
	@echo "  make fmt-fix       - Auto-fix code formatting"
	@echo "  make lint          - Run Clippy linter (deny warnings)"
	@echo "  make machete       - Check for unused dependencies"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run ALL tests sequentially (~112s)"
	@echo ""
	@echo "CI/CD:"
	@echo "  make check         - Run fmt + lint + machete + test (CI-ready)"
	@echo ""
	@echo "Docker:"
	@echo "  make up            - Start Docker Compose services"
	@echo "  make stop          - Stop Docker Compose services"
	@echo "  make down          - Remove Docker Compose services"
