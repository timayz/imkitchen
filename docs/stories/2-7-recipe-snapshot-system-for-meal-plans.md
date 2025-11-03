# Story 2.7: Recipe Snapshot System for Meal Plans

Status: ready-for-dev

## Story

As a user,
I want my generated meal plans to preserve original recipe data,
so that I can access historical meal plans even if the recipe owner modifies or deletes their recipe.

## Acceptance Criteria

1. Meal plan generation creates complete snapshot/copy of each referenced recipe
2. Snapshots stored in meal_plan_recipes table with week_id foreign key
3. Snapshots include all recipe fields: name, ingredients, instructions, dietary_restrictions, etc.
4. Calendar displays recipe data from snapshot, not original recipe
5. Recipe modifications by owner don't affect existing meal plan snapshots
6. Recipe deletion by owner doesn't affect existing meal plan snapshots
7. Tests verify snapshot creation, isolation from original recipe changes

## Tasks / Subtasks

- [ ] Create snapshot migration (AC: #2, #3)
  - [ ] Create migrations/queries/20250101000006_meal_plan_recipe_snapshots.sql
  - [ ] Define meal_plan_recipe_snapshots table with all recipe fields
  - [ ] Include: id, meal_plan_id (FK), day_index, meal_slot, original_recipe_id, recipe_type, name, ingredients, instructions, etc.
  - [ ] Add FK to meal_plans with ON DELETE CASCADE
  - [ ] Add index on meal_plan_id for fast lookups

- [ ] Create snapshot data structure (AC: #1, #3)
  - [ ] Define RecipeSnapshot struct with all recipe fields
  - [ ] Implement From<Recipe> trait to convert Recipe to RecipeSnapshot
  - [ ] Include original_recipe_id for reference (may be null if recipe deleted)
  - [ ] Add snapshot_at timestamp

- [ ] Implement snapshot creation function (AC: #1)
  - [ ] Create create_recipe_snapshot function in src/queries/snapshots.rs
  - [ ] Accept recipe_id and meal_plan_id
  - [ ] Query current recipe data from recipes table
  - [ ] Insert into meal_plan_recipe_snapshots table
  - [ ] Return snapshot_id

- [ ] Update meal plan generation to create snapshots (AC: #1)
  - [ ] Modify meal plan generation command (Story 3.1 dependency)
  - [ ] After selecting recipes, create snapshot for each recipe
  - [ ] Store snapshot_id references in MealPlanGenerated event
  - [ ] Call create_recipe_snapshot for each selected recipe

- [ ] Create snapshot query functions (AC: #4)
  - [ ] Create get_meal_plan_snapshots function
  - [ ] Query snapshots by meal_plan_id
  - [ ] Return snapshots ordered by day_index and meal_slot
  - [ ] Join with meal_plans table for week metadata

- [ ] Update calendar queries to use snapshots (AC: #4)
  - [ ] Modify calendar display queries to fetch from snapshots table
  - [ ] NEVER query recipes table for calendar display
  - [ ] Use snapshot data exclusively for meal plan visualization

- [ ] Test snapshot isolation from updates (AC: #5)
  - [ ] Create recipe, generate meal plan with snapshot
  - [ ] Update recipe fields (name, ingredients, etc.)
  - [ ] Verify snapshot unchanged
  - [ ] Verify calendar still shows original snapshot data

- [ ] Test snapshot isolation from deletion (AC: #6)
  - [ ] Create recipe, generate meal plan with snapshot
  - [ ] Delete recipe (soft delete with deleted_at)
  - [ ] Verify snapshot unchanged
  - [ ] Verify calendar still shows snapshot data
  - [ ] Verify original_recipe_id reference preserved

- [ ] Write integration tests (AC: #7)
  - [ ] Test snapshot creation during meal plan generation
  - [ ] Test snapshot includes all recipe fields
  - [ ] Test isolation from recipe updates
  - [ ] Test isolation from recipe deletion
  - [ ] Test calendar displays snapshot data correctly
  - [ ] Test ON DELETE CASCADE when meal plan deleted

## Dev Notes

- **Snapshot Storage**: Snapshots stored in separate table (not embedded in events) for query performance and event size optimization [Source: docs/architecture.md#ADR-003]
- **Complete Copy**: Snapshots include ALL recipe fields at generation time to ensure full isolation from future changes [Source: docs/PRD.md#FR029, docs/epics.md#Story 2.7]
- **Reference Preservation**: original_recipe_id preserved even if recipe deleted; enables "View original" link if recipe still exists [Source: docs/architecture.md#Core Tables]
- **Calendar Isolation**: Calendar queries NEVER access recipes table; use snapshots exclusively for historical accuracy [Source: docs/epics.md#Story 2.7]
- **Cascade Deletion**: When meal plan deleted, all snapshots automatically deleted via ON DELETE CASCADE FK constraint [Source: docs/architecture.md#Core Tables]
- **Future Integration**: This story lays foundation for Story 3.8 (Recipe Snapshot Integration with meal planning algorithm) [Source: docs/epics.md#Story 3.8]

### Project Structure Notes

- **Migrations**: `migrations/queries/20250101000006_meal_plan_recipe_snapshots.sql`
- **Queries**: `src/queries/snapshots.rs` (new file for snapshot functions)
- **Data Structures**: Define RecipeSnapshot in src/queries/snapshots.rs or src/lib.rs
- **Integration**: Meal plan generation command will call snapshot creation (Story 3.1 dependency)
- **Tests**: Add to `tests/mealplan_test.rs` (to be created in Epic 3)

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.7] - Full acceptance criteria and isolation requirements
- [docs/PRD.md#FR029] - Snapshot creation requirement
- [docs/architecture.md#ADR-003] - Recipe Snapshots in Separate Table
- [docs/architecture.md#Core Tables] - meal_plan_recipe_snapshots table schema
- [docs/epics.md#Story 3.8] - Recipe Snapshot Integration with meal planning

## Dev Agent Record

### Context Reference

- docs/stories/2-7-recipe-snapshot-system-for-meal-plans.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
