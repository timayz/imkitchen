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

- [ ] Task 1: Add admin flag to User aggregate (AC: 1)
  - [ ] Add is_admin field to UserRegistered event (defaults to false)
  - [ ] Add UserMadeAdmin event to crates/imkitchen-user/src/event.rs
  - [ ] Implement user_made_admin handler in aggregate.rs
  - [ ] Update User aggregate to track is_admin state
  - [ ] Add is_admin to JWT claims struct (used for authorization)

- [ ] Task 2: Update user projection with admin flag (AC: 1)
  - [ ] Modify migrations/queries/TIMESTAMP_users.sql to include is_admin column
  - [ ] Update on_user_registration_succeeded handler to set is_admin=false by default
  - [ ] Create query handler on_user_made_admin to update projection
  - [ ] Update JWT generation to include is_admin from projection

- [ ] Task 3: Define admin management events (AC: 4, 7)
  - [ ] Add UserSuspended event with reason field (optional)
  - [ ] Add UserActivated event (reactivation after suspension)
  - [ ] Add UserPremiumBypassToggled event with enabled (boolean)
  - [ ] Implement event handlers in aggregate.rs for suspension/activation state
  - [ ] Aggregate tracks is_suspended and premium_bypass state

- [ ] Task 4: Implement admin management commands (AC: 4, 7)
  - [ ] Create suspend_user command accepting user_id and optional reason
  - [ ] Create activate_user command accepting user_id
  - [ ] Create toggle_premium_bypass command accepting user_id and enabled flag
  - [ ] Commands use evento::save pattern with target user's aggregator_id
  - [ ] Only admins can call these commands (enforced at route level)

- [ ] Task 5: Update projections for suspension and premium bypass (AC: 4, 5, 6, 7)
  - [ ] Add is_suspended and premium_bypass columns to users table (migration)
  - [ ] Create query handlers: on_user_suspended, on_user_activated, on_user_premium_bypass_toggled
  - [ ] Handlers update users projection with new state
  - [ ] Suspended users: is_suspended=true prevents login
  - [ ] Premium bypass: premium_bypass=true grants premium access

- [ ] Task 6: Create admin-only middleware (AC: 2)
  - [ ] Create src/middleware/admin.rs
  - [ ] Middleware checks JWT claims for is_admin=true
  - [ ] If not admin, return 403 Forbidden or redirect to /auth/login
  - [ ] Apply middleware to all /admin/* routes

- [ ] Task 7: Implement admin user list route (AC: 3)
  - [ ] Create src/routes/admin/users.rs
  - [ ] GET /admin/users renders user list template
  - [ ] Query all users from projection with pagination (20 per page)
  - [ ] Display: email, is_admin, is_suspended, premium_bypass, created_at
  - [ ] Include search/filter by email or status
  - [ ] Add pagination controls (prev/next page)

- [ ] Task 8: Implement admin action routes (AC: 4, 7)
  - [ ] POST /admin/users/{id}/suspend calls suspend_user command
  - [ ] POST /admin/users/{id}/activate calls activate_user command
  - [ ] POST /admin/users/{id}/premium-bypass toggles bypass flag
  - [ ] Actions return updated user list partial (Twinspark pattern)
  - [ ] Success/error messages displayed inline

- [ ] Task 9: Create admin panel templates (AC: 3)
  - [ ] Create templates/pages/admin/users.html
  - [ ] User table with columns: email, status badges, action buttons
  - [ ] Action buttons: Suspend/Activate, Toggle Premium Bypass
  - [ ] Suspended users shown with red badge, active users green badge
  - [ ] Premium bypass shown with purple badge
  - [ ] Use Tailwind CSS for styling

- [ ] Task 10: Enforce suspension in authentication (AC: 5)
  - [ ] Update login command to check is_suspended flag from projection
  - [ ] If user is suspended, return "Account suspended" error (don't emit login event)
  - [ ] Suspended users cannot receive JWT tokens
  - [ ] Existing JWT tokens still valid until expiration (acceptable for MVP)

- [ ] Task 11: Hide suspended users' shared recipes (AC: 6)
  - [ ] Modify community recipe query to filter out recipes where owner is_suspended=true
  - [ ] Note: Recipe aggregate implementation comes in Epic 2
  - [ ] Document requirement for Epic 2: JOIN recipes with users table on owner_id, filter is_suspended=false

- [ ] Task 12: Testing (AC: 8)
  - [ ] Create tests/admin_test.rs
  - [ ] Test: Admin can view user list
  - [ ] Test: Admin can suspend user account
  - [ ] Test: Suspended user cannot log in
  - [ ] Test: Admin can reactivate suspended user
  - [ ] Test: Reactivated user can log in again
  - [ ] Test: Admin can toggle premium bypass flag
  - [ ] Test: Non-admin cannot access /admin routes (403)
  - [ ] Use unsafe_oneshot for synchronous event processing

- [ ] Task 13: Code quality validation
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt --all
  - [ ] Verify all tests pass: cargo test
  - [ ] Manual test: Create admin user, suspend/activate user, verify login blocks

## Dev Notes

### Architecture Patterns

**Admin Authorization:**
- JWT claims include is_admin boolean
- Admin middleware checks is_admin=true before allowing access
- Non-admins receive 403 Forbidden on /admin routes
- Admin status set during registration or via separate UserMadeAdmin event

**Suspension Flow:**
1. Admin clicks "Suspend" → POST /admin/users/{id}/suspend
2. Command emits UserSuspended event
3. Query handler sets is_suspended=true in projection
4. Login attempts check is_suspended, reject if true
5. Existing sessions remain valid until token expiration (7 days)

**Premium Bypass:**
- Separate from actual premium subscription (Story 5.6)
- Used for demo accounts, testing, staging environments
- Stored in user_profiles.premium_bypass (boolean)
- Access control checks: global bypass OR user bypass OR active premium

**Shared Recipe Hiding:**
- Community queries JOIN recipes with users table
- Filter: WHERE users.is_suspended = false
- Suspended users' recipes immediately hidden (no event needed)

### Project Structure Notes

New directories and files added:
- `src/middleware/admin.rs` - Admin-only authorization middleware
- `src/routes/admin/users.rs` - User management panel routes
- `templates/pages/admin/users.html` - Admin user list view
- `tests/admin_test.rs` - Admin authorization and suspension tests

**Migration updates:**
- Add is_admin, is_suspended columns to users table
- Add premium_bypass column to user_profiles table (or users table)

**First admin user creation:**
- Document in CLAUDE.md or README: Manually set is_admin=true in DB for first admin
- Or provide CLI command: `cargo run -- make-admin <email>`

### References

- [Source: docs/epics.md#Story 1.4] - Complete acceptance criteria
- [Source: docs/architecture.md#Security Architecture] - JWT claims structure
- [Source: docs/PRD.md#Requirements FR045-FR048] - Admin panel functionality
- [Source: CLAUDE.md#Axum Guidelines] - Route parameter format {id}
- [Source: CLAUDE.md#Server-Side Rendering] - Twinspark partial responses

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
