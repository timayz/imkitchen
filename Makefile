.PHONY: css css-watch dev lint fmt fmt-fix test build check help

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

# Run all checks: format, lint, and test (CI-ready)
check: fmt lint test
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
	@echo "  make test       - Run all tests"
	@echo "  make build      - Build the project"
	@echo "  make check      - Run fmt + lint + test (CI-ready)"
