# Story 2.1: Recipe Creation (Four Types)

Status: ready-for-dev

## Story

As a user,
I want to create recipes with explicit type selection (Appetizer, Main Course, Dessert, Accompaniment),
so that I can build my recipe library for meal planning.

## Acceptance Criteria

1. Recipe aggregate with RecipeCreated event including: recipe_type (enum), name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text
2. Main courses include accepts_accompaniment field (defaults to false)
3. Recipe creation command validates required fields using validator crate
4. Recipe creation form with type selector and conditional fields (accompaniment field shown only for Main Course)
5. Recipe projection table stores all recipe data for querying
6. User can view their recipe list filtered by type
7. Tests verify recipe creation for all four types with validation

## Tasks / Subtasks

- [ ] Create recipe bounded context crate `imkitchen-recipe` (AC: #1)
  - [ ] Add crate to workspace Cargo.toml with dependencies (evento, validator, serde, bincode)
  - [ ] Create src/lib.rs, src/event.rs, src/aggregate.rs, src/command.rs
  - [ ] Define RecipeType enum (Appetizer, MainCourse, Dessert, Accompaniment)

- [ ] Implement Recipe events (AC: #1, #2)
  - [ ] Define EventMetadata struct with user_id (optional) and request_id (ULID)
  - [ ] Define RecipeCreated event with all required fields
  - [ ] Include accepts_accompaniment field for main courses (default false)
  - [ ] Use bincode derive macros (Encode, Decode)

- [ ] Implement Recipe aggregate (AC: #1)
  - [ ] Define Recipe struct with recipe_type, name, owner_id, etc.
  - [ ] Implement evento aggregator with recipe_created handler
  - [ ] Apply event to populate aggregate state

- [ ] Implement Recipe creation command (AC: #1, #3)
  - [ ] Define CreateRecipeInput struct with validator constraints
  - [ ] Create Command struct with Executor and validation_pool
  - [ ] Implement create_recipe method using evento::create
  - [ ] Validate required fields (name, recipe_type, ingredients, instructions)

- [ ] Create recipe migrations (AC: #5)
  - [ ] Create migrations/queries/20250101000002_recipes.sql
  - [ ] Define recipes table with all columns including owner_id FK
  - [ ] Add indexes on owner_id and recipe_type

- [ ] Implement recipe query handler (AC: #5)
  - [ ] Create src/queries/recipes.rs
  - [ ] Define on_recipe_created handler to insert into recipes table
  - [ ] Create subscribe_recipe_query function returning SubscriptionBuilder
  - [ ] Use event.timestamp for created_at field

- [ ] Create recipe creation route and template (AC: #4)
  - [ ] Create src/routes/recipes/create.rs with GET/POST handlers
  - [ ] Create templates/pages/recipes/create.html with Askama template
  - [ ] Add recipe type selector (dropdown or radio buttons)
  - [ ] Show accepts_accompaniment toggle only for Main Course type
  - [ ] Use Twinspark for conditional field display
  - [ ] Include dietary restrictions multi-select

- [ ] Create recipe list route and template (AC: #6)
  - [ ] Create src/routes/recipes/list.rs
  - [ ] Implement get_user_recipes query function with type filter
  - [ ] Create templates/pages/recipes/list.html
  - [ ] Add filter UI for recipe types (All, Appetizer, Main, Dessert, Accompaniment)
  - [ ] Display recipe cards with type badges (color-coded per mockup)

- [ ] Write integration tests (AC: #7)
  - [ ] Create tests/recipes_test.rs
  - [ ] Set up test database using sqlx::migrate! and evento::sql_migrator
  - [ ] Test creating all four recipe types (Appetizer, MainCourse, Dessert, Accompaniment)
  - [ ] Test validation failures (missing required fields)
  - [ ] Test accepts_accompaniment defaults to false
  - [ ] Verify projection updates using get_user_recipes query

- [ ] Add recipe routes to server (AC: #4, #6)
  - [ ] Register recipe routes in src/server.rs
  - [ ] Create recipe router with /recipes/new, /recipes, /recipes/{id}
  - [ ] Protect routes with auth middleware

## Dev Notes

- **Event-Driven Architecture**: Use evento::create for new recipe aggregates. Commands return immediately after emitting events; query handlers update projections asynchronously [Source: docs/architecture.md#Command Pattern]
- **Bounded Context**: Recipe domain lives in `crates/imkitchen-recipe/` following DDD principles with complete isolation from user/mealplan domains [Source: docs/architecture.md#Project Structure]
- **Database Separation**: Write DB (evento.db) managed by evento, read DB (queries.db) for projections, validation DB for uniqueness checks [Source: docs/architecture.md#Database Separation]
- **Form Handling**: Use `axum_extra::extract::Form` for form data extraction (Axum 0.8+) [Source: CLAUDE.md#Axum Guidelines]
- **Recipe Types**: Four types map to meal slots: Appetizer, Main Course (with optional accompaniment), Dessert, Accompaniment [Source: docs/PRD.md#FR004]
- **Accepts Accompaniment**: Main courses default to `accepts_accompaniment=false`; users must explicitly enable for pairing eligibility [Source: docs/epics.md#Story 2.1]

### Project Structure Notes

- **Crate Location**: `crates/imkitchen-recipe/` - new bounded context crate following workspace pattern
- **Routes**: `src/routes/recipes/create.rs`, `src/routes/recipes/list.rs`
- **Queries**: `src/queries/recipes.rs` with projection handlers
- **Templates**: `templates/pages/recipes/create.html`, `templates/pages/recipes/list.html`
- **Migrations**: `migrations/queries/20250101000002_recipes.sql`
- **Tests**: `tests/recipes_test.rs` in workspace root

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/architecture.md#Epic to Architecture Mapping] - Recipe bounded context mapping
- [docs/architecture.md#Core Tables (Read DB)] - recipes table schema
- [docs/architecture.md#Command Pattern] - Command implementation pattern
- [docs/architecture.md#Query Pattern] - Query handler pattern
- [docs/epics.md#Story 2.1] - Full acceptance criteria and user story
- [docs/PRD.md#FR004-FR006] - Recipe management functional requirements
- [CLAUDE.md#Command Guidelines] - evento command patterns
- [CLAUDE.md#Query Guidelines] - Query projection patterns
- [CLAUDE.md#Axum Guidelines] - Form extraction and route parameters

## Dev Agent Record

### Context Reference

- docs/stories/2-1-recipe-creation-four-types.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
