# Story 1.2: User Registration and Authentication

Status: drafted

## Story

As a new user,
I want to register an account with email and password,
So that I can access the meal planning platform securely.

## Acceptance Criteria

1. User aggregate created with evento: UserRegistered, UserLoggedIn events
2. Registration command validates email format and password requirements
3. JWT cookie-based authentication implemented using evento metadata pattern
4. Login route returns JWT token stored in secure HTTP-only cookie
5. Protected routes verify JWT token and extract user_id
6. Registration/login forms rendered with Askama templates
7. User projection table created in queries DB with email, hashed_password, created_at
8. Tests verify registration, login, and protected route access

## Tasks / Subtasks

- [ ] Task 1: Create User bounded context crate (AC: 1)
  - [ ] Create crates/imkitchen-user/ directory with Cargo.toml
  - [ ] Create src/lib.rs, src/event.rs, src/aggregate.rs, src/command.rs
  - [ ] Define EventMetadata struct with user_id (optional) and request_id (ULID)
  - [ ] Define UserRegistered event with email field
  - [ ] Define UserLoggedIn event with email and timestamp
  - [ ] Define User aggregate with Default implementation
  - [ ] Implement event handlers in aggregate: user_registered, user_logged_in
  - [ ] Add imkitchen-user to workspace members in root Cargo.toml

- [ ] Task 2: Implement registration command with validation (AC: 2)
  - [ ] Define RegisterUserInput struct with email and password fields
  - [ ] Add validator derive to RegisterUserInput (email format, password min 8 chars)
  - [ ] Create Command struct with Executor and validation_pool (SqlitePool)
  - [ ] Implement register_user command accepting RegisterUserInput and EventMetadata
  - [ ] Use evento::create pattern to emit UserRegistered event
  - [ ] Command returns user_id (aggregator_id) on success
  - [ ] Write unit tests for validation (invalid email, weak password)

- [ ] Task 3: Implement command handler for async email validation (AC: 2)
  - [ ] Create on_user_registered handler in crates/imkitchen-user/src/command.rs
  - [ ] Query validation DB to check if email already exists (user_emails table)
  - [ ] If email exists, emit UserRegistrationFailed event with error message
  - [ ] If email unique, insert into user_emails validation table
  - [ ] Hash password using argon2 (time_cost=2, mem_cost=19456, parallelism=1)
  - [ ] Emit UserRegistrationSucceeded event with hashed_password
  - [ ] Create subscribe_user_command function returning SubscriptionBuilder

- [ ] Task 4: Create user projection in queries DB (AC: 7)
  - [ ] Create migration: migrations/queries/TIMESTAMP_users.sql
  - [ ] Define users table: id (TEXT PK), email (TEXT UNIQUE), hashed_password (TEXT), is_admin (BOOLEAN DEFAULT 0), is_suspended (BOOLEAN DEFAULT 0), created_at (INTEGER)
  - [ ] Create migration: migrations/validation/TIMESTAMP_user_emails.sql
  - [ ] Define user_emails validation table: user_id (TEXT PK), email (TEXT UNIQUE)
  - [ ] Create src/queries/users.rs with query handler on_user_registration_succeeded
  - [ ] Query handler inserts into users table using event.timestamp for created_at
  - [ ] Create subscribe_user_query function with skip for UserRegistered

- [ ] Task 5: Implement JWT authentication (AC: 3, 4, 5)
  - [ ] Add jsonwebtoken and argon2 dependencies to workspace Cargo.toml
  - [ ] Create JWT claims struct with user_id, is_admin, exp (7 days)
  - [ ] Implement JWT signing in login handler using configured secret key
  - [ ] Set JWT in secure HTTP-only cookie with SameSite=Strict
  - [ ] Create auth middleware to verify JWT and populate Extension<User>
  - [ ] Middleware redirects to /auth/login if token invalid/missing
  - [ ] Add JWT secret to config/default.toml (override in config/dev.toml)

- [ ] Task 6: Create registration and login routes (AC: 6)
  - [ ] Create src/routes/auth/ module with login.rs and register.rs
  - [ ] Create templates/pages/auth/login.html with Askama template
  - [ ] Create templates/pages/auth/register.html with Askama template
  - [ ] GET /auth/register renders registration form
  - [ ] POST /auth/register calls register_user command, handles errors in template
  - [ ] GET /auth/login renders login form
  - [ ] POST /auth/login verifies credentials, sets JWT cookie, redirects to dashboard
  - [ ] Use axum_extra::extract::Form for form handling

- [ ] Task 7: Implement login command and query (AC: 3)
  - [ ] Define LoginInput struct with email and password
  - [ ] Implement login_user command that verifies credentials against projection
  - [ ] Query users table for email, retrieve hashed_password
  - [ ] Use argon2 to verify password hash
  - [ ] If valid, emit UserLoggedIn event and return user_id
  - [ ] If invalid, return authentication error
  - [ ] Create query function get_user_by_email in src/queries/users.rs

- [ ] Task 8: Testing (AC: 8)
  - [ ] Create tests/auth_test.rs with database helper functions
  - [ ] Test: Successful registration creates user projection
  - [ ] Test: Duplicate email registration fails with validation error
  - [ ] Test: Successful login returns JWT cookie
  - [ ] Test: Invalid credentials return error
  - [ ] Test: Protected route accessible with valid JWT
  - [ ] Test: Protected route redirects without JWT
  - [ ] Use SubscriptionBuilder.unsafe_oneshot for synchronous event processing in tests

- [ ] Task 9: Code quality validation
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt --all
  - [ ] Verify all tests pass: cargo test
  - [ ] Manual test: Register user via browser, login, access protected route

## Dev Notes

### Architecture Patterns

**Event-Driven CQRS with Async Validation:**
- Registration command emits UserRegistered immediately, returns user_id
- Command handler performs async email validation in validation DB
- Success path: UserRegistrationSucceeded → creates user projection
- Failure path: UserRegistrationFailed → error shown to user (if still polling)
- CRITICAL: Never use projections in commands - only evento or validation tables

**Authentication Flow:**
- JWT tokens contain: user_id, is_admin, exp (7 days expiration)
- Tokens stored in secure HTTP-only cookies (SameSite=Strict for CSRF protection)
- Middleware validates JWT on protected routes, populates Extension<User>
- Invalid/missing tokens redirect to /auth/login

**Password Security:**
- Argon2id hashing (OWASP recommended)
- Cost parameters: time_cost=2, mem_cost=19456, parallelism=1
- Never store plaintext passwords
- Password requirements: min 8 chars (validator enforces)

### Project Structure Notes

New directories and files added:
- `crates/imkitchen-user/` - User bounded context
- `src/routes/auth/` - Authentication routes
- `src/queries/users.rs` - User projections and queries
- `templates/pages/auth/` - Registration/login forms
- `migrations/queries/TIMESTAMP_users.sql` - User projection table
- `migrations/validation/TIMESTAMP_user_emails.sql` - Email uniqueness validation
- `tests/auth_test.rs` - Authentication integration tests

### References

- [Source: docs/epics.md#Story 1.2] - Complete acceptance criteria
- [Source: docs/architecture.md#Authentication] - JWT cookie-based auth pattern
- [Source: docs/architecture.md#Security Architecture] - OWASP compliance, Argon2 settings
- [Source: docs/architecture.md#Command Pattern] - Input struct + metadata pattern
- [Source: docs/architecture.md#Query Pattern] - Projection table structure
- [Source: CLAUDE.md#Command Guidelines] - Async validation in command handlers
- [Source: CLAUDE.md#Evento Handler Guidelines] - SubscriptionBuilder pattern
- [Source: CLAUDE.md#Axum Guidelines] - Use axum_extra Form extractors

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
