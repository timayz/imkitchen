# Story 1.3: User Profile Management

Status: drafted

## Story

As a logged-in user,
I want to configure my dietary restrictions and preferences,
So that meal plan generation respects my dietary needs and cuisine preferences.

## Acceptance Criteria

1. UserProfileUpdated event stores dietary restrictions (array), complexity preferences, cuisine_variety_weight (default 0.7), household_size
2. Profile update command accepts input struct with validation
3. Profile page displays current preferences with edit form
4. Query handler projects profile data to user_profiles table
5. Profile data accessible via query function for meal planning algorithm
6. Twinspark form submission with optimistic UI update
7. Tests verify profile creation, update, and query retrieval

## Tasks / Subtasks

- [ ] Task 1: Define profile events in User aggregate (AC: 1)
  - [ ] Add UserProfileUpdated event to crates/imkitchen-user/src/event.rs
  - [ ] Event fields: dietary_restrictions (Vec<String>), cuisine_variety_weight (f32), household_size (Option<u32>), complexity_preferences (Option<String>)
  - [ ] Implement user_profile_updated handler in aggregate.rs
  - [ ] Aggregate tracks profile state (can be None initially)

- [ ] Task 2: Implement profile update command (AC: 2)
  - [ ] Define UpdateProfileInput struct with validation rules
  - [ ] Validate dietary_restrictions are from allowed list (gluten-free, vegan, vegetarian, dairy-free, nut-free, etc.)
  - [ ] Validate cuisine_variety_weight is between 0.0 and 1.0 (default 0.7)
  - [ ] Validate household_size is positive integer if provided
  - [ ] Implement update_profile command in Command struct
  - [ ] Use evento::save pattern (not create, since user already exists)
  - [ ] Command emits UserProfileUpdated event with input data

- [ ] Task 3: Create user_profiles projection table (AC: 4, 5)
  - [ ] Create migration: migrations/queries/TIMESTAMP_user_profiles.sql
  - [ ] Define user_profiles table: user_id (TEXT PK FK), dietary_restrictions (TEXT JSON array), cuisine_variety_weight (REAL DEFAULT 0.7), household_size (INTEGER), updated_at (INTEGER)
  - [ ] Create query handler on_user_profile_updated in src/queries/users.rs
  - [ ] Handler inserts or updates user_profiles row (UPSERT pattern)
  - [ ] Use event.timestamp for updated_at field
  - [ ] Create query function get_user_profile(pool, user_id) returning UserProfile struct

- [ ] Task 4: Create profile page routes (AC: 3)
  - [ ] Create src/routes/auth/profile.rs
  - [ ] GET /auth/profile renders profile form with current data (query get_user_profile)
  - [ ] Show default values if profile not yet created (cuisine_variety_weight=0.7)
  - [ ] POST /auth/profile submits UpdateProfileInput via Form extractor
  - [ ] Call update_profile command with user_id from JWT claims
  - [ ] Return success template or error message on validation failure
  - [ ] Add route to Axum router in server.rs

- [ ] Task 5: Create profile templates (AC: 3, 6)
  - [ ] Create templates/pages/auth/profile.html with Askama template
  - [ ] Form fields: dietary restrictions (checkboxes), cuisine variety slider (0-1), household size (number input)
  - [ ] Use Twinspark ts-req and ts-target for form submission
  - [ ] Display current values pre-populated in form
  - [ ] Success message displayed inline after update
  - [ ] Style with Tailwind CSS classes

- [ ] Task 6: Implement Twinspark reactivity (AC: 6)
  - [ ] Add ts-req="/auth/profile" to form element
  - [ ] Add ts-req-method="POST" for POST submission
  - [ ] Add ts-target="#profile-form" to replace form with response
  - [ ] Create partial template: templates/partials/auth/profile-success.html
  - [ ] Partial shows success message and updated form
  - [ ] Test form submission updates UI without full page reload

- [ ] Task 7: Testing (AC: 7)
  - [ ] Create tests/profile_test.rs
  - [ ] Test: Profile creation via command creates projection
  - [ ] Test: Profile update modifies existing projection (UPSERT)
  - [ ] Test: Query function retrieves profile data correctly
  - [ ] Test: Invalid dietary restriction rejected by validation
  - [ ] Test: cuisine_variety_weight outside 0-1 range rejected
  - [ ] Test: Profile page accessible only to authenticated users
  - [ ] Use unsafe_oneshot for synchronous event processing

- [ ] Task 8: Code quality validation
  - [ ] Run cargo clippy and fix all warnings
  - [ ] Run cargo fmt --all
  - [ ] Verify all tests pass: cargo test
  - [ ] Manual test: Update profile via browser, verify data persists

## Dev Notes

### Architecture Patterns

**Profile Update Flow:**
- User submits profile form → POST /auth/profile
- Handler calls update_profile command with user_id from JWT
- Command emits UserProfileUpdated event using evento::save
- Query handler updates user_profiles projection
- Twinspark replaces form with success message (no page reload)

**Default Values:**
- cuisine_variety_weight defaults to 0.7 (70% variety preference)
- dietary_restrictions defaults to empty array (no restrictions)
- household_size optional (null if not provided)

**Data Validation:**
- Dietary restrictions: predefined list (gluten-free, vegan, vegetarian, dairy-free, nut-free, shellfish-free, soy-free, egg-free)
- cuisine_variety_weight: 0.0 to 1.0 range (0=repeat frequently, 1=maximum variety)
- household_size: positive integer (1-20 realistic range)

**Twinspark Integration:**
- Form submission via ts-req (POST to /auth/profile)
- Response replaces form div with ts-target
- No full page reload needed for profile updates
- Optimistic UI: show "Saving..." state during request

### Project Structure Notes

New directories and files added:
- `src/routes/auth/profile.rs` - Profile management routes
- `templates/pages/auth/profile.html` - Profile edit form
- `templates/partials/auth/profile-success.html` - Success message partial
- `migrations/queries/TIMESTAMP_user_profiles.sql` - Profile projection table
- `tests/profile_test.rs` - Profile integration tests

**User aggregate state tracking:**
- Aggregate now tracks profile data for business logic (if needed in future commands)
- Profile preferences will be used by meal planning algorithm in Epic 3

### References

- [Source: docs/epics.md#Story 1.3] - Complete acceptance criteria
- [Source: docs/architecture.md#Command Pattern] - evento::save for existing aggregates
- [Source: docs/architecture.md#Query Pattern] - UPSERT pattern for projections
- [Source: docs/PRD.md#Requirements FR002] - Dietary restrictions and preferences storage
- [Source: CLAUDE.md#Server-Side Rendering] - Twinspark form submission patterns
- [Source: CLAUDE.md#Askama Guidelines] - Template syntax and filters

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
