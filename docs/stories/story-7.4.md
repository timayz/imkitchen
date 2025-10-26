# Story 7.4: Single Week Generation

Status: Approved

## Story

As a **meal planning system**,
I want to **generate a complete week's meal plan**,
so that **users receive 21 meal assignments (7 days × 3 courses) respecting all constraints**.

## Acceptance Criteria

1. Function `generate_single_week` implemented
2. Generates 21 assignments (7 days × 3 courses)
3. Assigns: appetizer, main (with optional accompaniment), dessert per day
4. Appetizer/dessert rotation (can repeat after exhausting full list)
5. Main course uses `select_main_course_with_preferences`
6. Accompaniment assigned if `accepts_accompaniment=true`
7. `RotationState` updated after each assignment (marks used recipes)
8. Returns `WeekMealPlan` with `status=Future`, `is_locked=false`
9. Unit tests cover full week generation

## Tasks / Subtasks

- [ ] Implement single week generation function (AC: 1)
  - [ ] Create function in `crates/meal_planning/src/algorithm.rs`
  - [ ] Signature: `pub fn generate_single_week(recipes: Vec<Recipe>, preferences: &UserPreferences, rotation_state: &mut RotationState, week_start_date: Date) -> Result<WeekMealPlan, Error>`
  - [ ] Return `Result<WeekMealPlan, Error>`

- [ ] Generate 21 meal assignments (AC: 2, 3)
  - [ ] Loop through 7 days (Monday-Sunday)
  - [ ] For each day, create 3 assignments: Appetizer, MainCourse, Dessert
  - [ ] Calculate date for each day (week_start_date + day_offset)
  - [ ] Total: 21 `MealAssignment` structs

- [ ] Implement appetizer rotation logic (AC: 4)
  - [ ] Filter recipes where `recipe_type == RecipeType::Appetizer`
  - [ ] Exclude appetizers already used: `rotation_state.used_appetizer_ids`
  - [ ] Select first available appetizer (cyclic)
  - [ ] Mark as used: `rotation_state.mark_used_appetizer(recipe.id)`
  - [ ] If all exhausted, reset: `rotation_state.reset_appetizers_if_all_used(total_appetizers)`

- [ ] Implement main course selection (AC: 5)
  - [ ] Call `select_main_course_with_preferences(available_mains, preferences, rotation_state, date, day_of_week)`
  - [ ] Filter main courses NOT already used (main courses never repeat)
  - [ ] If no compatible main course, return `Error::NoCompatibleRecipes`
  - [ ] Mark as used: `rotation_state.mark_used_main_course(recipe.id)`
  - [ ] Update complexity tracking: `rotation_state.update_last_complex_meal_date(date)` if Complex

- [ ] Implement accompaniment pairing (AC: 6)
  - [ ] Check `main_course.accepts_accompaniment`
  - [ ] If true, call `select_accompaniment(main_course, available_accompaniments)`
  - [ ] Set `meal_assignment.accompaniment_recipe_id = Some(accompaniment.id)`
  - [ ] If false or no compatible, set `accompaniment_recipe_id = None`

- [ ] Implement dessert rotation logic (AC: 4)
  - [ ] Filter recipes where `recipe_type == RecipeType::Dessert`
  - [ ] Exclude desserts already used: `rotation_state.used_dessert_ids`
  - [ ] Select first available dessert (cyclic)
  - [ ] Mark as used: `rotation_state.mark_used_dessert(recipe.id)`
  - [ ] If all exhausted, reset: `rotation_state.reset_desserts_if_all_used(total_desserts)`

- [ ] Update RotationState throughout generation (AC: 7)
  - [ ] Mark appetizers used
  - [ ] Mark main courses used (never reset)
  - [ ] Mark desserts used
  - [ ] Update cuisine usage: `rotation_state.increment_cuisine_usage(&main.cuisine)`
  - [ ] Track last complex meal date if applicable

- [ ] Construct WeekMealPlan result (AC: 8)
  - [ ] Generate UUID for `id`
  - [ ] Set `user_id` from parameters
  - [ ] Set `start_date` = week_start_date (Monday)
  - [ ] Calculate `end_date` = week_start_date + 6 days (Sunday)
  - [ ] Set `status = WeekStatus::Future`
  - [ ] Set `is_locked = false`
  - [ ] Generate `generation_batch_id` (UUID)
  - [ ] Assign `meal_assignments` (21 items)
  - [ ] Set `shopping_list_id` (placeholder, generated in Story 7.6)
  - [ ] Set `created_at` = now()

- [ ] Write comprehensive unit tests (AC: 9)
  - [ ] Test full week generation with sufficient recipes (21+ recipes)
  - [ ] Test appetizer cycling and reset
  - [ ] Test dessert cycling and reset
  - [ ] Test main course uniqueness (no repeats within week)
  - [ ] Test accompaniment pairing when accepted
  - [ ] Test 7 days × 3 courses = 21 assignments
  - [ ] Test WeekMealPlan metadata (status, dates, is_locked)
  - [ ] Test insufficient main courses returns error

## Dev Notes

### Architecture Patterns

**Week Generation Flow:**
```
1. Pre-filter recipes by dietary restrictions (Story 7.1)
2. Separate by type: appetizers, main_courses, desserts, accompaniments
3. Loop through 7 days (Mon-Sun):
   a. Select appetizer (cyclic rotation, reset if exhausted)
   b. Select main course (preference-aware, never repeat)
   c. Pair accompaniment if main accepts
   d. Select dessert (cyclic rotation, reset if exhausted)
   e. Update RotationState
4. Construct WeekMealPlan with 21 assignments
```

**Rotation Rules:**
- **Main Courses:** NEVER repeat (uniqueness enforced across all weeks)
- **Appetizers/Desserts:** CAN repeat after exhausting full list (reset logic)
- **Accompaniments:** CAN repeat freely (not tracked)

**Error Handling:**
- `Error::InsufficientRecipes` - Not enough recipes to generate week
- `Error::NoCompatibleRecipes` - No main course meets constraints for specific day

**Date Calculations:**
```rust
use chrono::{NaiveDate, Duration};

let monday = week_start_date; // Must be Monday
for day_offset in 0..7 {
    let date = monday + Duration::days(day_offset);
    let day_of_week = date.weekday(); // Mon, Tue, Wed, etc.
    // Generate assignments for this date
}
```

### Project Structure Notes

**File Location:**
- `crates/meal_planning/src/algorithm.rs` - Week generation logic
- `crates/meal_planning/src/rotation.rs` - RotationState (Epic 6 Story 6.5)

**Data Models:**
```rust
pub struct WeekMealPlan {
    id: String,                    // UUID
    user_id: UserId,
    start_date: Date,              // Monday
    end_date: Date,                // Sunday
    status: WeekStatus,            // Future | Current | Past | Archived
    is_locked: bool,
    generation_batch_id: String,   // UUID (links multi-week plans)
    meal_assignments: Vec<MealAssignment>,  // 21 items
    shopping_list_id: String,      // Generated in Story 7.6
    created_at: DateTime,
}

pub struct MealAssignment {
    id: String,                    // UUID
    meal_plan_id: String,
    date: Date,
    course_type: CourseType,       // Appetizer | MainCourse | Dessert
    recipe_id: RecipeId,
    accompaniment_recipe_id: Option<RecipeId>,
    prep_required: bool,           // If recipe has advance prep
}

pub enum CourseType {
    Appetizer,
    MainCourse,
    Dessert,
}

pub enum WeekStatus {
    Future,
    Current,
    Past,
    Archived,
}
```

**Dependencies:**
- `chrono` for date manipulation
- `uuid` for ID generation
- `evento` for event emission (Story 7.5 integration)

### Testing Standards

**Test Data Setup:**
- Create test recipes: 10 appetizers, 15 main courses, 10 desserts
- Vary complexity, cuisines, time constraints
- Test with UserPreferences variations

**Test Scenarios:**
1. Full week with sufficient recipes
2. Appetizer/dessert exhaustion and reset
3. Main course exhaustion mid-week (error case)
4. Accompaniment pairing
5. RotationState mutations verified
6. WeekMealPlan structure validation

**Integration with Rotation:**
- Use real RotationState (not mock)
- Verify state changes persist across days

### References

- [Tech Spec: Section 3.4 - Single Week Generation](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.4](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Tech Spec: Week Generation Flow](../tech-spec-epic-7.md#workflows-and-sequencing)
- [Domain Models: WeekMealPlan, MealAssignment](../tech-spec-epic-7.md#data-models-and-contracts)
- [Epic 6 Story 6.5: RotationState Module](./story-6.5.md)
- [Story 7.2: Main Course Selection](./story-7.2.md)
- [Story 7.3: Accompaniment Selection](./story-7.3.md)

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-7.4.xml`
  - Generated: 2025-10-26
  - Includes: Complete acceptance criteria, task breakdown, relevant documentation references, existing code artifacts, interface signatures, constraints, and test ideas

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
