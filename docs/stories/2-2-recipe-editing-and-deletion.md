# Story 2.2: Recipe Editing and Deletion

Status: ready-for-dev

## Story

As a user,
I want to edit and delete my recipes,
so that I can maintain my recipe library accurately.

## Acceptance Criteria

1. RecipeUpdated event stores changed fields with evento::save pattern
2. RecipeDeleted event marks recipe as deleted with soft delete timestamp
3. Recipe edit form pre-populated with current data
4. Deletion requires confirmation modal
5. Deleted recipes removed from user's favorites automatically
6. Deleted shared recipes hidden from community immediately
7. Query handlers update projections for edit/delete events
8. Tests verify edit, delete, and cascade deletion of favorites

## Tasks / Subtasks

- [ ] Define Recipe update and delete events (AC: #1, #2)
  - [ ] Define RecipeUpdated event with all editable fields
  - [ ] Define RecipeDeleted event with deleted_at timestamp
  - [ ] Use bincode derive macros (Encode, Decode)

- [ ] Implement Recipe aggregate handlers (AC: #1, #2)
  - [ ] Add recipe_updated handler to apply changes to aggregate state
  - [ ] Add recipe_deleted handler to mark aggregate as deleted
  - [ ] Update aggregate status field

- [ ] Implement Recipe update and delete commands (AC: #1, #2)
  - [ ] Define UpdateRecipeInput struct with validator constraints
  - [ ] Implement update_recipe method using evento::save
  - [ ] Implement delete_recipe method using evento::save
  - [ ] Validate ownership (only owner can edit/delete)

- [ ] Create query handlers for update/delete events (AC: #7)
  - [ ] Implement on_recipe_updated handler to UPDATE recipes table
  - [ ] Implement on_recipe_deleted handler to soft delete (deleted_at column)
  - [ ] Add on_recipe_deleted handler to cascade delete favorites
  - [ ] Update subscribe_recipe_query with new handlers

- [ ] Update recipes migration for soft delete (AC: #2, #6)
  - [ ] Create new migration or update existing recipes table
  - [ ] Add deleted_at INTEGER column (nullable)
  - [ ] Add index on deleted_at for filtering
  - [ ] Update query functions to exclude deleted recipes

- [ ] Create recipe edit route and template (AC: #3)
  - [ ] Create src/routes/recipes/edit.rs with GET/POST handlers
  - [ ] Create templates/pages/recipes/edit.html
  - [ ] Pre-populate form with current recipe data from projection
  - [ ] Reuse recipe form component from create.html
  - [ ] Verify user ownership before showing edit form

- [ ] Create recipe delete route with confirmation (AC: #4)
  - [ ] Add DELETE handler in src/routes/recipes/delete.rs or edit.rs
  - [ ] Create templates/components/delete-confirmation-modal.html
  - [ ] Use Twinspark for modal interaction
  - [ ] Confirm deletion with "Are you sure?" prompt

- [ ] Update recipe list to show edit/delete actions (AC: #3, #4)
  - [ ] Add Edit and Delete buttons to recipe cards
  - [ ] Show actions only for user's own recipes
  - [ ] Use Twinspark ts-action for delete confirmation

- [ ] Handle cascade deletion of favorites (AC: #5)
  - [ ] Update on_recipe_deleted handler to delete from recipe_favorites table
  - [ ] Use ON DELETE CASCADE in FK constraint or manual deletion
  - [ ] No notifications sent to users who favorited

- [ ] Handle shared recipe visibility (AC: #6)
  - [ ] Update queries to filter deleted recipes from community views
  - [ ] Add WHERE deleted_at IS NULL to all community queries
  - [ ] Test that deleted shared recipes disappear immediately

- [ ] Write integration tests (AC: #8)
  - [ ] Test recipe update (all fields editable)
  - [ ] Test ownership validation (cannot edit others' recipes)
  - [ ] Test recipe deletion (soft delete sets deleted_at)
  - [ ] Test cascade deletion of favorites
  - [ ] Test deleted shared recipes hidden from community
  - [ ] Use evento::load to verify aggregate state changes

## Dev Notes

- **Soft Delete**: Use deleted_at timestamp instead of hard delete to preserve event history and enable potential data recovery [Source: docs/epics.md#Story 2.2]
- **evento::save Pattern**: Use `evento::save::<Recipe>(&recipe_id)` for updates/deletes to existing aggregates (NOT evento::create) [Source: CLAUDE.md#Command Guidelines]
- **Cascade Deletion**: Favorites automatically removed when recipe deleted; NO notifications sent to users [Source: docs/epics.md#Story 2.2, docs/PRD.md#FR016]
- **Ownership Validation**: Commands must verify user_id matches recipe owner_id before allowing edit/delete [Source: docs/architecture.md#Security Architecture]
- **Query Filtering**: All queries must exclude deleted recipes using `WHERE deleted_at IS NULL` [Source: docs/epics.md#Story 2.2]

### Project Structure Notes

- **Routes**: `src/routes/recipes/edit.rs` (GET/POST)
- **Templates**: `templates/pages/recipes/edit.html`, `templates/components/delete-confirmation-modal.html`
- **Events**: Add RecipeUpdated and RecipeDeleted to `crates/imkitchen-recipe/src/event.rs`
- **Aggregate**: Update `crates/imkitchen-recipe/src/aggregate.rs` with new handlers
- **Commands**: Add methods to `crates/imkitchen-recipe/src/command.rs`
- **Migrations**: Update `migrations/queries/20250101000002_recipes.sql` or create new migration for deleted_at column
- **Tests**: Add to existing `tests/recipes_test.rs`

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.2] - Full acceptance criteria and cascade deletion requirements
- [docs/PRD.md#FR016] - Recipe deletion removes favorites without notifications
- [docs/architecture.md#Command Pattern] - evento::save usage for updates
- [docs/architecture.md#Query Pattern] - Projection update handlers
- [CLAUDE.md#Command Guidelines] - evento::save vs evento::create distinction
- [CLAUDE.md#Axum Guidelines] - Route parameter format {id}

## Dev Agent Record

### Context Reference

- docs/stories/2-2-recipe-editing-and-deletion.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
