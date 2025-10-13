# Story 1.1: User Registration

Status: Approved

## Story

As a new user,
I want to create an account with email and password,
so that I can access personalized meal planning features.

## Acceptance Criteria

1. Registration form displays email and password fields with clear validation rules
2. Password must be minimum 8 characters
3. Email format validation applied on client and server side
4. System prevents duplicate email registrations with clear error message
5. Successful registration creates user account and logs user in automatically
6. JWT token stored in HTTP-only secure cookie with CSRF protection
7. User redirected to onboarding/profile setup after registration
8. Failed registration displays specific validation errors (weak password, duplicate email, invalid format)

## Tasks / Subtasks

- [ ] Create User domain crate structure (AC: 1-8)
  - [ ] Set up `crates/user/` module with `lib.rs`, `aggregate.rs`, `commands.rs`, `events.rs`, `error.rs`
  - [ ] Add Cargo.toml dependencies: evento, argon2, validator, serde
  - [ ] Define UserAggregate struct with user_id, email, password_hash, created_at fields
  - [ ] Implement evento aggregator trait on UserAggregate

- [ ] Implement UserAggregate event handlers (AC: 5)
  - [ ] Create UserCreated event struct (email, password_hash, created_at)
  - [ ] Implement `user_created` event handler to rebuild aggregate state
  - [ ] Add bincode::Encode and bincode::Decode derives for evento serialization

- [ ] Implement password hashing utilities (AC: 2, 8)
  - [ ] Create `password.rs` module with `hash_password` function using Argon2
  - [ ] Implement `verify_password` function for login validation
  - [ ] Add unit tests for password hashing and verification

- [ ] Create RegisterUserCommand and handler (AC: 1-4, 8)
  - [ ] Define RegisterUserCommand struct with email and password fields
  - [ ] Add validator derives: email format validation, password min length 8 chars
  - [ ] Implement `register_user` command handler
  - [ ] Add email uniqueness check via read model query
  - [ ] Return UserError::EmailAlreadyExists if duplicate found
  - [ ] Hash password with Argon2 before creating event
  - [ ] Generate UUID for user_id
  - [ ] Commit UserCreated event to evento stream

- [ ] Implement read model projection (AC: 5)
  - [ ] Create `read_model.rs` with `project_user_created` handler
  - [ ] Set up evento subscription for UserCreated events
  - [ ] Create `users` table migration with SQLx
  - [ ] Insert user record on UserCreated event with tier='free', recipe_count=0
  - [ ] Add query function `query_user_by_email` for duplicate check

- [ ] Create registration route handler (AC: 1, 6, 7, 8)
  - [ ] Create `src/routes/auth.rs` with GET /register endpoint
  - [ ] Create POST /register endpoint with RegisterForm validation
  - [ ] Call `register_user` command on form submission
  - [ ] Handle validation errors and return error messages
  - [ ] Generate JWT token with user_id, email, tier claims on success
  - [ ] Set HTTP-only, Secure, SameSite=Lax cookie with 7-day expiration
  - [ ] Redirect to /dashboard on success

- [ ] Create JWT token generation utilities (AC: 6)
  - [ ] Implement `generate_jwt` function using jsonwebtoken crate
  - [ ] Add Claims struct with sub (user_id), email, tier, exp, iat fields
  - [ ] Use RS256 algorithm with secret from config
  - [ ] Set token expiration to 7 days
  - [ ] Add unit tests for JWT generation and validation

- [ ] Create registration page template (AC: 1, 8)
  - [ ] Create `templates/pages/register.html` with Askama
  - [ ] Add email input with HTML5 validation (type="email")
  - [ ] Add password input with minlength="8" client-side validation
  - [ ] Add password confirmation field with validation
  - [ ] Display validation errors from server using Askama error handling
  - [ ] Style form with Tailwind CSS utilities
  - [ ] Add "Already have an account? Login" link

- [ ] Add comprehensive tests (AC: 1-8)
  - [ ] Unit tests: UserAggregate event handlers
  - [ ] Unit tests: password hashing/verification
  - [ ] Unit tests: JWT generation/validation
  - [ ] Integration test: POST /register with valid inputs creates user
  - [ ] Integration test: POST /register with duplicate email returns error
  - [ ] Integration test: POST /register with short password returns error
  - [ ] Integration test: POST /register with invalid email returns error
  - [ ] Integration test: Successful registration sets JWT cookie
  - [ ] Integration test: Successful registration redirects to /dashboard

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing Pattern:**
- Use evento aggregate pattern for User domain
- UserCreated event is immutable and forms the audit trail
- All state changes flow through events (no direct database writes in commands)
- Read model projections update `users` table asynchronously via evento subscriptions

**CQRS Separation:**
- Commands (`register_user`) write events to evento stream
- Queries (`query_user_by_email`) read from materialized `users` table
- Email uniqueness check queries read model before event commit

**Security Constraints:**
- Password hashing: Argon2id with OWASP-recommended parameters (memory=65536 KB, iterations=3)
- JWT cookies: HTTP-only (prevents XSS), Secure (HTTPS only), SameSite=Lax (CSRF protection)
- Password minimum 8 characters enforced via validator crate
- Email validation both client-side (HTML5) and server-side (validator)
- Never expose password hash in logs or error messages

**Performance Considerations:**
- Argon2 hashing targets ~100ms per operation (acceptable for registration)
- Email uniqueness check via indexed query on `users.email` (<50ms)
- evento event commit + projection < 200ms total
- JWT generation < 10ms

### Source Tree Components to Touch

**New Files:**
```
crates/user/
├── Cargo.toml                     # Dependencies: evento, argon2, validator, serde, uuid
├── src/
│   ├── lib.rs                     # Public API exports
│   ├── aggregate.rs               # UserAggregate with evento trait
│   ├── commands.rs                # RegisterUserCommand and handler
│   ├── events.rs                  # UserCreated event
│   ├── read_model.rs              # Projection handler for users table
│   ├── password.rs                # Argon2 utilities
│   └── error.rs                   # UserError enum (EmailAlreadyExists, etc.)
└── tests/
    ├── aggregate_tests.rs         # Aggregate behavior tests
    └── commands_tests.rs          # Command handler tests

src/routes/auth.rs                 # GET/POST /register endpoints
src/middleware/auth.rs             # JWT validation middleware (future stories)
templates/pages/register.html     # Registration form template
migrations/001_create_users_table.sql  # SQLx migration for users table

tests/auth_integration_tests.rs   # Integration tests for registration flow
```

**Modifications:**
- `src/main.rs`: Add `/register` route registration
- `src/routes/mod.rs`: Export auth module
- `Cargo.toml` (root): Add `user` crate to workspace members
- `.env`: Add `JWT_SECRET` environment variable

### Testing Standards Summary

**TDD Approach:**
1. Write failing unit test for UserAggregate.user_created event handler
2. Implement event handler to make test pass
3. Write failing unit test for password hashing
4. Implement Argon2 password utilities
5. Write failing integration test for POST /register
6. Implement route handler and command
7. Repeat for all acceptance criteria

**Test Coverage Targets:**
- User crate: 90% (security-critical code requires higher coverage)
- Route handlers: 85%
- Templates: Manual verification via E2E tests

**Integration Test Setup:**
```rust
// tests/common/mod.rs
pub async fn setup_test_db() -> Pool<Sqlite> {
    let pool = SqlitePoolOptions::new()
        .connect(":memory:")
        .await
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    pool
}

pub fn create_test_app(pool: Pool<Sqlite>) -> Router {
    // Initialize Axum app with test database
}
```

**E2E Test (Playwright):**
```typescript
test('user registration flow', async ({ page }) => {
  await page.goto('/register');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.fill('input[name="password_confirm"]', 'password123');
  await page.click('button[type="submit"]');

  await expect(page).toHaveURL('/dashboard');
  const cookie = await page.context().cookies();
  expect(cookie.find(c => c.name === 'auth_token')).toBeDefined();
  expect(cookie.find(c => c.name === 'auth_token')?.httpOnly).toBe(true);
});
```

### Project Structure Notes

**Alignment with unified-project-structure.md:**

Based on solution-architecture.md, the project follows a monorepo workspace structure:

```
imkitchen/
├── Cargo.toml (workspace root)
├── crates/
│   ├── user/          # User domain crate (NEW)
│   └── ...            # Future domain crates (recipe, meal_plan, etc.)
├── src/
│   ├── main.rs        # Root binary CLI entry point
│   ├── routes/        # HTTP route handlers
│   │   └── auth.rs    # Registration, login, logout routes (NEW)
│   ├── middleware/    # Axum middleware
│   │   └── auth.rs    # JWT validation (future)
│   └── ...
├── templates/
│   └── pages/
│       └── register.html  # Registration form (NEW)
├── migrations/
│   └── 001_create_users_table.sql  # SQLite schema (NEW)
└── tests/
    └── auth_integration_tests.rs   # Registration tests (NEW)
```

**Naming Conventions:**
- Domain crates: snake_case (e.g., `user`, not `User`)
- Event structs: PascalCase past tense (e.g., `UserCreated`)
- Command structs: PascalCase imperative (e.g., `RegisterUserCommand`)
- Route handlers: snake_case with `_handler` suffix (e.g., `register_handler`)
- Templates: kebab-case (e.g., `register.html`)

**Detected Conflicts/Variances:**
- None detected. This is the first story, establishing baseline structure.

### References

**Technical Specifications:**
- [Source: docs/tech-spec-epic-1.md#user-domain-crate] - Complete UserAggregate implementation
- [Source: docs/tech-spec-epic-1.md#apis-and-interfaces] - POST /register route handler
- [Source: docs/tech-spec-epic-1.md#data-models-and-contracts] - users table schema
- [Source: docs/tech-spec-epic-1.md#workflows-and-sequencing] - Registration flow diagram
- [Source: docs/tech-spec-epic-1.md#acceptance-criteria] - AC-1.1 to AC-1.5

**Architecture Decisions:**
- [Source: docs/solution-architecture.md#application-architecture] - Event-sourced monolith pattern
- [Source: docs/solution-architecture.md#technology-stack] - evento, Argon2, JWT, Askama dependencies
- [Source: docs/solution-architecture.md#security-architecture] - Cookie security, password hashing standards

**Requirements:**
- [Source: docs/epics.md#story-11-user-registration] - User story and acceptance criteria
- [Source: docs/epics.md#epic-1-technical-summary] - User aggregate, events, security requirements
- [Source: docs/PRD.md] - (If additional business context needed)

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-12 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-1.md |
| 2025-10-12 | Jonathan | Status updated to Approved |

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.1.xml) - Generated 2025-10-12

### Agent Model Used

<!-- Will be populated by dev agent -->

### Debug Log References

<!-- Dev agent will add links to tracing logs here -->

### Completion Notes List

<!-- Dev agent will document completion status, deviations, issues encountered -->

### File List

<!-- Dev agent will list all files created/modified during implementation -->
