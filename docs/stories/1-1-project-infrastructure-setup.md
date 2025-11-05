# Story 1.1: Project Infrastructure Setup

Status: drafted

## Story

As a developer,
I want a properly configured Rust workspace with evento, axum, and database setup,
So that the project foundation supports event-driven architecture and web server capabilities.

## Acceptance Criteria

1. Workspace Cargo.toml configured with all required dependencies (evento 1.5+, axum 0.8+, sqlx, askama, etc.)
2. CLI commands implemented: serve, migrate, reset
3. Configuration system using TOML files (config/default.toml committed, config/dev.toml in .gitignore)
4. Separate databases created: write DB (evento), read DB (queries), validation DB
5. Migration structure created: migrations/queries/ and migrations/validation/
6. Playwright configured with example E2E test (tests/e2e/ directory created)
7. Rust test helper functions created for database setup (using sqlx::migrate! and evento::sql_migrator)
8. Project compiles without errors and passes clippy/fmt checks

## Tasks / Subtasks

- [ ] Task 1: Initialize Rust workspace (AC: 1)
  - [ ] Create workspace Cargo.toml with all required dependencies as workspace.dependencies
  - [ ] Configure workspace members array (root binary + future crates)
  - [ ] Add all dependencies per architecture.md: evento 1.5+, axum 0.8.6, axum-extra 0.12+, askama 0.14+, askama_web 0.14+, sqlx 0.8.2, validator 0.20+, ulid 1.2+, clap 4.5.23, config 0.15+, tracing, tokio 1.42+, serde, chrono

- [ ] Task 2: Implement CLI commands (AC: 2)
  - [ ] Create src/main.rs with clap CLI parser
  - [ ] Implement `serve` command to start Axum server
  - [ ] Implement `migrate` command to run database migrations (must create databases if they don't exist)
  - [ ] Implement `reset` command to drop databases and rerun migrate
  - [ ] Create src/server.rs with basic Axum server setup (empty routes for now)
  - [ ] Create src/migrate.rs with migration runner logic

- [ ] Task 3: Configuration system setup (AC: 3)
  - [ ] Create config/default.toml with default settings (server port, database paths, etc.)
  - [ ] Add config/dev.toml to .gitignore
  - [ ] Implement configuration loading in main.rs using config crate
  - [ ] Document configuration structure in config/default.toml comments

- [ ] Task 4: Database initialization (AC: 4, 5)
  - [ ] Create migrations/queries/ directory for read database migrations
  - [ ] Create migrations/validation/ directory for validation database migrations
  - [ ] Configure sqlx migrations in migrate.rs for queries and validation DBs
  - [ ] Configure evento migration support for write DB (evento.db)
  - [ ] Test migration commands create all three databases correctly

- [ ] Task 5: Testing infrastructure (AC: 6, 7)
  - [ ] Create tests/ directory for integration tests
  - [ ] Create tests/e2e/ directory for Playwright tests
  - [ ] Install Playwright dependencies (package.json with playwright 1.56+)
  - [ ] Create example E2E test (tests/e2e/smoke.spec.ts) verifying server starts
  - [ ] Create test helper functions in tests/helpers.rs for database setup using sqlx::migrate! and evento::sql_migrator
  - [ ] Document DRY pattern for database setup in test helpers

- [ ] Task 6: Code quality validation (AC: 8)
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix all issues
  - [ ] Run `cargo fmt --all` to format code
  - [ ] Verify project compiles: `cargo build --workspace`
  - [ ] Run basic integration test verifying CLI commands work

## Dev Notes

### Architecture Patterns

**Event-Driven CQRS Architecture:**
- Three separate SQLite databases: evento.db (write), queries.db (read), validation.db
- evento manages write DB exclusively - never query directly
- Query handlers update projections in read DB
- Validation DB used for async uniqueness checks in command handlers

**Project Structure:**
- Workspace with root binary + bounded context crates (added in later stories)
- All dependencies managed in workspace Cargo.toml for version consistency
- Configuration via TOML files (default.toml committed, dev.toml local)

**CLI Commands:**
- `serve` - Start web server (Axum on configured port, default 3000)
- `migrate` - Run all migrations (creates databases if missing)
- `reset` - Drop all databases and rerun migrate (development only)

**Testing Standards:**
- Integration tests in tests/ folder (NOT src/)
- Use sqlx::migrate! and evento::sql_migrator for test database setup
- DRY principle: Create reusable helper functions for database initialization
- Playwright for E2E critical flows

### Project Structure Notes

This story establishes the foundational structure per architecture.md:

```
imkitchen/
├── Cargo.toml                 # Workspace definition
├── config/
│   ├── default.toml          # Committed defaults
│   └── dev.toml              # .gitignore (local overrides)
├── src/
│   ├── main.rs               # CLI entry point
│   ├── server.rs             # Axum server
│   └── migrate.rs            # Migration runner
├── migrations/
│   ├── queries/              # Read DB migrations
│   └── validation/           # Validation DB migrations
├── tests/
│   ├── helpers.rs            # Reusable test utilities
│   └── e2e/                  # Playwright tests
├── .gitignore                # Must include config/dev.toml
└── package.json              # Playwright dependencies
```

Additional directories (crates/, templates/, static/) will be added in subsequent stories.

### References

- [Source: docs/architecture.md#Project Structure] - Complete directory layout
- [Source: docs/architecture.md#Technology Stack Details] - Dependency versions
- [Source: docs/architecture.md#Database Separation] - Three-database pattern
- [Source: docs/architecture.md#Development Environment] - Setup commands
- [Source: docs/PRD.md#Goals and Background Context] - Project overview
- [Source: CLAUDE.md#CLI and Configuration] - CLI command requirements
- [Source: CLAUDE.md#Database Guidelines] - Database separation rules
- [Source: CLAUDE.md#Testing Guidelines] - TDD and migration-based test setup

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

<!-- Will be populated during implementation -->

### Completion Notes List

<!-- Will be populated during implementation -->

### File List

<!-- Will be populated during implementation -->
