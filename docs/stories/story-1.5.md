# Story 1.5: Profile Editing

Status: Approved

## Story

As a registered user,
I want to update my profile preferences,
so that meal planning reflects my current needs.

## Acceptance Criteria

1. Profile page displays current preferences in editable form
2. User can modify dietary restrictions, household size, skill level, availability
3. Changes validated before saving
4. Successful save updates profile and shows confirmation message
5. Updated preferences immediately affect future meal plan generations
6. Active meal plans remain unchanged until regenerated
7. Profile change history tracked for audit purposes

## Tasks / Subtasks

- [ ] Create profile edit page template (AC: 1, 2)
  - [ ] Create GET /profile handler in src/routes/profile.rs
  - [ ] Query user by ID from Auth middleware claims
  - [ ] Create ProfilePageTemplate with Askama
  - [ ] Pre-populate form fields with current user profile data
  - [ ] Dietary restrictions as checkboxes (pre-checked from user.dietary_restrictions JSON)
  - [ ] Household size as number input (value from user.household_size)
  - [ ] Skill level as radio buttons (selected from user.skill_level)
  - [ ] Weeknight availability as time picker + duration slider (parsed from user.weeknight_availability JSON)
  - [ ] Style with Tailwind CSS utility classes
  - [ ] Add "Save Changes" button

- [ ] Implement PUT /profile handler (AC: 2, 3, 4)
  - [ ] Create UpdateProfileForm struct with validator derives
  - [ ] Parse dietary_restrictions from comma-separated string to Vec<String>
  - [ ] Validate household_size range (1-20 per validator)
  - [ ] Parse skill_level string to SkillLevel enum
  - [ ] Parse weeknight_availability as JSON string
  - [ ] Create UpdateProfileCommand with form data
  - [ ] Call user::update_profile command handler (crates/user)
  - [ ] Handle validation errors (re-render form with inline error messages)
  - [ ] On success: redirect to /profile?updated=true
  - [ ] Display success toast notification on redirect

- [ ] Implement domain command and event handling (AC: 3, 4, 7)
  - [ ] Implement update_profile command in crates/user/src/commands.rs
  - [ ] Load UserAggregate from evento stream
  - [ ] Validate command (form validation already done in route)
  - [ ] Append ProfileUpdated event with changed fields only
  - [ ] Commit event to evento executor
  - [ ] Add profile_updated event handler to UserAggregate
  - [ ] Update aggregate state with new profile fields (COALESCE logic for optional updates)
  - [ ] ProfileUpdated event includes timestamp for audit trail

- [ ] Add read model projection (AC: 4, 7)
  - [ ] Create project_profile_updated handler in crates/user/src/read_model.rs
  - [ ] Subscribe to ProfileUpdated events via evento::handler
  - [ ] Parse dietary_restrictions Vec to JSON string for storage
  - [ ] Map SkillLevel enum to string ("beginner"|"intermediate"|"expert")
  - [ ] Update users table: SET dietary_restrictions, household_size, skill_level, weeknight_availability, updated_at WHERE id = ?
  - [ ] Use COALESCE to only update non-null fields
  - [ ] Update updated_at timestamp to track change history

- [ ] Test profile editing flow (AC: 1, 2, 3, 4, 5, 6, 7)
  - [ ] Unit test: ProfileUpdated event handler updates aggregate state correctly
  - [ ] Unit test: update_profile command validates input and emits ProfileUpdated event
  - [ ] Integration test: GET /profile renders pre-populated form with user data
  - [ ] Integration test: PUT /profile with valid changes updates users table via projection
  - [ ] Integration test: PUT /profile with household_size > 20 returns 422 with validation error
  - [ ] Integration test: PUT /profile with invalid skill_level returns 422
  - [ ] Integration test: Profile changes don't affect active meal plans (query meal_plans table)
  - [ ] E2E test: Complete user flow - register, onboard, edit profile, verify changes persist

## Dev Notes

### Architecture Patterns

**CQRS Implementation**:
- **Command**: `UpdateProfileCommand` modifies UserAggregate by appending ProfileUpdated event
- **Query**: GET /profile reads from users table read model (fast lookup by indexed user_id)
- **Event Sourcing**: All profile changes recorded as ProfileUpdated events in evento stream
- **Read Model Projection**: evento subscription updates users table asynchronously (<100ms lag)

**Validation Strategy**:
- **Client-side**: HTML5 validation attributes (required, min, max, pattern) for immediate feedback
- **Server-side**: validator crate on UpdateProfileForm enforces constraints before domain command
- **Domain-level**: Minimal validation in command handler (structural checks already done in route)

**Partial Updates**:
- Form allows updating individual fields (dietary restrictions only, household size only, etc.)
- ProfileUpdated event includes only changed fields (Option<T> for each field)
- Read model projection uses COALESCE to update only non-null values
- Aggregate event handler preserves existing values for null fields

### Source Tree Components

**Route Handlers** (src/routes/profile.rs):
- `GET /profile`: Query user from read model, render ProfilePageTemplate
- `PUT /profile`: Validate form, invoke update_profile command, redirect

**Domain Crate** (crates/user/):
- commands.rs: `update_profile(cmd, executor)` - Load aggregate, append ProfileUpdated event
- aggregate.rs: `profile_updated(&mut self, event)` - Apply event to aggregate state
- events.rs: `ProfileUpdated { dietary_restrictions, household_size, skill_level, weeknight_availability }`
- read_model.rs: `project_profile_updated(context, event)` - Update users table

**Templates** (templates/pages/profile.html):
- Extends base.html
- Pre-populated form with {% if %} checks for existing values
- Error display blocks for validation feedback
- Success notification on ?updated=true query param

**Database** (users table):
- dietary_restrictions: TEXT (JSON array)
- household_size: INTEGER
- skill_level: TEXT ("beginner"|"intermediate"|"expert")
- weeknight_availability: TEXT (JSON time range)
- updated_at: TEXT (ISO 8601 timestamp)

### Testing Standards

**Unit Tests** (crates/user/tests/aggregate_tests.rs):
- Test ProfileUpdated event handler with partial updates (e.g., only household_size changed)
- Verify COALESCE behavior: unchanged fields retain original values

**Integration Tests** (tests/profile_tests.rs):
- Spin up in-memory SQLite, run migrations
- Create test user with onboarded profile
- PUT /profile with various field combinations
- Verify users table updated correctly
- Test validation errors (household_size = 25, invalid skill_level)

**E2E Tests** (e2e/tests/profile.spec.ts):
- Register user → Onboard → Navigate to /profile → Change dietary restrictions → Save → Reload page → Verify changes persist

### References

**Architecture**:
- [Source: docs/solution-architecture.md#Section 3.2: Data Models] - users table schema with profile fields
- [Source: docs/solution-architecture.md#Section 3.3: Data Migrations] - SQLx migrations for read models
- [Source: docs/solution-architecture.md#Section 6.1: Server State] - User aggregate event sourcing patterns

**Epic Specification**:
- [Source: docs/epics.md#Story 1.5] - Original story definition and acceptance criteria
- [Source: docs/tech-spec-epic-1.md#APIs/PUT /profile] - Detailed route implementation spec
- [Source: docs/tech-spec-epic-1.md#AC-7.1 to AC-7.5] - Authoritative acceptance criteria with examples

**Domain Events**:
- [Source: docs/tech-spec-epic-1.md#Events/ProfileUpdated] - Event struct definition and field descriptions
- [Source: docs/tech-spec-epic-1.md#Read Model Projections/project_profile_updated] - Projection implementation code example

### Project Structure Notes

**Alignment with unified-project-structure.md**:
- Route handlers in src/routes/profile.rs (GET /profile, PUT /profile)
- Domain logic in crates/user/ (commands, events, aggregate)
- Templates in templates/pages/profile.html
- Tests in crates/user/tests/ (unit) and tests/profile_tests.rs (integration)

**No detected conflicts**. Structure follows established patterns from stories 1.1-1.4.

**Rationale for structure**:
- Profile management naturally belongs in user domain crate
- GET/PUT routes separate from auth routes (logical grouping)
- Read model projections colocated with domain events for maintainability

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.5.xml`
- Generated: 2025-10-13T14:14:35-04:00
- Epic ID: 1, Story ID: 5

### Agent Model Used

<!-- Model version will be recorded here -->

### Debug Log References

<!-- Links to debug logs will be added during implementation -->

### Completion Notes List

<!-- Implementation notes will be added as story progresses -->

### File List

<!-- Files created/modified will be tracked here -->
