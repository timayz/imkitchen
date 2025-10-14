# Story 2.1: Create Recipe

Status: ContextReadyDraft

## Story

As a user,
I want to create a new recipe with all details,
so that I can add it to my meal planning rotation.

## Acceptance Criteria

1. Recipe creation form includes: title, ingredients (quantity/unit/name), step-by-step instructions, prep time, cook time, advance prep requirements, serving size
2. Ingredients list allows adding/removing rows dynamically
3. Instructions allow numbered step entry with reordering capability
4. Each instruction step includes optional timer field (duration in minutes)
5. Advance prep field accepts text description (e.g., "Marinate 4 hours")
6. All required fields validated before save
7. Successful save creates recipe and displays confirmation
8. User redirected to recipe detail page after creation
9. Recipe automatically owned by creating user
10. Default privacy set to "private"

## Tasks / Subtasks

- [ ] Design and implement recipe creation form UI (AC: 1, 2, 3, 4, 5)
  - [ ] Create Askama template for recipe form at `templates/pages/recipe-form.html`
  - [ ] Implement dynamic ingredient row addition/removal with TwinSpark
  - [ ] Implement instruction step entry with reordering (drag-drop or up/down buttons)
  - [ ] Add optional timer field per instruction step
  - [ ] Add advance prep text field with validation
  - [ ] Apply Tailwind CSS styling consistent with design system

- [ ] Implement Recipe domain aggregate and events (AC: 6, 7, 8, 9, 10)
  - [ ] Create `crates/recipe/src/aggregate.rs` with RecipeAggregate struct
  - [ ] Define RecipeCreated event in `crates/recipe/src/events.rs`
  - [ ] Implement CreateRecipe command in `crates/recipe/src/commands.rs`
  - [ ] Add evento aggregate handler for RecipeCreated event
  - [ ] Validate required fields: non-empty title, at least 1 ingredient, at least 1 instruction
  - [ ] Enforce free tier 10-recipe limit (check user tier from JWT claims)
  - [ ] Set default privacy to "private" and owner_id from authenticated user

- [ ] Create recipe HTTP route handlers (AC: 6, 7, 8)
  - [ ] Add GET /recipes/new route in `src/routes/recipes.rs` to serve recipe form
  - [ ] Add POST /recipes route to handle form submission
  - [ ] Parse multipart form data with ingredients and instructions as JSON strings
  - [ ] Validate form inputs using validator crate
  - [ ] Invoke CreateRecipe command via recipe domain crate
  - [ ] Return 302 redirect to /recipes/:id on success
  - [ ] Return 422 with form re-rendered with errors on validation failure
  - [ ] Implement auth middleware requirement for both routes

- [ ] Implement recipe read model projection (AC: 8)
  - [ ] Create migration `migrations/002_create_recipes_table.sql` with recipes table schema
  - [ ] Define read model projection in `crates/recipe/src/read_model.rs`
  - [ ] Register evento subscription handler for RecipeCreated event
  - [ ] Project event data to recipes table (INSERT query via SQLx)
  - [ ] Include fields: id, user_id, title, ingredients (JSON), instructions (JSON), prep_time_min, cook_time_min, advance_prep_hours, serving_size, is_private, created_at, updated_at

- [ ] Create recipe detail page (AC: 8)
  - [ ] Add GET /recipes/:id route in `src/routes/recipes.rs`
  - [ ] Query recipe from read model by id
  - [ ] Verify ownership or public visibility
  - [ ] Create Askama template at `templates/pages/recipe-detail.html`
  - [ ] Display recipe title, ingredients, instructions, timing, advance prep
  - [ ] Add "Edit Recipe" button (only for owner)
  - [ ] Add "Delete Recipe" button (only for owner)
  - [ ] Style with Tailwind CSS

- [ ] Test recipe creation (AC: 1-10)
  - [ ] Unit test: RecipeAggregate validates required fields
  - [ ] Unit test: RecipeCreated event applied correctly
  - [ ] Unit test: Free tier recipe limit enforced (11th recipe fails)
  - [ ] Integration test: POST /recipes creates recipe and redirects
  - [ ] Integration test: Recipe read model projection inserts into database
  - [ ] Integration test: GET /recipes/:id displays created recipe
  - [ ] E2E test: Complete recipe creation flow from form to detail page
  - [ ] E2E test: Validation errors displayed when required fields missing

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- All recipe state changes captured as immutable events
- RecipeCreated event is the initial event for the aggregate
- evento handles event persistence to SQLite event store
- Read model projections update recipes table via subscriptions

**CQRS Pattern:**
- Commands (CreateRecipe) write events to event stream
- Queries (GetRecipe) read from materialized recipes table
- Subscription handlers project events to read models asynchronously

**Domain-Driven Design:**
- Recipe domain crate (`crates/recipe/`) contains all business logic
- HTTP route handlers are thin - validate, invoke domain, render template
- Business rules (recipe limit, ownership) enforced in aggregate

**Server-Side Rendering:**
- Askama templates compile to Rust at build time
- Forms submit via standard POST (progressive enhancement)
- TwinSpark can enhance with AJAX for dynamic ingredient rows
- No client-side state management

**Validation Strategy:**
- Client-side: HTML5 validation attributes (required, min, max)
- Server-side: validator crate with derive macros
- Form re-rendered with inline errors on validation failure (422 status)

**Database Schema:**
- SQLite with SQLx for read models
- evento manages event store schema automatically
- Ingredients and instructions stored as JSON for flexibility
- Foreign key to users table for ownership

### Project Structure Notes

**New Files to Create:**
```
crates/recipe/
├── Cargo.toml (new crate)
├── src/
│   ├── lib.rs
│   ├── aggregate.rs (RecipeAggregate)
│   ├── commands.rs (CreateRecipe)
│   ├── events.rs (RecipeCreated)
│   ├── read_model.rs (projections)
│   └── error.rs (domain errors)
├── tests/
│   └── create_recipe_tests.rs

src/routes/
├── recipes.rs (new file - recipe routes)

templates/pages/
├── recipe-form.html (new template)
├── recipe-detail.html (new template)

migrations/
├── 002_create_recipes_table.sql (new migration)
```

**Existing Files to Modify:**
- `src/main.rs`: Register recipe crate and routes
- `src/routes/mod.rs`: Add recipes module
- `Cargo.toml`: Add recipe crate to workspace members
- `templates/base.html`: Add "My Recipes" navigation link

### Testing Standards Summary

**Unit Tests (crates/recipe/tests/):**
- Test aggregate command handlers in isolation
- Test event application to aggregate state
- Test business rules (recipe limit, validation)
- Mock evento dependencies with in-memory event store

**Integration Tests (tests/recipe_tests.rs):**
- Test full request/response cycle for POST /recipes
- Test read model projection from RecipeCreated event
- Use in-memory SQLite database for isolation
- Test authentication middleware enforcement

**E2E Tests (e2e/tests/recipe-management.spec.ts):**
- Test complete recipe creation flow in real browser
- Test form validation error display
- Test free tier recipe limit (create 10, 11th fails)
- Test recipe ownership (owner can edit, others cannot)

**Coverage Target:** 80% code coverage per NFRs

### References

- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships] - Recipe table schema
- [Source: docs/solution-architecture.md#3.1 Database Schema] - evento event store and read models
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Recipe crate organization
- [Source: docs/tech-spec-epic-2.md#1 Recipe Domain Crate] - Detailed recipe aggregate design
- [Source: docs/epics.md#Story 2.1] - Acceptance criteria and prerequisites
- [Source: docs/PRD.md#FR-1] - Recipe creation functional requirements

## Dev Agent Record

### Context Reference

- [Story Context 2.1](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.1.xml)

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
