# Story 2.10: Copy Community Recipe to Personal Library

Status: Approved

## Story

As a user browsing community recipes,
I want to add a community recipe to my library,
So that I can use it in my meal planning.

## Acceptance Criteria

1. "Add to My Recipes" button visible on community recipe detail page (GET /discover/:id)
2. Clicking button copies recipe to user's personal library with full recipe data duplicated
3. Copied recipe becomes owned by user (user_id set to current user, editable by owner only)
4. Original creator attribution maintained in metadata (original_recipe_id, original_author fields)
5. Copy counts as new recipe toward free tier limit (10 recipe maximum for free users)
6. Copied recipe defaults to private (is_shared = false, user must explicitly share separately)
7. Modifications to copy don't affect original recipe (independent Recipe aggregate created)
8. User can mark copied recipe as favorite for meal planning inclusion
9. Confirmation message displayed: "Recipe added to your library"
10. Button disabled if user already copied this recipe (prevent duplicate copies)
11. Button disabled if free tier user at recipe limit (show "Upgrade to add more recipes" message)
12. Copying recipe from discover page redirects to user's personal recipe detail page

## Tasks / Subtasks

- [ ] Create RecipeCopied event and copy command (AC: #2, #3, #4, #7)
  - [ ] Define RecipeCopied event struct (original_recipe_id, new_recipe_id, user_id, original_author)
  - [ ] Implement copy_recipe command in recipe domain crate
  - [ ] Load original recipe aggregate from event stream
  - [ ] Create new Recipe aggregate with RecipeCreated event (full data duplication)
  - [ ] Store original attribution metadata (original_recipe_id, original_author)

- [ ] Implement freemium limit checking (AC: #5, #11)
  - [ ] Check user's current recipe count before copying
  - [ ] Return RecipeLimitReached error if free tier user at 10 recipes
  - [ ] Display upgrade prompt instead of "Add to My Recipes" button when at limit

- [ ] Create recipe copy route (AC: #1, #2, #9, #12)
  - [ ] Implement POST /discover/:id/copy handler in src/routes/recipes.rs
  - [ ] Add auth middleware to protect endpoint (require JWT)
  - [ ] Call recipe::copy_recipe command with CopyRecipeCommand
  - [ ] Handle freemium limit errors with inline message
  - [ ] Return 302 redirect to /recipes/:new_id after successful copy
  - [ ] Display confirmation toast/message on redirect

- [ ] Update community recipe detail template (AC: #1, #10, #11)
  - [ ] Add "Add to My Recipes" button on templates/pages/discover-detail.html
  - [ ] Check if user already copied recipe (query copied_recipes table)
  - [ ] Disable button if recipe already in user's library ("Already in your library")
  - [ ] Show upgrade prompt if at recipe limit instead of button
  - [ ] Style button prominently for clear call-to-action

- [ ] Implement duplicate copy prevention (AC: #10)
  - [ ] Add copied_recipes tracking table or query existing recipes by original_recipe_id
  - [ ] Check if user_id + original_recipe_id combination exists
  - [ ] Display "Already in your library" message if duplicate detected

- [ ] Handle privacy and favoriting defaults (AC: #6, #8)
  - [ ] Set is_shared = false on copied recipe by default
  - [ ] Set is_favorite = false on copy (user can favorite afterwards)
  - [ ] Ensure copied recipe fully independent (separate aggregate ID)

- [ ] Write tests (AC: all)
  - [ ] Unit test: copy_recipe command creates new aggregate with correct data
  - [ ] Unit test: original attribution metadata stored correctly
  - [ ] Unit test: freemium limit enforced (error at 10 recipes for free users)
  - [ ] Unit test: modifications to copy don't affect original
  - [ ] Integration test: POST /discover/:id/copy creates recipe and redirects
  - [ ] Integration test: duplicate copy prevention works
  - [ ] Integration test: premium users can copy unlimited recipes

## Dev Notes

### Architecture Patterns

**Event Sourcing:**
- RecipeCopied event records copy operation with original attribution
- New Recipe aggregate created via standard RecipeCreated event
- Original recipe aggregate remains unchanged (no cross-aggregate modification)

**CQRS:**
- Command: CopyRecipeCommand { original_recipe_id, user_id }
- Query: Check if recipe already copied (SELECT * FROM recipes WHERE user_id = ? AND original_recipe_id = ?)
- Query: Count user recipes for freemium enforcement

**Domain Logic:**
- Copy operation loads original aggregate from event stream
- Full recipe data duplicated (ingredients, instructions, timing, tags)
- Attribution metadata added (original_recipe_id, original_author)
- Freemium limit enforced before copy (consistent with CreateRecipe)

### Source Tree Components

**Domain Crate (crates/recipe/):**
- `events.rs`: RecipeCopied event struct
- `commands.rs`: CopyRecipeCommand, copy_recipe handler
- `aggregate.rs`: RecipeAggregate (no changes - uses existing RecipeCreated event)
- `read_model.rs`: Query to check if recipe already copied by user

**Root Binary (src/):**
- `routes/recipes.rs`: POST /discover/:id/copy handler
- `middleware/auth.rs`: Protect copy endpoint (already exists)

**Templates (templates/):**
- `pages/discover-detail.html`: Add "Add to My Recipes" button with conditional rendering

**Database:**
- No new migrations needed - uses existing `recipes` table
- New optional fields in recipes table (if not already present):
  - `original_recipe_id TEXT` - References original community recipe
  - `original_author TEXT` - Username of original creator

### Testing Standards

**TDD Approach:**
1. Write failing test for copy_recipe command
2. Implement CopyRecipeCommand and RecipeCopied event
3. Test passes, add freemium limit test
4. Implement limit check, test passes
5. Add route handler test, implement handler

**Coverage:**
- Unit tests: Command logic, freemium enforcement, duplicate prevention (>=80% coverage)
- Integration tests: All routes (POST /discover/:id/copy), database interactions
- E2E tests: Full user journey (browse, copy, verify in library)

**Test Data:**
- Use shared test fixtures from tests/common/fixtures.rs
- Create test community recipe (shared = true)
- Create test user (free tier and premium tier)
- Verify copy creates independent aggregate

### Project Structure Notes

**Alignment with solution-architecture.md:**
- Event sourcing via evento with RecipeCopied event (section 3.1)
- Recipe domain crate in crates/recipe/ (section 11.1)
- Server-rendered HTML with Askama templates (section 7.1)
- Auth middleware protection for authenticated actions (section 5.3)
- Freemium enforcement pattern (section 5.4)

**Alignment with tech-spec-epic-2.md:**
- RecipeAggregate pattern already established (Story 2.1-2.3)
- Copy follows same event sourcing pattern as Create
- Attribution metadata extends existing recipe model
- Community discovery routes already exist (Story 2.8)

**No Detected Conflicts:**
- Recipe domain already established in crates/recipe/
- Copy operation reuses existing RecipeCreated event for new aggregate
- Original recipe immutability preserved (no cross-aggregate updates)
- Follows existing pattern from Stories 2.1-2.9

### References

- [Source: docs/epics.md#Story 2.10] - Story definition and acceptance criteria
- [Source: docs/tech-spec-epic-2.md#Commands] - Recipe command patterns and freemium enforcement
- [Source: docs/solution-architecture.md#Section 3.2] - Event sourcing pattern with evento
- [Source: docs/solution-architecture.md#Section 11.3] - Inter-domain communication (no direct crate calls)
- [Source: docs/PRD.md#FR-15] - Freemium access controls (10 recipe limit)
- [Source: docs/solution-architecture.md#Section 4.2] - HTML endpoints and form handling patterns

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.10.xml`

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
