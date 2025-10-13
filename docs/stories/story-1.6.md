# Story 1.6: Freemium Tier Enforcement (10 Recipe Limit)

Status: Approved

## Story

As a free tier user,
I want to understand my recipe limit,
so that I know when to upgrade to premium.

## Acceptance Criteria

1. Recipe count displayed on recipe management page (e.g., "7/10 recipes")
2. User can create recipes until limit reached
3. At 10th recipe, system shows "10/10 recipes - Upgrade for unlimited"
4. Attempting to create 11th recipe prevents creation, displays upgrade prompt
5. User can edit or delete existing recipes within limit
6. Deleting recipe frees up slot for new recipe
7. Recipe limit applies only to user-created recipes (not community-discovered)
8. Premium users see "Unlimited recipes" indicator

## Tasks / Subtasks

- [ ] Add recipe_count tracking to User aggregate (AC: 1, 2, 6)
  - [ ] Add recipe_count field to UserAggregate (crates/user/src/aggregate.rs)
  - [ ] Initialize recipe_count to 0 in user_created event handler
  - [ ] Add RecipeCreated event handler to increment recipe_count
  - [ ] Add RecipeDeleted event handler to decrement recipe_count
  - [ ] Update read model projection to maintain recipe_count in users table

- [ ] Implement validate_recipe_creation command (AC: 2, 4)
  - [ ] Create validate_recipe_creation function in crates/user/src/commands.rs
  - [ ] Query user by ID from read model
  - [ ] Check if tier == Free AND recipe_count >= 10
  - [ ] Return UserError::RecipeLimitReached if limit exceeded
  - [ ] Return Ok(()) if premium or under limit

- [ ] Display recipe count on recipe pages (AC: 1, 3, 8)
  - [ ] Add recipe count query to recipe list page handler
  - [ ] Create recipe_count_badge component in templates/components/
  - [ ] Show "X/10 recipes" for free users
  - [ ] Show "Unlimited recipes" badge for premium users
  - [ ] Display on recipe library page header

- [ ] Integrate validation in recipe creation flow (AC: 4)
  - [ ] Call validate_recipe_creation before recipe creation command
  - [ ] Handle UserError::RecipeLimitReached in route handler
  - [ ] Display upgrade prompt modal/message on error
  - [ ] Include "Upgrade to Premium" button in error message
  - [ ] Prevent recipe creation form submission if limit reached

- [ ] Test freemium enforcement (AC: 1-8)
  - [ ] Unit test: validate_recipe_creation with free user at 9 recipes → Ok
  - [ ] Unit test: validate_recipe_creation with free user at 10 recipes → RecipeLimitReached
  - [ ] Unit test: validate_recipe_creation with premium user at 50 recipes → Ok
  - [ ] Integration test: Create 10 recipes as free user, 11th attempt returns 422
  - [ ] Integration test: Delete recipe, recipe_count decrements, can create new recipe
  - [ ] Integration test: Premium user can create unlimited recipes
  - [ ] E2E test: Free user hits limit → sees upgrade prompt → upgrades → can create more

## Dev Notes

### Architecture Patterns

**Domain Event Sourcing**:
- `RecipeCreated` and `RecipeDeleted` events trigger recipe_count updates
- User aggregate tracks recipe_count for quick validation
- Read model (users table) mirrors recipe_count for query optimization

**Freemium Business Logic**:
- Validation at domain boundary (validate_recipe_creation)
- Recipe limit enforced BEFORE RecipeCreated event is emitted
- Premium tier bypasses all freemium restrictions
- Limit applies only to user-owned recipes (not community copies)

**Error Handling**:
- `UserError::RecipeLimitReached` returned from validation
- Route handler converts to 422 Unprocessable Entity with upgrade prompt
- Frontend displays modal/toast with "Upgrade to Premium" CTA

### Source Tree Components

**Domain Crate** (crates/user/):
- commands.rs: `validate_recipe_creation(user_id, executor)`
- aggregate.rs: `recipe_created(&mut self)`, `recipe_deleted(&mut self)` event handlers
- error.rs: `UserError::RecipeLimitReached` variant
- read_model.rs: Projection updates recipe_count in users table

**Recipe Domain** (crates/recipe/):
- commands.rs: `create_recipe` calls `user::validate_recipe_creation` before proceeding
- Integration point: Recipe creation blocked if validation fails

**Templates** (templates/):
- components/recipe-count-badge.html: Recipe count display component
- pages/recipe-list.html: Displays badge in header
- components/upgrade-modal.html: Upgrade prompt when limit reached

**Routes** (src/routes/):
- recipe.rs: GET /recipes shows recipe count, POST /recipes validates limit
- Error handler converts RecipeLimitReached to upgrade prompt response

### Testing Standards

**Unit Tests** (crates/user/tests/):
- Test validate_recipe_creation with various tier/count combinations
- Test RecipeCreated/RecipeDeleted event handlers update recipe_count
- Verify premium tier bypasses limit

**Integration Tests** (tests/recipe_tests.rs):
- Create 10 recipes as free user, verify 11th fails
- Delete recipe, verify count decrements and slot freed
- Premium user creates 50+ recipes without error

**E2E Tests** (e2e/tests/freemium.spec.ts):
- Complete user journey: Register → Create 10 recipes → Hit limit → Upgrade → Create more

### References

**Architecture**:
- [Source: docs/solution-architecture.md#Section 3.2] - users table schema with recipe_count field
- [Source: docs/solution-architecture.md#ADR-006] - Freemium model with 10 recipe limit rationale

**Epic Specification**:
- [Source: docs/epics.md#Story 1.6] - Original story definition
- [Source: docs/tech-spec-epic-1.md#AC-8.1 to AC-8.4] - Authoritative acceptance criteria for freemium enforcement
- [Source: docs/tech-spec-epic-1.md#Commands/validate_recipe_creation] - Implementation specification

**Domain Events**:
- [Source: docs/tech-spec-epic-1.md#Events] - RecipeCreated, RecipeDeleted event definitions
- [Source: docs/tech-spec-epic-1.md#Traceability/AC-8.1 to AC-8.4] - Test approach for freemium enforcement

### Project Structure Notes

**Alignment with unified-project-structure.md**:
- User domain enforces recipe limit (business rule ownership)
- Recipe domain queries user domain for validation (cross-domain dependency)
- Clear separation: user owns tier/limits, recipe owns recipe data

**Cross-Domain Integration**:
- Recipe creation command calls `user::validate_recipe_creation` before proceeding
- Loose coupling via function call (not direct aggregate access)
- User domain error propagated to recipe route handler

**Rationale for structure**:
- Freemium logic belongs in user domain (authentication/authorization concern)
- Recipe domain focuses on recipe CRUD, delegates tier validation to user domain
- Maintains single responsibility principle

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-1.6.xml`
- Generated: 2025-10-13T19:45:00Z
- Epic ID: 1, Story ID: 6

### Agent Model Used

<!-- Model version will be recorded here -->

### Debug Log References

<!-- Links to debug logs will be added during implementation -->

### Completion Notes List

<!-- Implementation notes will be added as work progresses -->

### File List

<!-- Created/modified files will be listed here during implementation -->
