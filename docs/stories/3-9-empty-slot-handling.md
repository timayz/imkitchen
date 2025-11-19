# Story 3.9: Empty Slot Handling

Status: drafted

## Story

As a user,
I want the algorithm to gracefully handle insufficient recipes,
So that I can still generate plans even with a small recipe library.

## Acceptance Criteria

1. Algorithm leaves meal slots empty when insufficient favorited recipes available
2. No minimum recipe count enforced (can generate with 1 favorite recipe)
3. Empty slots don't block generation or throw errors
4. Empty slot metadata stored in meal plan for UI display
5. Tests verify generation with 0, 1, 5, and 50 favorited recipes

## Tasks / Subtasks

- [ ] Update generation algorithm to allow empty slots (AC: #1, #3)
  - [ ] Remove any minimum recipe count checks
  - [ ] When recipe pool exhausted, set slot ID to None
  - [ ] Continue generation without errors
  - [ ] Don't attempt to select from empty pool

- [ ] Handle empty appetizer pool (AC: #1, #2, #3)
  - [ ] Filter favorited recipes for appetizers
  - [ ] If empty, set all appetizer_id fields to None
  - [ ] Log info message about empty appetizer slots
  - [ ] Continue with main course and dessert selection

- [ ] Handle empty main course pool (AC: #1, #2, #3)
  - [ ] Filter favorited recipes for main courses
  - [ ] If empty, set all main_id fields to None
  - [ ] Log info message about empty main slots
  - [ ] Continue with other meal types

- [ ] Handle empty dessert pool (AC: #1, #2, #3)
  - [ ] Filter favorited recipes for desserts
  - [ ] If empty, set all dessert_id fields to None
  - [ ] Log info message about empty dessert slots
  - [ ] Continue generation

- [ ] Handle partial exhaustion (AC: #1)
  - [ ] Track remaining recipes in each pool during generation
  - [ ] When pool exhausted mid-generation, leave remaining slots empty
  - [ ] Example: 10 main courses, 28 days → 18 empty main slots
  - [ ] Store None for empty slots in DayData

- [ ] Store empty slot metadata in event (AC: #4)
  - [ ] Empty slots represented as None in DayData (appetizer_id, main_id, dessert_id)
  - [ ] No additional metadata needed
  - [ ] Query handler recognizes None as empty slot

- [ ] Update query handler for empty slots (AC: #4)
  - [ ] When appetizer_id/main_id/dessert_id is None, skip snapshot creation
  - [ ] Don't insert snapshot row for empty slots
  - [ ] Calendar query handles missing slots gracefully

- [ ] Update calendar UI for empty slots (AC: #4)
  - [ ] Display placeholder for empty slots
  - [ ] Show message: "Browse community recipes" with link
  - [ ] Distinct styling for empty slots (border dashed, lighter background)
  - [ ] No error state, just information

- [ ] Write unit tests for empty pool handling (AC: #1, #2, #3, #5)
  - [ ] Test generation with 0 favorited recipes (all slots empty)
  - [ ] Test generation with 1 appetizer, 0 mains, 0 desserts
  - [ ] Test generation with 1 recipe of each type
  - [ ] Verify no errors thrown, generation succeeds

- [ ] Write integration tests for partial exhaustion (AC: #1, #5)
  - [ ] Create user with 5 main courses (appetizer/dessert counts vary)
  - [ ] Generate meal plan for 4 weeks (28 meals)
  - [ ] Use evento::load to verify 5 filled main slots + 23 empty main slots
  - [ ] Verify generation succeeds without errors

- [ ] Write integration test with 50 recipes (AC: #5)
  - [ ] Create user with 50+ favorited recipes (balanced across types)
  - [ ] Generate meal plan for 4 weeks
  - [ ] Verify all slots filled (no empty slots)
  - [ ] Verify main course uniqueness still enforced

## Dev Notes

### Architecture Patterns

- **Graceful Degradation**: Empty slots are expected behavior, not errors
- **Optional Fields**: Use Option<String> for recipe IDs in DayData
- **No Minimum Validation**: Allow generation with 0 recipes
- **UI Guidance**: Empty slots show helpful links (Epic 4)

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/generator.rs` - Remove minimum checks, handle empty pools
- `crates/imkitchen-mealplan/src/event.rs` - Ensure DayData uses Option<String> for IDs
- `src/queries/mealplans.rs` - Handle None values in query handler
- `templates/pages/mealplan/calendar.html` - Empty slot placeholder (Epic 4 Story 4.8)
- `tests/mealplan_test.rs` - Integration tests for empty handling

### Technical Constraints

**Empty Slot Representation** [Source: epics.md Story 3.9 ACs]:
```rust
#[derive(Encode, Decode, Clone)]
pub struct DayData {
    pub day_index: u8,  // 0-6 (Monday-Sunday)
    pub appetizer_id: Option<String>,  // None = empty slot
    pub main_id: Option<String>,
    pub dessert_id: Option<String>,
    pub accompaniment_id: Option<String>,
}
```

**Generation with Empty Pools** [Source: epics.md Story 3.9 ACs]:
```rust
fn generate_week(
    appetizers: &[Recipe],
    mains: &[Recipe],
    desserts: &[Recipe],
    accompaniments: &[Recipe],
    used_mains: &mut HashSet<String>,
) -> WeekData {
    let mut days = Vec::new();

    for day_index in 0..7 {
        // Select appetizer (if available)
        let appetizer_id = if !appetizers.is_empty() {
            appetizers.choose(&mut rng).map(|r| r.id.clone())
        } else {
            None  // Empty slot
        };

        // Select main (if available and not used)
        let main_id = if !mains.is_empty() {
            select_unique_main(mains, used_mains).map(|r| r.id.clone())
        } else {
            None  // Empty slot
        };

        // Select dessert (if available)
        let dessert_id = if !desserts.is_empty() {
            desserts.choose(&mut rng).map(|r| r.id.clone())
        } else {
            None  // Empty slot
        };

        days.push(DayData {
            day_index,
            appetizer_id,
            main_id,
            dessert_id,
            accompaniment_id: None,
        });
    }

    WeekData { days, /* ... */ }
}
```

**Query Handler for Empty Slots** [Source: CLAUDE.md Query Guidelines]:
```rust
#[evento::handler(MealPlan)]
async fn on_meal_plan_generated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    // Insert meal plan
    for week in &event.data.weeks {
        sqlx::query("INSERT INTO meal_plans (...) VALUES (...)")
            .execute(&pool)
            .await?;

        // Insert snapshots for non-empty slots only
        for day in &week.days {
            if let Some(snapshot) = &day.appetizer_snapshot {
                // Insert snapshot
            }
            // Skip if None (empty slot)
        }
    }

    Ok(())
}
```

**Calendar UI Empty State** [Source: PRD.md FR034, Epic 4 Story 4.8]:
```html
{% if day.main_id %}
  <div class="meal-card">
    <h3>{{ recipe.name }}</h3>
    <!-- Full recipe display -->
  </div>
{% else %}
  <div class="meal-card empty-slot">
    <p class="text-gray-500">No main course</p>
    <a href="/recipes/community?type=main" class="text-blue-600">
      Browse community recipes
    </a>
  </div>
{% endif %}
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Test empty pool logic
  - Pure function tests
  - Test with empty arrays for each recipe type
  - Verify no panics or errors
  - No database needed
- **Integration Tests**: Full generation with various recipe counts
  - Test with 0, 1, 5, 10, 50 recipes
  - Verify evento::load returns correct state
  - Measure empty slot counts
  - NEVER use direct SQL
- **Edge Case Tests**: Partial exhaustion
  - Generate more meals than available unique mains
  - Verify correct number of empty slots
  - Verify no duplicates before exhaustion

### References

- [Source: epics.md#Epic 3 Story 3.9]
- [Source: PRD.md FR021 - Graceful handling of insufficient recipes]
- [Source: PRD.md FR034 - Empty slots display community recipe links]
- [Source: architecture.md Data Architecture - DayData with Option<String>]
- [Source: CLAUDE.md Query Guidelines - Handle None values]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
