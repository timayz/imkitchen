# Story 3.3: Recipe Rotation System

Status: Approved

## Story

As a **user**,
I want **recipes to rotate without duplicates**,
so that **I experience maximum variety before repeating meals**.

## Acceptance Criteria

1. Meal planning algorithm tracks which recipes have been used in current rotation cycle
2. Each favorite recipe used exactly once before any recipe repeats
3. After all favorites used once, rotation cycle resets and recipes become available again
4. Rotation state persists across meal plan regenerations
5. Manually replacing individual meals respects rotation (only offers unused recipes)
6. Adding new favorite mid-rotation includes it in pool immediately
7. Un-favoriting recipe removes from rotation without disrupting active plan
8. Rotation progress visible to user: "12 of 20 favorite recipes used this cycle"

## Tasks / Subtasks

### Task 1: Implement RotationState Data Model (AC: 1-4)
- [ ] Create `RotationState` struct in `crates/meal_planning/src/rotation.rs`
  - [ ] Fields: `cycle_number`, `cycle_started_at`, `used_recipe_ids` (HashSet), `total_favorite_count`
  - [ ] Implement `is_recipe_available()` method to check if recipe unused in current cycle
  - [ ] Implement `mark_recipe_used()` method to add recipe to used set
  - [ ] Implement `should_reset_cycle()` method to determine if all favorites used
  - [ ] Implement `reset_cycle()` method to clear used set and increment cycle number
- [ ] Add `RotationState` field to `MealPlan` aggregate
- [ ] Write unit tests for `RotationState` methods:
  - [ ] Test: Recipe marked as used becomes unavailable
  - [ ] Test: Cycle resets when all favorites used
  - [ ] Test: After reset, all recipes available again
  - [ ] Test: Concurrent marking of recipes (thread safety if needed)

### Task 2: Create Domain Events for Rotation Tracking (AC: 1-3)
- [ ] Define `RecipeUsedInRotation` event in `crates/meal_planning/src/events.rs`
  - [ ] Fields: `recipe_id`, `cycle_number`, `used_at` timestamp
  - [ ] Implement `bincode::Encode` and `bincode::Decode` traits
- [ ] Define `RotationCycleReset` event
  - [ ] Fields: `user_id`, `old_cycle_number`, `new_cycle_number`, `favorite_count`, `reset_at`
- [ ] Update `MealPlan` aggregate event handlers:
  - [ ] Add `recipe_used_in_rotation()` handler to update aggregate rotation state
  - [ ] Add `rotation_cycle_reset()` handler to reset aggregate rotation state
- [ ] Write unit tests for event handlers:
  - [ ] Test: `RecipeUsedInRotation` event correctly updates aggregate state
  - [ ] Test: `RotationCycleReset` event clears used recipes and increments cycle

### Task 3: Create recipe_rotation_state Read Model Table (AC: 4, 8)
- [ ] Create migration `migrations/003_add_recipe_rotation_state.sql` (or append to existing meal plans migration)
  - [ ] Table schema: `id`, `user_id`, `cycle_number`, `cycle_started_at`, `recipe_id`, `used_at`
  - [ ] Unique constraint: `(user_id, cycle_number, recipe_id)` to prevent duplicates
  - [ ] Foreign keys: `user_id` → `users(id)`, `recipe_id` → `recipes(id)`
  - [ ] Indexes: `(user_id, cycle_number)` for fast rotation state queries
- [ ] Run migration to create table in development database
- [ ] Write integration test:
  - [ ] Test: Query rotation state after recipe marked used
  - [ ] Test: Verify unique constraint prevents duplicate entries

### Task 4: Implement Evento Projection for Rotation Events (AC: 4)
- [ ] Create `project_recipe_used_in_rotation()` subscription handler in `crates/meal_planning/src/read_model.rs`
  - [ ] Insert row into `recipe_rotation_state` table when `RecipeUsedInRotation` event emitted
  - [ ] Use `ON CONFLICT DO NOTHING` to handle duplicate events gracefully
  - [ ] Extract `user_id` from event metadata
- [ ] Create `project_rotation_cycle_reset()` subscription handler
  - [ ] Delete all rows for user with old cycle number
  - [ ] (Optional) Archive old cycle data for analytics
- [ ] Register subscriptions in `src/server.rs` or domain crate initialization
- [ ] Write integration test:
  - [ ] Test: Emit `RecipeUsedInRotation` event, verify read model updated
  - [ ] Test: Emit `RotationCycleReset` event, verify old cycle rows cleared

### Task 5: Integrate Rotation Logic into MealPlanningAlgorithm (AC: 1-2)
- [ ] Update `MealPlanningAlgorithm::generate()` in `crates/meal_planning/src/algorithm.rs`
  - [ ] Load current `RotationState` from aggregate or query read model
  - [ ] Filter favorite recipes by `rotation_state.is_recipe_available()` before assignment
  - [ ] After successful meal plan generation, emit `RecipeUsedInRotation` event for each assigned recipe
  - [ ] Check if all favorites used (`rotation_state.should_reset_cycle()`), emit `RotationCycleReset` if needed
- [ ] Update `GenerateMealPlanCommand` to accept `rotation_state` parameter
- [ ] Update route handler `src/routes/meal_plan.rs::generate_meal_plan()` to query rotation state before invoking command
- [ ] Write unit tests:
  - [ ] Test: Algorithm only selects from available (unused) recipes
  - [ ] Test: After all favorites assigned once, cycle resets automatically
  - [ ] Test: Regeneration respects rotation state (doesn't duplicate recently used recipes)

### Task 6: Integrate Rotation into Meal Replacement Flow (AC: 5)
- [ ] Update `ReplaceMealSlotCommand` handler in `crates/meal_planning/src/commands.rs`
  - [ ] Query rotation state to find unused recipes for replacement
  - [ ] When meal replaced, emit `RecipeUsedInRotation` event for new recipe
  - [ ] Mark old recipe as available again (remove from `used_recipe_ids` set if needed)
- [ ] Update route handler `src/routes/meal_plan.rs::replace_meal_slot()`
  - [ ] Query available (unused) recipes for replacement candidates
  - [ ] Filter candidates by meal type and user constraints
  - [ ] Display 3-5 replacement options to user (only unused recipes)
- [ ] Write integration test:
  - [ ] Test: Replace meal, verify old recipe available again, new recipe marked used
  - [ ] Test: Replacement only offers recipes not used in current cycle

### Task 7: Handle Favorite Recipe Changes Mid-Rotation (AC: 6-7)
- [ ] Subscribe to `RecipeFavorited` event from Recipe domain
  - [ ] When new recipe favorited, automatically include in rotation pool (no marking as used required)
  - [ ] Update `total_favorite_count` in rotation state
- [ ] Subscribe to `RecipeUnfavorited` event from Recipe domain
  - [ ] Remove recipe from rotation state `used_recipe_ids` if present
  - [ ] Update `total_favorite_count` in rotation state
  - [ ] If recipe currently assigned in active meal plan, keep assignment but exclude from future rotations
- [ ] Write integration tests:
  - [ ] Test: Favorite new recipe mid-cycle, verify it appears in next meal plan generation
  - [ ] Test: Un-favorite recipe mid-cycle, verify it removed from rotation but active meal plan unaffected

### Task 8: Display Rotation Progress in UI (AC: 8)
- [ ] Update `MealCalendarTemplate` in `templates/pages/meal-calendar.html`
  - [ ] Add rotation progress section: "Recipe variety: {used} of {total} favorites used this cycle"
  - [ ] Query rotation state in route handler: `COUNT(recipe_rotation_state WHERE user_id = ? AND cycle_number = current_cycle)`
  - [ ] Query total favorites: `COUNT(recipes WHERE user_id = ? AND is_favorite = TRUE)`
  - [ ] Pass `used_recipes_count` and `total_favorites` to template
- [ ] Style rotation progress with visual indicator (progress bar or fraction)
- [ ] Write E2E test:
  - [ ] Test: Generate meal plan, verify rotation progress displays "7 of 20 favorites used"
  - [ ] Test: Regenerate plan, verify progress updates to "14 of 20 favorites used"
  - [ ] Test: After cycle reset, verify progress resets to "0 of 20 favorites used"

### Task 9: Add Rotation State Queries (AC: 4, 8)
- [ ] Create `query_rotation_state()` function in `crates/meal_planning/src/read_model.rs`
  - [ ] Query: `SELECT recipe_id FROM recipe_rotation_state WHERE user_id = ? AND cycle_number = (SELECT MAX(cycle_number) FROM recipe_rotation_state WHERE user_id = ?)`
  - [ ] Return `RotationState` struct with current cycle and used recipe IDs
  - [ ] If no rotation state exists, return fresh `RotationState::new()`
- [ ] Create `query_available_recipes_for_rotation()` function
  - [ ] Query favorite recipes NOT IN current rotation state used set
  - [ ] Return list of `Recipe` objects available for assignment
- [ ] Write integration tests:
  - [ ] Test: Query rotation state after several recipes used
  - [ ] Test: Query available recipes, verify only unused recipes returned

### Task 10: Write Comprehensive Test Suite (AC: 1-8, TDD)
- [ ] **Unit tests** (rotation logic):
  - [ ] Test: `RotationState::mark_recipe_used()` prevents duplicates
  - [ ] Test: `RotationState::should_reset_cycle()` triggers reset at correct threshold
  - [ ] Test: `RotationState::reset_cycle()` clears used set and increments cycle number
  - [ ] Test: Rotation state persists across aggregate reloads (event sourcing)
- [ ] **Integration tests** (end-to-end flows):
  - [ ] Test: Generate meal plan with 15 favorites, verify 7 used, 8 available for next generation
  - [ ] Test: Regenerate meal plan, verify next 7 recipes used, 1 available
  - [ ] Test: Regenerate again, verify cycle resets automatically (all 15 available again)
  - [ ] Test: Replace meal, verify old recipe available, new recipe marked used
  - [ ] Test: Favorite new recipe mid-cycle, verify included in next generation
  - [ ] Test: Un-favorite recipe mid-cycle, verify removed from rotation
- [ ] **E2E tests** (Playwright):
  - [ ] Test: User generates meal plan, sees rotation progress "7 of 15 favorites used"
  - [ ] Test: User regenerates plan twice, sees cycle reset message "Rotation cycle complete, starting fresh!"
  - [ ] Test: User replaces meal, sees updated rotation progress
- [ ] **Property-based tests** (rotation invariants):
  - [ ] Property: No recipe used twice before cycle reset
  - [ ] Property: After cycle reset, all favorites available again
  - [ ] Property: Total used + available = total favorites (conservation)
- [ ] Ensure test coverage ≥80% for rotation module (target: NFR requirement)

## Dev Notes

### Architecture Patterns
- **Event Sourcing:** Rotation state managed via `RecipeUsedInRotation` and `RotationCycleReset` events
- **CQRS:** Read model (`recipe_rotation_state` table) optimized for rotation queries
- **Domain Service:** `RotationManager` encapsulates rotation logic (mark used, check availability, reset cycle)
- **Aggregate Ownership:** `MealPlan` aggregate owns rotation state, emits rotation events

### Key Components
- **Domain Crate:** `crates/meal_planning/src/rotation.rs` (RotationManager, RotationState struct)
- **Events:** `RecipeUsedInRotation`, `RotationCycleReset` (in `events.rs`)
- **Read Model:** `recipe_rotation_state` table with projection handlers
- **Algorithm Integration:** `MealPlanningAlgorithm` filters recipes by rotation availability

### Data Flow
1. **Meal Plan Generation:**
   - Algorithm queries `RotationState` from aggregate or read model
   - Filters favorite recipes: only `is_recipe_available() == true`
   - Assigns recipes to meal slots
   - Emits `RecipeUsedInRotation` event for each assigned recipe
   - Checks `should_reset_cycle()`, emits `RotationCycleReset` if needed

2. **Meal Replacement:**
   - User requests meal replacement
   - Query available (unused) recipes for replacement candidates
   - User selects replacement
   - Emit `RecipeUsedInRotation` for new recipe
   - Mark old recipe available again (remove from used set)

3. **Read Model Projection:**
   - Evento subscription: `RecipeUsedInRotation` → INSERT into `recipe_rotation_state`
   - Evento subscription: `RotationCycleReset` → DELETE old cycle rows

### Testing Standards
- **TDD Required:** Write failing test first, then implement to pass
- **Coverage Target:** ≥80% for rotation module (use `cargo tarpaulin`)
- **Test Types:**
  - Unit: Rotation logic (mark used, reset cycle, availability check)
  - Integration: Event projection, rotation state persistence
  - E2E: UI rotation progress display, cycle reset messaging
  - Property-based: Rotation invariants (no duplicates, conservation)

### Performance Considerations
- **Rotation Query Optimization:** Index on `(user_id, cycle_number)` for fast lookups
- **Aggregate Load:** Rotation state stored in aggregate, no additional event stream traversal needed
- **Read Model Freshness:** Eventual consistency acceptable (<500ms projection lag typical)
- **Unique Constraint:** Database enforces no duplicate `(user_id, cycle_number, recipe_id)` entries

### Rotation Cycle Reset Logic
```rust
// Pseudocode for rotation reset decision
fn should_reset_cycle(&self, current_favorite_count: usize) -> bool {
    // Reset if:
    // 1. All favorites used once
    self.used_recipe_ids.len() >= self.total_favorite_count
    // 2. OR user significantly reduced favorites (cleanup orphaned state)
    || current_favorite_count < self.used_recipe_ids.len()
}
```

### Edge Cases
- **Insufficient Favorites:** If <7 favorites available after rotation filter, display error (handled in Story 3.10)
- **Recipe Deletion:** If recipe deleted mid-rotation, remove from used set (recipe no longer exists, can't be assigned)
- **Favorite Count Decrease:** If user un-favorites many recipes, reset cycle to avoid stale state
- **Concurrent Meal Plan Operations:** Aggregate-level locking via evento ensures rotation state consistency

### Project Structure Notes
- **Migration:** Add `recipe_rotation_state` table in `migrations/003_create_meal_plans_table.sql` (or new migration file)
- **Aggregate File:** `crates/meal_planning/src/aggregate.rs` (add `rotation_state` field to `MealPlan` struct)
- **Domain Service:** `crates/meal_planning/src/rotation.rs` (new file for `RotationManager` and `RotationState`)
- **Event Handlers:** `crates/meal_planning/src/read_model.rs` (projection handlers for rotation events)
- **Route Integration:** `src/routes/meal_plan.rs` (query rotation state before command invocation)

### References
- [Source: docs/epics.md#Story 3.3] Recipe Rotation System requirements and acceptance criteria
- [Source: docs/tech-spec-epic-3.md#RotationManager] Detailed rotation logic design and data structures (lines 233-280)
- [Source: docs/tech-spec-epic-3.md#Events] `RecipeUsedInRotation` and `RotationCycleReset` event schemas (lines 318-331)
- [Source: docs/tech-spec-epic-3.md#Read Models] `recipe_rotation_state` table schema (lines 464-479)
- [Source: docs/solution-architecture.md#Event Sourcing] Event sourcing and CQRS patterns (lines 113-120)
- [Source: docs/solution-architecture.md#Evento Integration] Evento subscription setup (lines 446-453)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.3.xml` (Generated: 2025-10-16)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
