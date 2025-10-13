# Story 1.3: Password Reset Flow

Status: Approved

## Story

As a user who forgot my password,
I want to reset it via email,
so that I can regain access to my account.

## Acceptance Criteria

1. "Forgot Password" link available on login page
2. User enters email address to request reset
3. System sends password reset email with time-limited token (valid 1 hour)
4. Reset link directs to password reset form with token validation
5. User enters new password (min 8 characters) and confirms
6. Successful reset invalidates old password and all existing sessions
7. User redirected to login page with success message
8. Expired or invalid tokens display clear error message

## Tasks / Subtasks

- [ ] Create password reset request page template (AC: 1, 2)
  - [ ] Create `templates/pages/password-reset-request.html` with Askama
  - [ ] Add email input with HTML5 validation (type="email")
  - [ ] Display success message after submission
  - [ ] Style form with Tailwind CSS utilities
  - [ ] Add "Back to Login" link

- [ ] Implement POST /password-reset request handler (AC: 2, 3)
  - [ ] Create PasswordResetRequestForm struct with email field
  - [ ] Add validator derives for email format validation
  - [ ] Query user by email via `query_user_by_email` function
  - [ ] Generate secure time-limited reset token (1 hour expiration)
  - [ ] Store token with expiration in database or JWT
  - [ ] Send password reset email via SMTP (lettre)
  - [ ] Return success response regardless of email existence (prevent user enumeration)
  - [ ] Log password reset requests for security audit

- [ ] Create password reset completion page template (AC: 4, 5)
  - [ ] Create `templates/pages/password-reset-complete.html` with Askama
  - [ ] Add new password and confirm password fields
  - [ ] Extract token from URL query parameter
  - [ ] Display generic error for invalid/expired tokens
  - [ ] Style form with Tailwind CSS utilities

- [ ] Implement POST /password-reset/:token completion handler (AC: 4, 5, 6, 7, 8)
  - [ ] Create PasswordResetCompleteForm struct with new_password and password_confirm fields
  - [ ] Validate token (JWT signature and expiration)
  - [ ] Validate password minimum length (8 characters)
  - [ ] Verify password confirmation matches
  - [ ] Query user by token claims (user_id or email)
  - [ ] Update password via domain command (hash with Argon2)
  - [ ] Emit PasswordChanged event
  - [ ] Invalidate all existing JWT sessions (optional: track session IDs)
  - [ ] Redirect to login page with success flash message
  - [ ] Handle expired or invalid token errors

- [ ] Add GET /password-reset route (AC: 1)
  - [ ] Create route handler in `src/routes/auth.rs`
  - [ ] Render password reset request page template

- [ ] Add GET /password-reset/:token route (AC: 4)
  - [ ] Create route handler in `src/routes/auth.rs`
  - [ ] Validate token before rendering form
  - [ ] Render password reset completion page template
  - [ ] Display error page for invalid tokens

- [ ] Implement email sending via SMTP (AC: 3)
  - [ ] Configure lettre SMTP client with environment variables
  - [ ] Create password reset email template (HTML + plain text)
  - [ ] Include reset link with token: `https://imkitchen.app/password-reset/{token}`
  - [ ] Set email subject: "Password Reset Request - imkitchen"
  - [ ] Handle SMTP errors gracefully (log and return generic success)

- [ ] Add domain events and aggregate updates (AC: 6)
  - [ ] Create PasswordResetRequested event in `crates/user/src/events.rs`
  - [ ] Create PasswordChanged event in `crates/user/src/events.rs`
  - [ ] Add event handlers to UserAggregate
  - [ ] Update password_hash field in aggregate state
  - [ ] Add reset_token_expiration field to track active tokens

- [ ] Add comprehensive tests (AC: 1-8)
  - [ ] Integration test: POST /password-reset with valid email sends email
  - [ ] Integration test: POST /password-reset with invalid email returns success (no enumeration)
  - [ ] Integration test: GET /password-reset/:token with valid token renders form
  - [ ] Integration test: GET /password-reset/:token with expired token shows error
  - [ ] Integration test: POST /password-reset/:token with valid password updates password
  - [ ] Integration test: POST /password-reset/:token with mismatched passwords shows error
  - [ ] Integration test: Password reset invalidates old password (login fails)
  - [ ] Integration test: Reset token can only be used once
  - [ ] Unit test: Token generation and validation logic
  - [ ] Unit test: Token expiration (1 hour)

- [ ] Security validations (AC: 4, 8)
  - [ ] Verify token expiration enforced (1 hour)
  - [ ] Verify no user enumeration (same response for valid/invalid email)
  - [ ] Log all password reset attempts for security audit
  - [ ] Ensure tokens are cryptographically secure (JWT with HS256 or random bytes)
  - [ ] Confirm old sessions invalidated (optional: implement session tracking)

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing Pattern** (from solution-architecture.md):
- Password reset does NOT create events for request (stateless approach via JWT tokens)
- `PasswordChanged` event records successful password updates
- Event sourcing maintains audit trail of all password changes

**Security Requirements** (from solution-architecture.md, tech-spec-epic-1.md):
- Argon2 password hashing (OWASP recommended)
- HTTP-only, Secure, SameSite=Lax cookies for JWT
- Generic error messages prevent user enumeration
- Reset tokens valid for 1 hour only
- SMTP email delivery via lettre crate

**CQRS Pattern**:
- Commands: `RequestPasswordResetCommand`, `ResetPasswordCommand`
- Queries: `query_user_by_email` from read model
- Read model updated via evento subscriptions

**Server-Side Rendering** (from solution-architecture.md):
- Askama templates: `password-reset-request.html`, `password-reset-complete.html`
- TwinSpark progressive enhancement (optional for this flow)
- Traditional POST/Redirect/Get pattern for form handling

### Source Tree Components to Touch

**Root Binary Routes** (`src/routes/auth.rs`):
- Add `GET /password-reset` handler
- Add `POST /password-reset` handler (request)
- Add `GET /password-reset/:token` handler
- Add `POST /password-reset/:token` handler (complete)

**User Domain Crate** (`crates/user/`):
- `commands.rs`: Add `RequestPasswordResetCommand`, `ResetPasswordCommand`
- `events.rs`: Add `PasswordResetRequested`, `PasswordChanged` events
- `aggregate.rs`: Add event handlers for password changes
- `read_model.rs`: Query user by email (already exists from Story 1.1)
- `password.rs`: Reuse `hash_password` utility (already exists from Story 1.1)
- `error.rs`: Add password reset specific errors (InvalidToken, TokenExpired, etc.)

**Templates** (`templates/pages/`):
- `password-reset-request.html`: Email input form
- `password-reset-complete.html`: New password form with token

**Email Configuration**:
- SMTP settings in `config/default.toml` or environment variables
- Lettre SMTP client configuration in `src/config.rs`
- Email template (HTML + plain text)

**Tests** (`tests/`):
- `auth_integration_tests.rs`: Add password reset flow tests (10+ tests)

### Project Structure Notes

**Alignment with unified project structure**:
- Routes follow RESTful pattern: `/password-reset` (request), `/password-reset/:token` (complete)
- Templates follow naming convention: `password-reset-*.html`
- Domain crate structure: events, commands, aggregate handlers
- Integration tests in root `tests/` directory

**Token Implementation Options**:
1. **JWT Token** (preferred): Self-contained, stateless, includes user_id and expiration
   - Pros: No database storage, easy validation, matches existing auth pattern
   - Cons: Cannot be revoked before expiration (acceptable for 1-hour tokens)
2. **Random Token + Database**: Store token with expiration in `password_reset_tokens` table
   - Pros: Can be revoked, explicit expiration tracking
   - Cons: Additional database complexity, requires migration

**Recommendation**: Use JWT tokens (matches existing auth infrastructure, simpler implementation)

### Testing Standards Summary

**TDD Approach** (per architecture requirements):
1. Write tests first for each handler and domain command
2. Implement handlers to pass tests
3. Refactor while maintaining passing tests

**Test Coverage Goals** (per NFRs):
- 80% code coverage minimum
- Integration tests for all AC (8 acceptance criteria → 8+ tests)
- Unit tests for token generation/validation logic
- Security property tests (no enumeration, token expiration)

**Test Structure**:
- Use existing `tests/common/mod.rs` test harness
- Add tests to `tests/auth_integration_tests.rs`
- Mock SMTP email sending in tests (capture sent emails for assertions)

### References

- [Source: docs/solution-architecture.md#Section 5.1] - JWT Cookie-Based Authentication
- [Source: docs/solution-architecture.md#Section 17.1] - Authentication Security (Argon2)
- [Source: docs/tech-spec-epic-1.md#Section: Commands] - Command structures and patterns
- [Source: docs/epics.md#Story 1.3] - Acceptance criteria and technical notes
- [Source: docs/stories/story-1.1.md] - Registration implementation (password hashing pattern)
- [Source: docs/stories/story-1.2.md] - Login implementation (JWT generation pattern)
- [Source: crates/user/src/password.rs] - Existing password hashing utilities
- [Source: crates/user/src/jwt.rs] - Existing JWT utilities (to be extended for reset tokens)

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-12 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-1.md |
| 2025-10-12 | Bob (SM) | Generated story context XML with documentation and code artifacts; Status updated to Approved |

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.3.xml` - Generated 2025-10-12 - Story context with documentation, code artifacts, interfaces, constraints, and testing standards

### Agent Model Used

<!-- Will be populated by dev agent -->

### Debug Log References

<!-- Dev agent will add links to tracing logs here -->

### Completion Notes List

<!-- Dev agent will document completion status, deviations, issues encountered -->

### File List

<!-- Dev agent will list all files created/modified during implementation -->
