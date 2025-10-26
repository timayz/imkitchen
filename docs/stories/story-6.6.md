# Story 6.6: Update Read Models and Projections

Status: Approved

## Story

As a **backend developer**,
I want **to create evento projection handlers for all new Epic 6 events**,
so that **read models stay in sync with domain events and queries return current state**.

## Acceptance Criteria

1. Projection handler created for `MultiWeekMealPlanGenerated` event
2. Handler `project_multi_week_meal_plan_generated` inserts all weeks into `meal_plans` table
3. Projection handler created for `RecipeCreated` event extension (new Epic 6 fields)
4. Handler `project_recipe_created` handles new recipe fields (accepts_accompaniment, preferred_accompaniments, accompaniment_category, cuisine, dietary_tags)
5. Projection handler created for `UserMealPlanningPreferencesUpdated` event (Epic 6 user preferences)
6. Handler `project_user_meal_planning_preferences_updated` updates user preferences columns
7. JSON serialization works for Vec<T> fields stored as TEXT (preferred_accompaniments, dietary_tags, dietary_restrictions)
8. Integration tests verify database updates after evento events
9. Evento subscriptions registered for all new Epic 6 event handlers
10. All tests pass with `unsafe_oneshot` for sync subscription processing

## Tasks / Subtasks

- [ ] Create projection handler for MultiWeekMealPlanGenerated (AC: 1, 2)
  - [ ] Add `project_multi_week_meal_plan_generated` function in `crates/meal_planning/src/read_model.rs`
  - [ ] Iterate over `weeks: Vec<WeekMealPlanData>` from event
  - [ ] Insert each week into `meal_plans` table with fields: id, user_id, start_date, end_date, is_locked, generation_batch_id, status, rotation_state_json, created_at
  - [ ] Insert meal_assignments (21 per week) into `meal_assignments` table with accompaniment_recipe_id field
  - [ ] Calculate week status from dates: Future (start_date > today), Current (start_date <= today <= end_date), Past (end_date < today)
  - [ ] Store rotation_state as JSON string in rotation_state_json column
  - [ ] Add integration test: emit MultiWeekMealPlanGenerated → query meal_plans table → verify all weeks inserted
  - [ ] Add integration test: verify meal_assignments count = weeks × 21

- [ ] Create projection handler for RecipeCreated with Epic 6 fields (AC: 3, 4, 7)
  - [ ] Update existing `project_recipe_created` handler in `crates/recipe/src/read_model.rs`
  - [ ] Add new columns to INSERT: accepts_accompaniment (BOOLEAN), preferred_accompaniments (TEXT JSON), accompaniment_category (TEXT nullable), cuisine (TEXT nullable), dietary_tags (TEXT JSON)
  - [ ] Use `serde_json::to_string()` for Vec<AccompanimentCategory> → TEXT (preferred_accompaniments)
  - [ ] Use `serde_json::to_string()` for Vec<DietaryTag> → TEXT (dietary_tags)
  - [ ] Handle NULL for Option<AccompanimentCategory> and Option<Cuisine>
  - [ ] Add integration test: create recipe with all new fields → query recipes table → verify JSON deserialization works
  - [ ] Add edge case test: create recipe with empty preferred_accompaniments (should store "[]")

- [ ] Create projection handler for UserMealPlanningPreferencesUpdated (AC: 5, 6, 7)
  - [ ] Add `project_user_meal_planning_preferences_updated` function in `crates/user/src/read_model.rs`
  - [ ] UPDATE users table columns: max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight, dietary_restrictions (TEXT JSON)
  - [ ] Use `serde_json::to_string()` for Vec<DietaryRestriction> → TEXT (dietary_restrictions)
  - [ ] WHERE clause: user_id = event.user_id
  - [ ] Add integration test: emit UserMealPlanningPreferencesUpdated → query users table → verify preferences updated
  - [ ] Add integration test: verify dietary_restrictions JSON round-trip (serialize → store → deserialize)

- [ ] Register evento subscriptions for all handlers (AC: 9)
  - [ ] Add subscription for MultiWeekMealPlanGenerated in meal_planning read_model setup
  - [ ] Add subscription for RecipeCreated (enhanced) in recipe read_model setup
  - [ ] Add subscription for UserMealPlanningPreferencesUpdated in user read_model setup
  - [ ] Use `evento::subscribe("subscription-name").aggregator::<AggregateType>().handler(handler_fn).run(&executor)` pattern
  - [ ] Ensure subscription names unique per handler

- [ ] Write integration tests for all projections (AC: 8, 10)
  - [ ] Test: emit MultiWeekMealPlanGenerated with 3 weeks → query meal_plans → verify 3 rows
  - [ ] Test: emit MultiWeekMealPlanGenerated → query meal_assignments → verify 63 assignments (3 weeks × 21)
  - [ ] Test: verify generation_batch_id links all weeks from same batch
  - [ ] Test: emit RecipeCreated with cuisine="italian" → query recipes → verify cuisine field
  - [ ] Test: emit UserMealPlanningPreferencesUpdated → query users → verify max_prep_time_weeknight updated
  - [ ] Use `unsafe_oneshot(&executor)` instead of `run(&executor)` for sync subscription processing in tests
  - [ ] All integration tests use in-memory SQLite database
  - [ ] All tests cleanup after execution (drop tables)

- [ ] Add helper functions for JSON serialization (AC: 7)
  - [ ] Add `serialize_to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error>` in shared_kernel or meal_planning::read_model
  - [ ] Add `deserialize_from_json<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error>` helper
  - [ ] Use helpers in all projection handlers for Vec<T> fields
  - [ ] Add unit test: round-trip serialization for Vec<String>, Vec<DietaryTag>, Vec<AccompanimentCategory>

## Dev Notes

**Architecture Context:**

This story completes the CQRS read model infrastructure for Epic 6, ensuring all new domain events update corresponding database tables via evento projections. Read models enable fast queries without loading full event streams.

**Key Business Rules:**

- Projections are eventually consistent (not immediate)
- Multi-week meal plans store each week as separate row in meal_plans table
- generation_batch_id links weeks generated together
- Week status calculated from dates: Future/Current/Past/Archived
- JSON fields (Vec<T>) stored as TEXT for SQLite compatibility

**Technical Constraints:**

- SQLite doesn't have native JSON column type (use TEXT)
- SQLx query macros disabled (compile-time verification off per architecture)
- Use serde_json for Vec<T> serialization to TEXT
- Evento subscriptions process events async by default
- Tests use `unsafe_oneshot` for synchronous event processing (per architecture doc section 13 testing strategy)
- All projections must be idempotent (replaying events produces same state)

**Testing Standards:**

- Integration tests required for all projection handlers
- Use in-memory SQLite for fast test execution
- Test JSON round-trip serialization explicitly
- Verify database state after eventi event emission
- Coverage target: >90% per Story 6.7

### Project Structure Notes

**Files to Create/Modify:**

```
crates/meal_planning/src/
├── read_model.rs           # ADD project_multi_week_meal_plan_generated handler

crates/recipe/src/
├── read_model.rs           # UPDATE project_recipe_created (add Epic 6 fields)

crates/user/src/
├── read_model.rs           # ADD project_user_meal_planning_preferences_updated handler

migrations/
├── XXX_enhanced_meal_planning.sql  # Already exists from Story 6.1 (tables created)
```

**Dependencies:**

- `serde_json` (JSON serialization for Vec<T> fields)
- `sqlx` (database queries with runtime verification)
- `evento` (subscription registration)
- `tokio` (async test runtime)

**Alignment with Unified Structure:**

- Follows CQRS pattern: events → projections → read models
- Evento subscriptions handle async event processing
- Read models optimized for queries (not normalized)
- Separation: domain layer (events) vs persistence layer (projections)

### References

- [Source: docs/architecture-update-meal-planning-enhancements.md#51-crate-meal_planning] - Domain model files
- [Source: docs/architecture-update-meal-planning-enhancements.md#14-events] - MultiWeekMealPlanGenerated event structure
- [Source: docs/architecture-update-meal-planning-enhancements.md#52-crate-recipe] - RecipeCreated event updates
- [Source: docs/architecture-update-meal-planning-enhancements.md#53-crate-user] - UserMealPlanningPreferencesUpdated event
- [Source: docs/epics.md#story-66-update-read-models-and-projections] - Story acceptance criteria
- [Source: docs/solution-architecture-compact.md#469-projection-testing] - Testing with unsafe_oneshot for subscriptions
- [Source: crates/meal_planning/src/read_model.rs] - Existing projection patterns

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-6.6.xml` (Generated: 2025-10-26T01:15:00Z)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List

### Change Log

---
