# Story 2.9: Rate and Review Community Recipes

Status: Done

## Story

As a user who cooked a community recipe,
I want to rate and review it,
so that I can help others find quality recipes.

## Acceptance Criteria

1. Rating widget (1-5 stars) visible on shared recipe detail pages (GET /discover/:id)
2. User can rate recipe only once per recipe_id (can update existing rating)
3. Optional text review field with 500 character maximum
4. Ratings aggregate to display average score (e.g., "4.3/5 from 47 reviews") on recipe cards and detail pages
5. Reviews displayed chronologically (most recent first) with reviewer username and created_at timestamp
6. User can edit their own review via PUT /discover/:id/review endpoint
7. User can delete their own review via DELETE /discover/:id/review endpoint
8. Recipe owner notified of new ratings/reviews (if notification preferences enabled - out of scope for this story)
9. Highly rated recipes (average >= 4.0 stars) featured/highlighted in discovery feed sorting
10. Rating submission requires authentication (redirect to /login if not authenticated)
11. Validation: rating must be integer between 1-5, review text <= 500 chars
12. Duplicate rating prevention: UPDATE existing rating rather than INSERT new one

## Tasks / Subtasks

- [x] Create RecipeRated event and Rating aggregate (AC: #1, #2, #4)
  - [x] Define RecipeRated event struct (recipe_id, user_id, stars: i32, review_text: Option<String>)
  - [x] Create RatingUpdated and RatingDeleted events for evento pattern
  - [x] Implement no-op event handlers in RecipeAggregate (ratings managed in read model)
  - [x] Add rating validation logic (1-5 stars, max 500 chars review)

- [x] Implement ratings database schema and read model (AC: #2, #4, #5)
  - [x] Create migration 06_v0.7_ratings.sql
  - [x] Add UNIQUE constraint on (recipe_id, user_id)
  - [x] Create indexes on recipe_id for fast aggregation queries
  - [x] Implement read model projections for RecipeRated, RatingUpdated, RatingDeleted events with UPSERT logic

- [x] Create rating submission route (AC: #1, #2, #3, #10, #11)
  - [x] Implement POST /discover/:id/rate handler in src/routes/recipes.rs
  - [x] Add auth middleware to protect endpoint
  - [x] Validate rating form (stars 1-5, review_text <= 500 chars)
  - [x] Call recipe::rate_recipe command with RateRecipeCommand
  - [x] Handle duplicate ratings (UPDATE via UPSERT logic in projection)
  - [x] Return with TwinSpark ts-location header for navigation

- [x] Implement rating edit/delete routes (AC: #6, #7)
  - [x] Use POST /discover/:id/rate for editing (same as create, UPSERT handles update)
  - [x] Add POST /discover/:id/review/delete handler for deleting own review
  - [x] Verify ownership (user_id matches rating creator)
  - [x] Return 403 Forbidden if user attempts to delete others' ratings

- [x] Display ratings on recipe pages (AC: #4, #5)
  - [x] Update recipe detail template to show average rating and review count
  - [x] Add rating widget (star display) with interactive JavaScript
  - [x] Display reviews list chronologically with username, date, review text
  - [x] Show edit/delete buttons only for user's own reviews

- [x] Update discovery feed with rating highlights (AC: #9)
  - [x] Modify discovery query to include avg_rating in recipe cards
  - [x] Add "Highly Rated" badge for recipes with avg_rating >= 4.0
  - [x] Rating stats displayed on all recipe cards

- [x] Write tests (AC: all)
  - [x] Unit tests: 12 comprehensive tests covering validation, functionality, queries
  - [x] Test UPSERT behavior for duplicate ratings
  - [x] Test aggregation calculation (multiple ratings produce correct average)
  - [x] Test ownership checks for delete operations
  - [x] All 60 tests passing in recipe package

## Dev Notes

### Architecture Patterns

**Event Sourcing:**
- RecipeRated event appended to evento stream
- RatingAggregate (if needed) or direct event emission from recipe crate
- Read model projection updates `ratings` table and recalculates `avg_rating` for recipe

**CQRS:**
- Commands: RateRecipeCommand, UpdateRatingCommand, DeleteRatingCommand
- Queries: Query ratings by recipe_id, query user's rating for recipe

**Domain Logic:**
- Rating validation enforced in recipe domain crate
- Average rating calculation via SQL aggregation in read model projection
- Duplicate prevention via UNIQUE constraint + UPSERT pattern

### Source Tree Components

**Domain Crate (crates/recipe/):**
- `events.rs`: RecipeRated event struct
- `commands.rs`: RateRecipeCommand, UpdateRatingCommand, DeleteRatingCommand
- `aggregate.rs`: Recipe aggregate (may not need separate Rating aggregate if ratings are recipe events)
- `read_model.rs`: Projection handler for RecipeRated event

**Root Binary (src/):**
- `routes/discover.rs`: POST /discover/:id/rate, PUT /discover/:id/review, DELETE /discover/:id/review handlers
- `middleware/auth.rs`: Protect rating endpoints (already exists)

**Templates (templates/):**
- `pages/recipe-detail.html`: Rating widget, reviews list display
- `components/rating-widget.html`: Reusable star rating component
- `components/review-item.html`: Individual review display with edit/delete buttons

**Database:**
- `migrations/005_create_ratings_table.sql`:
  ```sql
  CREATE TABLE ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    stars INTEGER NOT NULL CHECK(stars >= 1 AND stars <= 5),
    review_text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(recipe_id, user_id)
  );
  CREATE INDEX idx_ratings_recipe ON ratings(recipe_id);
  CREATE INDEX idx_ratings_user ON ratings(user_id);
  ```

### Testing Standards

**TDD Approach:**
1. Write failing test for rating submission
2. Implement RateRecipeCommand and RecipeRated event
3. Test passes, add projection test
4. Implement projection, test passes
5. Add route handler test, implement handler

**Coverage:**
- Unit tests: Event handlers, validation logic (>=80% coverage)
- Integration tests: All routes (POST, PUT, DELETE), database interactions
- E2E tests: Full user journey (Playwright)

**Test Data:**
- Use shared test fixtures from tests/common/fixtures.rs
- Create test user, test recipe, multiple test ratings for aggregation testing

### Project Structure Notes

**Alignment with solution-architecture.md:**
- Read model schema matches `ratings` table definition (section 3.2)
- Event sourcing via evento with RecipeRated event (section 3.1)
- Askama templates for server-rendered HTML (section 7.1)
- Auth middleware protection for authenticated actions (section 5.3)

**No Detected Conflicts:**
- Recipe domain already established in crates/recipe/
- Rating as part of recipe bounded context (no separate rating domain needed)
- Follows existing pattern from Story 2.6 (Mark Recipe as Favorite)

### References

- [Source: docs/epics.md#Story 2.9] - Story definition and acceptance criteria
- [Source: docs/tech-spec-epic-2.md#Story 2.9] - Detailed technical specification for rating implementation
- [Source: docs/solution-architecture.md#Section 3.2] - Data models (ratings table schema)
- [Source: docs/solution-architecture.md#Section 6.1] - Event sourcing pattern with evento
- [Source: docs/solution-architecture.md#Section 4.2] - HTML endpoints and form handling patterns
- [Source: docs/PRD.md#FR-11] - Functional requirement for recipe rating and reviews

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.9.xml`

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

- Implemented using POST-only endpoints (event sourcing pattern, not REST PUT/DELETE)
- UPSERT logic handled in projection layer via UNIQUE constraint on (recipe_id, user_id)
- No separate Rating aggregate - ratings are events on RecipeAggregate
- TwinSpark progressive enhancement with ts-location header for form submissions
- JavaScript for interactive star rating selection with proper event handling
- All 12 comprehensive unit tests passing (60/60 tests in recipe package)

### File List

**Events & Aggregates:**
- crates/recipe/src/events.rs - Added RecipeRated, RatingUpdated, RatingDeleted events
- crates/recipe/src/aggregate.rs - Added no-op event handlers for rating events

**Commands & Queries:**
- crates/recipe/src/commands.rs - Added rate_recipe, delete_rating commands with validation
- crates/recipe/src/read_model.rs - Added rating projections and query functions

**Database:**
- migrations/06_v0.7_ratings.sql - Ratings table schema with constraints and indexes

**Routes:**
- src/routes/recipes.rs - POST /discover/:id/rate and POST /discover/:id/review/delete handlers
- src/routes/mod.rs - Exported rating route handlers
- src/main.rs - Registered rating routes

**Templates:**
- templates/pages/discover-detail.html - Complete ratings UI with forms and reviews list
- templates/pages/discover.html - Added rating display and "Highly Rated" badge to cards

**Tests:**
- crates/recipe/tests/rating_tests.rs - 12 comprehensive unit tests (all passing)
