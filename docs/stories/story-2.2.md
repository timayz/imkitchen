# Story 2.2: Edit Recipe

Status: Approved

## Story

As a recipe owner,
I want to modify my recipe details,
so that I can correct errors or improve instructions.

## Acceptance Criteria

1. Recipe edit page pre-populated with current recipe data
2. All fields editable (title, ingredients, instructions, timing, advance prep, serving size)
3. Changes validated before saving
4. Successful save updates recipe and shows confirmation
5. Recipe version history maintained via event sourcing
6. Updated recipe immediately reflects in meal plans (if currently scheduled)
7. Only recipe owner can edit their recipes
8. Community-shared recipes remain editable by owner only

## Tasks / Subtasks

- [ ] Create recipe edit route and handler (AC: 1, 7, 8)
  - [ ] Add GET `/recipes/:id/edit` route in `src/routes/recipes.rs`
  - [ ] Implement authorization check: verify user_id matches recipe owner
  - [ ] Query recipe read model to fetch current recipe data
  - [ ] Render Askama template with pre-populated form data
  - [ ] Return 403 Forbidden if user is not recipe owner

- [ ] Design and implement recipe edit form template (AC: 1, 2)
  - [ ] Create or reuse `templates/pages/recipe-form.html` (same as creation form)
  - [ ] Pre-populate all form fields with existing recipe data
  - [ ] Support dynamic ingredient row editing (add/remove/reorder)
  - [ ] Support instruction step editing (add/remove/reorder)
  - [ ] Include all editable fields: title, ingredients, instructions, prep_time, cook_time, advance_prep, serving_size

- [ ] Implement form validation (AC: 3)
  - [ ] Use validator crate for server-side validation
  - [ ] Validate required fields: title (non-empty), at least 1 ingredient, at least 1 instruction
  - [ ] Validate data types: prep_time and cook_time as positive integers
  - [ ] Return 422 Unprocessable Entity with inline error messages on validation failure

- [ ] Implement update recipe command handler (AC: 4, 5)
  - [ ] Add PUT `/recipes/:id` route in `src/routes/recipes.rs`
  - [ ] Parse and validate form data
  - [ ] Load Recipe aggregate from evento event stream
  - [ ] Execute UpdateRecipeCommand with changed fields
  - [ ] Emit RecipeUpdated event with delta (changed fields only)
  - [ ] Commit event to evento event store
  - [ ] Redirect to recipe detail page on success (PRG pattern)

- [ ] Update Recipe aggregate to handle RecipeUpdated event (AC: 5)
  - [ ] Add `recipe_updated` event handler in `crates/recipe/src/aggregate.rs`
  - [ ] Apply changes to aggregate state (update title, ingredients, instructions, etc.)
  - [ ] Ensure event sourcing maintains full history of all edits

- [ ] Create evento subscription to update read model (AC: 4, 6)
  - [ ] Implement subscription handler in `crates/recipe/src/read_model.rs`
  - [ ] On RecipeUpdated event, update `recipes` table with new values
  - [ ] Use SQLx to execute UPDATE query with parameterized values
  - [ ] Ensure read model reflects changes immediately for subsequent queries

- [ ] Handle meal plan cascading updates (AC: 6)
  - [ ] Implement evento subscription in `crates/meal_planning` to listen for RecipeUpdated
  - [ ] Update `meal_assignments` read model with refreshed recipe data (if scheduled)
  - [ ] Ensure meal calendar displays updated recipe details without user intervention

- [ ] Add confirmation message and redirect (AC: 4)
  - [ ] Display success message: "Recipe updated successfully"
  - [ ] Redirect to GET `/recipes/:id` (recipe detail page)
  - [ ] Use PRG pattern to prevent duplicate submissions

- [ ] Write unit tests for Recipe aggregate update logic (TDD)
  - [ ] Test RecipeUpdated event application
  - [ ] Test partial updates (only changed fields)
  - [ ] Test validation edge cases (empty title, no ingredients)
  - [ ] Test ownership verification (cannot update other users' recipes)

- [ ] Write integration tests for edit recipe flow (TDD)
  - [ ] Test GET /recipes/:id/edit returns pre-populated form
  - [ ] Test PUT /recipes/:id with valid data updates recipe
  - [ ] Test PUT with invalid data returns 422 with errors
  - [ ] Test unauthorized user receives 403 Forbidden
  - [ ] Test read model updated after RecipeUpdated event

- [ ] Write E2E tests for edit recipe user flow (TDD)
  - [ ] Test user navigates to edit page, modifies recipe, saves successfully
  - [ ] Test validation errors displayed inline on form
  - [ ] Test recipe detail page shows updated information after save

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- Recipe aggregate rebuilt from event stream on each load
- RecipeUpdated event stores delta (changed fields) for efficiency
- Full edit history maintained automatically via event log
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]

**CQRS Read Model Projection:**
- `recipes` table updated via evento subscription
- Subscription handler listens for RecipeUpdated events and applies changes to read model
- Ensures eventual consistency for queries
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]

**Server-Side Rendering:**
- Askama templates for type-safe HTML rendering
- Recipe edit form likely reuses `templates/pages/recipe-form.html` with mode="edit" flag
- Form validation errors rendered inline with field-specific messages
- [Source: docs/solution-architecture.md#2.2 Server-Side Rendering Strategy, lines 122-141]

**Authorization:**
- JWT auth middleware verifies user authentication
- Route handler checks ownership: `recipe.user_id == auth.user_id`
- Return 403 Forbidden if ownership check fails
- [Source: docs/solution-architecture.md#5.3 Protected Routes, lines 656-692]

**Form Validation:**
- validator crate for derive-based validation
- Server-side validation mandatory (no client-side bypass)
- Validation errors: 422 Unprocessable Entity with form re-rendered
- [Source: docs/solution-architecture.md#4.3 Form Actions and Mutations, lines 576-612]

**Meal Plan Cascading:**
- When recipe updated, meal plans referencing it must reflect changes
- Implement cross-domain evento subscription in `meal_planning` crate
- Listen for RecipeUpdated events and refresh meal_assignments read model
- [Source: docs/solution-architecture.md#11.3 Key Integrations, Inter-Domain Communication, lines 1471-1482]

### Project Structure Notes

**Codebase Alignment:**

**Route Handlers:**
- File: `src/routes/recipes.rs`
- GET `/recipes/:id/edit` - Render edit form
- PUT `/recipes/:id` - Handle recipe update
- [Source: docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 166-173]

**Domain Crate:**
- Crate: `crates/recipe/`
- Aggregate: `crates/recipe/src/aggregate.rs` (Recipe aggregate with evento)
- Commands: `crates/recipe/src/commands.rs` (UpdateRecipeCommand)
- Events: `crates/recipe/src/events.rs` (RecipeUpdated event)
- Read Model: `crates/recipe/src/read_model.rs` (evento subscription for projections)
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]

**Templates:**
- Template: `templates/pages/recipe-form.html` (shared for create and edit)
- Pass `mode: "edit"` variable to template to distinguish behavior
- Pre-populate form fields with `recipe` data from read model
- [Source: docs/solution-architecture.md#7.1 Component Structure, lines 752-819]

**Database:**
- Read Model Table: `recipes` (SQLite)
- evento Event Store: `events` table (managed automatically by evento)
- [Source: docs/solution-architecture.md#3.1 Database Schema, lines 253-382]

**Testing:**
- Unit tests: `crates/recipe/tests/aggregate_tests.rs`
- Integration tests: `tests/recipe_tests.rs` (root level)
- E2E tests: `e2e/tests/recipe-management.spec.ts` (Playwright)
- [Source: docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]

### References

- **Event Sourcing Pattern**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]
- **CQRS Read Model Projections**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]
- **Server-Side Rendering Strategy**: [docs/solution-architecture.md#2.2 Server-Side Rendering Strategy, lines 122-141]
- **Route Structure**: [docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 143-200]
- **Form Validation Pattern**: [docs/solution-architecture.md#4.3 Form Actions and Mutations, lines 576-612]
- **Authorization Middleware**: [docs/solution-architecture.md#5.3 Protected Routes, lines 656-692]
- **Domain Crate Organization**: [docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.2, lines 286-308]

## Dev Agent Record

### Context Reference

- [Story Context 2.2](../story-context-2.2.xml) - Generated 2025-10-14

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
