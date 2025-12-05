# Story 5.1: Recipe Sharing (Public/Private)

Status: drafted

## Story

As a user,
I want to share my recipes publicly with the community,
so that other users can discover and favorite my recipes.

## Acceptance Criteria

1. Recipe aggregate includes `is_shared` field (boolean, defaults to false) in RecipeCreated event
2. RecipeShared and RecipeUnshared events toggle public visibility using evento::save pattern
3. Recipe edit form includes "Share with community" toggle switch
4. Shared recipes appear in community browse page for all users (free and premium tiers)
5. Recipe owner can unshare at any time through recipe detail page or edit form
6. All four recipe types (Appetizer, Main Course, Dessert, Accompaniment) are sharable
7. Query handler projects `is_shared` flag to recipes projection table
8. Tests verify sharing toggle, community visibility query filtering, and unsharing

## Tasks / Subtasks

- [ ] Update Recipe aggregate and events (AC: #1, #2)
  - [ ] Add `is_shared: bool` field to RecipeCreated event in `crates/imkitchen-recipe/src/event.rs`
  - [ ] Create RecipeShared event struct with metadata
  - [ ] Create RecipeUnshared event struct with metadata
  - [ ] Update Recipe aggregate in `crates/imkitchen-recipe/src/aggregate.rs` to handle RecipeShared event
  - [ ] Update Recipe aggregate to handle RecipeUnshared event
- [ ] Implement sharing commands (AC: #2, #5)
  - [ ] Add `share_recipe()` command method in `crates/imkitchen-recipe/src/command.rs` accepting ShareRecipeInput
  - [ ] Add `unshare_recipe()` command method accepting UnshareRecipeInput
  - [ ] Use evento::save pattern with RecipeShared/RecipeUnshared events
  - [ ] Add validation ensuring only recipe owner can share/unshare
- [ ] Update query handlers and projections (AC: #4, #7)
  - [ ] Update on_recipe_created handler in `src/queries/recipes.rs` to store is_shared flag
  - [ ] Create on_recipe_shared handler to update recipes.is_shared = 1
  - [ ] Create on_recipe_unshared handler to update recipes.is_shared = 0
  - [ ] Update subscribe_recipe_query subscription with new handlers
  - [ ] Add database migration for `is_shared` column: `migrations/queries/{timestamp}_recipes.sql`
- [ ] Update recipe forms and templates (AC: #3, #5)
  - [ ] Add "Share with community" toggle to recipe edit form in `templates/pages/recipes/edit.html`
  - [ ] Add Twinspark ts-req for share/unshare actions on recipe detail page
  - [ ] Create share toggle component in `templates/components/share-toggle.html`
  - [ ] Display sharing status badge on recipe cards in user's recipe list
- [ ] Implement route handlers (AC: #2, #5)
  - [ ] Create POST `/recipes/{id}/share` route handler in `src/routes/recipes/share.rs`
  - [ ] Create POST `/recipes/{id}/unshare` route handler in same file
  - [ ] Extract user_id from JWT for authorization check
  - [ ] Return updated recipe detail partial template after share/unshare
- [ ] Update community query (AC: #4, #6)
  - [ ] Add `get_community_recipes()` query function in `src/queries/recipes.rs`
  - [ ] Filter recipes WHERE is_shared = 1
  - [ ] Support all four recipe types in results
  - [ ] Add pagination support (limit/offset parameters)
- [ ] Write tests (AC: #8)
  - [ ] Test RecipeShared event updates aggregate state correctly
  - [ ] Test RecipeUnshared event updates aggregate state correctly
  - [ ] Test share_recipe command emits RecipeShared event
  - [ ] Test unshare_recipe command emits RecipeUnshared event
  - [ ] Test only recipe owner can share/unshare (authorization check)
  - [ ] Test query handler updates is_shared flag in projection
  - [ ] Test community query filters shared recipes correctly
  - [ ] Test all four recipe types can be shared

## Dev Notes

### Architecture Patterns

**Bounded Context:** `imkitchen-recipe` crate handles Recipe aggregate, commands, and events

**Event-Driven Flow:**
1. User clicks "Share with community" toggle → POST `/recipes/{id}/share`
2. Route handler calls `command.share_recipe(input, metadata)`
3. Command emits RecipeShared event using evento::save
4. Query handler processes event, updates recipes.is_shared = 1 in projection
5. Community browse page queries recipes WHERE is_shared = 1

**Database Tables:**
- `recipes` table in queries.db requires `is_shared BOOLEAN DEFAULT 0` column
- Use migration to add column: `ALTER TABLE recipes ADD COLUMN is_shared BOOLEAN DEFAULT 0`
- Create index for community queries: `CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = 1`

**Authorization:**
- Only recipe owner can share/unshare their own recipes
- Extract user_id from JWT cookie in route handler
- Verify user_id matches recipe.owner_id before calling command

**UI/UX Pattern:**
- Use Twinspark for toggle switch with immediate visual feedback
- Display "Shared" badge on recipe cards when is_shared = true
- Toggle switch in edit form and on recipe detail page
- No confirmation modal needed (can easily unshare)

### Project Structure Notes

**Files to Create/Modify:**
- `crates/imkitchen-recipe/src/event.rs` - Add RecipeShared/RecipeUnshared events
- `crates/imkitchen-recipe/src/aggregate.rs` - Handle sharing events
- `crates/imkitchen-recipe/src/command.rs` - Add share_recipe/unshare_recipe methods
- `src/routes/recipes/share.rs` - Route handlers for share/unshare actions
- `src/queries/recipes.rs` - Query handlers and community query function
- `migrations/queries/{timestamp}_recipes.sql` - Add is_shared column
- `templates/pages/recipes/edit.html` - Add share toggle
- `templates/components/share-toggle.html` - Reusable toggle component
- `tests/recipes_test.rs` - Add sharing tests

**Dependencies:**
- No new dependencies required
- Uses existing evento, axum, askama, twinspark stack

### References

- [Source: docs/epics.md#Story 5.1] - Story acceptance criteria and prerequisites
- [Source: docs/PRD.md#FR015] - Recipe sharing functional requirement
- [Source: docs/architecture.md#Epic 5] - Bounded context mapping for community features
- [Source: docs/architecture.md#Command Pattern] - evento::save pattern for events
- [Source: CLAUDE.md#Event Guidelines] - Event naming and structure conventions
- [Source: CLAUDE.md#Askama Guidelines] - Template structure and inheritance

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
