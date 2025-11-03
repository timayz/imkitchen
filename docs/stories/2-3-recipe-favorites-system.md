# Story 2.3: Recipe Favorites System

Status: ready-for-dev

## Story

As a user,
I want to favorite recipes (my own and community recipes),
so that I can designate which recipes should be included in meal plan generation.

## Acceptance Criteria

1. RecipeFavorited and RecipeUnfavorited events store user-recipe relationship
2. Free tier users limited to maximum 10 favorited recipes
3. Attempting to exceed 10 favorites shows upgrade modal (no unfavoriting option)
4. Premium tier users have unlimited favorites
5. Recipe cards show favorite button with toggle state
6. User profile displays favorited recipes list
7. When recipe owner deletes recipe, all favorites automatically removed (no notifications)
8. Query projection tracks favorite_count per recipe for community sorting
9. Tests verify favorite limits, premium bypass, and cascade deletion

## Tasks / Subtasks

- [ ] Define Recipe favorite events (AC: #1)
  - [ ] Define RecipeFavorited event with user_id and recipe_id
  - [ ] Define RecipeUnfavorited event with user_id and recipe_id
  - [ ] Add to `crates/imkitchen-recipe/src/event.rs`

- [ ] Implement Recipe aggregate handlers for favorites (AC: #1)
  - [ ] Add recipe_favorited handler to track favorite_count
  - [ ] Add recipe_unfavorited handler to decrement favorite_count
  - [ ] Update Recipe aggregate with favorite_count field

- [ ] Implement favorite/unfavorite commands with tier limits (AC: #2, #3, #4)
  - [ ] Define FavoriteRecipeInput with user_id and recipe_id
  - [ ] Implement favorite_recipe command with pre-validation check
  - [ ] Query recipe_favorites table to count user's current favorites
  - [ ] Check access control service: is_premium_active OR premium_bypass OR global bypass
  - [ ] If free tier and count >= 10, return error with upgrade message
  - [ ] Implement unfavorite_recipe command (no tier restrictions)

- [ ] Create recipe_favorites migration (AC: #1, #7)
  - [ ] Create migrations/queries/20250101000003_recipe_favorites.sql
  - [ ] Define recipe_favorites table with composite PK (user_id, recipe_id)
  - [ ] Add ON DELETE CASCADE FK to recipes table
  - [ ] Add index on user_id for fast count queries

- [ ] Implement query handlers for favorite events (AC: #8)
  - [ ] Create on_recipe_favorited handler to INSERT into recipe_favorites
  - [ ] Create on_recipe_unfavorited handler to DELETE from recipe_favorites
  - [ ] Update subscribe_recipe_query with favorite handlers
  - [ ] Track favorite_count in recipes table or separate aggregate column

- [ ] Create favorite toggle route (AC: #5)
  - [ ] Create src/routes/recipes/favorite.rs with POST handler
  - [ ] Accept recipe_id parameter
  - [ ] Toggle favorite state (add if not favorited, remove if favorited)
  - [ ] Return partial HTML with updated button state for Twinspark
  - [ ] Return upgrade modal HTML if limit exceeded (free tier)

- [ ] Update recipe cards with favorite button (AC: #5)
  - [ ] Add favorite button to templates/components/recipe-card.html
  - [ ] Show filled heart icon if favorited, outline if not
  - [ ] Use Twinspark ts-req for toggle action
  - [ ] Display favorite count badge on card

- [ ] Create upgrade modal template (AC: #3)
  - [ ] Create templates/components/upgrade-modal.html
  - [ ] Show pricing: "You've reached your 10 favorites limit. Upgrade to Premium for unlimited favorites"
  - [ ] Include pricing info ($9.99/month or $59.94/year)
  - [ ] NO unfavoriting option in modal (strong conversion incentive)
  - [ ] Link to pricing page

- [ ] Add favorited recipes to user profile (AC: #6)
  - [ ] Update templates/pages/auth/profile.html
  - [ ] Query user's favorited recipes with get_user_favorites function
  - [ ] Display favorited recipes list with filter by type
  - [ ] Show count and tier limit (e.g., "8/10 favorites" for free tier)

- [ ] Cascade deletion handling (AC: #7)
  - [ ] Already handled by ON DELETE CASCADE in FK constraint
  - [ ] Verify no notifications sent (no event emitted)
  - [ ] Test cascade deletion in integration tests

- [ ] Write integration tests (AC: #9)
  - [ ] Test favoriting recipe (both own and community recipes)
  - [ ] Test free tier limit (10 max)
  - [ ] Test premium tier unlimited favorites
  - [ ] Test premium bypass configuration (global and per-user)
  - [ ] Test cascade deletion when recipe deleted
  - [ ] Test favorite_count projection updates
  - [ ] Use evento::load to verify aggregate state

## Dev Notes

- **Freemium Enforcement**: Access control service checks: is_premium_active OR premium_bypass OR global bypass. Free tier limited to 10 favorites [Source: docs/PRD.md#FR013, FR052]
- **Upgrade Modal**: Strong conversion incentive - modal has NO unfavoriting option, only upgrade CTA [Source: docs/PRD.md#FR014, docs/epics.md#Story 2.3]
- **Cascade Deletion**: ON DELETE CASCADE FK constraint automatically removes favorites when recipe deleted; no notifications [Source: docs/PRD.md#FR016, docs/epics.md#Story 2.3]
- **Access Control Service**: Centralized service at `src/access_control.rs` provides consistent freemium logic across features [Source: docs/architecture.md#ADR-004]
- **Premium Bypass**: Two levels - global config (entire environment) OR per-user flag (selective access for demo/staging) [Source: docs/PRD.md#FR051, docs/epics.md#Story 1.5]

### Project Structure Notes

- **Routes**: `src/routes/recipes/favorite.rs` (POST toggle)
- **Templates**: `templates/components/upgrade-modal.html`, update `templates/components/recipe-card.html`, update `templates/pages/auth/profile.html`
- **Events**: Add RecipeFavorited, RecipeUnfavorited to `crates/imkitchen-recipe/src/event.rs`
- **Aggregate**: Update `crates/imkitchen-recipe/src/aggregate.rs` with favorite_count field
- **Commands**: Add methods to `crates/imkitchen-recipe/src/command.rs`
- **Access Control**: Use existing `src/access_control.rs` service
- **Migrations**: Create `migrations/queries/20250101000003_recipe_favorites.sql`
- **Queries**: Add get_user_favorites, get_favorite_count functions to `src/queries/recipes.rs`
- **Tests**: Add to `tests/recipes_test.rs`

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.3] - Full acceptance criteria and freemium limits
- [docs/PRD.md#FR012-FR014] - Recipe favorites and tier limits
- [docs/PRD.md#FR016] - Cascade deletion without notifications
- [docs/architecture.md#ADR-004] - Centralized Access Control Service
- [docs/architecture.md#Core Tables] - recipe_favorites table schema
- [docs/epics.md#Story 1.5] - Premium bypass configuration

## Dev Agent Record

### Context Reference

- docs/stories/2-3-recipe-favorites-system.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
