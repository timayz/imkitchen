# Story 1.4: Admin User Management

Status: drafted

## Story

As an admin,
I want to view and manage user accounts,
So that I can suspend problematic users and manage premium bypass flags.

## Acceptance Criteria

1. is_admin flag added to user aggregate and projection
2. Admin panel route protected by admin-only middleware
3. Admin can view list of all users with pagination
4. Admin can suspend/activate user accounts (UserSuspended, UserActivated events)
5. Suspended users cannot log in (authentication check)
6. Suspended users' shared recipes hidden from community view
7. Admin can toggle premium_bypass flag per user (UserPremiumBypassToggled event)
8. Tests verify admin authentication, user suspension, and reactivation

## Tasks / Subtasks

- [ ] Task 1: Add is_admin support to User aggregate (AC: 1)
  - [ ] Update UserRegistered event to include is_admin field (default: false)
  - [ ] Update User aggregate to store is_admin state
  - [ ] Add is_admin column to users table in queries DB
  - [ ] Update registration flow to set first user as admin (optional: via config)

- [ ] Task 2: Define admin-related events (AC: 4, 7)
  - [ ] Add UserSuspended event with reason (optional) field
  - [ ] Add UserActivated event
  - [ ] Add UserPremiumBypassToggled event with new state (boolean)
  - [ ] Update User aggregate to handle all three events
  - [ ] Update aggregate state: is_suspended, premium_bypass

- [ ] Task 3: Implement admin commands (AC: 4, 7)
  - [ ] Create SuspendUserInput struct with user_id, reason fields
  - [ ] Implement suspend_user command (admin_user_id in metadata)
  - [ ] Create ActivateUserInput struct with user_id
  - [ ] Implement activate_user command
  - [ ] Create TogglePremiumBypassInput struct with user_id
  - [ ] Implement toggle_premium_bypass command
  - [ ] Validate requesting user is admin before executing commands

- [ ] Task 4: Create admin middleware (AC: 2)
  - [ ] Create src/middleware/admin.rs
  - [ ] Middleware verifies JWT is_admin claim
  - [ ] Returns 403 Forbidden if not admin
  - [ ] Apply middleware to all /admin/* routes

- [ ] Task 5: Update query handlers for admin events (AC: 4, 6, 7)
  - [ ] Update users table to add is_suspended column (BOOLEAN)
  - [ ] Create migration for column addition
  - [ ] Implement on_user_suspended handler (sets is_suspended = true)
  - [ ] Implement on_user_activated handler (sets is_suspended = false)
  - [ ] Implement on_user_premium_bypass_toggled handler
  - [ ] Update recipe query handlers to filter out recipes from suspended users

- [ ] Task 6: Create admin panel UI (AC: 3)
  - [ ] Create templates/pages/admin/users.html
  - [ ] Display user list: email, is_admin, is_suspended, is_premium_active, premium_bypass, created_at
  - [ ] Add pagination controls (20 users per page)
  - [ ] Add search/filter by email, status
  - [ ] Style with Tailwind CSS
  - [ ] Show stats: total users, premium users, suspended users

- [ ] Task 7: Implement admin route handlers (AC: 3, 4, 7)
  - [ ] Create src/routes/admin/users.rs
  - [ ] GET /admin/users - List all users with pagination
  - [ ] POST /admin/users/{id}/suspend - Suspend user account
  - [ ] POST /admin/users/{id}/activate - Activate user account
  - [ ] POST /admin/users/{id}/premium-bypass - Toggle bypass flag
  - [ ] Return updated user row template (Twinspark partial)

- [ ] Task 8: Update authentication to check suspension (AC: 5)
  - [ ] Modify login command to check is_suspended flag
  - [ ] Return error if user is suspended ("Account suspended. Contact support.")
  - [ ] Middleware checks suspension status on protected routes
  - [ ] Suspended users automatically logged out

- [ ] Task 9: Write comprehensive tests (AC: 8)
  - [ ] Create tests/admin_test.rs
  - [ ] Test: Admin can view user list
  - [ ] Test: Admin can suspend user
  - [ ] Test: Admin can activate suspended user
  - [ ] Test: Suspended user cannot log in
  - [ ] Test: Admin can toggle premium bypass flag
  - [ ] Test: Non-admin user blocked from admin routes (403)
  - [ ] Test: Suspended user's shared recipes hidden from community

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md):

**Admin Authorization Pattern:**
- Middleware extracts JWT and checks `is_admin` claim
- Admin routes protected at router level:
```rust
let admin_routes = Router::new()
    .route("/admin/users", get(admin::users::list))
    .route("/admin/users/{id}/suspend", post(admin::users::suspend))
    .layer(middleware::from_fn(admin_middleware));
```

**User Suspension Flow:**
1. Admin clicks "Suspend" button → POST /admin/users/{id}/suspend
2. Command verifies requesting user is admin
3. Emits UserSuspended event
4. Query handler updates users.is_suspended = true
5. Recipe query handlers hide suspended user's shared recipes
6. Login attempts by suspended user rejected

**Database Schema Updates:**

Add to users table:
```sql
ALTER TABLE users ADD COLUMN is_suspended BOOLEAN NOT NULL DEFAULT 0;
```

Add to user_profiles table (already exists from Story 1.3):
```sql
-- premium_bypass column already defined in Story 1.3
```

### Admin Access Management

From [PRD: FR003](../PRD.md#functional-requirements):
- System shall support admin users identified by `is_admin` flag
- Admin panel provides user management capabilities

From [PRD: FR045-FR048](../PRD.md#functional-requirements):
- Admin can view, edit, suspend/activate accounts, manage premium bypass flags
- Suspended users cannot log in
- Suspended users' shared recipes hidden from community

**First Admin Setup:**
- Option 1: Manual database update after first user registration
- Option 2: Configuration flag: `first_user_is_admin = true` in config/default.toml
- Option 3: CLI command: `cargo run -- create-admin user@example.com`

### Recipe Visibility Rules (AC: 6)

Community recipe query must filter:
```sql
SELECT * FROM recipes
WHERE is_shared = 1
  AND owner_id NOT IN (SELECT id FROM users WHERE is_suspended = 1)
```

From [epics.md](../epics.md):
- Story 1.4 affects Story 2.1 (recipe creation) and Story 5.2 (community browse)
- Suspended users' recipes hidden immediately (no grace period)

### References

- [PRD: Admin Panel Requirements](../PRD.md#admin-panel) - FR045-FR048
- [Architecture: Route Protection](../architecture.md#security-architecture) - Middleware patterns
- [CLAUDE.md: Axum Guidelines](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#axum-guidelines) - Route parameter format
- [Mockups: admin-users.html](../../mockups/admin-users.html) - Visual reference for admin panel

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
