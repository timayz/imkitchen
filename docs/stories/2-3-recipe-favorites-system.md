# Story 2.3: Recipe Favorites System

Status: drafted

## Story

As a user,
I want to save community recipes as favorites,
So that they join my own recipes in the candidate pool used by recipe list generation.

## Acceptance Criteria

1. `favorite::Saved { recipe_id, recipe_owner }` and `favorite::Unsaved { recipe_id }` events store the user-recipe relationship, with the saving user carried as `requested_by` metadata
2. Favorites are unlimited on both free and premium tiers
3. Recipe pages show a "Save" / "Saved" button with toggle state; only shared recipes can be saved
4. Saving a shared recipe copies it into the user's candidate pool (`meal_plan_recipe` table); unsaving removes it, so the pool always reflects own recipes plus saved ones
5. The recipes index "Saved" filter lists the user's candidate pool (own recipes plus saved community recipes)
6. When the recipe owner deletes a recipe, it disappears from every user's candidate pool automatically (no notifications)
7. Query projection tracks favorite_count per recipe for community sorting
8. Tests verify save, unsave, idempotence and cascade removal from the pool

## Tasks / Subtasks

- [ ] Define the Favorite aggregate (AC: #1)
  - [ ] Add `Favorite` aggregate in `crates/types/src/favorite.rs` with `Saved { recipe_id, recipe_owner }` and `Unsaved { recipe_id }` variants
  - [ ] Derive bitcode Encode/Decode via `#[evento::aggregate]`
  - [ ] Add the shapes to `events.lock` (run tests with `EVENTO_LOCK=update` once)

- [ ] Implement the Favorite projection (AC: #1)
  - [ ] Create `crates/core/src/recipe/favorite/mod.rs` with projection `Favorite { id, saved }`
  - [ ] Aggregate id is `evento::hash_ids([recipe_id, user_id])` so one stream exists per (recipe, user) pair
  - [ ] `Module::load(id, user_id)` returns an unsaved default when no stream exists yet
  - [ ] Handlers `handle_saved` / `handle_unsaved` flip `saved`

- [ ] Implement save and unsave commands (AC: #1, #2, #8)
  - [ ] `crates/core/src/recipe/favorite/save.rs`: `save(id, owner_id, user_id)` emits `Saved` only when not already saved
  - [ ] `crates/core/src/recipe/favorite/unsave.rs`: `unsave(id, user_id)` emits `Unsaved` only when currently saved
  - [ ] No tier check of any kind: the commands never consult subscription status
  - [ ] Expose the module as `core.recipe.favorite`

- [ ] Maintain the candidate pool from favorite events (AC: #4, #6)
  - [ ] In `crates/core/src/shopping/pool.rs`, add `handle_favorite_saved`: insert into `meal_plan_recipe` a copy of the owner's row (id, type, name, dietary restrictions, advance prep, cook/prep time, accepts_accompaniment) keyed by the saving user
  - [ ] Add `handle_favorite_unsaved`: delete the `(recipe_id, user_id)` row from `meal_plan_recipe`
  - [ ] Confirm the existing `handle_recipe_deleted` deletes every `meal_plan_recipe` row for the recipe id, which removes it from savers' pools too
  - [ ] Register both handlers on the pool subscription

- [ ] Implement save / unsave routes (AC: #3)
  - [ ] `POST /recipes/{id}/save` and `POST /recipes/{id}/unsave` in `web/recipe/src/routes/detail.rs`, registered in `web/recipe/src/lib.rs`
  - [ ] `save` loads the recipe, returns NotFound when it is not shared, then calls `core.recipe.favorite.save(id, recipe.owner_id, user.id)`
  - [ ] `unsave` calls `core.recipe.favorite.unsave(id, user.id)`
  - [ ] Both return the save button partial with `ts-swap: skip` so Twinspark swaps the button in place

- [ ] Save button partial (AC: #3)
  - [ ] `templates/partials/recipes-detail-save-button.html` renders a `#save-btn` button with `ts-req="/recipes/{id}/save"` or `/unsave` depending on `saved`, `ts-req-method="post"`, `ts-swap-push="#save-btn"`
  - [ ] Filled bookmark + "Saved" when saved, outline bookmark + "Save" otherwise
  - [ ] Recipe detail page loads the favorite state via `core.recipe.favorite.load(id, user.id)` to render the initial button

- [ ] Recipes index "Saved" filter (AC: #5)
  - [ ] Query field `in_meal_plan=true` in `web/recipe/src/routes/index.rs` filters recipes to those present in the user's `meal_plan_recipe` rows
  - [ ] Visible label in `templates/recipes-index.html` is "Saved"; it is mutually exclusive with the "Mine" toggle
  - [ ] Demo sessions get the sign-up prompt instead of the filter

- [ ] Add favorite_count projection for community sorting (AC: #7)
  - [ ] Track favorite_count per recipe from `Saved` / `Unsaved` events in the recipe read projection
  - [ ] Community browse sort "Most favorited" orders by this count (Story 5.2)

- [ ] Write core tests (AC: #8)
  - [ ] `save` then `load` reports `saved = true`; `unsave` reports `saved = false`
  - [ ] Saving twice emits a single `Saved` event; unsaving an unsaved recipe emits nothing
  - [ ] After `Saved`, the saver has a `meal_plan_recipe` row for the recipe; after `Unsaved` it is gone
  - [ ] Deleting the recipe removes it from the saver's pool
  - [ ] Generating a list for the saver can pick the saved recipe (ties into `crates/core/tests/shopping/generate.rs`)

- [ ] Write E2E test for the save flow (AC: #3, #5, #8)
  - [ ] Playwright test: open a shared recipe → Save → button reads "Saved" → recipe appears under the "Saved" filter → Unsave → it disappears

## Dev Notes

### Architecture Patterns

**One stream per (recipe, user):**
- The Favorite aggregate id is `evento::hash_ids([recipe_id, user_id])`, so each pair has its own event stream and the projection is a single `saved` boolean
- Commands are idempotent: they load the projection first and emit only on an actual state change

**Candidate pool, not a favorites table:**
- There is no `recipe_favorites` table. The read-side effect of saving is a row in `meal_plan_recipe` (the candidate pool used by recipe list generation, Story 2.7 / Story 3.1) keyed by the saving user
- `handle_favorite_saved` copies the owner's pool row (so type, dietary restrictions and accepts_accompaniment travel with it); subsequent recipe edits propagate through the existing pool handlers because they update by recipe id across all users' rows
- The "Saved" filter on the recipes index is simply "recipes present in my pool"

**No tier limit:**
- Favorites are unlimited on free and premium tiers. The free tier is ad-supported (Story 5.4); nothing in the save path checks subscription status

**Cascade on deletion (per epics.md AC #6):**
- `handle_recipe_deleted` in the pool subscription deletes every `meal_plan_recipe` row for that recipe id, which clears it from the owner's pool and every saver's pool
- Silent removal: no notifications to savers or owner (Story 5.9)

**Favorite count (per epics.md AC #7):**
- Denormalized count per recipe, maintained from `Saved` / `Unsaved` events, used by the community "Most favorited" sort

### Project Structure Notes

**Files to Create/Modify:**
```
crates/types/src/
└── favorite.rs                      # Favorite aggregate: Saved, Unsaved

crates/core/src/recipe/favorite/
├── mod.rs                           # Module, Favorite projection, handlers
├── save.rs                          # save(id, owner_id, user_id)
└── unsave.rs                        # unsave(id, user_id)

crates/core/src/shopping/
└── pool.rs                          # handle_favorite_saved / handle_favorite_unsaved

web/recipe/src/
├── lib.rs                           # POST /recipes/{id}/save, /unsave
└── routes/detail.rs                 # save, unsave handlers + SaveButtonTemplate

web/recipe/src/routes/index.rs       # in_meal_plan ("Saved") filter

templates/
├── partials/recipes-detail-save-button.html   # Save / Saved toggle
└── recipes-index.html                         # "Saved" filter toggle
```

**Twinspark Integration:**
- Save button: `ts-req="/recipes/{{ id }}/save"` (or `/unsave`), `ts-req-method="post"`, `ts-swap-push="#save-btn"`
- Route responses set the `ts-swap: skip` header and return the button partial

### Technical Constraints

**Bounded Context Decision:**
- Favorite lives under the Recipe context (`crates/core/src/recipe/favorite/`) and is exposed as `core.recipe.favorite`
- The pool side effect lives in the Shopping context because the pool exists to feed list generation

**Shared-only rule:**
- Only shared recipes can be saved; the route returns NotFound for private recipes. Own recipes are already in the owner's pool and need no save

**Event shape lock:**
- Adding or changing `Saved` / `Unsaved` fields changes persisted bitcode shapes; update `events.lock` deliberately

### References

- [Source: docs/epics.md#Story-2.3] Story acceptance criteria
- [Source: docs/epics.md#Story-2.7] Recipe candidate pool
- [Source: docs/epics.md#Story-3.1] List generation picks from the pool
- [Source: docs/epics.md#Story-5.4] Ad-supported free tier (no feature limits)
- [Source: docs/epics.md#Story-5.9] Deletion cascade without notifications
- [Source: CLAUDE.md#Query-Guidelines] Idempotent query handlers

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
