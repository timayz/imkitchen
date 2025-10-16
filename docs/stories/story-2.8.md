# Story 2.8: Community Recipe Discovery

Status: Approved

## Story

As a user,
I want to browse recipes shared by other users,
so that I can expand my culinary repertoire and discover new meal ideas.

## Acceptance Criteria

1. "Discover Recipes" page accessible at GET /discover (public, no auth required)
2. Recipes displayed in card view with: title, image, rating, creator name, complexity, cuisine, prep time
3. Only shared recipes displayed (WHERE is_shared = TRUE AND deleted_at IS NULL)
4. Filter controls available: rating (All, 4+ stars, 3+ stars), cuisine type, preparation time (<30min, 30-60min, >60min), dietary preferences (vegetarian, vegan, gluten-free)
5. Search by recipe title or ingredient name (text input with submit)
6. Sorting options: highest rated, most recent, alphabetical
7. Pagination with 20 recipes per page
8. Clicking recipe card opens detail view at GET /discover/:id
9. Recipe detail shows full recipe with attribution ("By {creator_email}")
10. Community recipes read-only for non-owners (no edit/delete buttons)
11. "Add to My Recipes" button visible on recipe detail (copies to user's library)
12. Guest users can browse /discover without authentication
13. Guest users see "Sign Up to Add Recipes" button instead of "Add to My Recipes"
14. SEO meta tags included: title, description, Open Graph tags
15. Schema.org Recipe JSON-LD markup for rich snippets in search results

## Tasks / Subtasks

- [ ] Implement /discover route handler (AC: 1, 2, 3, 12)
  - [ ] Create `src/routes/discover.rs` module
  - [ ] Implement `get_discover` handler (public access, no auth middleware)
  - [ ] Query `recipes` table: `WHERE is_shared = 1 AND deleted_at IS NULL`
  - [ ] LEFT JOIN `users` table for creator email attribution
  - [ ] Map results to `CommunityRecipeView` struct
  - [ ] Render `templates/pages/discover.html` with recipe list
  - [ ] Register route in `src/main.rs`: `.route("/discover", get(get_discover))`

- [ ] Implement filtering logic (AC: 4, 5, 6, 7)
  - [ ] Define `DiscoveryParams` query struct (cuisine, min_rating, max_prep_time, dietary, search, sort, page)
  - [ ] Build dynamic SQL WHERE clauses based on filter params
  - [ ] Cuisine filter: `WHERE cuisine = ?` (dropdown: Italian, Asian, Mexican, etc.)
  - [ ] Rating filter: `WHERE avg_rating >= ?` (3 or 4)
  - [ ] Prep time filter: `WHERE (prep_time_min + cook_time_min) <= ?` (30, 60, 999)
  - [ ] Dietary filter: `WHERE dietary_tags LIKE '%vegetarian%'` (JSON text search)
  - [ ] Search filter: `WHERE title LIKE '%?%' OR ingredients LIKE '%?%'`
  - [ ] Sorting: ORDER BY avg_rating DESC, created_at DESC, title ASC
  - [ ] Pagination: LIMIT 20 OFFSET (page-1)*20

- [ ] Create discover.html template (AC: 2, 8, 13)
  - [ ] Create `templates/pages/discover.html`
  - [ ] Header: "Discover Community Recipes"
  - [ ] Filter controls as form with dropdowns and text input
  - [ ] Recipe grid: responsive layout (1-3 columns based on screen size)
  - [ ] Recipe card component: image, title, creator email, rating stars, complexity badge, cuisine tag, prep time
  - [ ] Pagination controls: Previous/Next buttons with page numbers
  - [ ] Empty state message: "No recipes found. Be the first to share!"
  - [ ] Guest user conditional: {% if auth %} show "Add" button {% else %} show "Sign Up" {% endif %}

- [ ] Implement /discover/:id route handler (AC: 8, 9, 10, 11)
  - [ ] Create `get_discover_detail` handler (public access)
  - [ ] Query recipe by ID: `WHERE id = ? AND is_shared = 1 AND deleted_at IS NULL`
  - [ ] Return 404 if recipe not found or not shared
  - [ ] LEFT JOIN users for creator attribution
  - [ ] Render `templates/pages/discover-detail.html` with full recipe
  - [ ] Show "Add to My Recipes" button if authenticated
  - [ ] Hide edit/delete buttons (is_owner = false)
  - [ ] Register route: `.route("/discover/:id", get(get_discover_detail))`

- [ ] Implement "Add to My Recipes" functionality (AC: 11)
  - [ ] Create POST /discover/:id/add handler (requires auth)
  - [ ] Load source recipe aggregate from event stream
  - [ ] Verify is_shared = true
  - [ ] Create new recipe via `create_recipe` command with copied data
  - [ ] Check freemium limit (10 recipes for free tier)
  - [ ] Append "(from community)" to title for attribution
  - [ ] Redirect to /recipes/:new_id on success
  - [ ] Return 401 if not authenticated, 403 if limit reached

- [ ] Add SEO meta tags and structured data (AC: 14, 15)
  - [ ] Add Open Graph meta tags to discover-detail.html:
    - og:title = recipe title
    - og:description = recipe instructions preview (first 150 chars)
    - og:image = recipe.image_url
    - og:url = https://imkitchen.app/discover/:id
  - [ ] Add Schema.org Recipe JSON-LD script:
    - @type: Recipe
    - name: recipe title
    - author: creator email
    - prepTime: PT{prep_time_min}M
    - cookTime: PT{cook_time_min}M
    - recipeIngredient: array of ingredient strings
    - recipeInstructions: array of instruction strings
    - aggregateRating: average rating and review count
  - [ ] Add robots meta tag: index, follow (allow crawling)
  - [ ] Add canonical URL meta tag

- [ ] Write unit tests for read model queries (TDD)
  - [ ] Test list_shared_recipes query returns only is_shared = true recipes
  - [ ] Test list_shared_recipes excludes deleted recipes (deleted_at IS NULL)
  - [ ] Test cuisine filter returns matching recipes
  - [ ] Test rating filter returns recipes >= min_rating
  - [ ] Test prep time filter returns recipes <= max_prep_time
  - [ ] Test dietary filter matches dietary_tags JSON field
  - [ ] Test search filter matches title and ingredients
  - [ ] Test pagination (LIMIT/OFFSET) returns correct page
  - [ ] Test sorting by rating, date, title

- [ ] Write integration tests for discovery routes (TDD)
  - [ ] Test GET /discover returns 200 OK with HTML
  - [ ] Test GET /discover excludes private recipes
  - [ ] Test GET /discover/:id returns 200 OK for shared recipe
  - [ ] Test GET /discover/:id returns 404 for private recipe
  - [ ] Test GET /discover/:id returns 404 for deleted recipe
  - [ ] Test POST /discover/:id/add creates new recipe for authenticated user
  - [ ] Test POST /discover/:id/add returns 401 for unauthenticated user
  - [ ] Test POST /discover/:id/add enforces freemium limit (10 recipes)
  - [ ] Test filtering by cuisine, rating, prep time, dietary
  - [ ] Test search by recipe title
  - [ ] Test pagination with multiple pages

## Dev Notes

### Architecture Patterns and Constraints

**Public Routes (No Auth Required):**
- GET /discover and GET /discover/:id are public for guest access and SEO crawling
- Authentication optional (auth middleware not applied to these routes)
- "Add to My Recipes" action requires authentication (POST /discover/:id/add uses auth middleware)
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1537-1581]

**CQRS Read Model Query:**
- Query `recipes` table directly (read model projection from RecipeAggregate events)
- Filter: `WHERE is_shared = 1 AND deleted_at IS NULL`
- LEFT JOIN `users` table for creator attribution (creator_email field)
- Pagination: LIMIT 20 OFFSET (page-1)*20
- [Source: docs/tech-spec-epic-2.md#Database Schema, lines 979-1006]

**Filtering and Search:**
- Dynamic SQL WHERE clauses built from query params
- Cuisine: exact match on cuisine column
- Rating: >= min_rating (requires aggregate rating calculation - Story 2.9 dependency)
- Prep time: (prep_time_min + cook_time_min) <= max_prep_time
- Dietary: JSON text search on dietary_tags column (LIKE '%vegetarian%')
- Search: title LIKE '%query%' OR ingredients LIKE '%query%'
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1542-1553]

**Add to Library Pattern:**
- Copy recipe data from shared recipe to new RecipeAggregate
- Invoke `create_recipe` command with copied data
- New recipe owned by authenticated user (user_id = auth.user_id)
- Copied recipe defaults to private (is_shared = false)
- Original recipe unmodified (immutable event sourcing)
- [Source: docs/tech-spec-epic-2.md#HTTP Routes, lines 914-947]

**SEO Optimization:**
- Open Graph meta tags for social sharing previews
- Schema.org Recipe JSON-LD for Google rich snippets
- Server-rendered HTML (no client-side rendering) for crawler access
- Public routes (no auth wall) allow search engine indexing
- robots.txt allows /discover/* (configured separately)
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1575-1581]

**Guest vs. Authenticated User Experience:**
- Guests can browse /discover and view recipe details (read-only)
- Guests see "Sign Up to Add Recipes" CTA instead of "Add to My Recipes" button
- Authenticated users see "Add to My Recipes" button
- POST /discover/:id/add requires JWT (auth middleware enforces)
- [Source: docs/tech-spec-epic-2.md#Community Discovery Workflow, lines 1537-1573]

### Project Structure Notes

**Codebase Alignment:**

**HTTP Routes:**
- Create new module: `src/routes/discover.rs`
- Handlers: `get_discover` (community feed), `get_discover_detail` (recipe detail), `post_add_to_library` (copy recipe)
- Register routes in `src/main.rs`:
  - `.route("/discover", get(get_discover))` (no auth middleware)
  - `.route("/discover/:id", get(get_discover_detail))` (no auth middleware)
  - `.route("/discover/:id/add", post(post_add_to_library))` (with auth middleware)
- Export handlers from `src/routes/mod.rs`
- [Source: docs/tech-spec-epic-2.md#HTTP Routes, lines 849-969]

**Read Model Queries:**
- File: `crates/recipe/src/read_model.rs`
- Add query function: `list_shared_recipes` with filters (cuisine, min_rating, max_prep_time, dietary, search, sort, page)
- Query: `SELECT r.*, u.email as creator_email FROM recipes r LEFT JOIN users u ON r.user_id = u.id WHERE r.is_shared = 1 AND r.deleted_at IS NULL`
- Apply dynamic WHERE clauses based on filter params
- Return `Vec<CommunityRecipeView>` struct with recipe data + creator_email
- [Source: docs/tech-spec-epic-2.md#Community Discovery Routes, lines 872-888]

**Templates:**
- Create: `templates/pages/discover.html` (community feed with filters and recipe grid)
- Create: `templates/pages/discover-detail.html` (recipe detail with SEO tags and "Add to My Recipes" button)
- Use existing `templates/components/recipe-card.html` (extend for community view with creator attribution)
- [Source: docs/solution-architecture.md#7.1 Component Structure, lines 1756-1800]

**Database Schema:**
- Table: `recipes` (existing - no migration needed)
- Columns used: id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min, image_url, complexity, cuisine, dietary_tags, is_shared, deleted_at
- Index: `idx_recipes_shared` on (is_shared) WHERE is_shared = TRUE AND deleted_at IS NULL (existing)
- [Source: docs/tech-spec-epic-2.md#Database Schema, lines 979-1006]

**Testing:**
- Unit tests: `crates/recipe/tests/recipe_tests.rs` (extend with list_shared_recipes query tests)
- Integration tests: Create `tests/community_tests.rs` (new file for /discover route tests)
- E2E tests: `e2e/tests/community.spec.ts` (guest browsing, recipe discovery, add to library flow)
- [Source: docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]

**Lessons from Previous Stories:**
- Story 2.7 implemented `/discover` route with is_shared filtering (build on existing route)
- Use `Optional<Auth>` extractor for optional authentication (guest vs. authenticated users)
- Use TwinSpark for live filter updates without full page reload (progressive enhancement)
- Test soft delete filtering (deleted_at IS NULL) in all queries
- Write tests first (TDD) - integration tests for /discover routes before implementation
- [Source: docs/stories/story-2.7.md#Dev Agent Record, lines 230-380]

### References

- **Community Discovery Workflow**: [docs/tech-spec-epic-2.md#Community Recipe Discovery Workflow, lines 1534-1581]
- **HTTP Routes**: [docs/tech-spec-epic-2.md#Community Discovery Routes, lines 849-969]
- **Database Schema**: [docs/tech-spec-epic-2.md#Database Schema, lines 979-1006]
- **SEO Optimization**: [docs/tech-spec-epic-2.md#SEO Optimization, lines 1575-1581]
- **Privacy Controls**: [docs/tech-spec-epic-2.md#Security, lines 1692-1702]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.8, lines 430-451]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]
- **Solution Architecture**: [docs/solution-architecture.md#2.3 Page Routing and Navigation, lines 145-202]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.8.xml` - Generated on 2025-10-15 - Comprehensive story context with code references, documentation snippets, architecture constraints, interfaces, and testing guidance

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
