# Story 2.4: Organize Recipes into Collections

Status: ContextReadyDraft

## Story

As a user with multiple recipes,
I want to organize them into collections,
so that I can find related recipes easily.

## Acceptance Criteria

1. Collections management page displays all user collections
2. User can create new collection with name and optional description
3. User can add/remove recipes to/from collections
4. Recipe can belong to multiple collections
5. Collections displayed in recipe library sidebar for filtering
6. Clicking collection filters recipe list to show only that collection
7. Default "All Recipes" view shows uncategorized and all collections
8. Collections deletable (removes collection but not recipes)

## Tasks / Subtasks

- [ ] Create Collection aggregate and domain model (AC: 1, 2, 8)
  - [ ] Define `CollectionAggregate` in `crates/recipe/src/collections.rs`
  - [ ] Implement evento event handlers: `collection_created`, `collection_updated`, `collection_deleted`
  - [ ] Define events: `CollectionCreated`, `CollectionUpdated`, `CollectionDeleted`
  - [ ] Validate collection name (min 3 chars, max 100 chars)
  - [ ] Ensure soft delete maintains data integrity

- [ ] Create read model tables for collections (AC: 1, 3, 4)
  - [ ] Migration: `recipe_collections` table (id, user_id, name, description, created_at)
  - [ ] Migration: `recipe_collection_assignments` table (collection_id, recipe_id, assigned_at)
  - [ ] Many-to-many relationship enables recipes in multiple collections
  - [ ] Indexes: user_id, collection_id, recipe_id for fast filtering

- [ ] Implement evento subscription handlers (AC: 1, 3)
  - [ ] Subscription: `CollectionCreated` → Insert into `recipe_collections` table
  - [ ] Subscription: `RecipeAddedToCollection` → Insert into `recipe_collection_assignments`
  - [ ] Subscription: `RecipeRemovedFromCollection` → Delete from `recipe_collection_assignments`
  - [ ] Subscription: `CollectionDeleted` → Soft delete collection + remove assignments

- [ ] Create collections management page (AC: 1, 2, 8)
  - [ ] Template: `templates/pages/collections.html`
  - [ ] Display list of user's collections with name, description, recipe count
  - [ ] "Create Collection" form with name and description fields
  - [ ] Delete button per collection with confirmation dialog
  - [ ] Responsive design: mobile (list), desktop (grid)

- [ ] Implement collection CRUD routes (AC: 2, 8)
  - [ ] Route: POST `/collections` - Create new collection
  - [ ] Route: PUT `/collections/:id` - Update collection name/description
  - [ ] Route: DELETE `/collections/:id` - Delete collection (soft delete)
  - [ ] Ownership verification (user can only manage their own collections)
  - [ ] Structured logging for collection operations

- [ ] Implement recipe-collection assignment routes (AC: 3, 4)
  - [ ] Route: POST `/collections/:id/recipes/:recipe_id` - Add recipe to collection
  - [ ] Route: DELETE `/collections/:id/recipes/:recipe_id` - Remove recipe from collection
  - [ ] Validate recipe ownership (user can only assign their own recipes)
  - [ ] Handle duplicate assignments gracefully (idempotent operation)

- [ ] Add collection sidebar filtering to recipe library (AC: 5, 6, 7)
  - [ ] Update `templates/pages/recipe-list.html` with sidebar
  - [ ] Sidebar displays: "All Recipes" (default) + user's collections
  - [ ] Each collection shows recipe count in parentheses
  - [ ] Clicking collection filters recipe list to show only recipes in that collection
  - [ ] Active filter highlighted in sidebar
  - [ ] URL param: `?collection=:id` for bookmarkable filtered views
  - [ ] "All Recipes" shows all non-deleted recipes (uncategorized + all collections)

- [ ] Add collection assignment UI on recipe detail page (AC: 3, 4)
  - [ ] Update `templates/pages/recipe-detail.html`
  - [ ] "Manage Collections" section with checkboxes for each collection
  - [ ] Checked = recipe in collection, unchecked = not in collection
  - [ ] Toggle checkboxes to add/remove recipe from collections
  - [ ] TwinSpark AJAX updates without page reload
  - [ ] Display current collections: "In Collections: Favorites, Weeknight Meals"

- [ ] Write unit tests for Collection aggregate (TDD)
  - [ ] Test CollectionCreated event application
  - [ ] Test collection name validation (min/max length)
  - [ ] Test ownership verification (user can only delete own collections)
  - [ ] Test recipe assignment/unassignment to collections
  - [ ] Test collection deletion preserves recipes

- [ ] Write integration tests for collection CRUD (TDD)
  - [ ] Test POST /collections creates collection and read model syncs
  - [ ] Test DELETE /collections soft deletes and removes assignments
  - [ ] Test POST /collections/:id/recipes/:recipe_id adds recipe to collection
  - [ ] Test DELETE /collections/:id/recipes/:recipe_id removes recipe from collection
  - [ ] Test unauthorized collection access returns 403

- [ ] Write E2E tests for collection management flow (TDD)
  - [ ] Test user creates collection, adds recipes, views filtered list
  - [ ] Test recipe appears in multiple collections
  - [ ] Test deleting collection does not delete recipes
  - [ ] Test sidebar filtering shows correct recipes per collection

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- Collection aggregate rebuilt from event stream on each load
- CollectionCreated, RecipeAddedToCollection, RecipeRemovedFromCollection events
- Full deletion and assignment history maintained automatically via event log
- Soft delete for collections (deleted_at timestamp)
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, ADR-001]

**CQRS Read Model Projection:**
- `recipe_collections` table updated via evento subscription
- `recipe_collection_assignments` many-to-many table for filtering
- Subscription handlers listen for collection events and sync read models
- Read model queries filter by collection_id for fast recipe list filtering
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]

**Many-to-Many Relationship:**
- Recipe can belong to multiple collections (no exclusivity constraint)
- Assignment table stores: collection_id, recipe_id, assigned_at timestamp
- Deletion of collection removes assignments but preserves recipes
- Deletion of recipe cascades to remove assignments
- [Source: docs/tech-spec-epic-2.md#Collection Management, lines 130-145]

**Server-Side Rendering:**
- Askama templates for type-safe HTML rendering
- Collections management page with forms for create/update
- Sidebar filtering in recipe library with active state highlighting
- [Source: docs/solution-architecture.md#2.2 Server-Side Rendering Strategy]

**TwinSpark Pattern:**
- Use POST method for collection creation/assignment (not PUT/DELETE verbs)
- Success response: 200 OK with HTML fragment or ts-location header
- TwinSpark intercepts and updates UI without page reload
- Recipe detail page collection checkboxes update via AJAX
- [Source: Story 2.2 Technical Correction notes, TwinSpark Pattern Summary]

**Authorization:**
- JWT auth middleware verifies user authentication
- Route handlers check ownership: collection.user_id == auth.user_id
- User can only assign their own recipes to collections
- Return 403 Forbidden if ownership check fails
- [Source: docs/solution-architecture.md#5.3 Protected Routes]

**Soft Delete Pattern:**
- Never hard delete collections from database (preserves data integrity)
- Set `deleted_at` timestamp on collection
- Remove assignments when collection deleted
- Enables audit trail and potential recovery
- [Source: docs/tech-spec-epic-2.md, Story 2.3 patterns]

**Filtering and Query Optimization:**
- Collection filtering via query param: ?collection=:id
- Read model query: SELECT recipes WHERE recipe_id IN (SELECT recipe_id FROM recipe_collection_assignments WHERE collection_id = ?)
- Indexed queries for performance (collection_id, recipe_id)
- "All Recipes" shows union of all non-deleted recipes
- [Source: docs/solution-architecture.md#8.4 Database Performance]

### Project Structure Notes

**Codebase Alignment:**

**Domain Crate:**
- Crate: `crates/recipe/`
- Collections Module: `crates/recipe/src/collections.rs` (Collection aggregate with evento)
- Aggregate: `CollectionAggregate` with ownership and name validation
- Commands: `CreateCollection`, `UpdateCollection`, `DeleteCollection`, `AddRecipeToCollection`, `RemoveRecipeFromCollection`
- Events: `CollectionCreated`, `CollectionUpdated`, `CollectionDeleted`, `RecipeAddedToCollection`, `RecipeRemovedFromCollection`
- Read Model: `crates/recipe/src/read_model.rs` (subscription handlers for projections)
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]

**Route Handlers:**
- File: `src/routes/collections.rs` (new file)
- Routes: POST/PUT/DELETE `/collections`, POST/DELETE `/collections/:id/recipes/:recipe_id`
- Export routes in `src/routes/mod.rs`
- Register routes in `src/main.rs`
- [Source: docs/solution-architecture.md#2.3 Page Routing and Navigation]

**Templates:**
- New Template: `templates/pages/collections.html` (collections management page)
- Update Template: `templates/pages/recipe-list.html` (add sidebar filtering)
- Update Template: `templates/pages/recipe-detail.html` (add collection assignment UI)
- [Source: docs/solution-architecture.md#7.1 Component Structure]

**Database:**
- New Migration: `migrations/008_create_recipe_collections_table.sql`
- Tables: `recipe_collections` (id, user_id, name, description, deleted_at, created_at)
- Tables: `recipe_collection_assignments` (collection_id, recipe_id, assigned_at)
- Indexes: collection_id, recipe_id, user_id for fast filtering
- [Source: docs/solution-architecture.md#3.1 Database Schema]

**Testing:**
- Unit tests: `crates/recipe/tests/collection_tests.rs`
- Integration tests: `tests/collection_integration_tests.rs` (root level)
- E2E tests: `e2e/tests/recipe-management.spec.ts` (Playwright - extend existing)
- [Source: docs/solution-architecture.md#15 Testing Strategy]

**Lessons from Story 2.3:**
- Use POST method for all mutations (not PUT/DELETE verbs)
- Success response: 200 OK + `ts-location` header (TwinSpark pattern)
- Structured logging for all operations (include user_id, collection_id, recipe_id, event fields)
- Explicit error handling (no silent failures)
- Write tests first (TDD) before implementation
- Document cross-domain integration patterns
- [Source: Story 2.3 completion notes, Technical Correction section]

### References

- **Event Sourcing Pattern**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]
- **CQRS Read Model Projections**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]
- **Server-Side Rendering Strategy**: [docs/solution-architecture.md#2.2 Server-Side Rendering Strategy, lines 122-141]
- **Route Structure**: [docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 143-200]
- **Authorization Middleware**: [docs/solution-architecture.md#5.3 Protected Routes, lines 656-692]
- **Domain Crate Organization**: [docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]
- **Collection Management**: [docs/tech-spec-epic-2.md#Collection Management, lines 120-160]
- **Many-to-Many Relationships**: [docs/tech-spec-epic-2.md#Database Schema, lines 130-145]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.4, lines 334-357]
- **Technical Specification**: [docs/tech-spec-epic-2.md#Story 2.4]

## Dev Agent Record

### Context Reference

- [Story Context 2.4](../story-context-2.4.xml) - Generated 2025-10-14

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
