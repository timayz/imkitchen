# Story 2.2: Recipe Editing and Deletion

Status: drafted

## Story

As a user,
I want to edit and delete my recipes,
So that I can maintain my recipe library accurately.

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

- [ ] Define RecipeUpdated and RecipeDeleted events (AC: #1, #2)
  - [ ] Add RecipeUpdated event to `crates/imkitchen-recipe/src/event.rs`
  - [ ] Add RecipeDeleted event with deleted_at timestamp field
  - [ ] Derive bincode Encode/Decode for both events

- [ ] Implement Recipe aggregate handlers for edit/delete (AC: #1, #2)
  - [ ] Add recipe_updated handler in `crates/imkitchen-recipe/src/aggregate.rs`
  - [ ] Add recipe_deleted handler to set deleted flag in aggregate
  - [ ] Update Recipe aggregate state on edit events

- [ ] Implement update_recipe command (AC: #1)
  - [ ] Add UpdateRecipeInput struct with validator derives
  - [ ] Implement Command::update_recipe using evento::save pattern
  - [ ] Validate changed fields before emitting event
  - [ ] Use existing recipe_id for evento::save

- [ ] Implement delete_recipe command (AC: #2)
  - [ ] Add DeleteRecipeInput with recipe_id
  - [ ] Implement Command::delete_recipe using evento::save
  - [ ] Emit RecipeDeleted event with timestamp from metadata

- [ ] Update recipes table migration for soft delete (AC: #2, #6)
  - [ ] Add deleted_at INTEGER column (nullable) to recipes table
  - [ ] Update migration file: `migrations/queries/{timestamp}_recipes.sql`
  - [ ] Add index on deleted_at for filtering queries

- [ ] Implement query handlers for edit/delete events (AC: #7)
  - [ ] Add on_recipe_updated handler in `src/queries/recipes.rs`
  - [ ] Add on_recipe_deleted handler to set deleted_at timestamp
  - [ ] Update subscribe_recipe_query to include new handlers

- [ ] Implement cascade deletion for favorites (AC: #5)
  - [ ] Create on_recipe_deleted handler in recipe favorites subscription
  - [ ] Delete all recipe_favorites rows matching deleted recipe_id
  - [ ] No notifications sent to users (silent removal per spec)

- [ ] Update queries to filter deleted recipes (AC: #6)
  - [ ] Modify get_user_recipes to exclude deleted_at IS NOT NULL
  - [ ] Modify community recipe query to exclude deleted recipes
  - [ ] Add get_recipe_by_id query checking deleted status

- [ ] Create recipe edit form route (AC: #3)
  - [ ] Create `src/routes/recipes/edit.rs` with GET handler
  - [ ] Load existing recipe data using get_recipe_by_id query
  - [ ] Create `templates/pages/recipes/edit.html` pre-populated with data
  - [ ] Include all fields from recipe creation form

- [ ] Implement recipe edit POST handler (AC: #3)
  - [ ] Add POST /recipes/{id}/edit route handler
  - [ ] Extract recipe_id from path parameter
  - [ ] Call Command::update_recipe with changed fields
  - [ ] Return success or error template

- [ ] Create recipe deletion confirmation modal (AC: #4)
  - [ ] Add delete button to recipe detail/list pages
  - [ ] Create `templates/components/delete-confirmation-modal.html`
  - [ ] Use Twinspark for modal show/hide interactions
  - [ ] Include warning text about permanent deletion

- [ ] Implement recipe deletion POST handler (AC: #4)
  - [ ] Add POST /recipes/{id}/delete route handler
  - [ ] Call Command::delete_recipe with recipe_id
  - [ ] Redirect to recipe list on success

- [ ] Write unit tests for edit command (AC: #8)
  - [ ] Test update_recipe with valid changes
  - [ ] Test validation failures for invalid updates
  - [ ] Use evento::load to verify aggregate state after update

- [ ] Write unit tests for delete command (AC: #8)
  - [ ] Test delete_recipe emits RecipeDeleted event
  - [ ] Verify aggregate marked as deleted
  - [ ] Use evento::load to validate deletion

- [ ] Write integration tests for cascade deletion (AC: #5, #8)
  - [ ] Create recipe, favorite it, then delete recipe
  - [ ] Verify recipe_favorites entry removed automatically
  - [ ] Test with multiple users favoriting same recipe
  - [ ] Use subscribe queries with unsafe_oneshot for synchronous processing

- [ ] Write E2E test for edit and delete flows (AC: #8)
  - [ ] Create Playwright test in `tests/e2e/recipe_management.spec.ts`
  - [ ] Test full edit flow: login → view recipe → edit → save → verify changes
  - [ ] Test full delete flow: view recipe → click delete → confirm → verify removed from list
  - [ ] Test deleted recipe not visible in community

## Dev Notes

### Architecture Patterns

**evento::save Pattern (per CLAUDE.md):**
- Use `evento::save::<Recipe>(&recipe_id)` for update/delete operations
- This updates existing aggregate (not creating new one like evento::create)
- Example:
```rust
pub async fn update_recipe(&self, input: UpdateRecipeInput) -> anyhow::Result<()> {
    evento::save::<Recipe>(&input.recipe_id)
        .data(&RecipeUpdated { ... })?
        .metadata(&metadata)?
        .commit(&self.evento)
        .await?;
    Ok(())
}
```

**Soft Delete Pattern:**
- Don't physically delete rows from database
- Set deleted_at timestamp instead
- Filter deleted recipes in all queries: `WHERE deleted_at IS NULL`
- Preserves data for audit trail and potential recovery

**Cascade Deletion (per epics.md AC #5):**
- Recipe deletion triggers automatic removal from all users' favorites
- NO notifications sent (silent removal per PRD)
- Users discover missing favorite organically during meal generation
- Implemented via query handler listening to RecipeDeleted event

### Project Structure Notes

**Files to Modify:**
```
crates/imkitchen-recipe/src/
├── event.rs         # Add RecipeUpdated, RecipeDeleted events
├── aggregate.rs     # Add event handlers
└── command.rs       # Add update_recipe, delete_recipe methods

src/queries/recipes.rs  # Add on_recipe_updated, on_recipe_deleted handlers

src/routes/recipes/
├── edit.rs          # New file for edit form and handler
└── delete.rs        # New file for delete handler

templates/pages/recipes/
└── edit.html        # New file for edit form

templates/components/
└── delete-confirmation-modal.html  # New reusable modal
```

**Database Schema Changes:**
```sql
-- Add to existing recipes table migration
ALTER TABLE recipes ADD COLUMN deleted_at INTEGER;
CREATE INDEX idx_recipes_deleted ON recipes(deleted_at);
```

### Technical Constraints

**Authorization Check:**
- Verify recipe.owner_id matches authenticated user_id before allowing edit/delete
- Return 403 Forbidden if user attempts to edit/delete someone else's recipe
- Admin users may have override permission (implement if needed)

**Partial Updates:**
- UpdateRecipeInput should include all fields (not just changed ones)
- Simpler than tracking field-level changes
- Event stores complete updated state

**Confirmation Modal UX:**
- Use Twinspark for modal interactions (no JavaScript)
- Modal should clearly state consequences
- Example text: "This will permanently delete your recipe and remove it from all users who favorited it. This action cannot be undone."

**Query Performance:**
- deleted_at index enables efficient filtering
- Community queries: `WHERE is_shared = 1 AND deleted_at IS NULL`
- User recipe queries: `WHERE owner_id = ? AND deleted_at IS NULL`

### References

- [Source: docs/PRD.md#FR004] Recipe management functional requirements
- [Source: docs/epics.md#Story-2.2] Story acceptance criteria
- [Source: docs/architecture.md#Error-Handling] evento::save pattern
- [Source: CLAUDE.md#Command-Guidelines] evento::save vs evento::create
- [Source: CLAUDE.md#Query-Guidelines] Idempotent query handlers
- [Source: mockups/recipe-detail.html] Visual design reference

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
