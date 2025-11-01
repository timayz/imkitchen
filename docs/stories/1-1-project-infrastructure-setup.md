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

- [ ] Task 1: Initialize Rust workspace with bounded context crates (AC: 1)
  - [ ] Create workspace Cargo.toml in project root with all dependencies as workspace dependencies
  - [ ] Create bounded context crates: crates/imkitchen-user/, crates/imkitchen-recipe/, crates/imkitchen-mealplan/
  - [ ] Each crate Cargo.toml configured to use workspace dependencies
  - [ ] Verify dependency versions: evento 1.5+, axum 0.8.6, sqlx 0.8.2, askama 0.14.0, etc.

- [ ] Task 2: Implement CLI commands (serve, migrate, reset) (AC: 2)
  - [ ] Create src/main.rs with clap CLI parser
  - [ ] Implement `serve` command that starts axum server
  - [ ] Implement `migrate` command using sqlx::migrate! and evento::sql_migrator
  - [ ] Migrate command creates databases if they don't exist
  - [ ] Implement `reset` command that drops databases and runs migrate
  - [ ] Add command help text and argument validation

- [ ] Task 3: Set up configuration system (AC: 3)
  - [ ] Create config/ directory
  - [ ] Create config/default.toml with all default settings (committed to git)
  - [ ] Add config/dev.toml to .gitignore
  - [ ] Implement config loading using config crate
  - [ ] Document required config fields in default.toml

- [ ] Task 4: Create database structure (AC: 4, 5)
  - [ ] Set up three SQLite databases: evento.db (write), queries.db (read), validation.db
  - [ ] Create migrations/queries/ directory for read database migrations
  - [ ] Create migrations/validation/ directory for validation database migrations
  - [ ] Ensure migrate command creates databases if they don't exist
  - [ ] Document database separation in README or architecture docs

- [ ] Task 5: Configure Playwright E2E testing (AC: 6)
  - [ ] Initialize npm package.json for Playwright
  - [ ] Install Playwright dependencies
  - [ ] Create tests/e2e/ directory
  - [ ] Create example E2E test (e.g., health check or home page load)
  - [ ] Add Playwright config file (playwright.config.ts)
  - [ ] Document E2E test execution in README

- [ ] Task 6: Create Rust test helper functions (AC: 7)
  - [ ] Create tests/ directory in project root
  - [ ] Implement database setup helpers using sqlx::migrate!
  - [ ] Implement evento setup helpers using evento::sql_migrator
  - [ ] Create test fixtures for common test scenarios
  - [ ] Document test helper usage

- [ ] Task 7: Verify code quality standards (AC: 8)
  - [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - [ ] Run `cargo fmt --all`
  - [ ] Fix all clippy warnings (no #[allow(...)] suppressions)
  - [ ] Ensure project compiles without errors
  - [ ] Document code quality commands in README

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md):
- **Workspace Structure**: Main binary + bounded context crates pattern (imkitchen-user, imkitchen-recipe, imkitchen-mealplan)
- **Database Separation**: Three SQLite databases following CQRS pattern:
  - evento.db (write DB) - managed exclusively by evento
  - queries.db (read DB) - projections for queries
  - validation.db - async validation constraints
- **CLI Commands**: serve (start server), migrate (run migrations + create DBs), reset (drop + migrate)
- **Configuration**: TOML-based (config/default.toml committed, config/dev.toml gitignored)

### Dependency Versions

All dependencies must be managed using workspace dependencies in root Cargo.toml:
- evento: 1.5+ (feature: sqlite)
- axum: 0.8.6
- axum-extra: 0.12+ (features: form, query)
- askama: 0.14+
- askama_web: 0.14+
- sqlx: 0.8.2 (features: runtime-tokio-rustls, sqlite)
- validator: 0.20+
- ulid: 1.2+
- clap: 4.5.23
- config: 0.15.0

From [CLAUDE.md](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md):
- Migration files must follow format: `{timestamp}_{table_name}.sql` (timestamp format: YYYYMMDDHHmmss)
- Always create database if it doesn't exist (migrate command requirement)

### Project Structure Notes

Expected project structure aligned with architecture.md:

```
imkitchen/
├── Cargo.toml                 # Workspace definition
├── config/
│   ├── default.toml          # Default configuration (committed)
│   └── dev.toml              # Local dev config (.gitignore)
├── src/
│   ├── main.rs               # CLI entry point
│   ├── lib.rs                # Shared app types
│   ├── server.rs             # Web server (serve command)
│   └── migrate.rs            # Database migrations
├── crates/
│   ├── imkitchen-user/
│   │   └── Cargo.toml
│   ├── imkitchen-recipe/
│   │   └── Cargo.toml
│   └── imkitchen-mealplan/
│       └── Cargo.toml
├── migrations/
│   ├── queries/              # Read database migrations
│   └── validation/           # Validation database migrations
└── tests/
    └── e2e/                  # Playwright tests
```

### Testing Standards

From [CLAUDE.md](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md):
- Always use migrations for database setup (sqlx::migrate! and evento::sql_migrator)
- Never create tables directly using sqlx::query in tests
- Apply DRY principle for test database setup - create reusable helpers
- Test files in tests/ folder (not in src/)

### References

- [PRD: Epic 1 Overview](../PRD.md#epic-1-foundation--user-management) - Project foundation requirements
- [Architecture: Project Structure](../architecture.md#project-structure) - Detailed folder structure
- [Architecture: Technology Stack](../architecture.md#technology-stack-details) - Dependency versions
- [CLAUDE.md: CLI Configuration](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#cli-and-configuration) - CLI command requirements
- [CLAUDE.md: Database Guidelines](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#database-guidelines) - Database separation rules
- [CLAUDE.md: Testing Guidelines](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#testing-guidelines) - Test structure requirements

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Will be filled by Dev agent -->

### Debug Log References

<!-- Dev agent logs will be added here -->

### Completion Notes List

<!-- Dev agent completion notes will be added here -->

### File List

<!-- List of files created/modified will be added here -->
