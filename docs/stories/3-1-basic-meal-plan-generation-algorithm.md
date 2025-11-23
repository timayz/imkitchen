# Story 3.1: Basic Meal Plan Generation Algorithm

Status: ready

## Story

As a user,
I want to generate meal plans for all future weeks of the current month,
So that I can have my meals automatically scheduled without manual planning.

## Acceptance Criteria

1. MealPlanGenerated event stores week data with 7 days × 3 courses (appetizer, main, dessert)
2. Generation command calculates future weeks from next Monday until end of current month
3. Algorithm randomly selects recipes from user's favorited recipes for each meal slot
4. Main courses must be unique across all generated weeks until all favorites exhausted
5. Appetizers/desserts can repeat after all are used once
6. Empty slots left when insufficient favorited recipes available
7. Generation completes in <5 seconds for 5 weeks (P95)
8. Tests verify week calculation, recipe selection, and performance

## Tasks / Subtasks

- [ ] Create meal plan bounded context crate (AC: #1, #2)
  - [ ] Create `crates/imkitchen-mealplan/` directory structure
  - [ ] Add Cargo.toml with workspace dependencies
  - [ ] Create `src/lib.rs` with public exports
  - [ ] Add crate to workspace Cargo.toml

- [ ] Define meal plan events (AC: #1)
  - [ ] Create `event.rs` with `EventMetadata` struct containing user_id and request_id
  - [ ] Define `MealPlanGenerated` event with week metadata and meal slots
  - [ ] Define `WeekData` structure with 7 days × 3 courses
  - [ ] Use bincode Encode/Decode for event serialization

- [ ] Create meal plan aggregate (AC: #1)
  - [ ] Create `aggregate.rs` with `MealPlan` struct
  - [ ] Implement evento aggregator for `meal_plan_generated` event handler
  - [ ] Store generated weeks in aggregate state
  - [ ] Update aggregate status on generation

- [ ] Implement week calculation logic (AC: #2)
  - [ ] Create helper function to calculate next Monday from today
  - [ ] Determine last day of current month
  - [ ] Generate list of Monday dates between next Monday and month end
  - [ ] Return list of week start dates with week numbers (1-5)

- [ ] Create meal plan generation command (AC: #2, #3)
  - [ ] Create `command.rs` with `Command<E: Executor>` struct
  - [ ] Add `GenerateMealPlanInput` with user_id
  - [ ] Implement `generate_meal_plan()` method accepting input and metadata
  - [ ] Load user's favorited recipes from validation database
  - [ ] Call week calculation logic to determine weeks to generate
  - [ ] Use evento::create to emit MealPlanGenerated event
  - [ ] Return meal_plan_id

- [ ] Implement basic recipe selection algorithm (AC: #3, #4, #5, #6)
  - [ ] Create `generator.rs` module for pure Rust algorithm
  - [ ] Load favorited recipes into Vec for in-memory processing
  - [ ] Implement random selection with weighted distribution
  - [ ] Track used main courses to enforce uniqueness constraint
  - [ ] Allow appetizer/dessert repetition after first cycle
  - [ ] Leave slots empty when recipes exhausted
  - [ ] Return structured week data with meal assignments

- [ ] Create meal plan query projections (AC: #1)
  - [ ] Define migration for `meal_plans` table in `migrations/queries/`
  - [ ] Implement query handler for `MealPlanGenerated` event
  - [ ] Store week metadata: user_id, week_start_date, week_number, generated_at
  - [ ] Use event.timestamp for generated_at field
  - [ ] Create subscription builder for meal plan query handlers

- [ ] Create query functions for meal plans (AC: #1)
  - [ ] Add `queries/mealplans.rs` module
  - [ ] Implement `get_user_meal_plans(pool, user_id)` query
  - [ ] Implement `get_week_meals(pool, meal_plan_id)` query
  - [ ] Return meal data structures for template rendering

- [ ] Implement performance optimization (AC: #7)
  - [ ] Use Vec for recipe storage (better cache locality)
  - [ ] Pre-allocate RNG for random selection
  - [ ] Minimize database round-trips in command
  - [ ] Profile generation with 100 favorited recipes
  - [ ] Verify P95 latency < 5 seconds

- [ ] Write unit tests for algorithm (AC: #3, #4, #5, #6, #8)
  - [ ] Test week calculation logic with various dates
  - [ ] Test main course uniqueness constraint
  - [ ] Test appetizer/dessert repetition behavior
  - [ ] Test empty slot handling with insufficient recipes
  - [ ] Use evento::SubscriptionBuilder.unsafe_oneshot for synchronous event processing

- [ ] Write integration tests for command (AC: #1, #2, #7, #8)
  - [ ] Create test database with migrations
  - [ ] Insert test user with favorited recipes
  - [ ] Execute generate_meal_plan command
  - [ ] Verify evento::load returns correct meal plan state
  - [ ] Verify query projections match event data
  - [ ] Measure generation performance with timing

## Dev Notes

### Architecture Patterns

- **Bounded Context**: Create `imkitchen-mealplan` crate following DDD principles per architecture.md
- **Pure Rust Algorithm**: Implement generation logic in `generator.rs` as pure in-memory algorithm (ADR-002)
- **Event-Driven**: Use evento::create pattern for MealPlanGenerated event emission
- **CQRS**: Separate command (generation) from queries (viewing meal plans)

### Project Structure Notes

Per architecture.md project structure:
- Crate location: `crates/imkitchen-mealplan/`
- Required files: `lib.rs`, `command.rs`, `event.rs`, `aggregate.rs`, `generator.rs`
- Query location: `src/queries/mealplans.rs` in main binary
- Migration location: `migrations/queries/YYYYMMDDHHMMSS_meal_plans.sql`

### Technical Constraints

**Performance Requirements** [Source: architecture.md, ADR-002]:
- P95 latency: <5 seconds for 5-week generation with 100 favorited recipes
- Use in-memory processing with Vec (not HashMap) for cache locality
- Pre-compute week boundaries before recipe selection
- Profile with `cargo flamegraph` if performance issues arise

**Database Design** [Source: architecture.md Data Architecture]:
```sql
CREATE TABLE meal_plans (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,  -- ISO date
    week_number INTEGER NOT NULL,  -- 1-5
    is_current_week BOOLEAN NOT NULL DEFAULT 0,
    generated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**Event Structure** [Source: CLAUDE.md Command Guidelines]:
- EventMetadata must include user_id (optional) and request_id (ULID)
- Use bincode Encode/Decode for serialization
- MealPlanGenerated event structure:
  ```rust
  pub struct MealPlanGenerated {
      pub weeks: Vec<WeekData>,
  }

  pub struct WeekData {
      pub week_number: u8,
      pub week_start_date: String,  // ISO format
      pub days: Vec<DayData>,
  }

  pub struct DayData {
      pub day_index: u8,  // 0-6 (Monday-Sunday)
      pub appetizer_id: Option<String>,
      pub main_id: Option<String>,
      pub dessert_id: Option<String>,
  }
  ```

**Recipe Selection Algorithm** [Source: epics.md Story 3.1 ACs]:
1. Load user's favorited recipes from database
2. Separate by type: appetizers, mains, desserts
3. For each week's 7 days:
   - Randomly select main (must be unique until exhausted)
   - Randomly select appetizer (can repeat after first cycle)
   - Randomly select dessert (can repeat after first cycle)
4. Leave slots empty (None) when insufficient recipes

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Test pure algorithm logic in `generator.rs`
  - Week calculation with edge cases (month boundaries)
  - Recipe selection with various favorite counts (0, 1, 10, 50, 100)
  - Uniqueness constraints for main courses
- **Integration Tests**: Test full command execution in `tests/mealplan_test.rs`
  - Use sqlx::migrate! for database setup
  - Use evento::SubscriptionBuilder.unsafe_oneshot for synchronous event processing
  - Verify evento::load returns correct aggregate state
  - NEVER use direct SQL INSERT/SELECT in tests
- **Performance Tests**: Measure P95 latency with 100 favorited recipes
  - Use `std::time::Instant` for timing
  - Run 100 iterations to calculate P95
  - Fail test if P95 > 5 seconds

### References

- [Source: epics.md#Epic 3 Story 3.1]
- [Source: PRD.md FR018-FR027 - Meal Plan Generation requirements]
- [Source: architecture.md ADR-002 - Pure Rust Meal Planning Algorithm]
- [Source: architecture.md Data Architecture - meal_plans table schema]
- [Source: CLAUDE.md Command Guidelines - Command pattern with input struct]
- [Source: CLAUDE.md Query Guidelines - Query handler idempotency]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
