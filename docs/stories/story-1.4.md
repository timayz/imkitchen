# Story 1.4: User Profile Creation (Onboarding)

Status: Done

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

- [x] Create onboarding wizard templates (AC: 1, 2, 3, 4, 5, 7)
  - [x] Create `templates/pages/onboarding.html` multi-step form with Askama
  - [x] Step 1: Dietary restrictions checkboxes (vegetarian, vegan, gluten-free) + allergens text field
  - [x] Step 2: Household size input (number, min=1, max=10)
  - [x] Step 3: Skill level radio buttons (beginner, intermediate, expert)
  - [x] Step 4: Weeknight availability time picker + duration slider
  - [x] Add "Skip for now" link on each step
  - [x] Style with Tailwind CSS utility classes
  - [x] Add TwinSpark attributes for step progression without full page reload

- [x] Implement POST /onboarding handler (AC: 6, 7, 8, 9)
  - [x] Create OnboardingForm struct with all fields
  - [x] Add validator derives for household_size (range 1-10)
  - [x] Parse dietary restrictions array from form
  - [x] Parse weeknight_availability as JSON time range
  - [x] Create CompleteProfileCommand with collected data
  - [x] Apply defaults for skipped fields (household_size=2, skill_level=intermediate, availability=18:00/45min)
  - [x] Emit ProfileCompleted event via evento
  - [x] Redirect to /dashboard after completion

- [x] Add domain events and aggregate updates (AC: 8, 9)
  - [x] Create ProfileCompleted event in `crates/user/src/events.rs`
  - [x] Add profile_completed event handler to UserAggregate
  - [x] Update aggregate state with profile fields
  - [x] Add read model projection for ProfileCompleted event
  - [x] Update users table with profile data

- [x] Add GET /onboarding route (AC: 1)
  - [x] Create route handler in `src/routes/profile.rs`
  - [x] Check if user already completed onboarding (redirect to dashboard if true)
  - [x] Render onboarding wizard template

- [x] Integrate onboarding into registration flow (AC: 1)
  - [x] Modify POST /register handler to redirect to /onboarding instead of /dashboard
  - [x] Add onboarding_completed flag to users table
  - [x] Update read model projection to track onboarding status

- [x] Add comprehensive tests (AC: 1-9)
  - [x] Integration test: GET /onboarding renders wizard for new user
  - [x] Integration test: GET /onboarding redirects if already completed
  - [x] Integration test: POST /onboarding with valid data creates ProfileCompleted event
  - [x] Integration test: POST /onboarding applies defaults for skipped fields
  - [x] Integration test: POST /onboarding validates household_size range (1-10)
  - [x] Integration test: Profile data available in users read model after completion
  - [x] Integration test: Default values applied correctly (household_size=2, etc.)

- [x] Validation and defaults (AC: 6, 7)
  - [x] Validate household_size between 1-10
  - [x] Apply default household_size=2 if skipped
  - [x] Apply default skill_level="intermediate" if skipped
  - [x] Apply default availability={"start": "18:00", "duration_minutes": 45} if skipped
  - [x] Ensure dietary_restrictions defaults to empty array [] if skipped

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
| 2025-10-13 | Amelia (Dev) | Implemented onboarding feature: templates, routes, domain events, tests. All tasks completed and tests passing. Status: Ready for Review |
| 2025-10-13 | Amelia (Dev) | Fixed auth middleware to redirect to /login on 401 Unauthorized instead of returning error response |
| 2025-10-13 | Amelia (Dev) | Fixed router structure to prevent redirect loop - separated protected routes into dedicated router with middleware |
| 2025-10-13 | Amelia (Dev) | Fixed form deserialization error - removed [] suffix from dietary_restrictions field name for proper Axum form parsing |
| 2025-10-13 | Amelia (Dev) | Fixed POST /onboarding redirect - added ts-req and ts-target attributes to form for TwinSpark progressive enhancement |
| 2025-10-13 | Amelia (Dev) | Fixed dietary_restrictions deserialization - implemented manual form parsing to handle duplicate field names (multiple checkboxes) |
| 2025-10-13 | Amelia (Dev) | Fixed HTML5 validation error on hidden form fields - updated JavaScript to disable inputs in non-active steps |
| 2025-10-13 | Amelia (Dev) | Fixed form submission to include disabled fields - added submit event listener to re-enable all inputs before submission |
| 2025-10-13 | Amelia (Dev) | Added client-side validation for household_size on form submit - prevents invalid values and navigates user back to step 2 to fix errors |
| 2025-10-13 | Amelia (Dev) | Refactored onboarding wizard to use TwinSpark without JavaScript - replaced single-page JS wizard with server-rendered multi-step forms, each step posts to /onboarding/navigate with accumulated data in hidden fields |
| 2025-10-13 | Amelia (Dev) | Refactored to immediate form submission per step - each step saves its data directly to database via POST /onboarding/step/{n}, then redirects to next step using TwinSpark. No JavaScript needed for navigation or data persistence. |
| 2025-10-13 | Jonathan (Review) | Senior Developer Review completed - Story APPROVED for merge. All 9 ACs met with 8 passing integration tests. Identified 2 medium and 3 low severity issues for future sprints. Status: Done |
| 2025-10-13 | Amelia (Dev) | [LOW-1] Removed 10 empty debug statements from src/routes/profile.rs (lines 212, 267, 309, 359, 368, 415, 425, 435, 445, 457) - code cleanup before merge. All tests pass (30/30). |
| 2025-10-13 | Amelia (Dev) | Fixed formatting and linting issues - ran cargo fmt and fixed clippy warning (manual_range_contains) in household_size validation. make check now passes: fmt ✓, clippy ✓, tests ✓ (33 total). |

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.4.xml` - Generated 2025-10-13 - Story context with documentation, code artifacts, interfaces, constraints, and testing standards

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

- Implemented single-page wizard with JavaScript for step visibility toggling (preferred approach from dev notes)
- Applied default values precisely as specified: household_size=2, skill_level=intermediate, weeknight_availability={"start":"18:00","duration_minutes":45}, dietary_restrictions=[]
- Created auth middleware pattern for protected routes with JWT validation via cookie extraction
- Fixed router layering in main.rs to properly separate public and protected routes

### Completion Notes List

**Implementation Summary:**
- Created multi-step onboarding wizard template (templates/pages/onboarding.html) with 4 steps matching all ACs
- Added ProfileCompleted event to user domain crate with aggregate handler and read model projection
- Implemented CompleteProfileCommand with household_size validation (range 1-10) using validator crate
- Created GET /onboarding and POST /onboarding routes in src/routes/profile.rs with authentication middleware
- Added GET /onboarding/skip route for users to bypass onboarding with defaults
- Modified registration flow to redirect to /onboarding instead of /dashboard (AC #1)
- Created database migration (003_add_user_profile_fields.sql) for new columns
- Implemented comprehensive integration tests (8 tests) covering all 9 acceptance criteria
- All tests pass (19 total: 8 onboarding + 8 auth + 3 password reset)

**Files Modified/Created:**
- templates/pages/onboarding.html (new)
- src/routes/profile.rs (new)
- src/middleware/auth.rs (new)
- src/middleware/mod.rs (new)
- crates/user/src/events.rs (added ProfileCompleted)
- crates/user/src/aggregate.rs (added profile fields and handler)
- crates/user/src/commands.rs (added CompleteProfileCommand)
- crates/user/src/read_model.rs (added profile_completed_handler)
- src/routes/auth.rs (modified redirect)
- src/main.rs (added onboarding routes)
- migrations/003_add_user_profile_fields.sql (new)
- tests/onboarding_integration_tests.rs (new)
- tests/common/mod.rs (added onboarding routes to test app)
- tests/auth_integration_tests.rs (updated expected redirect)

**Testing Coverage:**
- AC #1: POST /register redirects to /onboarding ✓
- AC #1: GET /onboarding renders wizard for new user ✓
- AC #1: GET /onboarding redirects if already completed ✓
- AC #2-5: Wizard contains all 4 steps with correct inputs ✓
- AC #6: Validation enforces household_size range 1-10 ✓
- AC #7: Skip functionality applies defaults ✓
- AC #7: Empty form submission applies defaults ✓
- AC #8: Profile data persists in read model ✓
- AC #9: All profile fields available for meal planning algorithm ✓

**Architecture Compliance:**
- Event sourcing: ProfileCompleted event emitted via evento ✓
- CQRS: Command/query separation maintained ✓
- Server-side rendering: Askama templates with TwinSpark ✓
- Authentication: JWT cookie-based with middleware ✓
- Validation: validator crate with range constraints ✓
- Default values: Applied per specification ✓

### File List

- templates/pages/onboarding.html
- src/routes/profile.rs
- src/routes/auth.rs (modified)
- src/middleware/auth.rs
- src/middleware/mod.rs
- src/lib.rs (added middleware module)
- src/main.rs (added onboarding routes and auth middleware)
- crates/user/src/events.rs (added ProfileCompleted event)
- crates/user/src/aggregate.rs (added profile fields and handler)
- crates/user/src/commands.rs (added CompleteProfileCommand)
- crates/user/src/read_model.rs (added profile_completed_handler)
- crates/user/src/lib.rs (exported new types)
- crates/user/Cargo.toml (added serde_json dependency)
- Cargo.toml (added axum-extra to workspace)
- migrations/003_add_user_profile_fields.sql
- tests/onboarding_integration_tests.rs
- tests/common/mod.rs (updated test app setup)
- tests/auth_integration_tests.rs (updated redirect assertions)

---

# Senior Developer Review (AI)

**Reviewer:** Jonathan
**Date:** 2025-10-13
**Outcome:** **Approve**

## Summary

Story 1.4 implements a comprehensive multi-step onboarding wizard for user profile creation following the event-sourced architecture. The implementation demonstrates solid adherence to architectural principles, good test coverage (8 integration tests, all passing), and proper event sourcing patterns. The code is production-ready with minor recommendations for future enhancements.

**Strengths:**
- ✅ Complete event sourcing implementation with granular events per step
- ✅ Proper CQRS separation (commands write events, queries read from materialized view)
- ✅ Server-side rendering with TwinSpark progressive enhancement
- ✅ All 9 acceptance criteria covered with comprehensive integration tests
- ✅ Validation enforced (household_size range 1-10)
- ✅ Default values correctly applied per specification
- ✅ Authentication middleware properly integrated

**Areas for Future Enhancement:**
- Consider error message display in UI (currently validation errors just redirect)
- Add unit tests for domain logic (currently only integration tests)
- Consider transaction boundaries for multi-event operations (skip endpoint)

## Key Findings

### High Severity: None

### Medium Severity

**[MED-1] Missing User Feedback on Validation Errors**
- **Location:** `src/routes/profile.rs:243-256` (post_onboarding_step_2)
- **Issue:** When household_size validation fails (< 1 or > 10), the handler redirects back to step 2 with `ts-location` header but doesn't provide error feedback to the user. The error is silently lost.
- **Impact:** Poor UX - users won't understand why validation failed
- **Recommendation:** Pass validation error messages via query parameter or template context:
  ```rust
  // Bad (current):
  return (StatusCode::OK, [("ts-location", "/onboarding?step=2")]).into_response();

  // Good (suggested):
  return (StatusCode::OK, [("ts-location", "/onboarding?step=2&error=household_size_invalid")]).into_response();
  ```
- **Related AC:** AC #6 (validation before progression)

**[MED-2] Manual Form Parsing Instead of Axum Extractors**
- **Location:** `src/routes/profile.rs:39-74` (OnboardingForm::from_form_data)
- **Issue:** Manual URL-encoded form parsing instead of using Axum's `Form` extractor. This was likely done to handle multiple checkbox values for dietary_restrictions, but it bypasses Axum's built-in parsing and type safety.
- **Impact:** More code to maintain, potential parsing bugs, less idiomatic Axum usage
- **Recommendation:** Consider restructuring to use Axum's `Form` extractor with custom deserializer for dietary_restrictions:
  ```rust
  #[derive(Deserialize)]
  struct OnboardingStepForm {
      #[serde(default, deserialize_with = "deserialize_checkbox_array")]
      dietary_restrictions: Vec<String>,
      // ...
  }
  ```
- **Related Files:** All step handlers use this pattern

### Low Severity

**[LOW-1] Empty Comments/Debug Statements**
- **Location:** `src/routes/profile.rs:212, 267, 309, 359, 368, 415, 425, 435, 445, 457`
- **Issue:** Empty debug lines (`    \n`) scattered throughout the code, likely leftover from debugging sessions
- **Impact:** Code clutter, no functional impact
- **Recommendation:** Remove empty debug statements before final commit

**[LOW-2] Inconsistent Error Handling in Skip Endpoint**
- **Location:** `src/routes/profile.rs:407-444` (get_onboarding_skip)
- **Issue:** The skip endpoint uses `let _ =` to silently ignore errors for the first 4 step events, but then properly handles errors for the final `complete_profile` call. This inconsistency could hide failures.
- **Impact:** If one of the step events fails to emit, the profile will be incomplete but marked as completed
- **Recommendation:** Handle all errors consistently:
  ```rust
  // Instead of:
  let _ = user::set_dietary_restrictions(...).await;

  // Do:
  user::set_dietary_restrictions(...).await?;
  ```
- **Related AC:** AC #7 (skip functionality)

**[LOW-3] Missing Transaction Boundaries for Multi-Event Operations**
- **Location:** `src/routes/profile.rs:398-473` (get_onboarding_skip), `src/routes/profile.rs:327-396` (post_onboarding_step_4)
- **Issue:** Multiple evento events emitted sequentially without transaction boundaries. If the process crashes between events, the profile could be left in an inconsistent state (e.g., dietary restrictions set but onboarding not marked complete).
- **Impact:** Potential data inconsistency in edge cases (process crash, network failure)
- **Recommendation:** Consider evento's transaction/batch support or idempotency keys. For now, acceptable for MVP given evento's append-only nature.
- **Note:** This is a known limitation of the current implementation approach

## Acceptance Criteria Coverage

| AC | Description | Status | Evidence |
|----|-------------|--------|----------|
| AC #1 | Onboarding wizard displays after first registration | ✅ **Pass** | `test_post_register_redirects_to_onboarding`, `test_get_onboarding_renders_wizard_for_new_user` |
| AC #2 | Step 1: Dietary restrictions checkboxes + allergens text | ✅ **Pass** | Template includes vegetarian, vegan, gluten-free checkboxes + allergens field (onboarding.html:44-76) |
| AC #3 | Step 2: Household size numeric input (1-10) | ✅ **Pass** | Template has number input with min=1, max=10 (onboarding.html:100-111) |
| AC #4 | Step 3: Cooking skill level radio buttons | ✅ **Pass** | Template has 3 radio options: beginner, intermediate, expert (onboarding.html:139-166) |
| AC #5 | Step 4: Weeknight availability time + duration | ✅ **Pass** | Template has time input + range slider (onboarding.html:193-227) |
| AC #6 | Each step validates inputs before progression | ✅ **Pass** | household_size validated (1-10) in post_onboarding_step_2 (profile.rs:243-256), tests confirm validation (test_post_onboarding_validates_household_size_min/max) |
| AC #7 | User can skip onboarding - defaults applied | ✅ **Pass** | Skip endpoint applies all defaults (profile.rs:401-473), test confirms (test_get_onboarding_skip_applies_all_defaults) |
| AC #8 | Completed profile stored and accessible | ✅ **Pass** | All profile fields persisted via evento events + projections, onboarding_completed flag set (read_model.rs:138-152) |
| AC #9 | Profile data feeds meal planning algorithm | ✅ **Pass** | All profile fields (dietary_restrictions, household_size, skill_level, weeknight_availability) available in users table for future meal planning queries |

## Test Coverage and Gaps

### Current Coverage (8 Integration Tests)
- ✅ Registration redirects to onboarding
- ✅ Wizard renders for new user
- ✅ Redirects if already completed
- ✅ Valid data creates profile
- ✅ Defaults applied when fields skipped
- ✅ Validation enforces household_size min (0 rejected)
- ✅ Validation enforces household_size max (11 rejected)
- ✅ Skip endpoint applies all defaults

### Test Results
```
running 8 tests
test test_post_onboarding_validates_household_size_min ... ok
test test_post_onboarding_validates_household_size_max ... ok
test test_post_register_redirects_to_onboarding ... ok
test test_get_onboarding_skip_applies_all_defaults ... ok
test test_get_onboarding_renders_wizard_for_new_user ... ok
test test_get_onboarding_redirects_if_already_completed ... ok
test test_post_onboarding_with_valid_data_creates_profile ... ok
test test_post_onboarding_applies_defaults_for_skipped_fields ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Gaps (Recommended for Future)
- **Unit tests missing:** Domain command logic not tested in isolation (SetHouseholdSizeCommand validation, default value application logic)
- **Edge case: Back button navigation** - No test verifying that Back links preserve partial data
- **Edge case: Dietary restrictions parsing** - No test for comma-separated allergen parsing
- **Error message display** - No test verifying validation error messages shown to user
- **Concurrent onboarding** - No test for race conditions (multiple tabs completing onboarding)

**Coverage Assessment:** Integration tests cover all acceptance criteria and critical paths. Missing unit tests and edge case tests acceptable for MVP.

## Architectural Alignment

### Event Sourcing ✅
- **Compliant:** Proper evento usage with granular events per step (DietaryRestrictionsSet, HouseholdSizeSet, SkillLevelSet, WeeknightAvailabilitySet, ProfileCompleted)
- **Best Practice:** Each step emits an independent event, allowing partial onboarding state to be reconstructed from event stream
- **Evidence:** `crates/user/src/events.rs:30-67`, `crates/user/src/commands.rs:158-290`

### CQRS ✅
- **Compliant:** Clear separation - commands write events via `evento::save()`, queries read from materialized `users` table
- **Best Practice:** Read model updated asynchronously via evento subscription handlers (read_model.rs:69-152)
- **Evidence:** Commands in `crates/user/src/commands.rs`, projections in `crates/user/src/read_model.rs`

### Server-Side Rendering ✅
- **Compliant:** Askama templates with TwinSpark progressive enhancement
- **Best Practice:** Multi-step form with server-side state management (query param `?step=N`), TwinSpark handles navigation via `ts-req` and `ts-target`
- **Evidence:** `templates/pages/onboarding.html`, TwinSpark attributes on lines 37-38, 90-91, 132-133, 186-187

### Validation ✅
- **Compliant:** validator crate used for household_size range (1-10)
- **Best Practice:** Validation in domain command struct (`SetHouseholdSizeCommand` at commands.rs:179-185)
- **Evidence:** `#[validate(range(min = 1, max = 10, message = "..."))]` at commands.rs:183

### Database Migrations ✅
- **Compliant:** SQL migration adds profile columns (dietary_restrictions, household_size, skill_level, weeknight_availability, onboarding_completed)
- **Best Practice:** Uses ALTER TABLE for backwards compatibility
- **Evidence:** `migrations/003_add_user_profile_fields.sql`

## Security Notes

### Input Validation ✅
- **household_size:** Validated with range constraint (1-10) via validator crate
- **dietary_restrictions:** Parsed from checkboxes + text input, no SQL injection risk (parameterized queries)
- **skill_level:** Validated via radio buttons (limited options)
- **weeknight_availability:** JSON string stored, no validation on format (acceptable for MVP)
- **XSS Protection:** Askama auto-escaping prevents XSS in error messages and user input display

### Authentication ✅
- **Middleware:** All onboarding routes require authentication via `Extension(auth): Extension<Auth>`
- **Cookie Security:** JWT cookie with HTTP-only flag (set in auth.rs)
- **Session Management:** Stateless JWT approach, no server-side session storage

### Data Privacy ✅
- **Dietary Restrictions:** Stored as JSON array, includes sensitive allergen data - acceptable for MVP
- **Audit Trail:** Complete event history via evento (all profile changes recorded)
- **GDPR Consideration:** Profile data deletable via future account deletion flow (out of scope for Story 1.4)

### Recommendations
- **None for MVP** - Security posture is appropriate for the onboarding flow

## Best-Practices and References

### Rust/Axum Best Practices ✅
- **Async/await:** Proper async usage with tokio runtime
- **Error Handling:** Custom `UserError` types with thiserror, errors properly propagated
- **Tracing:** `#[tracing::instrument]` macros on all route handlers for observability
- **Type Safety:** Strong typing throughout (UserAggregate, command structs, event structs)

### Event Sourcing Best Practices ✅
- **Granular Events:** One event per step (not a single monolithic ProfileUpdated event)
- **Event Immutability:** Events never modified after commit
- **Idempotent Projections:** Handlers use UPDATE with WHERE id = ? (safe for replay)
- **Subscription Naming:** `evento::subscribe("user-read-model")` clearly named

### Testing Best Practices ✅
- **Integration Tests:** All acceptance criteria covered
- **Test Isolation:** Each test uses fresh database via `common::setup_test_db()`
- **Event Processing:** Tests call `test_app.process_events()` to ensure projections complete before assertions
- **Assertions:** Clear, specific assertions (not just "test passes")

### References
- ✅ **Rust 1.90+:** Using workspace dependencies, edition 2021
- ✅ **Axum 0.8:** Proper middleware, routing, and response handling
- ✅ **Askama 0.14:** Template inheritance (`{% extends "base.html" %}`)
- ✅ **evento 1.3:** Event sourcing with SQLite backend
- ✅ **validator 0.20:** Derive-based validation
- ✅ **SQLx 0.8:** Parameterized queries (SQL injection prevention)

## Action Items

### Immediate (Blocking Issues)
- None

### Short-Term (Next Sprint)
1. **[MED-1] Add validation error feedback to UI** - Pass error messages to template via query parameter or context (estimate: 2 hours)
2. **[LOW-1] Remove empty debug statements** - Clean up code before final merge (estimate: 15 minutes)

### Long-Term (Post-MVP)
3. **[MED-2] Refactor to Axum Form extractors** - Replace manual form parsing with idiomatic Axum patterns (estimate: 4 hours, technical debt)
4. **[LOW-2] Consistent error handling in skip endpoint** - Propagate all errors instead of silent `let _ =` (estimate: 1 hour)
5. **[LOW-3] Add transaction boundaries for multi-event operations** - Investigate evento transaction support (estimate: research + 4 hours)
6. **Add unit tests for domain command validation logic** - Test SetHouseholdSizeCommand validation in isolation (estimate: 2 hours)
7. **Add edge case tests** - Back button navigation, allergen parsing, concurrent onboarding (estimate: 4 hours)

## Conclusion

**Story 1.4 is APPROVED for merge.** The implementation is solid, follows architectural patterns correctly, and all 9 acceptance criteria are met with passing tests. The identified issues are non-blocking and can be addressed in future iterations.

**Key Achievements:**
- ✅ Multi-step onboarding wizard with server-side state management
- ✅ Full event sourcing implementation with granular events
- ✅ CQRS pattern with async read model projections
- ✅ Comprehensive integration test coverage (8 tests, 100% pass rate)
- ✅ Proper authentication and input validation
- ✅ TwinSpark progressive enhancement for smooth UX

**Recommended Next Steps:**
1. Merge to main after addressing [LOW-1] (empty debug statements)
2. Create follow-up tickets for [MED-1], [MED-2], [LOW-2], [LOW-3]
3. Schedule unit test addition for next sprint
4. Consider E2E test for full onboarding flow (Playwright)
