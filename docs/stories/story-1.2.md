# Story 1.2: User Login

Status: Approved

## Story

As a registered user,
I want to log in with my credentials,
so that I can access my meal plans and recipes.

## Acceptance Criteria

1. Login form accepts email and password
2. System validates credentials against stored hashed password
3. Successful login issues JWT token in HTTP-only secure cookie
4. Failed login displays generic error "Invalid credentials" (no user enumeration)
5. Login redirected to home dashboard
6. Session persists across browser restarts until token expiration
7. JWT token includes user ID and role (user/premium-user)
8. Token expiration set to 7 days with sliding expiration on activity

## Tasks / Subtasks

- [ ] Create login page template (AC: 1)
  - [ ] Create `templates/pages/login.html` with Askama
  - [ ] Add email input with HTML5 validation (type="email")
  - [ ] Add password input field
  - [ ] Display generic error message from server
  - [ ] Style form with Tailwind CSS utilities
  - [ ] Add "Don't have an account? Register" link
  - [ ] Add "Forgot Password?" link for password reset

- [ ] Implement POST /login route handler (AC: 2, 3, 4, 5)
  - [ ] Create LoginForm struct with email and password fields
  - [ ] Add validator derives for email format validation
  - [ ] Validate form inputs server-side
  - [ ] Query user by email via `query_user_by_email` function
  - [ ] Return generic "Invalid credentials" error if email not found
  - [ ] Verify password hash using `verify_password` utility
  - [ ] Return generic "Invalid credentials" error if password incorrect
  - [ ] Generate JWT token on successful authentication
  - [ ] Set HTTP-only, Secure, SameSite=Lax cookie with 7-day expiration
  - [ ] Redirect to /dashboard on success
  - [ ] Handle and display validation errors

- [ ] Add GET /login route (AC: 1)
  - [ ] Create route handler in `src/routes/auth.rs`
  - [ ] Render login page template
  - [ ] Display flash messages for errors (e.g., "Please log in to continue")

- [ ] Enhance JWT utilities for login (AC: 6, 7, 8)
  - [ ] Verify `generate_jwt` includes user_id, email, tier claims
  - [ ] Confirm JWT expiration set to 7 days
  - [ ] Verify token persists across browser restarts via cookie
  - [ ] Add unit tests for JWT validation with expired tokens

- [ ] Add comprehensive tests (AC: 1-8)
  - [ ] Integration test: POST /login with valid credentials sets JWT cookie
  - [ ] Integration test: POST /login with invalid email returns "Invalid credentials"
  - [ ] Integration test: POST /login with incorrect password returns "Invalid credentials"
  - [ ] Integration test: Successful login redirects to /dashboard
  - [ ] Integration test: JWT cookie is HTTP-only, Secure, SameSite=Lax
  - [ ] Integration test: JWT includes correct claims (user_id, email, tier)
  - [ ] Integration test: JWT expiration set to 7 days
  - [ ] Integration test: Login with expired JWT redirects to /login
  - [ ] E2E test: Complete login flow from form to dashboard

- [ ] Security validations (AC: 4)
  - [ ] Verify no user enumeration (same error for invalid email or password)
  - [ ] Log failed login attempts for security audit
  - [ ] Test password verification timing attack resistance
  - [ ] Confirm error messages never expose password hash

## Dev Notes

### Architecture Patterns and Constraints

**Authentication Flow:**
- Login does not create new events (no UserLoggedIn event in MVP)
- Authentication is stateless via JWT cookies
- Password verification uses existing Argon2 `verify_password` utility from Story 1.1
- Query user from `users` read model (already projected from UserCreated events)

**Security Constraints:**
- Generic error messages prevent email enumeration ("Invalid credentials" for both cases)
- Password verification uses constant-time comparison (Argon2 handles)
- Failed login attempts logged for security monitoring (OpenTelemetry)
- JWT cookies: HTTP-only (prevents XSS), Secure (HTTPS only), SameSite=Lax (CSRF protection)

**Session Management:**
- Stateless JWT tokens (no server-side session store)
- 7-day expiration (acceptable for MVP, no refresh token pattern)
- Token includes tier claim (free/premium) for authorization checks

**Performance Considerations:**
- Email lookup via indexed query on `users.email` (<50ms)
- Password verification ~100ms (Argon2 default params)
- JWT generation <10ms
- Total login flow <200ms

### Source Tree Components to Touch

**Files to Modify:**
```
src/routes/auth.rs                 # Add GET/POST /login handlers
templates/pages/login.html         # Create login form template (NEW)
tests/auth_integration_tests.rs    # Add login flow tests
```

**Existing Files (from Story 1.1):**
```
crates/user/src/read_model.rs      # Use query_user_by_email (already exists)
crates/user/src/password.rs        # Use verify_password (already exists)
src/routes/auth.rs                 # JWT generation already implemented
```

**No New Domain Logic**: Login uses existing query and password utilities. No new events or aggregates.

### Testing Standards Summary

**TDD Approach:**
1. Write failing integration test for POST /login with valid credentials
2. Implement login_handler to make test pass
3. Write failing test for invalid credentials
4. Add error handling
5. Write E2E test for full login flow
6. Repeat for all acceptance criteria

**Test Coverage Targets:**
- Route handlers: 85%
- Templates: Manual verification via E2E tests

**Integration Test Setup:**
```rust
#[tokio::test]
async fn test_login_with_valid_credentials() {
    let pool = setup_test_db().await;
    let app = create_test_app(pool.clone()).await;

    // Pre-create user via registration
    let user_id = create_test_user(&pool, "test@example.com", "password123").await;

    // Attempt login
    let resp = app
        .post("/login")
        .form(&[
            ("email", "test@example.com"),
            ("password", "password123"),
        ])
        .await;

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert_eq!(resp.headers().get("Location").unwrap(), "/dashboard");

    let cookie = resp.cookies().find(|c| c.name() == "auth_token").unwrap();
    assert!(cookie.http_only());
    assert!(cookie.secure());
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
}
```

**E2E Test (Playwright):**
```typescript
test('user login flow', async ({ page }) => {
  // Precondition: User registered
  await registerUser(page, 'test@example.com', 'password123');

  await page.goto('/login');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.click('button[type="submit"]');

  await expect(page).toHaveURL('/dashboard');

  const cookies = await page.context().cookies();
  const authCookie = cookies.find(c => c.name === 'auth_token');
  expect(authCookie).toBeDefined();
  expect(authCookie?.httpOnly).toBe(true);
  expect(authCookie?.secure).toBe(true);
  expect(authCookie?.sameSite).toBe('Lax');
});
```

### Project Structure Notes

**Alignment with solution-architecture.md:**

Login implementation follows established auth patterns from Story 1.1:

```
imkitchen/
├── src/routes/auth.rs             # Add GET/POST /login handlers (modify)
├── templates/pages/
│   └── login.html                 # Login form template (NEW)
└── tests/
    └── auth_integration_tests.rs  # Add login tests (modify)
```

**Reused Components from Story 1.1:**
- `crates/user/src/read_model.rs::query_user_by_email` - Query user by email
- `crates/user/src/password.rs::verify_password` - Verify password hash
- `src/routes/auth.rs::generate_jwt` - Generate JWT token
- `src/middleware/auth.rs` - Auth middleware (future use for protected routes)

**Naming Conventions:**
- Route handlers: `login_handler` (GET), `login_post_handler` (POST)
- Template: `login.html` (kebab-case)
- Form struct: `LoginForm` (PascalCase)

**Detected Conflicts/Variances:**
- None. Login builds on Story 1.1 foundation.

### References

**Technical Specifications:**
- [Source: docs/tech-spec-epic-1.md#authentication-routes] - POST /login route handler implementation
- [Source: docs/tech-spec-epic-1.md#workflows-and-sequencing] - Login flow diagram
- [Source: docs/tech-spec-epic-1.md#acceptance-criteria] - AC-2.1 to AC-2.5

**Architecture Decisions:**
- [Source: docs/solution-architecture.md#authentication-and-authorization] - JWT cookie-based auth strategy
- [Source: docs/solution-architecture.md#security] - Password verification, cookie security
- [Source: docs/solution-architecture.md#api-design] - Form-based authentication pattern

**Requirements:**
- [Source: docs/epics.md#story-12-user-login] - User story and acceptance criteria
- [Source: docs/epics.md#epic-1-technical-summary] - Security requirements (OWASP compliance)

**Previous Story Lessons:**
- [Source: docs/stories/story-1.1.md] - User registration implementation (password hashing, JWT generation already done)

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-12 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-1.md |

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.2.xml` - Generated 2025-10-12 - Story context with documentation, code artifacts, interfaces, constraints, and testing standards

### Agent Model Used

<!-- Will be populated by dev agent -->

### Debug Log References

<!-- Dev agent will add links to tracing logs here -->

### Completion Notes List

<!-- Dev agent will document completion status, deviations, issues encountered -->

### File List

<!-- Dev agent will list all files created/modified during implementation -->
