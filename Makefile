# IMKitchen Development Makefile
.PHONY: help build check clean watch-check watch-test watch-full watch-web dev-setup ci

# Default target
help:
	@echo "IMKitchen Development Commands"
	@echo "=============================="
	@echo ""
	@echo "Development Commands:"
	@echo "  dev         - Start web server"
	@echo "  watch-check - Watch and run fast checks"
	@echo "  watch-test  - Watch and run tests"
	@echo "  watch-full  - Watch and run comprehensive checks"
	@echo "  watch-web   - Watch and restart web server"
	@echo "  dev-setup   - Install development tools and hooks"
	@echo ""
	@echo "Build & Test Commands:"
	@echo "  build       - Build the project"
	@echo "  check       - Fast syntax checking"
	@echo "  test        - Run all tests"
	@echo "  clean       - Clean build artifacts"
	@echo ""
	@echo "Quality Commands:"
	@echo "  lint        - Run clippy linting"
	@echo "  fmt         - Format code"
	@echo "  audit       - Security audit (with ignores)"
	@echo "  audit-strict- Security audit (no ignores)"
	@echo "  audit-report- Generate security report"
	@echo "  audit-json  - Generate JSON security report"
	@echo "  machete     - Find unused dependencies"
	@echo "  outdated    - Check for outdated dependencies"
	@echo ""
	@echo "Git Hooks Commands:"
	@echo "  hooks-install   - Install pre-commit hooks"
	@echo "  hooks-uninstall - Remove pre-commit hooks"
	@echo "  hooks-test      - Test pre-commit hooks"
	@echo "  hooks-status    - Show hooks status"
	@echo ""
	@echo "Other Commands:"
	@echo "  tailwind    - Watch Tailwind CSS"
	@echo "  cert        - Generate development certificates"
	@echo "  up/stop/down - Docker compose operations"
	@echo "  e2e         - Run end-to-end tests"

# Build commands
build:
	cargo build

check:
	cargo check

clean:
	cargo clean

# Watch commands for development
watch-check:
	bash scripts/dev-watch.sh check

watch-test:
	bash scripts/dev-watch.sh test

watch-full:
	bash scripts/dev-watch.sh full

watch-web:
	bash scripts/dev-watch.sh web

# Development setup
dev-setup:
	@echo "Setting up development environment..."
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit; \
	fi
	bash scripts/setup-hooks.sh --install
	@echo "Development environment ready!"

# Git hooks management
hooks-install:
	bash scripts/setup-hooks.sh --install

hooks-uninstall:
	bash scripts/setup-hooks.sh --uninstall

hooks-test:
	bash scripts/setup-hooks.sh --test

hooks-status:
	bash scripts/setup-hooks.sh --status

# CI simulation
ci:
	@echo "Running CI checks..."
	cargo build
	cargo test
	cargo clippy -- -D warnings
	cargo fmt --check
	@echo "All CI checks passed!"

dev:
	cargo run -- web start
	# cargo run -- --log error,imkitchen=debug,evento=debug migrate -c ./imkitchen.toml
	# cargo watch -x 'run -- --log error,imkitchen=debug,evento=debug serve -c ./imkitchen.toml'

tailwind:
	tailwindcss -i ./tailwind.css -o ./crates/imkitchen-web/static/css/main.css --watch

reset:
	cargo run -- --log error,imkitchen=debug,evento=debug reset -c ./imkitchen.toml

cert:
	mkdir -p .docker/traefik/certs
	mkcert -install
	mkcert -key-file .docker/traefik/certs/imkitchen.key -cert-file .docker/traefik/certs/imkitchen.crt imkitchen.localhost traefik.localhost *.imkitchen.localhost

up:
	sudo docker compose up -d --remove-orphans

stop:
	sudo docker compose stop

down:
	sudo docker compose down -v --rmi local --remove-orphans

lint:
	cargo clippy --fix --allow-dirty --workspace --all-features -- -D warnings

test:
	cargo test

e2e:
	npx playwright test --headed

fmt:
	cargo fmt -- --emit files

machete:
	cargo machete

advisory.clean:
	rm -rf ~/.cargo/advisory-db

pants: advisory.clean
	cargo pants

audit: advisory.clean
	bash scripts/security-audit.sh

audit-strict:
	bash scripts/security-audit.sh --strict

audit-report:
	bash scripts/security-audit.sh --report

audit-json:
	bash scripts/security-audit.sh --json

outdated:
	cargo outdated

claude.git:
	@echo "git create branch,commit,push,create PR that closed #[] when merged"
