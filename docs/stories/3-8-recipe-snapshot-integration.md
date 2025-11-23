# Story 3.8: Recipe Snapshot Integration

Status: drafted

## Story

As a user,
I want each generated meal plan to capture complete recipe data at generation time,
So that my historical meal plans remain intact even if recipes are modified or deleted.

## Acceptance Criteria

1. Generation algorithm creates snapshot of each selected recipe during generation
2. Snapshots include all recipe fields plus accompaniment recipe snapshots if paired
3. MealPlanGenerated event includes recipe snapshot IDs
4. Query handler stores snapshots in meal_plan_recipe_snapshots table
5. Calendar queries load recipe data from snapshots, not original recipes
6. Tests verify snapshot creation, storage, and isolation

## Tasks / Subtasks

- [ ] Create recipe snapshot data structure (AC: #1, #2)
  - [ ] Define `RecipeSnapshot` struct with all recipe fields
  - [ ] Include: id, original_recipe_id, recipe_type, name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text, snapshot_at
  - [ ] Add to event.rs or create separate snapshots.rs module
  - [ ] Use bincode Encode/Decode for serialization

- [ ] Implement snapshot creation during generation (AC: #1, #2)
  - [ ] After selecting each recipe (appetizer, main, dessert, accompaniment), create snapshot
  - [ ] Copy all recipe fields into RecipeSnapshot struct
  - [ ] Generate unique snapshot ID (separate from original recipe ID)
  - [ ] Include current timestamp as snapshot_at
  - [ ] Store snapshot data in meal plan event

- [ ] Update event structure to include snapshots (AC: #3)
  - [ ] Add `recipe_snapshots` field to MealPlanGenerated event
  - [ ] Store list of all RecipeSnapshot objects
  - [ ] Update DayData to reference snapshot IDs instead of recipe IDs
  - [ ] Ensure accompaniment snapshots included if paired

- [ ] Create migration for snapshots table (AC: #4)
  - [ ] Define meal_plan_recipe_snapshots table in migrations/queries/
  - [ ] Columns: id, meal_plan_id, day_index, meal_slot, original_recipe_id, recipe_type, name, ingredients, instructions, dietary_restrictions, cuisine_type, snapshot_at
  - [ ] Add foreign key constraint to meal_plans(id)
  - [ ] Add index on meal_plan_id for efficient queries

- [ ] Implement snapshot query handler (AC: #4)
  - [ ] Add handler for MealPlanGenerated event
  - [ ] Extract recipe snapshots from event
  - [ ] Insert each snapshot into meal_plan_recipe_snapshots table
  - [ ] Store day_index and meal_slot for accurate positioning
  - [ ] Use event.timestamp for snapshot_at

- [ ] Update calendar query to use snapshots (AC: #5)
  - [ ] Modify get_week_meals() query to JOIN with meal_plan_recipe_snapshots
  - [ ] Load recipe data from snapshots table, not recipes table
  - [ ] Return snapshot data with meal slot positioning
  - [ ] Ensure accompaniment snapshots loaded if present

- [ ] Write unit tests for snapshot creation (AC: #1, #2, #6)
  - [ ] Create test recipe with all fields populated
  - [ ] Call snapshot creation function
  - [ ] Verify all fields copied correctly
  - [ ] Verify unique snapshot ID generated
  - [ ] Verify original_recipe_id preserved

- [ ] Write integration tests for snapshot storage (AC: #3, #4, #6)
  - [ ] Generate meal plan with favorited recipes
  - [ ] Use evento::load to verify event includes snapshots
  - [ ] Verify snapshots stored in meal_plan_recipe_snapshots table
  - [ ] Query snapshots and compare to original recipes
  - [ ] Verify snapshot_at timestamp matches event timestamp

- [ ] Write integration test for snapshot isolation (AC: #5, #6)
  - [ ] Generate meal plan with recipe A
  - [ ] Modify original recipe A (change name, ingredients)
  - [ ] Query calendar via get_week_meals()
  - [ ] Verify calendar displays original snapshot data, not modified recipe
  - [ ] Delete original recipe A
  - [ ] Query calendar again
  - [ ] Verify calendar still displays snapshot data (no broken references)

## Dev Notes

### Architecture Patterns

- **Snapshot Pattern**: Copy entire recipe state at generation time for immutability
- **Separate Table**: Store snapshots in dedicated table (ADR-003)
- **Event Payload**: Include snapshot data in MealPlanGenerated event
- **Query Isolation**: Calendar queries only access snapshots, never original recipes

### Project Structure Notes

Files to modify/create:
- `crates/imkitchen-mealplan/src/event.rs` - Add RecipeSnapshot struct
- `crates/imkitchen-mealplan/src/generator.rs` - Snapshot creation logic
- `src/queries/mealplans.rs` - Snapshot query handler and get_week_meals()
- `src/queries/snapshots.rs` - New module for snapshot-specific queries
- `migrations/queries/YYYYMMDDHHMMSS_meal_plan_recipe_snapshots.sql` - Snapshots table
- `tests/mealplan_test.rs` - Integration tests for snapshots

### Technical Constraints

**Snapshot Data Structure** [Source: ADR-003, architecture.md Data Architecture]:
```rust
#[derive(Encode, Decode, Clone)]
pub struct RecipeSnapshot {
    pub id: String,                          // Unique snapshot ID
    pub original_recipe_id: String,          // Reference to original (may be deleted)
    pub recipe_type: String,                 // Appetizer | MainCourse | Dessert | Accompaniment
    pub name: String,
    pub ingredients: Vec<String>,            // Full ingredients list
    pub instructions: String,
    pub dietary_restrictions: Vec<String>,
    pub cuisine_type: String,
    pub complexity: String,
    pub advance_prep_text: Option<String>,
    pub snapshot_at: i64,                    // Unix timestamp
}
```

**Database Schema** [Source: architecture.md Data Architecture]:
```sql
CREATE TABLE meal_plan_recipe_snapshots (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    day_index INTEGER NOT NULL CHECK (day_index >= 0 AND day_index <= 6),
    meal_slot TEXT NOT NULL,  -- 'appetizer' | 'main' | 'dessert' | 'accompaniment'
    original_recipe_id TEXT,  -- Reference (may be deleted)
    recipe_type TEXT NOT NULL,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,  -- JSON array
    instructions TEXT NOT NULL,
    dietary_restrictions TEXT,  -- JSON array
    cuisine_type TEXT,
    snapshot_at INTEGER NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_snapshots_meal_plan ON meal_plan_recipe_snapshots(meal_plan_id);
```

**Snapshot Creation During Generation** [Source: epics.md Story 3.8 ACs]:
```rust
fn create_snapshot(recipe: &Recipe) -> RecipeSnapshot {
    RecipeSnapshot {
        id: generate_ulid(),
        original_recipe_id: recipe.id.clone(),
        recipe_type: recipe.recipe_type.clone(),
        name: recipe.name.clone(),
        ingredients: recipe.ingredients.clone(),
        instructions: recipe.instructions.clone(),
        dietary_restrictions: recipe.dietary_restrictions.clone(),
        cuisine_type: recipe.cuisine_type.clone(),
        complexity: recipe.complexity.clone(),
        advance_prep_text: recipe.advance_prep_text.clone(),
        snapshot_at: chrono::Utc::now().timestamp(),
    }
}

// During generation
for day in &mut week.days {
    if let Some(recipe) = selected_main_recipe {
        let snapshot = create_snapshot(recipe);
        day.main_snapshot_id = Some(snapshot.id.clone());
        week.snapshots.push(snapshot);
    }
}
```

**Calendar Query with Snapshots** [Source: epics.md Story 3.8 AC#5]:
```rust
pub async fn get_week_meals(
    pool: &SqlitePool,
    meal_plan_id: &str,
) -> anyhow::Result<Vec<DayMealData>> {
    sqlx::query_as::<_, DayMealData>(
        "SELECT day_index, meal_slot, name, ingredients, instructions
         FROM meal_plan_recipe_snapshots
         WHERE meal_plan_id = ?
         ORDER BY day_index, meal_slot"
    )
    .bind(meal_plan_id)
    .fetch_all(pool)
    .await
}
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Snapshot creation logic
  - Test all fields copied correctly
  - Test snapshot ID uniqueness
  - No database needed
- **Integration Tests**: Full generation with snapshots
  - Generate meal plan
  - Verify evento::load includes snapshots
  - Query snapshots and compare to originals
  - NEVER use direct SQL for assertions
- **Isolation Tests**: Verify immutability
  - Modify original recipe after generation
  - Verify snapshot unchanged
  - Delete original recipe
  - Verify snapshot still accessible

### References

- [Source: epics.md#Epic 3 Story 3.8]
- [Source: PRD.md FR029 - Recipe snapshots at generation time]
- [Source: architecture.md ADR-003 - Recipe Snapshots in Separate Table]
- [Source: architecture.md Data Architecture - meal_plan_recipe_snapshots schema]
- [Source: CLAUDE.md Query Guidelines - Use event.timestamp for timestamps]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
