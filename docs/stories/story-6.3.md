# Story 6.3: Update MealPlan Domain Model

Status: Approved

## Story

As a **backend developer**,
I want **to extend MealPlan aggregate for multi-week support**,
so that **the system can generate and track multiple weeks**.

## Acceptance Criteria

1. WeekMealPlan struct created with fields: end_date, is_locked, generation_batch_id, status
2. MultiWeekMealPlan struct created
3. WeekStatus enum created (Future, Current, Past, Archived)
4. RotationState struct created with tracking fields
5. MultiWeekMealPlanGenerated event created
6. SingleWeekRegenerated event created
7. AllFutureWeeksRegenerated event created
8. MealAssignment updated with accompaniment_recipe_id field
9. Unit tests cover all event handlers
10. All tests pass with >90% coverage

## Tasks / Subtasks

- [ ] Create WeekStatus enum (AC: 3)
  - [ ] Create enum with variants: Future, Current, Past, Archived
  - [ ] Add derives: Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Encode, Decode
  - [ ] Add unit test for serialization round-trip

- [ ] Create WeekMealPlan struct (AC: 1)
  - [ ] Add field: id (String)
  - [ ] Add field: user_id (UserId)
  - [ ] Add field: start_date (Date - Monday)
  - [ ] Add field: end_date (Date - Sunday)
  - [ ] Add field: status (WeekStatus)
  - [ ] Add field: is_locked (bool)
  - [ ] Add field: generation_batch_id (String)
  - [ ] Add field: meal_assignments (Vec<MealAssignment>)
  - [ ] Add field: shopping_list_id (String)
  - [ ] Add field: created_at (DateTime)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, Encode, Decode

- [ ] Create RotationState struct (AC: 4)
  - [ ] Add field: used_main_course_ids (Vec<RecipeId>) - MUST be unique
  - [ ] Add field: used_appetizer_ids (Vec<RecipeId>) - can repeat
  - [ ] Add field: used_dessert_ids (Vec<RecipeId>) - can repeat
  - [ ] Add field: cuisine_usage_count (HashMap<Cuisine, u32>)
  - [ ] Add field: last_complex_meal_date (Option<Date>)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, Encode, Decode
  - [ ] Implement RotationState::new() constructor
  - [ ] Add helper methods: mark_used_main_course, mark_used_appetizer, mark_used_dessert
  - [ ] Add validation method: is_main_course_used

- [ ] Create MultiWeekMealPlan struct (AC: 2)
  - [ ] Add field: user_id (UserId)
  - [ ] Add field: generation_batch_id (String)
  - [ ] Add field: generated_weeks (Vec<WeekMealPlan>)
  - [ ] Add field: rotation_state (RotationState)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, Encode, Decode

- [ ] Update MealAssignment struct (AC: 8)
  - [ ] Add field: accompaniment_recipe_id (Option<RecipeId>)
  - [ ] Ensure serde Serialize/Deserialize derives present
  - [ ] Ensure bincode Encode/Decode derives present
  - [ ] Update existing event handlers to include new field

- [ ] Create MultiWeekMealPlanGenerated event (AC: 5)
  - [ ] Add field: generation_batch_id (String)
  - [ ] Add field: user_id (UserId)
  - [ ] Add field: weeks (Vec<WeekMealPlanData>)
  - [ ] Add field: rotation_state (RotationState)
  - [ ] Add field: generated_at (DateTime)
  - [ ] Add derives: evento::AggregatorName, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize
  - [ ] Implement evento handler in MealPlanAggregate

- [ ] Create SingleWeekRegenerated event (AC: 6)
  - [ ] Add field: week_id (String)
  - [ ] Add field: week_start_date (Date)
  - [ ] Add field: meal_assignments (Vec<MealAssignment>)
  - [ ] Add field: updated_rotation_state (RotationState)
  - [ ] Add derives: evento::AggregatorName, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize
  - [ ] Implement evento handler in MealPlanAggregate

- [ ] Create AllFutureWeeksRegenerated event (AC: 7)
  - [ ] Add field: generation_batch_id (String)
  - [ ] Add field: user_id (UserId)
  - [ ] Add field: weeks (Vec<WeekMealPlanData>)
  - [ ] Add field: preserved_current_week_id (Option<String>)
  - [ ] Add derives: evento::AggregatorName, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize
  - [ ] Implement evento handler in MealPlanAggregate

- [ ] Update MealPlanAggregate with evento handlers (AC: 5, 6, 7)
  - [ ] Implement handler for MultiWeekMealPlanGenerated
  - [ ] Implement handler for SingleWeekRegenerated
  - [ ] Implement handler for AllFutureWeeksRegenerated
  - [ ] Verify aggregate state properly reconstructs from event stream
  - [ ] Test aggregate replay with new events

- [ ] Create comprehensive unit tests (AC: 9, 10)
  - [ ] Test WeekStatus enum serialization round-trip
  - [ ] Test WeekMealPlan struct serialization
  - [ ] Test RotationState struct serialization and helper methods
  - [ ] Test MultiWeekMealPlan struct serialization
  - [ ] Test MultiWeekMealPlanGenerated event serialization and handler
  - [ ] Test SingleWeekRegenerated event serialization and handler
  - [ ] Test AllFutureWeeksRegenerated event serialization and handler
  - [ ] Test MealAssignment with accompaniment_recipe_id field
  - [ ] Test RotationState methods: mark_used_main_course, is_main_course_used
  - [ ] Test backwards compatibility: old events still deserialize
  - [ ] Run `cargo tarpaulin --package meal_planning` and verify >90% coverage

- [ ] Verify compilation and existing tests (AC: 9, 10)
  - [ ] Run `cargo build --package meal_planning` and verify zero warnings
  - [ ] Run `cargo test --package meal_planning` and verify all tests pass
  - [ ] Run `cargo clippy --package meal_planning` and verify no lints

## Dev Notes

### Architecture Context

This story implements domain model updates for Epic 6: Enhanced Meal Planning System, specifically adding **multi-week meal plan support** to the MealPlan aggregate. These changes enable the system to:

1. **Generate multiple weeks simultaneously** (up to 5 weeks maximum)
2. **Track rotation state** across weeks (main courses never repeat, appetizers/desserts can repeat)
3. **Lock current week** to prevent disruption of in-progress meals
4. **Support accompaniment pairing** with main courses via MealAssignment

### Source Document References

- [Source: docs/architecture-update-meal-planning-enhancements.md#1-multi-week-meal-plan-generation] - Multi-week design and data model
- [Source: docs/architecture-update-meal-planning-enhancements.md#1.3-data-model-changes] - WeekMealPlan, MultiWeekMealPlan, RotationState specifications
- [Source: docs/architecture-update-meal-planning-enhancements.md#1.4-events] - MultiWeekMealPlanGenerated, SingleWeekRegenerated, AllFutureWeeksRegenerated event schemas
- [Source: docs/architecture-update-meal-planning-enhancements.md#5-domain-model-updates] - MealPlan aggregate changes
- [Source: docs/epics.md#story-6.3] - Acceptance criteria and technical notes

### Key Design Decisions

**Multi-Week Architecture:**
- Maximum 5 weeks generated per batch (hard cap to prevent excessive computation)
- `generation_batch_id` (UUID) links all weeks created together
- Each week has independent `WeekMealPlan` with 21 meal assignments (7 days × 3 courses)
- Current week (today falls within Monday-Sunday) becomes locked automatically

**Week Locking Rules:**
- `is_locked: bool` prevents regeneration of in-progress weeks
- Current week cannot be regenerated (safety constraint)
- Future weeks can be regenerated individually or all at once
- Locked weeks enforced at application layer (database trigger also present from Story 6.1)

**Rotation State Tracking:**
- `used_main_course_ids`: Main courses MUST be unique across ALL weeks (never repeat)
- `used_appetizer_ids`, `used_dessert_ids`: CAN repeat after exhausting full list
- `cuisine_usage_count`: Tracks cuisine variety for preference algorithm
- `last_complex_meal_date`: Avoids consecutive complex meals

**Week Status Lifecycle:**
- `Future`: Week hasn't started yet (start_date > today)
- `Current`: Today falls within week (start_date <= today <= end_date)
- `Past`: Week completed (end_date < today)
- `Archived`: User manually archived (optional future feature)

**Accompaniment Integration:**
- `MealAssignment.accompaniment_recipe_id: Option<RecipeId>` links main course to side dish
- Only main courses with `accepts_accompaniment=true` get accompaniment
- Accompaniments selected based on `preferred_accompaniments` from Recipe (Story 6.2)

### Event Sourcing Implications

**Backwards Compatibility:**
- Old `MealPlanGenerated` events (pre-Epic 6) lack multi-week structure
- Use `#[serde(default)]` or Option types for all Epic 6 fields
- Aggregate handlers must handle both old single-week and new multi-week events
- Default values:
  - `end_date: start_date + 6 days` (calculate from start_date)
  - `is_locked: false` (assume unlocked for old events)
  - `generation_batch_id: Uuid::new_v4()` (generate for old events)
  - `status: WeekStatus::Future` (default to future)

**Event Stream Strategy:**
- `MultiWeekMealPlanGenerated`: Single event creates all weeks in batch
- `SingleWeekRegenerated`: Updates one week, preserves rotation state
- `AllFutureWeeksRegenerated`: Regenerates multiple weeks, preserves current week

**Testing Event Replay:**
```rust
// Test case: Replay MultiWeekMealPlanGenerated event
let event = MultiWeekMealPlanGenerated {
    generation_batch_id: "batch-123".into(),
    user_id: "user-1".into(),
    weeks: vec![week1, week2, week3],
    rotation_state: RotationState::new(),
    generated_at: Utc::now(),
};

let aggregate = MealPlan::from_events(vec![event])?;
assert_eq!(aggregate.weeks.len(), 3);
assert_eq!(aggregate.generation_batch_id, "batch-123");
```

### Project Structure Notes

**File Locations:**
- Enums: `crates/meal_planning/src/types.rs` (or inline in aggregate.rs)
- MealPlan struct: `crates/meal_planning/src/aggregate.rs`
- Events: `crates/meal_planning/src/events.rs`
- RotationState: `crates/meal_planning/src/rotation.rs` (Story 6.5 will add logic)
- Tests: `crates/meal_planning/src/tests.rs` or `tests/meal_plan_epic6_tests.rs`

**Module Organization:**
```
crates/meal_planning/
├── src/
│   ├── lib.rs
│   ├── aggregate.rs         # MealPlanAggregate (update here)
│   ├── events.rs            # MultiWeekMealPlanGenerated, SingleWeekRegenerated, AllFutureWeeksRegenerated
│   ├── types.rs             # WeekStatus enum, WeekMealPlan struct
│   ├── rotation.rs          # RotationState struct (basic implementation, full logic in Story 6.5)
│   ├── commands.rs          # GenerateMultiWeekMealPlans, RegenerateSingleWeek commands
│   ├── read_model.rs        # Projection updates
│   └── error.rs
├── tests/
│   └── meal_plan_epic6_tests.rs   # Unit tests for Epic 6 functionality
```

**Naming Conventions:**
- Structs: `PascalCase` (WeekMealPlan, MultiWeekMealPlan, RotationState)
- Enums: `PascalCase` (WeekStatus)
- Enum variants: `PascalCase` (Future, Current, Past, Archived)
- Struct fields: `snake_case` (end_date, is_locked, generation_batch_id)
- Events: Past tense `PascalCase` (MultiWeekMealPlanGenerated)

### Serialization Standards

**Serde Configuration:**
```rust
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeekStatus {
    Future,    // Serializes as "future"
    Current,   // Serializes as "current"
    Past,      // Serializes as "past"
    Archived,  // Serializes as "archived"
}
```

**Bincode Configuration:**
```rust
#[derive(bincode::Encode, bincode::Decode)]
pub struct MultiWeekMealPlanGenerated {
    // Fields encoded in declaration order
    // Bincode format versioned via evento stream metadata
}
```

**JSON Storage (Database):**
- `rotation_state: RotationState` → JSON TEXT (used_main_course_ids, cuisine_usage_count, etc.)
- Application-layer validation (SQLite JSON functions optional)

### Testing Strategy

**Unit Test Coverage:**
1. ✅ WeekStatus enum serialization round-trip (serde JSON + bincode)
2. ✅ WeekMealPlan struct creation and serialization
3. ✅ RotationState struct creation and helper methods
4. ✅ MultiWeekMealPlan struct serialization
5. ✅ MultiWeekMealPlanGenerated event serialization and handler
6. ✅ SingleWeekRegenerated event serialization and handler
7. ✅ AllFutureWeeksRegenerated event serialization and handler
8. ✅ MealAssignment with accompaniment_recipe_id field
9. ✅ Backwards compatibility: old MealPlanGenerated events deserialize
10. ✅ Aggregate state reconstruction from multi-week event stream

**Test Data Examples:**
```rust
// Multi-week meal plan with 3 weeks
let batch_id = Uuid::new_v4().to_string();
let week1 = WeekMealPlan {
    id: Uuid::new_v4().to_string(),
    user_id: "user-1".into(),
    start_date: NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(),
    end_date: NaiveDate::from_ymd_opt(2025, 11, 3).unwrap(),
    status: WeekStatus::Current,
    is_locked: true,
    generation_batch_id: batch_id.clone(),
    meal_assignments: vec![/* 21 assignments */],
    shopping_list_id: "shopping-1".into(),
    created_at: Utc::now(),
};

// Rotation state tracking
let mut rotation = RotationState::new();
rotation.mark_used_main_course(&"recipe-123".into());
assert!(rotation.is_main_course_used(&"recipe-123".into()));
assert!(!rotation.is_main_course_used(&"recipe-456".into()));
```

**Use `unsafe_oneshot` for tests per user guidance:**
```rust
#[tokio::test]
async fn test_multi_week_meal_plan_projection() {
    let executor = setup_test_executor().await;

    // Generate multi-week meal plan
    let event = MultiWeekMealPlanGenerated { /* ... */ };

    // Subscribe with unsafe_oneshot for synchronous processing
    evento::subscribe("meal-plan-projections")
        .aggregator::<MealPlan>()
        .handler(project_multi_week_meal_plan)
        .unsafe_oneshot(&executor)  // Synchronous event processing for tests
        .await?;

    // Verify projection updated
    // ...
}
```

### Performance Considerations

**Memory Impact:**
- `WeekMealPlan`: ~1KB per week (21 meal assignments × 50 bytes/assignment)
- `RotationState`: ~500 bytes (used_ids vectors + cuisine map)
- `MultiWeekMealPlan`: ~5KB for 5 weeks (week data + rotation state)
- **Total per user**: ~5-10KB for full multi-week meal plan (acceptable)

**Serialization Performance:**
- Bincode: zero-copy deserialization for most fields (highly optimized)
- Serde JSON: negligible overhead for small structs
- Database: JSON TEXT fields use SQLite native storage (no perf impact)

**Event Stream Size:**
- `MultiWeekMealPlanGenerated`: ~10KB event size (5 weeks × 21 assignments)
- Evento compression recommended if event stream grows large (optional)

### Lessons Learned from Story 6.2

**From Story 6.2 Completion Notes:**
1. ✅ **Pre-existing Columns**: Verify database schema before adding fields
   - **Action**: Story 6.1 added `end_date`, `is_locked`, `generation_batch_id` to meal_plans table
   - **Implication**: Domain model can use these fields directly (no DB changes in this story)

2. ✅ **Backwards Compatibility**: Always use Option types or `#[serde(default)]`
   - **Action**: All Epic 6 fields in events should be Option or have default values

3. ✅ **Enum Serialization**: Use `#[serde(rename_all = "snake_case")]`
   - **Action**: Apply to WeekStatus enum for consistent JSON format

4. ✅ **Test Coverage**: Aim for >90% coverage with unit and integration tests
   - **Action**: Use `cargo tarpaulin` to measure coverage

5. ✅ **Clippy Warnings**: Fix all lints before marking story as done
   - **Action**: Run `cargo clippy` and address warnings

### Acceptance Criteria Verification Checklist

Before marking story as "Done", verify:

- [ ] **AC1**: Run `rg "struct WeekMealPlan" crates/meal_planning/src/` → struct defined with all 10 fields
- [ ] **AC2**: Run `rg "struct MultiWeekMealPlan" crates/meal_planning/src/` → struct defined
- [ ] **AC3**: Run `rg "enum WeekStatus" crates/meal_planning/src/` → enum with 4 variants
- [ ] **AC4**: Run `rg "struct RotationState" crates/meal_planning/src/` → struct with 5 tracking fields
- [ ] **AC5**: Run `rg "struct MultiWeekMealPlanGenerated" crates/meal_planning/src/events.rs` → event defined
- [ ] **AC6**: Run `rg "struct SingleWeekRegenerated" crates/meal_planning/src/events.rs` → event defined
- [ ] **AC7**: Run `rg "struct AllFutureWeeksRegenerated" crates/meal_planning/src/events.rs` → event defined
- [ ] **AC8**: Run `rg "accompaniment_recipe_id" crates/meal_planning/src/` → field added to MealAssignment
- [ ] **AC9**: Run `cargo test --package meal_planning` → includes tests for all 3 new events
- [ ] **AC10**: Run `cargo tarpaulin --package meal_planning` → coverage >90%

### Dependencies and Blockers

**Prerequisites:**
- ✅ Story 6.1 (Database Schema Migration) - **Status: Done**
  - Database columns `end_date`, `is_locked`, `generation_batch_id` exist in meal_plans table
  - Database table `meal_plan_rotation_state` created
- ✅ Story 6.2 (Update Recipe Domain Model) - **Status: Done**
  - Recipe struct has `accepts_accompaniment`, `preferred_accompaniments` fields
  - AccompanimentCategory, Cuisine, DietaryTag enums available

**No Blockers**: All database schema and dependency domain models complete.

**Downstream Dependencies** (Stories blocked by this one):
- Story 6.5: Create Rotation State Management Module (needs RotationState struct)
- Epic 7: Multi-Week Algorithm Implementation (needs all domain models complete)

### Technical Notes

**RotationState Design:**
- Main courses stored in `used_main_course_ids` - NEVER repeat across weeks
- Appetizers/desserts stored separately - CAN repeat after all used once
- `cuisine_usage_count` tracks how many times each cuisine used (for variety scoring)
- `last_complex_meal_date` avoids consecutive complex meals (user preference)

**MealAssignment Extension:**
- `accompaniment_recipe_id: Option<RecipeId>` links main course to side dish (rice, pasta, etc.)
- Only populated if main course has `accepts_accompaniment: true`
- Accompaniments selected based on `preferred_accompaniments` from Recipe

**AccompanimentCategory variants** (from Story 6.2):
- Pasta, Rice, Fries, Salad, Bread, Vegetable, Other

### References

- [Source: docs/architecture-update-meal-planning-enhancements.md#1-multi-week-meal-plan-generation] - Complete multi-week design
- [Source: docs/architecture-update-meal-planning-enhancements.md#1.3-data-model-changes] - WeekMealPlan, MultiWeekMealPlan, RotationState structs
- [Source: docs/architecture-update-meal-planning-enhancements.md#1.4-events] - Event schemas
- [Source: docs/architecture-update-meal-planning-enhancements.md#1.5-algorithm-multi-week-generation] - RotationState usage in algorithm
- [Source: docs/solution-architecture-compact.md#11-component-overview] - Domain crates structure
- [Source: docs/solution-architecture-compact.md#17-implementation-guidance] - Naming conventions
- [Evento Documentation](https://docs.rs/evento/latest/evento/) - Aggregator trait and event handling
- [Serde Documentation](https://serde.rs/) - Serialization/deserialization patterns
- [Bincode Documentation](https://docs.rs/bincode/latest/bincode/) - Binary encoding for evento events

## Dev Agent Record

### Context Reference

- `docs/story-context-6.3.xml` (Generated: 2025-10-25)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
