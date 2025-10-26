# Story 7.5: Multi-Week Meal Plan Generation

Status: Draft

## Story

As a **user**,
I want to **generate meal plans for multiple weeks (1-5)**,
so that **I can plan ahead and maximize recipe variety across weeks**.

## Acceptance Criteria

1. Function `generate_multi_week_meal_plans` implemented
2. Calculates `max_weeks = min(5, min(appetizers, mains, desserts))`
3. Returns `InsufficientRecipes` error if `max_weeks < 1`
4. Filters by dietary restrictions before counting recipes
5. Generates weeks sequentially (loop 0..max_weeks)
6. Week dates calculated from next Monday + offset (ISO 8601)
7. Shopping list generated per week via `generate_shopping_list_for_week`
8. Returns `MultiWeekMealPlan` with all weeks and rotation state
9. Performance: <5 seconds for 5 weeks (P95)
10. Unit tests cover various recipe counts (edge cases: 1 week, 5 weeks, insufficient)

## Tasks / Subtasks

- [ ] Implement multi-week generation function (AC: 1)
  - [ ] Create async function in `crates/meal_planning/src/algorithm.rs`
  - [ ] Signature: `pub async fn generate_multi_week_meal_plans(user_id: UserId, favorite_recipes: Vec<Recipe>, preferences: UserPreferences) -> Result<MultiWeekMealPlan, Error>`
  - [ ] Return `Result<MultiWeekMealPlan, Error>`

- [ ] Filter recipes by dietary restrictions (AC: 4)
  - [ ] Call `filter_by_dietary_restrictions(favorite_recipes, &preferences.dietary_restrictions)`
  - [ ] Use filtered list for all subsequent operations
  - [ ] If all recipes filtered, return `InsufficientRecipes` error

- [ ] Calculate maximum weeks (AC: 2)
  - [ ] Separate recipes by type: appetizers, main_courses, desserts
  - [ ] Count each type
  - [ ] Calculate: `max_weeks = min(5, min(appetizer_count, main_count, dessert_count))`
  - [ ] Hard cap at 5 weeks per architecture decision

- [ ] Validate sufficient recipes (AC: 3)
  - [ ] Check `max_weeks >= 1`
  - [ ] If false, return `Error::InsufficientRecipes { appetizers, main_courses, desserts }`
  - [ ] Error includes actual counts for user feedback

- [ ] Initialize RotationState (AC: 5)
  - [ ] Create `RotationState::new()`
  - [ ] Will be mutated across all week generations

- [ ] Generate weeks sequentially (AC: 5, 6)
  - [ ] Loop `for week_index in 0..max_weeks`
  - [ ] Calculate `week_start_date = calculate_next_monday() + Duration::weeks(week_index)`
  - [ ] Call `generate_single_week(recipes.clone(), &preferences, &mut rotation_state, week_start_date)`
  - [ ] Collect all `WeekMealPlan` results
  - [ ] If any week generation fails, return error and halt

- [ ] Generate shopping lists per week (AC: 7)
  - [ ] For each generated week, call `generate_shopping_list_for_week(&week.meal_assignments, &recipes, week.start_date)`
  - [ ] Attach shopping list ID to `week.shopping_list_id`
  - [ ] Store shopping lists in result

- [ ] Construct MultiWeekMealPlan result (AC: 8)
  - [ ] Generate UUID for `generation_batch_id`
  - [ ] Set `user_id`
  - [ ] Collect all `generated_weeks: Vec<WeekMealPlan>`
  - [ ] Include final `rotation_state` (for future regenerations)
  - [ ] Return `MultiWeekMealPlan`

- [ ] Write comprehensive unit tests (AC: 10)
  - [ ] Test with exactly 7 recipes per type (1 week)
  - [ ] Test with 35+ recipes per type (5 weeks, capped)
  - [ ] Test with 100+ recipes per type (still capped at 5)
  - [ ] Test insufficient recipes (6 appetizers, 7 mains, 7 desserts → 0 weeks)
  - [ ] Test dietary filtering reduces available recipes
  - [ ] Test week date calculations (Monday-Sunday, sequential)
  - [ ] Test RotationState persistence across weeks
  - [ ] Test error propagation from single week generation

- [ ] Performance benchmark (AC: 9)
  - [ ] Create criterion benchmark with 50 recipes (realistic dataset)
  - [ ] Measure end-to-end generation time for 5 weeks
  - [ ] Assert P95 < 5 seconds
  - [ ] Profile bottlenecks if needed

## Dev Notes

### Architecture Patterns

**Multi-Week Generation Flow:**
```
1. Filter recipes by dietary restrictions (Story 7.1)
2. Separate by type: appetizers, main_courses, desserts, accompaniments
3. Calculate max_weeks = min(5, min(counts))
4. Validate max_weeks >= 1 (else InsufficientRecipes error)
5. Initialize RotationState::new()
6. For each week (0..max_weeks):
   a. Calculate week_start_date (next Monday + offset)
   b. Generate single week (Story 7.4)
   c. Generate shopping list (Story 7.6)
   d. Collect week results
7. Return MultiWeekMealPlan with all weeks + rotation state
```

**Max Weeks Calculation Logic:**
```rust
let appetizer_count = recipes.iter().filter(|r| r.recipe_type == RecipeType::Appetizer).count();
let main_count = recipes.iter().filter(|r| r.recipe_type == RecipeType::MainCourse).count();
let dessert_count = recipes.iter().filter(|r| r.recipe_type == RecipeType::Dessert).count();

let max_weeks = [5, appetizer_count / 7, main_count / 7, dessert_count / 7]
    .into_iter()
    .min()
    .unwrap_or(0);

// Each week needs: 7 appetizers, 7 mains, 7 desserts
// Max weeks = min of all three quotients, capped at 5
```

**Week Start Date Calculation:**
```rust
use chrono::{Local, Datelike, Duration, Weekday};

fn calculate_next_monday() -> NaiveDate {
    let today = Local::now().date_naive();
    let days_until_monday = match today.weekday() {
        Weekday::Mon => 7,  // Next week if today is Monday
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1,
    };
    today + Duration::days(days_until_monday)
}

// For week_index > 0:
// week_start_date = calculate_next_monday() + Duration::weeks(week_index as i64)
```

**5-Week Hard Cap Rationale:**
- Balances planning horizon with computational cost
- More weeks = diminishing user value (plans change)
- Main courses NEVER repeat, so cap limits recipe library requirements
- Architecture decision from section 1.2 of tech spec

### Project Structure Notes

**File Location:**
- `crates/meal_planning/src/algorithm.rs` - Multi-week generation logic

**Data Models:**
```rust
pub struct MultiWeekMealPlan {
    user_id: UserId,
    generation_batch_id: String,   // UUID linking all weeks
    generated_weeks: Vec<WeekMealPlan>,
    rotation_state: RotationState, // Final state for future use
}

pub enum Error {
    InsufficientRecipes {
        appetizers: usize,
        main_courses: usize,
        desserts: usize,
    },
    NoCompatibleRecipes {
        course_type: CourseType,
        reason: String,
    },
    AlgorithmTimeout,
    InvalidPreferences(String),
}
```

**Event Integration (for Epic 8):**
- Function returns `MultiWeekMealPlan` struct
- Route handler (Epic 8) emits `MultiWeekMealPlanGenerated` event
- Projections update read models (meal_plans, meal_assignments, shopping_lists tables)

**Async Function:**
- Declared `async` for future database/event store integration
- Current implementation is CPU-bound (no I/O), could be sync
- Async signature allows Epic 8 routes to await without blocking

### Testing Standards

**Test Data Setup:**
- Realistic recipe library: 15 appetizers, 20 mains, 15 desserts, 10 accompaniments
- Varied complexity, cuisines, dietary tags
- UserPreferences: Vegan with weeknight constraints

**Test Scenarios:**
1. **Minimum Viable (1 week):** 7 of each type
2. **Multi-week (3 weeks):** 21 of each type
3. **Maximum (5 weeks):** 35+ of each type, verify cap
4. **Insufficient:** 6 appetizers, 7 mains, 7 desserts → error
5. **Dietary filtering impact:** 20 recipes, 10 filtered → insufficient
6. **Week date sequencing:** Verify Monday-Sunday, no gaps
7. **Rotation state:** Main courses never repeat across 5 weeks
8. **Error handling:** Single week failure propagates error

**Performance Validation:**
- Benchmark with 50 recipes (representative user library)
- 5 weeks × 21 assignments = 105 total assignments
- Measure wall-clock time, assert <5s P95
- Use `criterion` crate for statistical analysis

### References

- [Tech Spec: Section 3.5 - Multi-Week Generation](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.5](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Tech Spec: Multi-Week Flow Diagram](../tech-spec-epic-7.md#workflows-and-sequencing)
- [Tech Spec: 5-Week Hard Cap Rationale](../tech-spec-epic-7.md#objectives-and-scope)
- [Tech Spec: Performance Target <5s](../tech-spec-epic-7.md#performance)
- [Domain Models: MultiWeekMealPlan struct](../tech-spec-epic-7.md#data-models-and-contracts)
- [Story 7.1: Dietary Filtering](./story-7.1.md)
- [Story 7.4: Single Week Generation](./story-7.4.md)
- [Story 7.6: Shopping List Generation](./story-7.6.md)

## Dev Agent Record

### Context Reference

<!-- Story context XML will be added by story-context workflow -->

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
