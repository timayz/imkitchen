# Story 1.4: User Profile Creation (Onboarding)

Status: Approved

## Story

As a newly registered user,
I want to complete my profile with dietary and cooking preferences,
so that the meal planning algorithm can personalize recommendations.

## Acceptance Criteria

1. Onboarding wizard displays after first registration
2. Step 1: Dietary restrictions (checkboxes: vegetarian, vegan, gluten-free, allergens with text input)
3. Step 2: Household size (numeric input, 1-10)
4. Step 3: Cooking skill level (radio: beginner, intermediate, expert)
5. Step 4: Typical weeknight availability (time range picker, duration slider)
6. Each step validates inputs before allowing progression
7. User can skip onboarding (optional) - defaults applied
8. Completed profile stored and accessible for editing later
9. Profile data feeds meal planning optimization algorithm

## Tasks / Subtasks

- [ ] Create onboarding wizard templates (AC: 1, 2, 3, 4, 5, 7)
  - [ ] Create `templates/pages/onboarding.html` multi-step form with Askama
  - [ ] Step 1: Dietary restrictions checkboxes (vegetarian, vegan, gluten-free) + allergens text field
  - [ ] Step 2: Household size input (number, min=1, max=10)
  - [ ] Step 3: Skill level radio buttons (beginner, intermediate, expert)
  - [ ] Step 4: Weeknight availability time picker + duration slider
  - [ ] Add "Skip for now" link on each step
  - [ ] Style with Tailwind CSS utility classes
  - [ ] Add TwinSpark attributes for step progression without full page reload

- [ ] Implement POST /onboarding handler (AC: 6, 7, 8, 9)
  - [ ] Create OnboardingForm struct with all fields
  - [ ] Add validator derives for household_size (range 1-10)
  - [ ] Parse dietary restrictions array from form
  - [ ] Parse weeknight_availability as JSON time range
  - [ ] Create UpdateProfileCommand with collected data
  - [ ] Apply defaults for skipped fields (household_size=2, skill_level=intermediate, availability=18:00/45min)
  - [ ] Emit ProfileCompleted event via evento
  - [ ] Redirect to /dashboard after completion

- [ ] Add domain events and aggregate updates (AC: 8, 9)
  - [ ] Create ProfileCompleted event in `crates/user/src/events.rs`
  - [ ] Add profile_completed event handler to UserAggregate
  - [ ] Update aggregate state with profile fields
  - [ ] Add read model projection for ProfileCompleted event
  - [ ] Update users table with profile data

- [ ] Add GET /onboarding route (AC: 1)
  - [ ] Create route handler in `src/routes/profile.rs`
  - [ ] Check if user already completed onboarding (redirect to dashboard if true)
  - [ ] Render onboarding wizard template

- [ ] Integrate onboarding into registration flow (AC: 1)
  - [ ] Modify POST /register handler to redirect to /onboarding instead of /dashboard
  - [ ] Add onboarding_completed flag to users table
  - [ ] Update read model projection to track onboarding status

- [ ] Add comprehensive tests (AC: 1-9)
  - [ ] Integration test: GET /onboarding renders wizard for new user
  - [ ] Integration test: GET /onboarding redirects if already completed
  - [ ] Integration test: POST /onboarding with valid data creates ProfileCompleted event
  - [ ] Integration test: POST /onboarding applies defaults for skipped fields
  - [ ] Integration test: POST /onboarding validates household_size range (1-10)
  - [ ] Integration test: Profile data available in users read model after completion
  - [ ] Unit test: Default values applied correctly (household_size=2, etc.)

- [ ] Validation and defaults (AC: 6, 7)
  - [ ] Validate household_size between 1-10
  - [ ] Apply default household_size=2 if skipped
  - [ ] Apply default skill_level="intermediate" if skipped
  - [ ] Apply default availability={"start": "18:00", "duration_minutes": 45} if skipped
  - [ ] Ensure dietary_restrictions defaults to empty array [] if skipped

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing Pattern** (from solution-architecture.md):
- `ProfileCompleted` event records user profile setup
- Event sourcing maintains audit trail of all profile state changes
- UserAggregate rebuilds state from event stream

**CQRS Pattern**:
- Commands: `CompleteProfileCommand` (or extend `UpdateProfileCommand`)
- Queries: `query_user_by_id` from read model
- Read model updated via evento subscriptions

**Server-Side Rendering** (from solution-architecture.md):
- Askama templates: `onboarding.html` multi-step wizard
- TwinSpark progressive enhancement for step navigation
- Traditional POST/Redirect/Get pattern for final submission

**Validation** (from tech-spec-epic-1.md):
- validator crate for household_size range (1-10)
- Client-side HTML5 validation for numeric inputs
- Server-side validation enforced before event emission

### Source Tree Components to Touch

**Root Binary Routes** (`src/routes/profile.rs`):
- Add `GET /onboarding` handler
- Add `POST /onboarding` handler

**Root Binary Routes** (`src/routes/auth.rs`):
- Modify `POST /register` to redirect to `/onboarding` instead of `/dashboard`

**User Domain Crate** (`crates/user/`):
- `commands.rs`: Add `CompleteProfileCommand` (or extend UpdateProfileCommand)
- `events.rs`: Add `ProfileCompleted` event
- `aggregate.rs`: Add event handler for profile_completed
- `read_model.rs`: Add projection for ProfileCompleted event
- `error.rs`: Add validation errors for profile fields

**Templates** (`templates/pages/`):
- `onboarding.html`: Multi-step wizard with 4 steps

**Database**:
- Add `onboarding_completed` BOOLEAN column to `users` table (migration)

**Tests** (`tests/`):
- `profile_integration_tests.rs`: Add onboarding flow tests (7+ tests)

### Project Structure Notes

**Alignment with unified project structure**:
- Routes follow RESTful pattern: `GET /onboarding`, `POST /onboarding`
- Templates follow naming convention: `onboarding.html`
- Domain crate structure: events, commands, aggregate handlers
- Integration tests in root `tests/` directory

**Multi-Step Form Implementation**:
- Option 1: Single-page wizard with JavaScript/TwinSpark step visibility toggling (preferred)
- Option 2: Multi-page flow with state stored in session
- **Recommendation**: Single-page wizard with TwinSpark for progressive enhancement

**Default Values** (from epics.md):
- household_size=2
- skill_level="intermediate"
- weeknight_availability={"start": "18:00", "duration_minutes": 45}
- dietary_restrictions=[] (empty array)

### Testing Standards Summary

**TDD Approach** (per architecture requirements):
1. Write tests first for each handler and domain command
2. Implement handlers to pass tests
3. Refactor while maintaining passing tests

**Test Coverage Goals** (per NFRs):
- 80% code coverage minimum
- Integration tests for all AC (9 acceptance criteria → 7+ tests)
- Unit tests for default value logic
- Validation tests for household_size range

**Test Structure**:
- Use existing `tests/common/mod.rs` test harness
- Add tests to new `tests/profile_integration_tests.rs`
- Mock evento executor for aggregate tests

### References

- [Source: docs/solution-architecture.md#Section 2.1] - Event-Sourced Architecture
- [Source: docs/solution-architecture.md#Section 3.2] - Data Models (UserAggregate)
- [Source: docs/tech-spec-epic-1.md#Section: Commands] - UpdateProfileCommand structure
- [Source: docs/tech-spec-epic-1.md#Section: Events] - ProfileUpdated event pattern
- [Source: docs/epics.md#Story 1.4] - Acceptance criteria and defaults
- [Source: docs/stories/story-1.1.md] - Registration flow (redirect pattern)
- [Source: docs/stories/story-1.3.md] - Form handling with validation

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-13 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-1.md |
| 2025-10-13 | Bob (SM) | Generated story context XML with documentation and code artifacts; Status updated to Approved |
| 2025-10-13 | Bob (SM) | Fixed skill level terminology: "advanced" → "expert" to align with solution-architecture.md and tech-spec-epic-1.md |

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.4.xml` - Generated 2025-10-13 - Story context with documentation, code artifacts, interfaces, constraints, and testing standards

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
