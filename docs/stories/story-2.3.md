# Story 2.3: Delete Recipe

Status: Approved

## Story

As a recipe owner,
I want to delete a recipe I no longer use,
so that I can keep my library organized.

## Acceptance Criteria

1. Delete button available on recipe detail page
2. Confirmation dialog displays before deletion: "Are you sure? This cannot be undone."
3. Successful deletion removes recipe from user's library
4. Deleted recipe removed from any active meal plans
5. Meal plans with deleted recipes show empty slots requiring replacement
6. Recipe count decremented (frees slot for free tier users)
7. Community ratings/reviews retained for analytics but recipe no longer discoverable
8. Soft delete maintains data integrity for audit trail

## Tasks / Subtasks

- [ ] Add delete button to recipe detail page (AC: 1)
  - [ ] Add delete button in `templates/pages/recipe-detail.html`
  - [ ] Style as danger/destructive action (red styling)
  - [ ] Position in owner actions section (visible only to recipe owner)
  - [ ] Use form with POST method (TwinSpark pattern)

- [ ] Implement confirmation dialog (AC: 2)
  - [ ] Add JavaScript confirmation via `onsubmit` attribute
  - [ ] Display message: "Are you sure? This cannot be undone."
  - [ ] Return false if user cancels, submit if confirmed
  - [ ] Progressive enhancement: works without JS (server-side check)

- [ ] Create recipe deletion route and handler (AC: 3, 6, 7, 8)
  - [ ] Add POST `/recipes/:id/delete` route in `src/routes/recipes.rs`
  - [ ] Implement ownership verification (403 if not owner)
  - [ ] Load Recipe aggregate from evento event store
  - [ ] Execute DeleteRecipeCommand
  - [ ] Emit RecipeDeleted event with soft delete pattern
  - [ ] Return 200 OK with `ts-location: /recipes` header (TwinSpark pattern)

- [ ] Implement Recipe aggregate delete handler (AC: 8)
  - [ ] Add `recipe_deleted` event handler in `crates/recipe/src/aggregate.rs`
  - [ ] Apply soft delete to aggregate state (set deleted_at timestamp)
  - [ ] Ensure event sourcing maintains audit trail

- [ ] Create evento subscription to update read model (AC: 3, 6, 7)
  - [ ] Implement `recipe_deleted_handler` in `crates/recipe/src/read_model.rs`
  - [ ] On RecipeDeleted event, update `recipes` table: SET `deleted_at = NOW(), is_deleted = TRUE`
  - [ ] Ensure read model excludes deleted recipes from queries (WHERE is_deleted = FALSE)
  - [ ] Decrement user recipe count for freemium enforcement

- [ ] Handle meal plan cascading updates (AC: 4, 5)
  - [ ] Architecture supports cross-domain evento subscriptions
  - [ ] RecipeDeleted event available for future `meal_planning` crate to subscribe
  - [ ] **Note**: meal_planning crate not yet implemented - will handle empty slots when created
  - [ ] Document integration pattern in read_model.rs

- [ ] Handle community ratings/reviews (AC: 7)
  - [ ] Soft delete preserves recipe_id in ratings table
  - [ ] Update community discovery query to exclude deleted recipes (WHERE is_deleted = FALSE)
  - [ ] Ratings remain accessible for analytics but recipe not discoverable

- [ ] Write unit tests for Recipe aggregate delete logic (TDD)
  - [ ] Test RecipeDeleted event application
  - [ ] Test soft delete sets deleted_at timestamp
  - [ ] Test ownership verification (cannot delete other users' recipes)
  - [ ] Test deleted recipes cannot be updated/deleted again

- [ ] Write integration tests for delete recipe flow (TDD)
  - [ ] Test POST /recipes/:id/delete with valid ownership succeeds
  - [ ] Test POST with unauthorized user returns 403 Forbidden
  - [ ] Test read model updated after RecipeDeleted event (is_deleted = TRUE)
  - [ ] Test deleted recipe excluded from user recipe queries
  - [ ] Test recipe count decremented for freemium users

- [ ] Write E2E tests for delete recipe user flow (TDD)
  - [ ] Test user navigates to recipe detail, clicks delete, confirms, recipe removed
  - [ ] Test confirmation dialog can be cancelled
  - [ ] Test recipe list no longer shows deleted recipe

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- Recipe aggregate rebuilt from event stream on each load
- RecipeDeleted event uses soft delete pattern (deleted_at timestamp)
- Full deletion history maintained automatically via event log
- Hard delete not permitted - violates audit trail requirements
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, ADR-001]

**CQRS Read Model Projection:**
- `recipes` table updated via evento subscription
- Subscription handler listens for RecipeDeleted events and sets `is_deleted = TRUE, deleted_at = NOW()`
- Read model queries filter deleted recipes: WHERE is_deleted = FALSE
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]

**Server-Side Rendering:**
- Askama templates for type-safe HTML rendering
- Delete button in `templates/pages/recipe-detail.html` with confirmation
- JavaScript confirmation for UX, server-side ownership check for security
- [Source: docs/solution-architecture.md#2.2 Server-Side Rendering Strategy]

**TwinSpark Pattern:**
- Use POST method for delete action (not DELETE verb)
- Success response: 200 OK with `ts-location: /recipes` header
- TwinSpark intercepts and navigates client-side
- [Source: Story 2.2 Technical Correction notes, TwinSpark Pattern Summary]

**Authorization:**
- JWT auth middleware verifies user authentication
- Route handler checks ownership: `recipe.user_id == auth.user_id`
- Return 403 Forbidden if ownership check fails
- Structured logging for security events (deletion attempts)
- [Source: docs/solution-architecture.md#5.3 Protected Routes]

**Soft Delete Pattern:**
- Never hard delete from database (preserves data integrity)
- Set `deleted_at` timestamp and `is_deleted = TRUE` flag
- Exclude from queries via WHERE clause
- Enables recovery and audit trail
- [Source: docs/tech-spec-epic-2.md#AC-2.3]

**Meal Plan Cascading:**
- When recipe deleted, meal plans referencing it must handle empty slots
- Implement cross-domain evento subscription in `meal_planning` crate (future)
- Listen for RecipeDeleted events and mark slots as requiring replacement
- [Source: docs/solution-architecture.md#11.3 Key Integrations, Inter-Domain Communication]

**Freemium Enforcement:**
- Recipe count decremented when deleted (frees slot)
- Free tier users can create new recipe if count < 10 after deletion
- Read model tracks: COUNT(*) WHERE user_id = ? AND is_deleted = FALSE
- [Source: docs/PRD.md#FR-15, docs/tech-spec-epic-2.md#Freemium Controls]

### Project Structure Notes

**Codebase Alignment:**

**Route Handlers:**
- File: `src/routes/recipes.rs`
- POST `/recipes/:id/delete` - Handle recipe deletion
- Use POST (not DELETE verb) per TwinSpark pattern
- [Source: docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 166-173]

**Domain Crate:**
- Crate: `crates/recipe/`
- Aggregate: `crates/recipe/src/aggregate.rs` (Recipe aggregate with evento)
- Commands: `crates/recipe/src/commands.rs` (DeleteRecipeCommand)
- Events: `crates/recipe/src/events.rs` (RecipeDeleted event)
- Read Model: `crates/recipe/src/read_model.rs` (evento subscription for projections)
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]

**Templates:**
- Template: `templates/pages/recipe-detail.html`
- Add delete button with confirmation
- Display only for recipe owner (conditional rendering)
- [Source: docs/solution-architecture.md#7.1 Component Structure]

**Database:**
- Read Model Table: `recipes` with `is_deleted BOOLEAN DEFAULT FALSE`, `deleted_at TEXT`
- evento Event Store: `events` table (managed automatically by evento)
- [Source: docs/solution-architecture.md#3.1 Database Schema, lines 276-318]

**Testing:**
- Unit tests: `crates/recipe/tests/aggregate_tests.rs`
- Integration tests: `tests/recipe_integration_tests.rs` (root level)
- E2E tests: `e2e/tests/recipe-management.spec.ts` (Playwright)
- [Source: docs/solution-architecture.md#15 Testing Strategy]

**Lessons from Story 2.2:**
- Use POST method for all mutations (not PUT/DELETE verbs)
- Success response: 200 OK + `ts-location` header (TwinSpark pattern)
- Structured logging for security events (include user_id, recipe_id, event fields)
- Explicit error handling (no silent failures)
- Document cross-domain integration patterns
- Write tests first (TDD) before implementation
- [Source: Story 2.2 completion notes, Technical Correction section]

### References

- **Event Sourcing Pattern**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]
- **CQRS Read Model Projections**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]
- **Server-Side Rendering Strategy**: [docs/solution-architecture.md#2.2 Server-Side Rendering Strategy, lines 122-141]
- **Route Structure**: [docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 143-200]
- **Authorization Middleware**: [docs/solution-architecture.md#5.3 Protected Routes, lines 656-692]
- **Domain Crate Organization**: [docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]
- **Soft Delete Pattern**: [docs/tech-spec-epic-2.md#AC-2.3, lines 1963-1968]
- **TwinSpark Pattern**: [Story 2.2 Technical Correction, TwinSpark Pattern Summary]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.3, lines 310-331]
- **Technical Specification**: [docs/tech-spec-epic-2.md#AC-2.3]

## Dev Agent Record

### Context Reference

- [Story Context 2.3](../story-context-2.3.xml) - Generated 2025-10-14

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
