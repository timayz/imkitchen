# Story 6.7: Write Comprehensive Domain Model Tests

Status: Approved

## Story

As a **backend developer**,
I want **to achieve >90% test coverage on all Epic 6 domain models**,
so that **we have confidence in the foundation before implementing the algorithm**.

## Acceptance Criteria

1. Unit test coverage >90% measured by cargo tarpaulin across all Epic 6 domain crates
2. All enum variants tested (RecipeType, AccompanimentCategory, Cuisine, DietaryTag, DietaryRestriction, WeekStatus, Complexity)
3. All event handlers tested for Recipe, MealPlan, User, and RotationState aggregates
4. Edge cases tested: empty lists, null/Option values, boundary conditions (0 weeks, max 5 weeks)
5. Serialization round-trip tests for all domain events (serde + bincode)
6. All tests pass in CI without warnings
7. Test execution completes in <10 seconds total

## Tasks / Subtasks

- [ ] Write tests for Recipe aggregate (AC: 1, 2, 3, 5)
  - [ ] Test CreateRecipe command with all Epic 6 fields (accepts_accompaniment, preferred_accompaniments, accompaniment_category, cuisine, dietary_tags)
  - [ ] Test enum variants: RecipeType (Appetizer, MainCourse, Dessert, Accompaniment)
  - [ ] Test enum variants: AccompanimentCategory (Pasta, Rice, Fries, Bread, Salad, Vegetables)
  - [ ] Test enum variants: Cuisine (Italian, Mexican, Asian, Indian, Mediterranean, American)
  - [ ] Test enum variants: DietaryTag (Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher)
  - [ ] Test RecipeCreated event serialization/deserialization (serde + bincode)
  - [ ] Test edge cases: empty preferred_accompaniments Vec, None values for Option<Cuisine>
  - [ ] Verify test coverage >90% for crates/recipe/src/*.rs

- [ ] Write tests for MealPlan aggregate (AC: 1, 2, 3, 4, 5)
  - [ ] Test MultiWeekMealPlanGenerated event with 1 week (minimum)
  - [ ] Test MultiWeekMealPlanGenerated event with 5 weeks (maximum)
  - [ ] Test MultiWeekMealPlanGenerated event with boundary: 0 favorite recipes (should error)
  - [ ] Test week status calculation: Future (start_date > today), Current (start_date <= today <= end_date), Past (end_date < today)
  - [ ] Test enum variants: WeekStatus (Future, Current, Past, Archived)
  - [ ] Test is_locked behavior: true when week is Current or Past
  - [ ] Test generation_batch_id links all weeks from same generation
  - [ ] Test meal_assignments: exactly 21 per week (7 days × 3 courses)
  - [ ] Test accompaniment_recipe_id assignment for main courses
  - [ ] Test MultiWeekMealPlanGenerated serialization/deserialization (bincode)
  - [ ] Test SingleWeekRegenerated event
  - [ ] Test AllFutureWeeksRegenerated event
  - [ ] Verify test coverage >90% for crates/meal_planning/src/*.rs

- [ ] Write tests for User aggregate (AC: 1, 2, 3, 5)
  - [ ] Test UserMealPlanningPreferencesUpdated event with all preference fields
  - [ ] Test enum variants: DietaryRestriction (same as DietaryTag enum)
  - [ ] Test enum variants: Complexity (Simple, Moderate, Complex)
  - [ ] Test preference validation: max_prep_time_weeknight < max_prep_time_weekend
  - [ ] Test cuisine_variety_weight range: 0.0 (repeat OK) to 1.0 (max variety)
  - [ ] Test UserMealPlanningPreferencesUpdated serialization/deserialization (bincode)
  - [ ] Test edge cases: empty dietary_restrictions Vec
  - [ ] Verify test coverage >90% for crates/user/src/*.rs (preference-related code)

- [ ] Write tests for RotationState module (AC: 1, 3, 4, 5)
  - [ ] Test RotationState initialization (all fields empty)
  - [ ] Test add_used_main_course: verify uniqueness constraint (no duplicates allowed)
  - [ ] Test add_used_appetizer: allow repeats after all used once (cycle tracking)
  - [ ] Test add_used_dessert: allow repeats after all used once (cycle tracking)
  - [ ] Test cuisine_usage_count increment
  - [ ] Test last_complex_meal_date update
  - [ ] Test RotationState serialization to JSON (stored in database as TEXT)
  - [ ] Test RotationState deserialization from JSON
  - [ ] Test edge cases: empty used_*_ids Vecs, HashMap with 0 entries
  - [ ] Verify test coverage >90% for crates/meal_planning/src/rotation_state.rs

- [ ] Run cargo tarpaulin and verify coverage (AC: 1, 7)
  - [ ] Install cargo-tarpaulin: `cargo install cargo-tarpaulin`
  - [ ] Run coverage: `cargo tarpaulin --workspace --exclude-files 'tests/*' --timeout 120`
  - [ ] Verify overall coverage >90% for Epic 6 crates (recipe, meal_planning, user)
  - [ ] Generate HTML report: `cargo tarpaulin --workspace --out Html`
  - [ ] Review uncovered lines and add missing tests if needed
  - [ ] Ensure all tests pass: `cargo test --workspace`
  - [ ] Verify execution time <10 seconds: `time cargo test --workspace`

- [ ] Add CI integration for coverage enforcement (AC: 6)
  - [ ] Update `.github/workflows/ci.yml` to run cargo tarpaulin
  - [ ] Add coverage threshold check: fail CI if coverage <90%
  - [ ] Upload coverage report to CI artifacts
  - [ ] Add badge to README showing coverage percentage
  - [ ] Configure CI to fail on test warnings (--deny warnings flag)

## Dev Notes

**Architecture Context:**

Story 6.7 completes Epic 6 by establishing comprehensive test coverage for all domain models introduced in Stories 6.2-6.5. High test coverage is critical before implementing the algorithm (Epic 7) because domain models are the foundation of the meal planning system.

**Key Business Rules:**

- Recipe uniqueness: Main courses must never repeat across all weeks in a multi-week plan
- Recipe cycling: Appetizers and desserts can repeat after all favorites used once
- Week locking: Current week (today within Monday-Sunday range) is locked (is_locked=true)
- Week status: Calculated from dates (Future/Current/Past/Archived)
- Accompaniments: Only assigned to main courses with accepts_accompaniment=true

**Technical Constraints:**

- Rust test framework: Use `#[cfg(test)]` modules in each source file
- Coverage tool: cargo-tarpaulin (industry standard for Rust)
- Serialization: bincode for evento events (binary format), serde_json for database JSON fields
- Test execution: Must be fast (<10s) to avoid slowing down TDD workflow
- CI enforcement: Coverage must be measured and enforced automatically

**Testing Standards:**

This story follows TDD principles from solution-architecture-compact.md section 13:
- Unit tests focus on domain logic (no external dependencies like databases)
- Each enum variant tested at least once
- Edge cases explicitly tested (empty collections, nulls, boundaries)
- Serialization round-trips verified (serialize → deserialize → verify equality)
- Coverage target: >90% (per architecture doc and this story's AC #1)

**Testing Strategy:**

1. **Unit Tests (primary focus):**
   - Domain aggregates (Recipe, MealPlan, User)
   - Events (serialization round-trips)
   - RotationState module (state tracking logic)
   - Enum variants (all possible values)

2. **Property-Based Testing (optional enhancement):**
   - Use `quickcheck` or `proptest` for fuzzing inputs
   - Example: Generate random RotationState, verify serialization round-trip
   - Not required for AC coverage but recommended for robustness

3. **Integration Tests (deferred to Story 6.6):**
   - Database projections tested in Story 6.6 (multi_week_projection_tests.rs)
   - This story focuses only on domain layer unit tests

### Project Structure Notes

**Files to Create/Modify:**

```
crates/recipe/src/
├── aggregate.rs            # ADD #[cfg(test)] mod tests
├── events.rs               # ADD serialization round-trip tests

crates/meal_planning/src/
├── aggregate.rs            # ADD #[cfg(test)] mod tests
├── events.rs               # ADD serialization tests
├── rotation_state.rs       # ADD comprehensive unit tests

crates/user/src/
├── aggregate.rs            # ADD #[cfg(test)] mod tests for preferences
├── events.rs               # ADD serialization tests

crates/shared_kernel/src/
├── types.rs                # ADD enum variant tests (if not already present)

.github/workflows/
├── ci.yml                  # UPDATE to run cargo tarpaulin
```

**Dependencies (Cargo.toml dev-dependencies):**

```toml
[dev-dependencies]
cargo-tarpaulin = "0.27"    # Coverage measurement
quickcheck = "1.0"          # Optional: property-based testing
proptest = "1.4"            # Alternative to quickcheck
```

**Alignment with Unified Structure:**

- Tests colocated with source files using `#[cfg(test)]` modules (Rust best practice)
- Separate integration tests in `crates/*/tests/` (used in Story 6.6)
- CI enforces coverage minimum before merge
- Fast test execution ensures TDD workflow not disrupted

### References

- [Source: docs/epics.md#story-67-write-comprehensive-domain-model-tests] - Story acceptance criteria
- [Source: docs/solution-architecture-compact.md#13-testing-strategy] - TDD enforcement, 80% coverage goal, test pyramid
- [Source: docs/architecture-update-meal-planning-enhancements.md#14-events] - Event structures to test
- [Source: docs/architecture-update-meal-planning-enhancements.md#5-domain-model-updates] - Domain models (Recipe, MealPlan, User, RotationState)
- [Source: crates/meal_planning/src/rotation_state.rs] - RotationState implementation (created in Story 6.5)
- [Source: crates/recipe/src/aggregate.rs] - Recipe aggregate (updated in Story 6.2)
- [Source: crates/meal_planning/src/aggregate.rs] - MealPlan aggregate (updated in Story 6.3)
- [Source: crates/user/src/aggregate.rs] - User aggregate (updated in Story 6.4)
- [Cargo Tarpaulin Docs](https://github.com/xd009642/tarpaulin) - Coverage tool configuration

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-6.7.xml` (Generated: 2025-10-26)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
