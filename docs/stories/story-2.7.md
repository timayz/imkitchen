# Story 2.7: Share Recipe to Community

Status: Ready

## Story

As a recipe owner,
I want to share my recipe with the community,
so that others can discover and use it.

## Acceptance Criteria

1. "Share to Community" toggle visible on recipe edit page
2. Toggle changes privacy from "private" to "shared" (RecipeShared event)
3. Shared recipes appear in community discovery feed (`/discover` route)
4. Recipe attribution displays creator's username on community pages
5. Shared recipes remain editable only by owner
6. Owner can revert to private at any time (removes from community discovery)
7. Ratings and reviews visible only on shared recipes
8. User profile shows count of shared recipes
9. New recipes default to private (`is_shared = false`)
10. Direct URL to private recipe returns 404 for non-owners
11. Shared recipes excluded from owner's personal recipe limit enforcement
12. Community feed filters shared recipes (`is_shared = true AND deleted_at IS NULL`)

## Tasks / Subtasks

- [ ] Implement ShareRecipe command and event (AC: 1, 2, 6)
  - [ ] Verify `is_shared` boolean field exists in RecipeAggregate (should already exist)
  - [ ] Define `RecipeShared` event in `crates/recipe/src/events.rs` (should already exist)
  - [ ] Implement `recipe_shared` event handler in RecipeAggregate (should already exist)
  - [ ] Create `share_recipe` command function in `crates/recipe/src/commands.rs`
  - [ ] Ownership verification: verify user_id matches recipe.user_id
  - [ ] Toggle logic: takes boolean parameter `shared` (true = share, false = unshare)
  - [ ] Emit RecipeShared event with shared boolean value

- [ ] Update read model for share status (AC: 3, 9, 12)
  - [ ] Verify `recipes` table has `is_shared` boolean column (should already exist)
  - [ ] Verify default value `is_shared = FALSE` for new recipes
  - [ ] Create evento subscription handler for RecipeShared event (project_recipe_shared)
  - [ ] Project share status to read model on event
  - [ ] Index: `CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = TRUE AND deleted_at IS NULL` (verify exists)

- [ ] Add share toggle UI to recipe edit page (AC: 1)
  - [ ] Update `templates/pages/recipe-edit.html` (or recipe-form.html)
  - [ ] Add checkbox/toggle switch for "Share to Community"
  - [ ] Label: "Share this recipe with the community"
  - [ ] Helper text: "Shared recipes are visible to all users and can be rated"
  - [ ] Toggle state reflects current `is_shared` value
  - [ ] Submit form includes `is_shared` boolean parameter

- [ ] Implement share toggle route (AC: 2, 6)
  - [ ] Route: POST /recipes/:id/share
  - [ ] Handler: `post_share_recipe` in `src/routes/recipes.rs`
  - [ ] Ownership check: verify user owns recipe
  - [ ] Accept `shared` boolean in form data
  - [ ] Call `share_recipe` command with recipe_id, user_id, shared value
  - [ ] Return confirmation message or redirect to recipe detail

- [ ] Community discovery feed filtering (AC: 3, 10, 12)
  - [ ] Verify GET /discover route exists (should be implemented or planned)
  - [ ] Query filters: `WHERE is_shared = TRUE AND deleted_at IS NULL`
  - [ ] Public access (no authentication required for read-only browse)
  - [ ] Pagination: 20 recipes per page
  - [ ] Direct URL to private recipe (/recipes/:id) returns 404 if user not owner

- [ ] Recipe attribution display (AC: 4)
  - [ ] Community recipe cards show creator username (from users.email or username)
  - [ ] Community recipe detail page shows "By [creator name]"
  - [ ] Attribution persists even if recipe unshared later (for analytics)
  - [ ] Join query: `LEFT JOIN users ON recipes.user_id = users.id`

- [ ] Ownership enforcement for editing (AC: 5)
  - [ ] Verify existing edit route checks: `recipe.user_id == auth_user.id`
  - [ ] Shared recipes editable only by owner (not by community users)
  - [ ] Community users see read-only view with "Add to My Recipes" option
  - [ ] Edit button hidden for non-owners on shared recipe pages

- [ ] Profile shared recipe count (AC: 8)
  - [ ] Add shared_count query: `SELECT COUNT(*) FROM recipes WHERE user_id = ? AND is_shared = TRUE AND deleted_at IS NULL`
  - [ ] Display count on profile page: "X Shared Recipes"
  - [ ] Optional: Add shared_count to users table with evento subscription (performance optimization)

- [ ] Ratings visibility (AC: 7)
  - [ ] Rating widget only shown on shared recipes
  - [ ] Private recipes: hide rating/review section entirely
  - [ ] Conditional render in template: `{% if recipe.is_shared %} ... rating widget ... {% endif %}`
  - [ ] Document integration point for future Story 2.9 (Rating implementation)

- [ ] Write unit tests for share command (TDD)
  - [ ] Test RecipeShared event emitted when sharing recipe
  - [ ] Test RecipeShared event emitted when unsharing recipe (shared = false)
  - [ ] Test ownership verification (PermissionDenied for non-owners)
  - [ ] Test RecipeShared event applied to aggregate state (is_shared = true/false)
  - [ ] Test NotFound error for non-existent recipes

- [ ] Write integration tests for share toggle (TDD)
  - [ ] Test POST /recipes/:id/share toggles is_shared in read model
  - [ ] Test GET /discover returns only shared recipes (is_shared = true)
  - [ ] Test GET /recipes/:id returns 404 for private recipes (non-owner access)
  - [ ] Test shared recipe count in profile page query
  - [ ] Test unauthorized share attempt returns PermissionDenied error

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- RecipeShared event emitted on share/unshare toggle
- Event stores: recipe_id, shared (boolean), user_id metadata
- Aggregate rebuilds share status from event stream
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, ADR-001]

**CQRS Read Model Projection:**
- `recipes.is_shared` boolean column updated via evento subscription
- Read model query filters by is_shared for community discovery
- Indexed query: `idx_recipes_shared` on (is_shared) WHERE is_shared = TRUE
- [Source: docs/tech-spec-epic-2.md#Database Schema, lines 991-1006]

**Share Command Pattern:**
- Load recipe aggregate from event stream
- Verify ownership: recipe.user_id == auth_user.id
- Emit RecipeShared event with shared boolean parameter
- Single command handles both share and unshare (toggle via boolean)
- [Source: docs/tech-spec-epic-2.md#Command Handlers, lines 513-530]

**Community Discovery Query:**
- Public route GET /discover (no authentication required)
- Filter: `WHERE is_shared = TRUE AND deleted_at IS NULL`
- Join with users table for creator attribution
- Pagination for scalability (20 recipes per page)
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1537-1581]

**Privacy Controls:**
- Default `is_shared = false` on recipe creation
- Private recipes excluded from /discover queries
- Direct URL access to private recipe returns 404 if not owner
- Owner can toggle share status anytime (bidirectional)
- [Source: docs/tech-spec-epic-2.md#Security and Privacy, lines 1691-1702]

**SEO Optimization (Future Enhancement):**
- Community recipe pages server-rendered for SEO crawlers
- Open Graph meta tags for social sharing
- Schema.org Recipe markup for Google rich snippets
- Sitemap includes all shared recipes
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1574-1581]

### Project Structure Notes

**Codebase Alignment:**

**Domain Crate:**
- Crate: `crates/recipe/`
- Aggregate: `RecipeAggregate` in `crates/recipe/src/aggregate.rs`
- Field: `pub is_shared: bool` (should already exist from template)
- Event: `RecipeShared` in `crates/recipe/src/events.rs` (should already exist)
- Event Handler: `recipe_shared` in `crates/recipe/src/aggregate.rs`
- Command: `share_recipe` function in `crates/recipe/src/commands.rs`
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]

**Read Model:**
- Table: `recipes` (existing)
- Column: `is_shared BOOLEAN DEFAULT FALSE` (verify exists)
- Index: `idx_recipes_shared ON recipes(is_shared) WHERE is_shared = TRUE AND deleted_at IS NULL`
- Subscription Handler: `project_recipe_shared` in `crates/recipe/src/read_model.rs`
- Query Function: `list_shared_recipes` with filters (cuisine, rating, prep_time, dietary)
- [Source: docs/tech-spec-epic-2.md#Database Schema, lines 991-1006]

**Route Handlers:**
- File: `src/routes/recipes.rs`
- Route: POST `/recipes/:id/share` (share_recipe_handler)
- File: `src/routes/discover.rs` (public community routes)
- Route: GET `/discover` (community_feed_handler)
- Route: GET `/discover/:id` (community_recipe_detail_handler)
- [Source: docs/tech-spec-epic-2.md#HTTP Routes, lines 710-714]

**Templates:**
- Update: `templates/pages/recipe-edit.html` (add share toggle checkbox)
- Create/Update: `templates/pages/community-feed.html` (community discovery feed)
- Create/Update: `templates/pages/community-recipe-detail.html` (public recipe view)
- Update: `templates/pages/profile.html` (add shared recipe count display)
- [Source: docs/solution-architecture.md#7.1 Component Structure]

**Testing:**
- Unit tests: `crates/recipe/tests/recipe_tests.rs` (extend with share tests)
- Integration tests: `tests/recipe_integration_tests.rs` (extend with share toggle, discovery feed tests)
- E2E tests: `e2e/tests/recipe-management.spec.ts` (extend with share flow)
- [Source: docs/solution-architecture.md#15 Testing Strategy]

**Lessons from Previous Stories:**
- Use POST method for share toggle (not PUT)
- Return HTML fragment for TwinSpark swap on partial updates
- Structured logging: include user_id, recipe_id, shared status
- Write tests first (TDD)
- Verify evento subscription registration in main.rs
- Use `unsafe_oneshot` for synchronous projections in tests (evento 1.4)
- [Source: Story 2.5, Story 2.6 completion notes]

### References

- **Event Sourcing Pattern**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]
- **CQRS Read Model Projections**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]
- **Recipe Domain**: [docs/tech-spec-epic-2.md#Domain Logic, lines 194-341]
- **Share Recipe Command**: [docs/tech-spec-epic-2.md#Command Handlers, lines 513-530]
- **Database Schema**: [docs/tech-spec-epic-2.md#Database Schema, lines 991-1006]
- **Community Discovery Routes**: [docs/tech-spec-epic-2.md#HTTP Routes, lines 710-714, 869-900]
- **Privacy Controls**: [docs/tech-spec-epic-2.md#Security and Privacy, lines 1691-1702]
- **Community Discovery Workflow**: [docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1537-1581]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.7, lines 406-427]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.7.xml` (Generated: 2025-10-15)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
