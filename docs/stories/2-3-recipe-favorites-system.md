# Story 2.3: Recipe Favorites System

Status: drafted

## Story

As a user,
I want to favorite recipes (my own and community recipes),
So that I can designate which recipes should be included in meal plan generation.

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

- [ ] Define RecipeFavorited and RecipeUnfavorited events (AC: #1)
  - [ ] Add RecipeFavorited event to appropriate bounded context (likely User or Recipe)
  - [ ] Add RecipeUnfavorited event with user_id and recipe_id
  - [ ] Include timestamp fields in events
  - [ ] Derive bincode Encode/Decode

- [ ] Create recipe_favorites migration (AC: #1, #8)
  - [ ] Create `migrations/queries/{timestamp}_recipe_favorites.sql`
  - [ ] Define recipe_favorites table with (user_id, recipe_id) composite primary key
  - [ ] Add favorited_at timestamp column
  - [ ] Add CASCADE DELETE on recipe_id foreign key (for AC #7)
  - [ ] Create index on user_id for user favorites queries

- [ ] Implement favorite_recipe command with tier limits (AC: #2, #3, #4)
  - [ ] Add FavoriteRecipeInput struct with user_id and recipe_id
  - [ ] Implement Command::favorite_recipe method
  - [ ] Query current favorite count for user from projection
  - [ ] Check access control: if free tier AND count >= 10 → return error
  - [ ] If premium tier OR count < 10 → emit RecipeFavorited event

- [ ] Implement unfavorite_recipe command (AC: #1)
  - [ ] Add UnfavoriteRecipeInput struct
  - [ ] Implement Command::unfavorite_recipe method
  - [ ] Emit RecipeUnfavorited event

- [ ] Implement query handlers for favorite events (AC: #1, #8)
  - [ ] Add on_recipe_favorited handler in `src/queries/recipes.rs`
  - [ ] Insert row into recipe_favorites table
  - [ ] Increment favorite_count in recipes table
  - [ ] Add on_recipe_unfavorited handler
  - [ ] Delete row from recipe_favorites table
  - [ ] Decrement favorite_count in recipes table

- [ ] Update Access Control Service for tier limits (AC: #2, #3, #4)
  - [ ] Add can_add_favorite method to `src/access_control.rs`
  - [ ] Check: if premium_active OR premium_bypass OR global_bypass → return true
  - [ ] Else: query favorite count, return count < 10
  - [ ] Return (can_add: bool, reason: Option<String>)

- [ ] Create upgrade modal template (AC: #3)
  - [ ] Create `templates/components/upgrade-modal.html`
  - [ ] Include text: "You've reached your 10 favorites limit. Upgrade to Premium for unlimited favorites"
  - [ ] Show pricing: $9.99/month or $59.94/year
  - [ ] NO unfavoriting option in modal (strong conversion incentive per spec)
  - [ ] Include "Upgrade Now" CTA button

- [ ] Implement favorite toggle route (AC: #5)
  - [ ] Create `src/routes/recipes/favorite.rs`
  - [ ] Add POST /recipes/{id}/favorite handler
  - [ ] Check if already favorited → unfavorite, else → favorite
  - [ ] If favorite blocked by tier limit → return upgrade modal HTML
  - [ ] Return updated recipe card partial with new favorite state

- [ ] Update recipe card component with favorite button (AC: #5)
  - [ ] Add favorite button to `templates/components/recipe-card.html`
  - [ ] Show filled heart if favorited, outline heart if not
  - [ ] Use Twinspark ts-req for toggle action
  - [ ] Display favorite count badge

- [ ] Create user favorites list view (AC: #6)
  - [ ] Add get_user_favorites query function
  - [ ] Query recipe_favorites joined with recipes table
  - [ ] Create `templates/pages/recipes/favorites.html` template
  - [ ] Show favorite count and tier limit (e.g., "8/10 favorites" for free tier)

- [ ] Add favorite_count column to recipes table (AC: #8)
  - [ ] Update recipes migration to include favorite_count INTEGER DEFAULT 0
  - [ ] Update on_recipe_created handler to initialize favorite_count = 0
  - [ ] Ensure on_recipe_favorited increments count
  - [ ] Ensure on_recipe_unfavorited decrements count

- [ ] Update community query to sort by favorite_count (AC: #8)
  - [ ] Modify get_community_recipes query to include ORDER BY favorite_count DESC
  - [ ] Add favorite count display in community recipe cards
  - [ ] Show "X users favorited" badge

- [ ] Verify cascade deletion on RecipeDeleted (AC: #7)
  - [ ] Confirm foreign key CASCADE DELETE handles automatic removal
  - [ ] NO additional event handler needed (database handles it)
  - [ ] Document in dev notes that cascade is database-level

- [ ] Write unit tests for favorite command (AC: #9)
  - [ ] Test favorite_recipe with valid user and recipe
  - [ ] Test unfavorite_recipe removes favorite
  - [ ] Test free tier limit enforcement (10 max)
  - [ ] Test premium tier has no limit

- [ ] Write integration tests for tier limits (AC: #9)
  - [ ] Create user with free tier status
  - [ ] Favorite 10 recipes successfully
  - [ ] Attempt 11th favorite → verify error returned
  - [ ] Set premium_bypass = true → verify 11th favorite succeeds

- [ ] Write integration tests for cascade deletion (AC: #7, #9)
  - [ ] Create recipe, favorite by multiple users
  - [ ] Delete recipe via Command::delete_recipe
  - [ ] Verify all recipe_favorites rows removed automatically
  - [ ] Verify no notifications sent (check logs)

- [ ] Write E2E test for favorites flow (AC: #9)
  - [ ] Create Playwright test in `tests/e2e/favorites.spec.ts`
  - [ ] Test: login as free user → favorite 10 recipes → attempt 11th → see upgrade modal
  - [ ] Test: unfavorite recipe → count decreases
  - [ ] Test: favorited recipe appears in favorites list

## Dev Notes

### Architecture Patterns

**Access Control Integration:**
- Use centralized AccessControlService from `src/access_control.rs`
- Service checks: active premium subscription OR global bypass OR user bypass flag
- Command calls service before emitting event
- Example:
```rust
let can_add = access_control.can_add_favorite(&user_id, current_count).await?;
if !can_add {
    return Err(anyhow!("Favorite limit reached. Upgrade to premium."));
}
```

**Tier Limit Enforcement (per PRD FR013, FR014):**
- Free tier: maximum 10 favorited recipes
- Premium tier: unlimited favorites
- Limit checked in command before emitting RecipeFavorited event
- Upgrade modal shown on 11th attempt (no unfavorite option per spec)

**Cascade Deletion Pattern (per epics.md AC #7):**
- Database foreign key with ON DELETE CASCADE
- No event handler needed - database handles automatically
- Silent removal (no notifications sent per PRD)

**Favorite Count Projection (per epics.md AC #8):**
- Denormalized favorite_count stored in recipes table
- Updated by query handlers on RecipeFavorited/RecipeUnfavorited events
- Enables efficient sorting: `ORDER BY favorite_count DESC`

### Project Structure Notes

**Files to Create/Modify:**
```
crates/imkitchen-recipe/src/
├── event.rs         # Add RecipeFavorited, RecipeUnfavorited events (or in User context)
└── command.rs       # Add favorite_recipe, unfavorite_recipe methods

src/
├── access_control.rs  # Add can_add_favorite method
├── queries/recipes.rs # Add favorite event handlers
└── routes/recipes/
    └── favorite.rs    # New file for favorite toggle handler

migrations/queries/
└── {timestamp}_recipe_favorites.sql  # New migration

templates/
├── components/
│   ├── recipe-card.html        # Add favorite button
│   └── upgrade-modal.html      # New modal for tier limits
└── pages/recipes/
    └── favorites.html          # New page for user's favorites list
```

**Database Schema:**
```sql
CREATE TABLE recipe_favorites (
    user_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    favorited_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, recipe_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);

CREATE INDEX idx_recipe_favorites_user ON recipe_favorites(user_id);

-- Add to recipes table:
ALTER TABLE recipes ADD COLUMN favorite_count INTEGER NOT NULL DEFAULT 0;
```

### Technical Constraints

**Bounded Context Decision:**
- RecipeFavorited/RecipeUnfavorited could belong to User OR Recipe context
- Recommendation: Place in Recipe context since favorite is recipe-centric action
- Alternative: Create separate Favorites aggregate if complexity grows

**Premium Bypass Configuration (per PRD FR051, FR116):**
- Global bypass: `config/default.toml` → `access_control.global_premium_bypass = true`
- Per-user bypass: `user_profiles.premium_bypass = true`
- Access control checks: `is_premium_active OR premium_bypass OR global_bypass`

**Upgrade Modal UX (per epics.md AC #3):**
- NO unfavoriting option in modal (intentional friction for conversion)
- User must navigate back or close modal
- Modal persists until user makes choice (dismissible but returns on next favorite attempt)

**Freemium Touchpoint (per PRD FR056):**
- Favorite limit is one of multiple upgrade prompts
- Others: calendar week locking, dashboard restrictions
- All use same upgrade modal component with contextual messaging

### References

- [Source: docs/PRD.md#FR012-FR014] Favorites functional requirements
- [Source: docs/PRD.md#FR051] Premium bypass configuration
- [Source: docs/epics.md#Story-2.3] Story acceptance criteria
- [Source: docs/architecture.md#ADR-004] Centralized Access Control Service
- [Source: CLAUDE.md#Query-Guidelines] Idempotent query handlers
- [Source: mockups/recipes-list.html] Visual design with favorites counter

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
