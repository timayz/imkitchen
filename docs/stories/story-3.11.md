# Story 3.11: Meal Plan Persistence and Activation

Status: Approved

## Story

As a user,
I want my meal plan to persist across sessions,
so that I don't lose my schedule when I close the browser or switch devices.

## Acceptance Criteria

1. Generated meal plan stored in database immediately upon generation
2. Exactly one meal plan active per user at a time (enforced by database constraint)
3. Active meal plan automatically loaded and displayed on dashboard and calendar views
4. Meal plan persists across browser sessions and device switches (server-side storage)
5. Regenerating meal plan archives the old plan (sets is_active=false) and creates new active plan (is_active=true)
6. Active plan indicated by `is_active` boolean flag in database
7. Historical meal plans preserved in database for audit trail (event sourcing maintains full history)
8. User can access meal plan from any authenticated session without re-generation
9. Meal plan ID remains stable during replacements (only regeneration creates new plan ID)
10. Database enforces unique constraint: only one `is_active=true` meal plan per user

## Tasks / Subtasks

- [ ] Task 1: Implement meal plan persistence in read model projection (AC: #1, #7)
  - [ ] Subtask 1.1: Update `project_meal_plan_generated` handler to insert into `meal_plans` table with `is_active=true`
  - [ ] Subtask 1.2: Insert all meal assignments into `meal_assignments` table with proper foreign keys
  - [ ] Subtask 1.3: Verify evento subscription triggers projection on `MealPlanGenerated` event
  - [ ] Subtask 1.4: Write unit test for projection handler (verify SQL insert correctness)

- [ ] Task 2: Enforce single active meal plan constraint (AC: #2, #10)
  - [ ] Subtask 2.1: Add database migration with unique constraint: `UNIQUE(user_id, is_active) WHERE is_active = TRUE`
  - [ ] Subtask 2.2: Update projection handler to deactivate existing active plans before creating new one
  - [ ] Subtask 2.3: Add database-level check constraint or trigger to prevent multiple active plans
  - [ ] Subtask 2.4: Write integration test verifying constraint enforcement (attempt to create two active plans, expect error)

- [ ] Task 3: Implement active meal plan query for dashboard/calendar (AC: #3, #4, #8)
  - [ ] Subtask 3.1: Create `query_active_meal_plan(user_id, pool)` function in `meal_planning/read_model.rs`
  - [ ] Subtask 3.2: SQL query: `SELECT * FROM meal_plans WHERE user_id = ? AND is_active = TRUE`
  - [ ] Subtask 3.3: Join with `meal_assignments` to load full meal plan with assignments
  - [ ] Subtask 3.4: Return `Option<MealPlanView>` (None if no active plan exists)
  - [ ] Subtask 3.5: Write unit test for query (seed database, query active plan, verify results)

- [ ] Task 4: Update dashboard route to load and display active meal plan (AC: #3, #4)
  - [ ] Subtask 4.1: Modify `show_dashboard` handler in `src/routes/dashboard.rs` to call `query_active_meal_plan`
  - [ ] Subtask 4.2: Pass meal plan data to `DashboardTemplate` (handle None case with "Generate Meal Plan" CTA)
  - [ ] Subtask 4.3: Update dashboard template to render today's meals from active plan
  - [ ] Subtask 4.4: Write integration test for dashboard route (verify meal plan displayed correctly)

- [ ] Task 5: Update calendar route to load and display active meal plan (AC: #3, #4)
  - [ ] Subtask 5.1: Modify `show_meal_calendar` handler in `src/routes/meal_plan.rs` to call `query_active_meal_plan`
  - [ ] Subtask 5.2: Pass meal plan data to `MealCalendarTemplate` (handle None case with error message)
  - [ ] Subtask 5.3: Update calendar template to render 7-day view with meal assignments
  - [ ] Subtask 5.4: Write integration test for calendar route (verify week view rendered correctly)

- [ ] Task 6: Implement meal plan archival on regeneration (AC: #5)
  - [ ] Subtask 6.1: Update `project_meal_plan_regenerated` handler to set `is_active=false` on old plan before creating new one
  - [ ] Subtask 6.2: Verify `MealPlanRegenerated` event includes both old and new plan IDs
  - [ ] Subtask 6.3: SQL update: `UPDATE meal_plans SET is_active = FALSE WHERE user_id = ? AND is_active = TRUE`
  - [ ] Subtask 6.4: Write integration test for regeneration (verify old plan archived, new plan active)

- [ ] Task 7: Implement stable meal plan ID during replacements (AC: #9)
  - [ ] Subtask 7.1: Verify `project_meal_slot_replaced` handler updates existing `meal_assignments` row (not creates new)
  - [ ] Subtask 7.2: Verify meal_plan_id foreign key remains unchanged after replacement
  - [ ] Subtask 7.3: Write integration test for replacement (verify meal_plan_id stability)

- [ ] Task 8: Add comprehensive integration tests for persistence (AC: #1-#10)
  - [ ] Subtask 8.1: Test scenario: Generate meal plan → Close session → Reopen → Verify plan loaded
  - [ ] Subtask 8.2: Test scenario: Generate meal plan on device A → Switch to device B → Verify plan accessible
  - [ ] Subtask 8.3: Test scenario: Regenerate meal plan → Verify old plan archived, new plan active
  - [ ] Subtask 8.4: Test scenario: Replace meal slot → Verify plan ID stable, assignment updated
  - [ ] Subtask 8.5: Achieve 80% code coverage for meal plan persistence logic (run `cargo tarpaulin`)

- [ ] Task 9: Update error handling for missing active meal plan (AC: #8)
  - [ ] Subtask 9.1: Handle None case from `query_active_meal_plan` in dashboard (show "Generate Meal Plan" button)
  - [ ] Subtask 9.2: Handle None case in calendar view (redirect to dashboard with message)
  - [ ] Subtask 9.3: Write E2E test with Playwright: user with no meal plan visits calendar, sees helpful message

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing**: Meal plan persistence leverages evento's event sourcing pattern. All meal plan state changes (generation, replacement, regeneration) are recorded as immutable events in the evento event store. The read models (`meal_plans` and `meal_assignments` tables) are projections of these events, updated via evento subscription handlers.

**CQRS**: Commands (`GenerateMealPlan`, `RegenerateMealPlan`) write events to the event store. Queries (`query_active_meal_plan`) read from materialized view tables. This separation ensures read performance independence from write complexity.

**Single Active Plan Constraint**: Database-level enforcement via unique partial index ensures exactly one active meal plan per user. This prevents race conditions and guarantees data integrity. The projection handler must deactivate existing plans before creating new ones to avoid constraint violations.

**Referential Integrity**: `meal_assignments` foreign key to `meal_plans` with `ON DELETE CASCADE` ensures orphaned assignments are cleaned up. Foreign key to `recipes` uses `ON DELETE RESTRICT` to prevent deletion of recipes currently in meal plans.

### Source Tree Components to Touch

**Domain Crate** (`crates/meal_planning/`):
- `src/read_model.rs` - Add `query_active_meal_plan()` function
- `src/aggregate.rs` - Verify `MealPlan` aggregate handles `is_active` state correctly
- `src/events.rs` - Ensure `MealPlanGenerated` and `MealPlanRegenerated` events include all necessary fields
- `tests/read_model_tests.rs` - Unit tests for query functions

**Root Binary** (`src/`):
- `routes/dashboard.rs` - Update `show_dashboard` handler to load active meal plan
- `routes/meal_plan.rs` - Update `show_meal_calendar` handler to load active meal plan
- `routes/meal_plan.rs` - Verify `generate_meal_plan` handler creates active plan
- `routes/meal_plan.rs` - Verify `regenerate_meal_plan` handler archives old plan
- `middleware/error.rs` - Add error handling for `NoActiveMealPlan` case

**Templates** (`templates/`):
- `pages/dashboard.html` - Render today's meals from active plan (or "Generate Plan" CTA if none)
- `pages/meal-calendar.html` - Render 7-day calendar from active plan
- `components/meal-slot.html` - Display individual meal assignment with recipe details

**Database Migrations** (`migrations/`):
- `003_create_meal_plans_table.sql` - Verify unique constraint on (user_id, is_active) exists
- Add migration if constraint missing: `CREATE UNIQUE INDEX idx_meal_plans_unique_active ON meal_plans(user_id) WHERE is_active = TRUE;`

**Integration Tests** (`tests/`):
- `meal_plan_tests.rs` - Add tests for persistence, archival, cross-session access

**E2E Tests** (`e2e/tests/`):
- `meal-planning.spec.ts` - Add test for meal plan persistence across browser sessions

### Project Structure Notes

**Alignment with Unified Project Structure**:

Per `solution-architecture.md` sections 2.1 and 14:
- Domain crates organized by bounded context (DDD pattern): `crates/meal_planning/`
- Root binary handles HTTP routing and template rendering: `src/routes/meal_plan.rs`
- Read models in domain crate, queried from route handlers: `meal_planning::query_active_meal_plan()`
- Evento subscriptions registered in `main.rs` at startup: `evento::subscribe("meal-plan-projections")`
- SQLite database with evento event store + read model tables as per architecture decision ADR-003

**Database Schema Location**:
- Read model migrations: `migrations/003_create_meal_plans_table.sql`
- Evento event store schema managed automatically by evento library (no manual migration needed)
- Connection pooling via SQLx with max 5 connections (sufficient for MVP scale)

**Testing Standards**:

Per `solution-architecture.md` section 15:
- **Unit tests**: Domain aggregate logic in `crates/meal_planning/tests/`
- **Integration tests**: HTTP routes and database projections in `tests/meal_plan_tests.rs`
- **E2E tests**: Full user flows with Playwright in `e2e/tests/meal-planning.spec.ts`
- **Coverage target**: 80% via `cargo tarpaulin` (NFR requirement from PRD)
- **TDD enforced**: Write tests first (red), implement (green), refactor (maintain green)

**Naming Conventions**:

Per `solution-architecture.md` section 13.3:
- Events: Past tense (`MealPlanGenerated`, `MealPlanRegenerated`)
- Commands: Imperative (`GenerateMealPlan`, `RegenerateMealPlan`)
- Functions: `snake_case` (`query_active_meal_plan`)
- Database tables: `snake_case` (`meal_plans`, `meal_assignments`)
- Route paths: kebab-case (`/meal-plan`, `/plan/generate`)

### Testing Standards Summary

**Test Pyramid**:
1. **Unit Tests** (fast, isolated): Domain logic in `crates/meal_planning/tests/`
   - Test `MealPlan` aggregate event handlers
   - Test `query_active_meal_plan` with mock database
   - Test projection handlers with in-memory evento executor

2. **Integration Tests** (moderate speed, database): Route handlers in `tests/meal_plan_tests.rs`
   - Test `/dashboard` and `/plan` routes with real SQLite database
   - Test projection subscription end-to-end (emit event, verify read model updated)
   - Test unique active plan constraint enforcement

3. **E2E Tests** (slow, full stack): User flows in `e2e/tests/meal-planning.spec.ts`
   - Test: Generate meal plan → Close browser → Reopen → Verify plan persisted
   - Test: Regenerate meal plan → Verify old plan archived
   - Test: No active plan → Dashboard shows "Generate Plan" CTA

**Coverage Enforcement**:
- Run `cargo tarpaulin --all-features` in CI pipeline
- Fail build if coverage <80%
- Generate HTML coverage report for review

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-3.md#Data Models and Contracts] - `meal_plans` and `meal_assignments` read model schema
- [Source: docs/tech-spec-epic-3.md#Read Model Projections] - Evento subscription handlers for projections
- [Source: docs/tech-spec-epic-3.md#HTTP Routes] - `show_meal_calendar` and `show_dashboard` route implementations

**Solution Architecture**:
- [Source: docs/solution-architecture.md#2.1 Architecture Pattern] - Event-sourced monolith with DDD bounded contexts
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships] - `meal_plans` table schema and foreign keys
- [Source: docs/solution-architecture.md#3.3 Data Migrations Strategy] - Evento + SQLx dual migration system
- [Source: docs/solution-architecture.md#ADR-001] - Event sourcing with evento (audit trail, CQRS projections)

**Epic Requirements**:
- [Source: docs/epics.md#Story 3.11] - Acceptance criteria and technical notes for meal plan persistence
- [Source: docs/epics.md#Epic 3 Technical Summary] - Aggregates, events, and testing requirements

**PRD Constraints**:
- [Source: docs/PRD.md#Non-Functional Requirements] - 80% code coverage, TDD enforced, <5 second meal plan generation

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.11.xml) - Generated 2025-10-17

### Agent Model Used

<!-- Model name and version will be populated during story execution -->

### Debug Log References

<!-- Debug logs and trace IDs will be added during implementation -->

### Completion Notes List

<!-- Implementation notes, deviations, and lessons learned will be added here -->

### File List

<!-- Complete list of files created/modified will be added during implementation -->
