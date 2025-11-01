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

- [ ] Task 1: Define UserProfileUpdated event and aggregate handler (AC: 1)
  - [ ] Add UserProfileUpdated event to crates/imkitchen-user/src/event.rs
  - [ ] Event fields: dietary_restrictions (Vec<String>), cuisine_variety_weight (f32), household_size (i32)
  - [ ] Update User aggregate to handle UserProfileUpdated event
  - [ ] Store profile data in aggregate state

- [ ] Task 2: Implement profile update command (AC: 2)
  - [ ] Create UpdateProfileInput struct with all profile fields
  - [ ] Add validation: cuisine_variety_weight between 0.0 and 1.0, household_size > 0
  - [ ] Implement update_profile command using evento::save pattern
  - [ ] Command emits UserProfileUpdated event with metadata
  - [ ] Handle case where user_id from JWT doesn't match input

- [ ] Task 3: Create user_profiles projection table and migration (AC: 4)
  - [ ] Create migration: migrations/queries/20250101000001_user_profiles.sql
  - [ ] Table fields: user_id (PK, FK to users), dietary_restrictions (TEXT as JSON), cuisine_variety_weight (REAL), household_size (INTEGER), is_premium_active (BOOLEAN), premium_bypass (BOOLEAN)
  - [ ] Add indexes on user_id
  - [ ] Document JSON format for dietary_restrictions field

- [ ] Task 4: Implement query handler for UserProfileUpdated (AC: 4)
  - [ ] Create on_user_profile_updated handler in src/queries/users.rs
  - [ ] Handler inserts or updates user_profiles table
  - [ ] Serialize dietary_restrictions as JSON array
  - [ ] Handle NULL values for optional fields
  - [ ] Add handler to subscription builder

- [ ] Task 5: Create profile query function (AC: 5)
  - [ ] Implement get_user_profile function in src/queries/users.rs
  - [ ] Returns UserProfile struct with all fields deserialized
  - [ ] Handle case where profile doesn't exist (return defaults)
  - [ ] Add test to verify query retrieval

- [ ] Task 6: Create profile page templates (AC: 3, 6)
  - [ ] Create templates/pages/auth/profile.html with Askama template
  - [ ] Display current dietary restrictions as checkboxes/tags
  - [ ] Cuisine variety weight slider (0.0 to 1.0, default 0.7)
  - [ ] Household size number input
  - [ ] Style with Tailwind CSS
  - [ ] Twinspark form submission: ts-req="/auth/profile" ts-req-method="POST" ts-target="#profile-form"

- [ ] Task 7: Implement profile route handlers (AC: 3, 6)
  - [ ] Create src/routes/auth/profile.rs with GET and POST handlers
  - [ ] GET /auth/profile loads current profile from query DB
  - [ ] POST /auth/profile executes update_profile command
  - [ ] Return updated profile template with success message
  - [ ] Handle validation errors with error display in form

- [ ] Task 8: Write comprehensive tests (AC: 7)
  - [ ] Add to tests/auth_test.rs or create tests/profile_test.rs
  - [ ] Test: User can update dietary restrictions
  - [ ] Test: Cuisine variety weight validation (0.0-1.0)
  - [ ] Test: Household size validation (> 0)
  - [ ] Test: Profile query returns updated data
  - [ ] Test: Profile defaults when user has no profile
  - [ ] Test: Multiple updates preserve latest state

## Dev Notes

### Architecture Patterns

From [architecture.md](../architecture.md):

**Profile Data Model:**

user_profiles table:
```sql
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY,
    dietary_restrictions TEXT,  -- JSON array: ["gluten-free", "vegan"]
    cuisine_variety_weight REAL NOT NULL DEFAULT 0.7,
    household_size INTEGER,
    is_premium_active BOOLEAN NOT NULL DEFAULT 0,
    premium_bypass BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**Default Values:**
- cuisine_variety_weight: 0.7 (balances variety and repetition)
- dietary_restrictions: empty array []
- household_size: null (optional field)
- is_premium_active: false
- premium_bypass: false

### User Story Linkage

From [epics.md](../epics.md):
- Story 1.3 provides profile data consumed by Story 3.5 (Dietary Restriction Filtering) and Story 3.6 (Cuisine Variety Scheduling)
- Profile preferences directly influence meal plan generation algorithm behavior

### Twinspark Form Pattern

Optimistic UI update with Twinspark:
```html
<form ts-req="/auth/profile"
      ts-req-method="POST"
      ts-target="#profile-form"
      ts-req-before="class+ opacity-50"
      ts-req-after="class- opacity-50">
  <!-- Form fields -->
</form>
```

Server returns updated profile template fragment that replaces #profile-form

From [CLAUDE.md](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#server-side-rendering):
- Always define ts-target when using ts-req
- Always render HTML with status 200 (no REST API patterns)
- Use Twinspark for UI reactivity

### References

- [PRD: FR002 User Preferences](../PRD.md#functional-requirements) - Dietary restrictions, cuisine variety weight, household size
- [Architecture: user_profiles Table](../architecture.md#core-tables-read-db) - Table schema
- [CLAUDE.md: Query Guidelines](/home/snapiz/projects/github/timayz/imkitchen/CLAUDE.md#query-guidelines) - Query handler idempotency
- [Mockups: profile.html](../../mockups/profile.html) - Visual reference for profile page

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
