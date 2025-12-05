# Story 5.2: Community Recipe Browse & Discovery

Status: drafted

## Story

As a user,
I want to browse community-shared recipes with filters,
so that I can discover new recipes to add to my favorites.

## Acceptance Criteria

1. Community recipe page displays all shared recipes with pagination (20 recipes per page)
2. Filters available: recipe type (Appetizer/Main/Dessert/Accompaniment), cuisine type, dietary restrictions
3. Search functionality by recipe name or ingredients (case-insensitive partial matching)
4. Sort options: rating (highest first), newest, most favorited
5. Recipe cards show: name, recipe type badge, rating stars, favorite count, owner name
6. Quick-favorite button on recipe cards with Twinspark toggle behavior
7. Clicking recipe card navigates to recipe detail page
8. Tests verify filtering logic, search functionality, sorting, and pagination

## Acceptance Criteria

1. Community recipe page displays all shared recipes (is_shared = 1) with pagination
2. Filters: recipe type (appetizer/main/dessert/accompaniment), cuisine type, dietary restrictions
3. Search by recipe name or ingredients (case-insensitive LIKE matching)
4. Sort by rating (highest first), newest (created_at DESC), most favorited (favorite_count DESC)
5. Recipe cards show: name, type badge, rating, favorite count, owner name
6. Quick-favorite button on cards with toggle state
7. Pagination support (20 recipes per page)
8. Tests verify filtering, search, sorting, and pagination logic

## Tasks / Subtasks

- [ ] Create community query function (AC: #1, #2, #3, #4, #7)
  - [ ] Add `get_community_recipes()` function in `src/queries/recipes.rs`
  - [ ] Accept parameters: page, limit, recipe_type_filter, cuisine_filter, dietary_filter, search_query, sort_by
  - [ ] Query recipes WHERE is_shared = 1
  - [ ] Apply recipe_type filter if provided
  - [ ] Apply cuisine_type filter if provided
  - [ ] Apply dietary_restrictions JSON filter (NO overlap with user restrictions)
  - [ ] Apply search query with LIKE on name OR ingredients
  - [ ] Apply ORDER BY based on sort_by parameter (rating DESC, created_at DESC, favorite_count DESC)
  - [ ] Apply LIMIT and OFFSET for pagination
  - [ ] Join with recipe_favorites to get favorite_count
  - [ ] Join with recipe_ratings to get average rating
- [ ] Update recipe projection for community queries (AC: #5)
  - [ ] Add favorite_count computed column in query (COUNT aggregation)
  - [ ] Add average_rating computed column in query (AVG aggregation)
  - [ ] Add owner_name from users table JOIN
- [ ] Create community browse route handler (AC: #1, #2, #3, #4)
  - [ ] Create GET `/recipes/community` route in `src/routes/recipes/community.rs`
  - [ ] Extract query parameters: page, type, cuisine, dietary, search, sort
  - [ ] Call get_community_recipes() with extracted parameters
  - [ ] Render `templates/pages/recipes/community.html` with results
  - [ ] Include pagination metadata (current page, total pages, has_next, has_prev)
- [ ] Create community browse template (AC: #5, #6, #7)
  - [ ] Create `templates/pages/recipes/community.html`
  - [ ] Display recipe cards in grid layout (3 columns desktop, 1 column mobile)
  - [ ] Show recipe name, type badge (color-coded), owner name
  - [ ] Display star rating with average (e.g., "4.8 ★")
  - [ ] Display favorite count (e.g., "23 favorites")
  - [ ] Add quick-favorite button with Twinspark ts-req to toggle favorite status
  - [ ] Link each card to `/recipes/{id}` detail page
  - [ ] Add pagination controls (Previous/Next buttons with page numbers)
- [ ] Create filter and search UI (AC: #2, #3, #4)
  - [ ] Add filter sidebar with dropdowns for recipe type, cuisine, dietary restrictions
  - [ ] Add search input field with debounced Twinspark request (1s delay)
  - [ ] Add sort dropdown (Rating, Newest, Most Favorited)
  - [ ] Use Twinspark ts-req to update results without page reload
  - [ ] Update URL query parameters for shareable filter state
- [ ] Implement quick-favorite functionality (AC: #6)
  - [ ] Reuse POST `/recipes/{id}/favorite` endpoint from Story 2.3
  - [ ] Return partial template showing updated favorite button state
  - [ ] Display favorite limit warning if free tier user hits 10 favorites
  - [ ] Update favorite count in card after toggle
- [ ] Write tests (AC: #8)
  - [ ] Test get_community_recipes returns only shared recipes (is_shared = 1)
  - [ ] Test recipe type filter returns correct types
  - [ ] Test cuisine filter returns matching cuisine types
  - [ ] Test dietary filter excludes recipes with conflicting restrictions
  - [ ] Test search query matches recipe name (case-insensitive)
  - [ ] Test search query matches ingredients (case-insensitive)
  - [ ] Test sorting by rating (highest first)
  - [ ] Test sorting by newest (created_at DESC)
  - [ ] Test sorting by most favorited (favorite_count DESC)
  - [ ] Test pagination returns correct page with limit/offset
  - [ ] Test quick-favorite button toggles favorite status

## Dev Notes

### Architecture Patterns

**Query Complexity:**
- Community browse requires JOIN with users (owner_name), recipe_favorites (favorite_count), recipe_ratings (average_rating)
- Use single query with LEFT JOINs and GROUP BY for efficiency
- Index on recipes(is_shared) WHERE is_shared = 1 for performance

**Pagination Strategy:**
- Default: 20 recipes per page
- Calculate total_pages = CEIL(total_count / limit)
- Pass has_next, has_prev to template for navigation
- Use OFFSET = (page - 1) * limit

**Filter Application:**
- Apply filters incrementally with WHERE clauses
- Use JSON operators for dietary_restrictions array filtering
- Recipe type filter: simple equality check
- Cuisine filter: simple equality check
- Search: `(name LIKE ? OR ingredients LIKE ?)`

**Twinspark Integration:**
- Filter changes trigger ts-req to `/recipes/community?type=X&cuisine=Y`
- Results replace #community-results div
- URL updated with ts-req-history for shareable filter state
- Search input uses `ts-trigger="change delay 1s"` for debouncing

### Project Structure Notes

**Files to Create/Modify:**
- `src/queries/recipes.rs` - Add get_community_recipes() function with complex query
- `src/routes/recipes/community.rs` - Route handler for community browse
- `templates/pages/recipes/community.html` - Main community page with filters and grid
- `templates/partials/recipes/community-results.html` - Results grid for Twinspark updates
- `templates/components/recipe-card.html` - Reusable recipe card component
- `tests/recipes_test.rs` - Add community browse tests

**Dependencies:**
- No new dependencies required
- Uses existing sqlx, axum, askama, twinspark stack

**Query Example:**
```sql
SELECT
  r.id, r.name, r.recipe_type, r.cuisine_type, r.created_at,
  u.email as owner_name,
  COUNT(DISTINCT rf.user_id) as favorite_count,
  AVG(rr.rating) as average_rating
FROM recipes r
INNER JOIN users u ON r.owner_id = u.id
LEFT JOIN recipe_favorites rf ON r.id = rf.recipe_id
LEFT JOIN recipe_ratings rr ON r.id = rr.recipe_id
WHERE r.is_shared = 1
  AND (? IS NULL OR r.recipe_type = ?)
  AND (? IS NULL OR r.cuisine_type = ?)
  AND (? IS NULL OR r.name LIKE ? OR r.ingredients LIKE ?)
GROUP BY r.id
ORDER BY average_rating DESC
LIMIT ? OFFSET ?
```

### References

- [Source: docs/epics.md#Story 5.2] - Story acceptance criteria and prerequisites
- [Source: docs/PRD.md#FR015, FR017] - Community sharing and rating requirements
- [Source: docs/architecture.md#Data Architecture] - recipes table schema
- [Source: docs/architecture.md#Query Pattern] - Query function structure
- [Source: CLAUDE.md#Query Guidelines] - Query handler rules and projection patterns
- [Source: CLAUDE.md#Twinspark API Reference] - ts-req, ts-target, ts-trigger usage

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
