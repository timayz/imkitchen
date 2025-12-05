# Story 5.3: Recipe Rating System

Status: drafted

## Story

As a user,
I want to rate and review community recipes I've tried,
so that I can provide feedback and help others discover quality recipes.

## Acceptance Criteria

1. RecipeRated event stores user_id, recipe_id, rating (1-5 stars), review_text (optional), timestamp
2. Users can rate any shared recipe (is_shared = 1) but NOT their own recipes
3. Users can edit their existing rating by submitting a new RecipeRated event (overwrites previous)
4. Users can delete their rating via RecipeRatingDeleted event
5. Recipe detail page displays average rating and all reviews (paginated)
6. Community browse page sorts recipes by average rating (highest first) as default sort option
7. Low-rated recipes (average < 3 stars) are de-prioritized in community search results
8. Tests verify rating creation, authorization (no self-rating), average calculation, and sorting

## Tasks / Subtasks

- [ ] Create rating events and aggregate (AC: #1, #3, #4)
  - [ ] Create RecipeRating aggregate in `crates/imkitchen-recipe/src/aggregate.rs`
  - [ ] Create RecipeRated event in `crates/imkitchen-recipe/src/event.rs` with fields: user_id, recipe_id, rating (1-5), review_text, timestamp
  - [ ] Create RecipeRatingDeleted event with user_id, recipe_id
  - [ ] RecipeRating aggregate handles RecipeRated and RecipeRatingDeleted events
- [ ] Implement rating commands (AC: #1, #2, #3, #4)
  - [ ] Add `rate_recipe()` command in `crates/imkitchen-recipe/src/command.rs`
  - [ ] Accept RateRecipeInput struct with recipe_id, rating, review_text (optional)
  - [ ] Validate rating is between 1-5 using validator crate
  - [ ] Check recipe exists and is_shared = 1 (query read DB for validation)
  - [ ] Check user is NOT recipe owner (prevent self-rating) by comparing user_id with owner_id
  - [ ] Use evento::create for first rating, evento::save for updates (check if rating exists)
  - [ ] Add `delete_rating()` command accepting DeleteRatingInput with recipe_id
  - [ ] Emit RecipeRatingDeleted event
- [ ] Create rating projection and query handlers (AC: #5, #6, #7)
  - [ ] Create on_recipe_rated handler in `src/queries/recipes.rs`
  - [ ] Insert or UPDATE recipe_ratings table with rating data
  - [ ] Use UPSERT pattern: ON CONFLICT (user_id, recipe_id) DO UPDATE
  - [ ] Create on_recipe_rating_deleted handler to DELETE from recipe_ratings
  - [ ] Add handlers to subscribe_recipe_query subscription
  - [ ] Create database migration: `migrations/queries/{timestamp}_recipe_ratings.sql`
  - [ ] Table schema: id (PK), recipe_id (FK), user_id (FK), rating (1-5), review_text, created_at
  - [ ] Add UNIQUE constraint on (user_id, recipe_id)
- [ ] Implement rating query functions (AC: #5, #6, #7)
  - [ ] Add `get_recipe_ratings()` function in `src/queries/recipes.rs`
  - [ ] Return Vec<RatingRow> with user_id, rating, review_text, created_at
  - [ ] Support pagination (limit, offset)
  - [ ] Add `get_recipe_average_rating()` function returning Option<f64>
  - [ ] Query: SELECT AVG(rating) FROM recipe_ratings WHERE recipe_id = ?
  - [ ] Update get_community_recipes() to include average_rating in results
  - [ ] Apply de-prioritization: ORDER BY CASE WHEN avg_rating < 3 THEN 1 ELSE 0 END, avg_rating DESC
- [ ] Create rating UI components (AC: #5)
  - [ ] Create `templates/components/rating-form.html` for submitting ratings
  - [ ] Include 5-star radio buttons and optional review textarea
  - [ ] Use Twinspark ts-req for form submission to POST `/recipes/{id}/rate`
  - [ ] Create `templates/components/rating-display.html` for showing average rating
  - [ ] Display average as "4.8 ★ (23 reviews)"
  - [ ] Create `templates/components/review-list.html` for displaying all reviews
  - [ ] Show reviewer name (or anonymous), star rating, review text, date
- [ ] Implement rating route handlers (AC: #1, #3, #4)
  - [ ] Create POST `/recipes/{id}/rate` route in `src/routes/recipes/rate.rs`
  - [ ] Extract user_id from JWT cookie
  - [ ] Call command.rate_recipe(input, metadata)
  - [ ] Return updated recipe detail partial with new rating displayed
  - [ ] Create POST `/recipes/{id}/rate/delete` route for deleting user's rating
  - [ ] Call command.delete_rating(input, metadata)
- [ ] Update recipe detail page (AC: #5)
  - [ ] Add average rating display at top of recipe detail page
  - [ ] Show rating form for logged-in users (hidden if user is recipe owner)
  - [ ] Display all reviews below recipe details with pagination
  - [ ] Pre-populate form if user has already rated (edit mode)
  - [ ] Add delete button if user has rated (only visible to rating author)
- [ ] Write tests (AC: #8)
  - [ ] Test rate_recipe command emits RecipeRated event
  - [ ] Test rating validation: 1-5 range enforced
  - [ ] Test user cannot rate their own recipe (authorization check)
  - [ ] Test user cannot rate non-shared recipe (is_shared = 0)
  - [ ] Test updating existing rating overwrites previous rating
  - [ ] Test delete_rating command emits RecipeRatingDeleted event
  - [ ] Test query handler inserts/updates recipe_ratings table correctly
  - [ ] Test get_recipe_average_rating calculates average correctly
  - [ ] Test community browse sorts by rating (highest first)
  - [ ] Test low-rated recipes (< 3 stars) de-prioritized in results

## Dev Notes

### Architecture Patterns

**Aggregate Design:**
- RecipeRating is a separate aggregate from Recipe
- Aggregate root ID = concatenated "{user_id}_{recipe_id}" for uniqueness
- RecipeRated event can be replayed for updates (upsert pattern in projection)

**Authorization Rules:**
- Users can only rate shared recipes (is_shared = 1)
- Users cannot rate their own recipes
- Check in command by loading recipe from read DB and comparing owner_id with user_id from JWT

**Rating Calculation:**
- Average rating computed in query: AVG(rating) grouped by recipe_id
- Cached in community query results for performance
- Real-time updates when new ratings added

**De-prioritization Logic:**
- Community browse applies sort order:
  - CASE WHEN avg_rating < 3 THEN 1 ELSE 0 END (pushes low-rated to bottom)
  - Then sort by avg_rating DESC within groups

**Edit/Delete Pattern:**
- Editing: Submit new RecipeRated event with same user_id + recipe_id
- Query handler uses UPSERT (ON CONFLICT DO UPDATE) to overwrite
- Deleting: Emit RecipeRatingDeleted event, handler DELETEs from projection

### Project Structure Notes

**Files to Create/Modify:**
- `crates/imkitchen-recipe/src/aggregate.rs` - Add RecipeRating aggregate
- `crates/imkitchen-recipe/src/event.rs` - Add RecipeRated, RecipeRatingDeleted events
- `crates/imkitchen-recipe/src/command.rs` - Add rate_recipe, delete_rating commands
- `src/queries/recipes.rs` - Add rating query handlers and functions
- `src/routes/recipes/rate.rs` - Route handlers for rating actions
- `migrations/queries/{timestamp}_recipe_ratings.sql` - Create recipe_ratings table
- `templates/components/rating-form.html` - Rating submission form
- `templates/components/rating-display.html` - Average rating display
- `templates/components/review-list.html` - Review list with pagination
- `templates/pages/recipes/detail.html` - Update to include ratings section
- `tests/recipes_test.rs` - Add rating tests

**Database Schema:**
```sql
CREATE TABLE recipe_ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review_text TEXT,
    created_at INTEGER NOT NULL,
    UNIQUE (user_id, recipe_id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_recipe_ratings_recipe ON recipe_ratings(recipe_id);
CREATE INDEX idx_recipe_ratings_user ON recipe_ratings(user_id);
```

### References

- [Source: docs/epics.md#Story 5.3] - Story acceptance criteria and prerequisites
- [Source: docs/PRD.md#FR017] - Recipe rating functional requirement
- [Source: docs/architecture.md#Data Architecture] - recipe_ratings table schema
- [Source: docs/architecture.md#Command Pattern] - Authorization and validation patterns
- [Source: CLAUDE.md#Command Guidelines] - Validation and evento patterns
- [Source: CLAUDE.md#Query Guidelines] - Projection idempotency and timestamp usage

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
