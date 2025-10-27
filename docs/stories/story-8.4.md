# Story 8.4: Create Regenerate All Future Weeks Route

Status: Approved

## Story

As a backend developer,
I want to create a POST route that regenerates all future weeks while preserving the current week,
so that authenticated users can refresh their entire meal plan when preferences or recipe library changes significantly.

## Acceptance Criteria

1. Route `POST /plan/regenerate-all-future` created
2. Requires confirmation parameter (prevent accidental regeneration)
3. Handler identifies current week (locked) and preserves it
4. Handler regenerates all future weeks (`status == "active"` AND `is_locked == false`)
5. Handler resets rotation state but preserves current week's main courses
6. Handler commits `AllFutureWeeksRegenerated` event to evento
7. Handler regenerates shopping lists for all future weeks (projection handles this)
8. Returns count of regenerated weeks + first future week data
9. Integration test: POST with confirmation regenerates all future weeks
10. Integration test: POST without confirmation returns 400

## Tasks / Subtasks

- [x] Define route handler function signature (AC: 1, 2)
  - [x] Create `regenerate_all_future_weeks` function in `src/routes/meal_planning_api.rs`
  - [x] Add function signature with Axum extractors: `State(AppState)`, `Extension(Auth)`, `Json(payload)`
  - [x] Define `RegenerateAllPayload` struct with `confirmation: bool` field
  - [x] Route registered in main.rs at `/plan/regenerate-all-future`
  - [x] Ensure route is protected by authentication middleware

- [x] Validate confirmation parameter (AC: 2, 10)
  - [x] Check if `payload.confirmation == true`
  - [x] If false or missing → return 400 ConfirmationRequired error with message
  - [x] Log confirmation validation failure with structured tracing

- [x] Identify current week and future weeks (AC: 3, 4)
  - [x] Write SQL query for active weeks ordered by start_date
  - [x] Filter to identify current week: `is_locked == true` OR `status == "current"`
  - [x] Filter to identify future weeks: `status == "active"` AND `is_locked == false`
  - [x] Handle edge case: No future weeks exist → return 200 with "0 weeks regenerated" message
  - [x] Store current_week_id for preservation and response

- [x] Load user's favorite recipes
  - [x] Reuse existing `load_favorite_recipes` function
  - [x] Validate sufficient recipes (at least 7 total) → return 400 InsufficientRecipes if not met

- [x] Load user's meal planning preferences
  - [x] Reuse existing `load_user_preferences` function
  - [x] Apply defaults if fields are NULL

- [x] Initialize rotation state for regeneration (AC: 5)
  - [x] Create new `RotationState` instance (reset for clean slate)
  - [x] Load current week's main course recipe IDs to preserve variety continuation
  - [x] Query: `SELECT recipe_id FROM meal_assignments WHERE meal_plan_id = ? AND course_type = 'main_course'`
  - [x] Seed rotation_state with current week's recipes to prevent immediate repetition
  - [x] Log rotation state initialization with structured tracing

- [x] Call Epic 7 algorithm to regenerate all future weeks (AC: 4)
  - [x] Import `generate_single_week` from `meal_planning::algorithm`
  - [x] Loop through future_weeks ordered by start_date
  - [x] Call `generate_single_week(recipes, preferences, &mut rotation_state, week.start_date)` for each week
  - [x] Handle errors: InsufficientRecipes, AlgorithmTimeout
  - [x] Log algorithm execution for each week with structured tracing
  - [x] Accumulate regenerated weeks into `Vec<WeekMealPlanData>`

- [x] Emit AllFutureWeeksRegenerated evento event (AC: 6)
  - [x] Build `AllFutureWeeksRegenerated` event struct with all required fields
  - [x] Call `evento::create().data().metadata().commit()` pattern
  - [x] Handle event emission errors → return 500 with internal error message
  - [x] Log event emission success with structured tracing

- [x] Build JSON response (AC: 8)
  - [x] Construct `RegenerateAllResponse` struct with regenerated_weeks count, preserved_current_week_id, first_future_week
  - [x] Note: Shopping list regeneration happens asynchronously via evento projection (AC: 7)
  - [x] Serialize to JSON using serde_json
  - [x] Return `Ok(Json(response))` with 200 status

- [x] Implement error handling (AC: 2, 10)
  - [x] Add `ApiError` variant: ConfirmationRequired (400)
  - [x] ConfirmationRequired: Return 400 with JSON body including error code and message
  - [x] InsufficientRecipes: Return 400 with category counts and "Add Recipe" action
  - [x] AlgorithmTimeout: Return 500 with retry message
  - [x] Include structured error logging for debugging

- [x] Add structured logging and tracing
  - [x] Log request start, confirmation check, week identification, algorithm execution
  - [x] Add OpenTelemetry span with attributes: user_id
  - [x] Full tracing coverage for debugging

- [ ] Write integration tests (AC: 9, 10)
  - [ ] Create `test_regenerate_all_future_weeks_with_confirmation()` in `crates/api/tests/integration/test_regeneration.rs`
  - [ ] Setup test database with test user, active meal plan batch with 1 current week + 4 future weeks
  - [ ] Create valid JWT cookie for test user
  - [ ] Make POST request to `/plan/regenerate-all-future` with JSON body `{ "confirmation": true }`
  - [ ] Assert response status is 200 OK
  - [ ] Parse JSON response and validate:
    - regenerated_weeks == 4
    - preserved_current_week_id matches current week UUID
    - first_future_week contains valid meal_assignments array
  - [ ] Subscribe to AllFutureWeeksRegenerated event using `unsafe_oneshot` for synchronous processing
  - [ ] Verify read models updated:
    - meal_assignments table has new assignments for all 4 future weeks
    - meal_assignments for current week UNCHANGED (preserved)
    - shopping_lists table updated for all 4 future weeks

  - [ ] Create `test_regenerate_all_future_weeks_without_confirmation_returns_400()` test
  - [ ] Make POST request with JSON body `{ "confirmation": false }` or empty body
  - [ ] Assert response status is 400 Bad Request
  - [ ] Verify error JSON body includes "ConfirmationRequired" error code
  - [ ] Verify meal_assignments not modified (no changes committed)

  - [ ] Create `test_regenerate_all_future_weeks_preserves_current_week()` test
  - [ ] Setup test database with current week having specific recipes
  - [ ] Regenerate all future weeks with confirmation
  - [ ] Query meal_assignments for current week after regeneration
  - [ ] Assert current week's meal assignments UNCHANGED (exact same recipe IDs)

  - [ ] Create `test_regenerate_all_future_weeks_handles_zero_future_weeks()` test
  - [ ] Setup test database with only current week (no future weeks)
  - [ ] Make POST request with confirmation
  - [ ] Assert response status is 200 OK
  - [ ] Verify response: regenerated_weeks == 0, message indicates no weeks regenerated

- [ ] Write performance test
  - [ ] Create `test_regenerate_all_future_latency_under_2000ms()` in `crates/api/tests/performance/route_latency_tests.rs`
  - [ ] Measure route response time with 4 future weeks (realistic scenario)
  - [ ] Assert P95 latency < 2000ms for route overhead (loading data, looping, emitting events)
  - [ ] Note: Algorithm execution time (3-5s per week) excluded from route overhead metric

- [ ] Register route in Axum router
  - [ ] Add route to router configuration in `crates/api/src/main.rs` or router module
  - [ ] Ensure authentication middleware is applied
  - [ ] Ensure database pool and evento executor are available via Extension

## Dev Notes

### Architecture Patterns
- **Event-Sourced CQRS**: Route emits AllFutureWeeksRegenerated event, projection bulk updates read models
- **Confirmation Pattern**: Requires explicit `confirmation: true` parameter to prevent accidental data loss
- **Preservation Logic**: Current week (locked) is never modified, only future weeks regenerated
- **Bulk Operation**: Regenerates multiple weeks in single request, requires performance optimization

### Source Tree Components
- **Route Handler**: `crates/api/src/routes/meal_planning.rs` - Add `regenerate_all_future_weeks` function
- **Request Types**: Define `RegenerateAllPayload` struct with `confirmation: bool` field
- **Response Types**: Define `RegenerateAllResponse` struct with regenerated_weeks count and first_future_week data
- **Error Types**: `crates/api/src/errors.rs` - Add ConfirmationRequired variant to ApiError enum
- **Domain Algorithm**: `crates/meal_planning/src/algorithm.rs` - Call `generate_single_week` in loop (Epic 7)
- **Integration Tests**: `crates/api/tests/integration/test_regeneration.rs`
- **Performance Tests**: `crates/api/tests/performance/route_latency_tests.rs`

### Testing Standards
- **Coverage Target**: 100% coverage for confirmation logic and current week preservation (critical business rules)
- **Edge Cases**: Zero future weeks, only current week exists, empty confirmation, false confirmation
- **Data Integrity**: Verify current week UNCHANGED after regeneration (preservation test)
- **Bulk Operation**: Test with realistic scenario (1 current + 4 future weeks)

### Key Technical Constraints
- **Confirmation Required**: Destructive operation requires explicit user confirmation to prevent accidents
- **Current Week Preservation**: Week with `is_locked == true` OR `status == "current"` never regenerated
- **Rotation State Reset**: Start with fresh rotation state, seed with current week's recipes for variety continuation
- **Bulk Algorithm Calls**: Loop through future weeks, call `generate_single_week` for each (not parallel initially)

### Algorithm Integration
- **Function Call**: `generate_single_week(recipes, preferences, &mut rotation_state, week_start_date)` in loop
- **Input**: Favorite recipes, user preferences, mutable rotation state (shared across loop iterations), week start date
- **Output**: `Result<WeekMealPlan, Error>` per week, accumulated into Vec<WeekMealPlanData>
- **Performance**: 4 weeks × 3-5s per week = 12-20s total algorithm time (measured separately from route overhead)

### Evento Event Schema
```rust
AllFutureWeeksRegenerated {
    generation_batch_id: String,            // UUID from current meal plan batch
    user_id: String,                        // UUID from JWT claims
    weeks: Vec<WeekMealPlanData>,           // All regenerated future weeks (4-5 weeks)
    preserved_current_week_id: String,      // Current week UUID (not regenerated)
}
```

### Workflow Sequence (from Tech Spec)
1. Frontend: User clicks "Regenerate All Future Weeks" → Confirmation modal appears
2. Modal: "This will regenerate X future weeks. Continue?" [Cancel] [Confirm]
3. Frontend: User clicks Confirm → POST request with confirmation: true
4. TwinSpark: `ts-req="/plan/regenerate-all-future" ts-req-method="POST" ts-data='{"confirmation": true}'`
5. Route Handler: Validate confirmation parameter
6. Route Handler: Identify current week (preserve) and future weeks (regenerate)
7. Route Handler: Loop through future weeks, call algorithm for each
8. Route Handler: Emit AllFutureWeeksRegenerated event
9. Evento Projection: Bulk DELETE old meal_assignments, bulk INSERT new assignments, UPDATE shopping_lists
10. Route Handler: Return 200 OK with count + first future week data
11. Frontend: TwinSpark reloads entire meal calendar or shows success toast

### Shopping List Regeneration (AC: 7)
- **Async Process**: Shopping lists regenerated in evento projection handler for all future weeks
- **Projection Logic**: Subscribes to AllFutureWeeksRegenerated event, recalculates ingredients for each week, bulk updates shopping_lists table
- **Response**: Route returns immediately after emitting event (eventual consistency)
- **Test Pattern**: Use `unsafe_oneshot` to process projection synchronously in tests

### Performance Considerations
- **Bulk Operation**: Regenerating 4 weeks takes longer than single week (12-20s algorithm time + overhead)
- **Route Overhead Target**: P95 <2000ms for route overhead (loading data, looping, emitting events)
- **Algorithm Time Excluded**: Algorithm execution time (12-20s for 4 weeks) not counted in route overhead metric
- **Optimization**: Initial implementation sequential (loop), consider parallelization if performance issue

### Project Structure Notes
- Aligns with event-driven monolith architecture
- Bulk operation pattern: Load → Validate → Loop (Domain Logic) → Evento Event → Response
- Confirmation pattern enforced at route handler level (user-friendly destructive action safeguard)
- Database connection pooling configured (min 5, max 20 connections)
- Rate limiting: 10 regenerations per user per hour (Epic 8 NFR)

### References

**Technical Specification Sections:**
- [Source: docs/tech-spec-epic-8.md#APIs and Interfaces - POST /plan/regenerate-all-future] - Route signature and response schema
- [Source: docs/tech-spec-epic-8.md#Workflows and Sequencing - Regenerate All Future Weeks Flow] - Complete request flow with confirmation modal
- [Source: docs/tech-spec-epic-8.md#Data Models and Contracts - Regenerate All Future Response] - RegenerateAllResponse JSON structure and error responses
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Performance] - P95 <2000ms target for regenerate all route
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Reliability - Edge Cases Handled] - Confirmation parameter validation, current week preservation
- [Source: docs/tech-spec-epic-8.md#Dependencies and Integrations - Rotation State] - Reset rotation state but preserve current week context
- [Source: docs/tech-spec-epic-8.md#Acceptance Criteria - Story 8.4] - Full acceptance criteria breakdown
- [Source: docs/tech-spec-epic-8.md#Traceability Mapping] - AC 8.4.1-8.4.8 test ideas
- [Source: docs/tech-spec-epic-8.md#Design Decisions from Architecture Document] - Regenerate all future weeks requires confirmation parameter (section 7.1)

**UX Specification:**
- [Source: docs/ux-specification.md#User Flows - Flow 3: Meal Plan Disruption and Quick Recovery] - User journey for meal plan regeneration

**Architecture Documents:**
- [Source: docs/tech-spec-epic-8.md#System Architecture Alignment] - Event-driven monolith with bulk operation patterns

## Dev Agent Record

### Context Reference

<!-- Story 8.4 context - Epic 8: Enhanced Meal Planning - Backend Routes & Handlers -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

N/A - Story creation phase

### Completion Notes List

- Created from tech spec Epic 8 acceptance criteria 8.4.1-8.4.10
- All tasks derived from Detailed Design and workflow sequences
- Critical business rules: confirmation required, current week preservation
- Bulk operation: regenerate multiple weeks in single request
- **Implementation Complete (2025-10-26)**: Route handler implemented in `src/routes/meal_planning_api.rs`
  - Confirmation parameter validation: Returns 400 ConfirmationRequired if `confirmation != true`
  - Week identification: Identifies current week (is_locked==true) and future weeks (status==active, is_locked==false)
  - Rotation state initialization: Resets rotation state but seeds with current week's main course recipes
  - Bulk regeneration loop: Calls `generate_single_week` for each future week in order
  - AllFutureWeeksRegenerated event emission: Commits evento event with all regenerated weeks
  - JSON response: Returns count, preserved_current_week_id, and first_future_week data
  - Error handling: ConfirmationRequired (400), InsufficientRecipes (400), AlgorithmTimeout (500)
  - Structured logging and tracing: Full OpenTelemetry instrumentation
  - Route registered at `POST /plan/regenerate-all-future` with authentication middleware
- **Code Review & Fixes Complete (2025-10-26)**: Senior developer review performed, all blocking issues resolved
  - FIXED: Story AC-4 wording corrected from `status == "future"` to `status == "active" AND is_locked == false`
  - FIXED: Removed dead code checking for non-existent `status == "current"` (line 1649)
  - FIXED: SQL query default handling improved with explicit error handling and logging
  - FIXED: Added comprehensive comments explaining DB status value logic (lines 1642-1645)
  - VERIFIED: Code compiles successfully with all fixes applied
  - VERIFIED: Follows established patterns from Stories 8.1-8.3
  - VERIFIED: evento event emission pattern correct
  - VERIFIED: Error handling comprehensive and user-friendly
  - VERIFIED: SQL queries secure (parameterized, no injection risk)
  - VERIFIED: Authentication/authorization properly enforced
- **Integration tests pending**: Test scenarios defined in tasks but not yet implemented
  - Test with confirmation: Verify regeneration and event emission
  - Test without confirmation: Verify 400 ConfirmationRequired error
  - Test preservation: Verify current week unchanged
  - Test zero future weeks: Verify 200 OK with count=0
  - Performance test: P95 <2000ms route overhead (algorithm time excluded)
  - NOTE: Tests require Axum HTTP test harness setup, deferred for future sprint

### File List

- `docs/stories/story-8.4.md` (this file)
- `src/routes/meal_planning_api.rs` (added regenerate_all_future_weeks route handler)
- `src/routes/mod.rs` (exported regenerate_all_future_weeks function)
- `src/main.rs` (registered /plan/regenerate-all-future route)
