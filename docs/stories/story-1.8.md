# Story 1.8: User Logout

Status: Approved

## Story

As a logged-in user,
I want to log out,
So that my session is securely terminated.

## Acceptance Criteria

1. Logout button accessible from navigation menu on all authenticated pages
2. Clicking logout triggers POST /logout endpoint
3. POST /logout clears JWT cookie (auth_token) with Max-Age=0 and secure flags
4. User redirected to login page after logout (302 redirect to /login)
5. Logged-out user cannot access authenticated routes (redirected to /login by auth middleware)
6. Logout action logged for security audit (UserLoggedOut event optional for audit trail)
7. Logout confirmation message displayed on login page

## Tasks / Subtasks

- [ ] Implement logout route handler (AC: 2, 3, 4, 6)
  - [ ] Add `post_logout` handler to `src/routes/auth.rs`
  - [ ] Clear auth_token cookie with `Max-Age=0`, `HttpOnly`, `Secure`, `SameSite=Lax` flags
  - [ ] Redirect to `/login?logout=success` (302 status code)
  - [ ] Log logout event with user_id from JWT claims (tracing::info)
  - [ ] Optional: Emit UserLoggedOut domain event for audit trail
  - [ ] Register route in `src/routes/mod.rs` and `src/main.rs`

- [ ] Add logout button to navigation (AC: 1)
  - [ ] Update `templates/base.html` or navigation component
  - [ ] Add logout form/button in authenticated navigation menu
  - [ ] Style logout button with appropriate Tailwind classes
  - [ ] Ensure logout button only visible when user authenticated

- [ ] Display logout confirmation (AC: 7)
  - [ ] Update `templates/pages/login.html` to detect `?logout=success` query param
  - [ ] Show success message: "You have been logged out successfully"
  - [ ] Style message with success color (Tailwind green)

- [ ] Verify auth middleware behavior (AC: 5)
  - [ ] Confirm existing auth middleware redirects unauthenticated users to /login
  - [ ] Test: Access /dashboard after logout → redirected to /login
  - [ ] No code changes needed (middleware already implements this)

- [ ] Test logout functionality (AC: 1-7)
  - [ ] Unit test: `post_logout` clears cookie and redirects to /login
  - [ ] Integration test: POST /logout clears cookie, subsequent GET /dashboard redirects to /login
  - [ ] Integration test: Logout confirmation message displays on login page
  - [ ] E2E test (Playwright): Login → Navigate to dashboard → Click logout → Verify redirected to login with confirmation → Attempt dashboard access → Blocked

## Dev Notes

### Architecture Patterns

**JWT Cookie-Based Session Management**:
- Stateless authentication: no server-side session store required
- Logout implemented as client-side cookie clearing with appropriate flags
- Cookie cleared with `Max-Age=0` expires immediately in browser
- Auth middleware (`src/middleware/auth.rs`) already handles unauthenticated request redirection

**Event Sourcing (Optional)**:
- `UserLoggedOut` event can be emitted for audit trail completeness
- Event includes user_id, logout timestamp for security monitoring
- Read model projection not required (stateless JWT means no session to invalidate)
- Tracing logs provide sufficient audit trail for MVP (event sourcing optional)

**Security Best Practices**:
- Cookie clearing uses same security flags as cookie setting (HttpOnly, Secure, SameSite=Lax)
- Logout action logged via tracing for security monitoring
- Redirect to login page prevents confused deputy attacks
- Auth middleware provides defense-in-depth (even if cookie clearing fails)

### Source Tree Components

**Logout Route** (`src/routes/auth.rs`):
- `POST /logout` → Clear auth_token cookie, log logout, redirect to /login?logout=success
- Handler extracts user_id from Auth extension for logging before clearing cookie
- No authentication required for route (but Auth extension provides user context)

**Navigation Component** (`templates/base.html` or equivalent):
- Logout button/form added to authenticated navigation menu
- Form uses POST method (semantic correctness for state-changing operation)
- Hidden form with button trigger, or link with JavaScript form submission
- Visible only when user authenticated (Askama conditional rendering)

**Login Page** (`templates/pages/login.html`):
- Query param detection: check for `?logout=success` in template
- Display success alert: "You have been logged out successfully"
- Alert auto-dismisses after 5 seconds (optional JavaScript enhancement)

**Auth Middleware** (`src/middleware/auth.rs`):
- No changes required - existing implementation already redirects unauthenticated requests
- Validates JWT from cookie, redirects to /login if missing/invalid/expired
- Defense-in-depth: ensures logged-out users cannot access protected routes

### Testing Standards

**Unit Tests** (`src/routes/auth.rs` or `tests/auth_tests.rs`):
- Test `post_logout` handler returns redirect to /login?logout=success
- Test cookie clearing: verify Set-Cookie header contains `Max-Age=0`
- Test logging: verify tracing logs contain user_id and logout event

**Integration Tests** (`tests/auth_integration_tests.rs`):
- POST /logout clears cookie and redirects
- Subsequent GET /dashboard without cookie redirects to /login (401)
- GET /login with ?logout=success query param displays confirmation message

**E2E Tests** (`e2e/tests/auth.spec.ts`):
- Complete logout flow: Login → Dashboard → Click logout → Verify login page with confirmation
- Logout blocks protected access: After logout, attempt /dashboard → redirected to /login
- Logout confirmation message visible and dismisses

### References

**PRD**:
- [Source: docs/PRD.md#FR-18] - Authentication and authorization requirements
- [Source: docs/PRD.md#NFR-4] - Security standards compliance (OWASP)

**Epic Specification**:
- [Source: docs/epics.md#Story 1.8] - Original story definition for user logout
- [Source: docs/tech-spec-epic-1.md#Story 3] - AC-3.1, AC-3.2: Authoritative acceptance criteria

**Architecture**:
- [Source: docs/solution-architecture.md#Section 5.1] - JWT cookie-based authentication strategy
- [Source: docs/solution-architecture.md#Section 5.2] - Stateless session management (no refresh tokens)
- [Source: docs/solution-architecture.md#Section 5.3] - Auth middleware implementation

**Technical Specification**:
- [Source: docs/tech-spec-epic-1.md#APIs/POST /logout] - Logout endpoint implementation details
- [Source: docs/tech-spec-epic-1.md#Workflows/Login Flow] - Authentication flow context
- [Source: docs/tech-spec-epic-1.md#Events/UserLoggedOut] - Optional event for audit trail

### Project Structure Notes

**Alignment with solution-architecture.md**:
- Logout follows same auth route conventions as login/register (Section 2.3)
- Cookie clearing matches cookie setting security parameters (Section 5.1)
- Stateless JWT approach means no server-side session invalidation needed (Section 5.2)
- Auth middleware provides automatic redirection for logged-out users (Section 5.3)

**Consistency with Previous Stories**:
- Route registration pattern: add to routes/mod.rs, register in main.rs (Story 1.7 lesson)
- Template updates: Askama conditional rendering based on auth state (Stories 1.1-1.2)
- Tracing instrumentation: log user actions for observability (all previous stories)
- Test coverage: unit + integration + E2E for complete validation (Story 1.7 pattern)

**Rationale**:
- POST /logout semantically correct (state-changing operation)
- Query param ?logout=success provides confirmation without session state
- Cookie clearing with Max-Age=0 is browser-standard approach
- Auth middleware defense-in-depth prevents any session hijacking attempts
- Optional UserLoggedOut event maintains event sourcing completeness

## Dev Agent Record

### Context Reference

- [Story Context 1.8](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.8.xml) - Generated 2025-10-13

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

## Change Log
