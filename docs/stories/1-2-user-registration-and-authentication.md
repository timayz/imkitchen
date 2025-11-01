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

- [ ] Task 1: Create User bounded context with evento events (AC: 1)
  - [ ] Create User aggregate in crates/imkitchen-user/src/aggregate.rs
  - [ ] Define UserRegistered event with email, hashed_password fields
  - [ ] Define UserLoggedIn event with timestamp
  - [ ] Define EventMetadata struct with user_id (optional) and request_id (ULID)
  - [ ] Implement aggregate methods to apply events

- [ ] Task 2: Implement registration command with validation (AC: 2)
  - [ ] Create RegisterUserInput struct with email, password fields
  - [ ] Add validator crate validation: email format, password min 8 chars
  - [ ] Create Command struct with Executor and validation_pool
  - [ ] Implement register_user command using evento::create
  - [ ] Hash password with argon2 before emitting event
  - [ ] Validate email uniqueness in command handler (async validation)

- [ ] Task 3: Implement JWT authentication system (AC: 3, 4, 5)
  - [ ] Add jsonwebtoken and argon2 dependencies
  - [ ] Create JWT token generation function (user_id, is_admin, exp)
  - [ ] Implement login command that verifies password and emits UserLoggedIn
  - [ ] Create authentication middleware to extract JWT from HTTP-only cookie
  - [ ] Middleware populates Extension<User> for protected routes
  - [ ] Redirect to /auth/login if token missing or invalid

- [ ] Task 4: Create registration and login forms (AC: 6)
  - [ ] Create templates/pages/auth/register.html with Askama template
  - [ ] Create templates/pages/auth/login.html with Askama template
  - [ ] Add form validation display for errors
  - [ ] Style forms with Tailwind CSS
  - [ ] Implement Twinspark form submission with error handling

- [ ] Task 5: Create user projection table and query handlers (AC: 7)
  - [ ] Create migration: migrations/queries/20250101000000_users.sql
  - [ ] Define users table: id, email, hashed_password, is_admin, is_suspended, created_at
  - [ ] Create validation table: migrations/validation/20250101000000_user_emails.sql
  - [ ] Implement query handler for UserRegistered event
  - [ ] Implement query handler for UserLoggedIn event (optional: track last_login)
  - [ ] Create subscription builder function for user query handlers

- [ ] Task 6: Create command handler for async email validation (AC: 2)
  - [ ] Implement on_user_registered handler to check email uniqueness
  - [ ] Query validation DB for existing email
  - [ ] If exists, emit UserRegistrationFailed event with error message
  - [ ] If unique, insert into validation table and emit UserRegistrationSucceeded event
  - [ ] Create subscription builder for command handlers

- [ ] Task 7: Implement route handlers for registration and login (AC: 4)
  - [ ] Create src/routes/auth/register.rs with GET and POST handlers
  - [ ] Create src/routes/auth/login.rs with GET and POST handlers
  - [ ] POST /auth/register creates user and returns pending template (polls for success/failure)
  - [ ] POST /auth/login verifies credentials and sets JWT cookie
  - [ ] Create polling endpoint for registration status check
  - [ ] Implement logout route to clear JWT cookie

- [ ] Task 8: Write comprehensive tests (AC: 8)
  - [ ] Create tests/auth_test.rs with database setup helpers
  - [ ] Test: User can register with valid email and password
  - [ ] Test: Registration fails with invalid email format
  - [ ] Test: Registration fails with weak password
  - [ ] Test: User can login with correct credentials
  - [ ] Test: Login fails with incorrect password
  - [ ] Test: Protected route accessible with valid JWT
  - [ ] Test: Protected route redirects when JWT missing
  - [ ] Test: Email uniqueness validation prevents duplicate registration

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md):

**Event-Driven Authentication Flow:**
1. User submits registration form → POST /auth/register
2. Registration command emits UserRegistered event
3. Command handler validates email uniqueness asynchronously
4. Emits UserRegistrationSucceeded or UserRegistrationFailed
5. Query handler updates user projection
6. Polling endpoint returns success/failure template

**JWT Cookie Pattern:**
- Token lifetime: 7 days with sliding window refresh
- HTTP-only cookies prevent XSS attacks
- Token payload: { user_id, is_admin, exp }
- Middleware verifies signature and populates Extension<User>

**Database Tables:**

users table (queries.db):
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    is_suspended BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
```

user_emails table (validation.db):
```sql
CREATE TABLE user_emails (
    email TEXT PRIMARY KEY,
    user_id TEXT NOT NULL
);
```

### Security Requirements

From [architecture.md](../architecture.md#security-architecture):
- **Password Hashing**: Argon2id algorithm (OWASP recommended) with auto-generated salt
- **Password Requirements**: Minimum 8 characters (extend with uppercase, lowercase, number in validation)
- **JWT Security**: HTTP-only cookies with SameSite=Strict attribute
- **CSRF Protection**: Double-submit cookie pattern for state-changing operations

From [CLAUDE.md](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#command-guidelines):
- Commands MUST use input struct as first argument, metadata as second
- NEVER use projections in commands (eventually consistent)
- Async validation deferred to command handlers
- Metadata must include user_id (optional) and request_id (ULID)

### Twinspark Polling Pattern

Registration flow with async validation:
1. User submits form → UserRegistered event emitted immediately
2. Return pending template with polling setup: `ts-req="/auth/register/status/{user_id}" ts-trigger="load delay 1s"`
3. Pending template shows "Creating account, please wait..."
4. Command handler processes validation in background
5. Polling endpoint checks user status in projection
6. Returns success template (redirect to dashboard) or error template (back to form)

### References

- [PRD: FR001 Authentication Requirements](../PRD.md#functional-requirements) - JWT cookie-based auth
- [Architecture: Security Architecture](../architecture.md#security-architecture) - Authentication patterns
- [Architecture: Command Pattern](../architecture.md#command-pattern-per-claudemd) - Command structure
- [CLAUDE.md: Command Guidelines](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#command-guidelines) - Event metadata requirements
- [Mockups: login.html](../../mockups/login.html) - Visual reference for login form
- [Mockups: register.html](../../mockups/register.html) - Visual reference for registration form

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
