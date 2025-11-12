# Story 2.7: Recipe Snapshot System for Meal Plans

Status: drafted

## Story

As a user,
I want my generated meal plans to preserve original recipe data,
So that I can access historical meal plans even if the recipe owner modifies or deletes their recipe.

## Acceptance Criteria

1. Meal plan generation creates complete snapshot/copy of each referenced recipe
2. Snapshots stored in meal_plan_recipes table with week_id foreign key
3. Snapshots include all recipe fields: name, ingredients, instructions, dietary_restrictions, etc.
4. Calendar displays recipe data from snapshot, not original recipe
5. Recipe modifications by owner don't affect existing meal plan snapshots
6. Recipe deletion by owner doesn't affect existing meal plan snapshots
7. Tests verify snapshot creation, isolation from original recipe changes

## Tasks / Subtasks

- [ ] Create meal_plan_recipe_snapshots table migration (AC: #2, #3)
  - [ ] Create `migrations/queries/{timestamp}_meal_plan_recipe_snapshots.sql`
  - [ ] Define schema per architecture.md (line 629): id, meal_plan_id, day_index, meal_slot, original_recipe_id, recipe_type, name, ingredients, instructions, etc.
  - [ ] Add foreign key to meal_plans with ON DELETE CASCADE
  - [ ] Add snapshot_at timestamp field
  - [ ] Create index on meal_plan_id for efficient queries

- [ ] Define RecipeSnapshot struct (AC: #3)
  - [ ] Create RecipeSnapshot in `src/queries/mealplans.rs` or `src/queries/snapshots.rs`
  - [ ] Include all fields from Recipe projection
  - [ ] Add snapshot_at timestamp
  - [ ] Add original_recipe_id for reference (nullable)

- [ ] Implement snapshot creation during meal generation (AC: #1)
  - [ ] Modify meal plan generation algorithm (Story 3.x)
  - [ ] For each selected recipe, create full snapshot
  - [ ] Copy all recipe fields to snapshot
  - [ ] Set snapshot_at = generation timestamp
  - [ ] Store original_recipe_id for reference
  - [ ] Insert snapshot into meal_plan_recipe_snapshots table

- [ ] Update MealPlanGenerated event to reference snapshots (AC: #1)
  - [ ] Modify event in `crates/imkitchen-mealplan/src/event.rs`
  - [ ] Include snapshot_ids array instead of recipe_ids
  - [ ] Map day_index → meal_slot → snapshot_id
  - [ ] Store snapshot references in event data

- [ ] Implement query handler for snapshot storage (AC: #2)
  - [ ] Add on_meal_plan_generated handler in `src/queries/mealplans.rs`
  - [ ] Insert snapshots into meal_plan_recipe_snapshots table
  - [ ] Link snapshots to meal_plan_id via foreign key
  - [ ] Store day_index and meal_slot for calendar layout

- [ ] Update calendar queries to use snapshots (AC: #4)
  - [ ] Modify get_meal_plan_for_week query
  - [ ] Join meal_plans with meal_plan_recipe_snapshots
  - [ ] Return snapshot data (NOT original recipe data)
  - [ ] Group by day_index and meal_slot

- [ ] Write unit tests for snapshot creation (AC: #7)
  - [ ] Test snapshot includes all recipe fields
  - [ ] Test original_recipe_id stored correctly
  - [ ] Test snapshot_at timestamp set
  - [ ] Verify snapshot independent of Recipe aggregate

- [ ] Write integration tests for snapshot isolation (AC: #5, #6, #7)
  - [ ] Create recipe, generate meal plan with snapshot
  - [ ] Update original recipe (edit name, ingredients)
  - [ ] Query meal plan snapshot → verify unchanged
  - [ ] Delete original recipe
  - [ ] Query meal plan snapshot → verify still accessible

- [ ] Write integration tests for cascade deletion (AC: #2)
  - [ ] Create meal plan with snapshots
  - [ ] Delete meal plan
  - [ ] Verify all associated snapshots deleted (CASCADE)
  - [ ] Confirm original recipes unaffected

- [ ] Write E2E test for snapshot persistence (AC: #7)
  - [ ] Create Playwright test in `tests/e2e/meal_plan_snapshots.spec.ts`
  - [ ] Generate meal plan with recipe X
  - [ ] Edit recipe X (change name)
  - [ ] View meal plan calendar → verify original name shown (from snapshot)
  - [ ] Delete recipe X
  - [ ] View meal plan calendar → verify recipe still visible

## Dev Notes

### Architecture Patterns

**Snapshot vs Event Embedding (per architecture.md ADR-003):**
- Snapshots stored in separate table, NOT embedded in events
- Keeps events lightweight (only snapshot IDs)
- Better query performance with indexed table
- Easier to deduplicate identical snapshots across weeks
- Example event structure:
```rust
pub struct MealPlanGenerated {
    pub week_start_date: String,
    pub snapshot_ids: Vec<SnapshotReference>,  // day_index → meal_slot → snapshot_id
}
```

**Snapshot Isolation Principle:**
- Snapshots are immutable once created
- Changes to original Recipe aggregate don't propagate
- Deletion of original recipe doesn't affect snapshots
- Provides stable historical view of meal plans

**Cascade Deletion Strategy:**
- Snapshots tied to meal_plan_id with ON DELETE CASCADE
- When meal plan deleted, snapshots automatically removed
- Original recipes preserved (no reverse cascade)

### Project Structure Notes

**Files to Create/Modify:**
```
migrations/queries/
└── {timestamp}_meal_plan_recipe_snapshots.sql  # New migration

src/queries/
├── mealplans.rs      # Add snapshot query handlers
└── snapshots.rs      # Optional: separate file for snapshot logic

crates/imkitchen-mealplan/src/
├── event.rs          # Update MealPlanGenerated with snapshot references
└── generator.rs      # Create snapshots during generation
```

**Database Schema (per architecture.md line 629):**
```sql
CREATE TABLE meal_plan_recipe_snapshots (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    day_index INTEGER NOT NULL CHECK (day_index >= 0 AND day_index <= 6),
    meal_slot TEXT NOT NULL,  -- 'appetizer' | 'main' | 'dessert' | 'accompaniment'
    original_recipe_id TEXT,  -- Reference (may be deleted)
    recipe_type TEXT NOT NULL,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,
    instructions TEXT NOT NULL,
    dietary_restrictions TEXT,
    cuisine_type TEXT,
    complexity TEXT,
    advance_prep_text TEXT,
    snapshot_at INTEGER NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_snapshots_meal_plan ON meal_plan_recipe_snapshots(meal_plan_id);
```

### Technical Constraints

**Snapshot Deduplication (Optional Optimization):**
- Multiple meal plans may use same recipe
- Can deduplicate identical snapshots with content hash
- For MVP: create separate snapshot per meal slot (simpler)
- Post-MVP: Implement deduplication to save storage

**Snapshot Size:**
- Average recipe: ~2KB JSON
- 5 weeks × 7 days × 3 courses = 105 snapshots per generation
- Total: ~210KB per meal plan generation
- Storage scales linearly with users

**Query Performance:**
- Index on meal_plan_id enables fast lookup
- Calendar query: single SELECT with JOIN
- No N+1 queries - fetch all day snapshots at once

**Original Recipe Reference:**
- original_recipe_id stored for informational purposes
- Can show "Recipe no longer available" badge if deleted
- Optional: Link to community replacement suggestions

### Integration with Meal Plan Generation

**Snapshot Creation Flow:**
1. Meal plan algorithm selects recipe from favorites
2. Algorithm queries recipe data from recipes projection
3. Algorithm creates RecipeSnapshot struct with all fields
4. Algorithm stores snapshot in meal_plan_recipe_snapshots table
5. Algorithm includes snapshot_id in MealPlanGenerated event

**Calendar Display Flow:**
1. User views calendar for week
2. Query fetches meal_plans for week
3. Query JOINs with meal_plan_recipe_snapshots
4. Query returns snapshot data grouped by day_index/meal_slot
5. Template renders meals using snapshot fields (name, ingredients, etc.)

### References

- [Source: docs/PRD.md#FR029] Recipe snapshot requirement
- [Source: docs/epics.md#Story-2.7] Story acceptance criteria
- [Source: docs/architecture.md#ADR-003] Snapshot storage decision
- [Source: docs/architecture.md#Data-Architecture] Database schema
- [Source: CLAUDE.md#Query-Guidelines] Idempotent query handlers

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
