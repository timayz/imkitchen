# Story 2.1: Recipe Creation (Four Types)

Status: drafted

## Story

As a user,
I want to create recipes with explicit type selection (Appetizer, Main Course, Dessert, Accompaniment),
So that I can build my recipe library for meal planning.

## Acceptance Criteria

1. Recipe aggregate with RecipeCreated event including: recipe_type (enum), name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text
2. Main courses include accepts_accompaniment field (defaults to false)
3. Recipe creation command validates required fields using validator crate
4. Recipe creation form with type selector and conditional fields (accompaniment field shown only for Main Course)
5. Recipe projection table stores all recipe data for querying
6. User can view their recipe list filtered by type
7. Tests verify recipe creation for all four types with validation

## Tasks / Subtasks

- [ ] Create Recipe bounded context crate (Task AC: #1)
  - [ ] Set up `crates/imkitchen-recipe/` workspace crate with dependencies
  - [ ] Create `src/lib.rs` with public exports
  - [ ] Create `src/event.rs` with EventMetadata and RecipeCreated event
  - [ ] Create `src/aggregate.rs` with Recipe aggregate root
  - [ ] Create `src/command.rs` with Command struct and create_recipe method

- [ ] Implement RecipeCreated event schema (AC: #1, #2)
  - [ ] Define RecipeType enum (Appetizer, MainCourse, Dessert, Accompaniment)
  - [ ] Define RecipeCreated struct with all fields (recipe_type, name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text, accepts_accompaniment)
  - [ ] Derive bincode Encode/Decode for event serialization
  - [ ] Define EventMetadata with user_id and request_id (ULID)

- [ ] Implement Recipe aggregate (AC: #1)
  - [ ] Create Recipe struct with recipe_type, name, and other core fields
  - [ ] Implement #[evento::aggregator] on Recipe
  - [ ] Add recipe_created event handler to update aggregate state
  - [ ] Derive bincode Encode/Decode for aggregate

- [ ] Implement create_recipe command with validation (AC: #1, #2, #3)
  - [ ] Define CreateRecipeInput struct with validator derives
  - [ ] Add validation rules: required fields (name, ingredients, instructions, recipe_type)
  - [ ] Set accepts_accompaniment default to false for MainCourse type
  - [ ] Implement Command::create_recipe using evento::create pattern
  - [ ] Return recipe_id on successful creation

- [ ] Create read database migration for recipes table (AC: #5)
  - [ ] Create `migrations/queries/{timestamp}_recipes.sql`
  - [ ] Define recipes table schema with all fields from RecipeCreated event
  - [ ] Add indexes on owner_id and recipe_type for filtering
  - [ ] Include is_shared BOOLEAN field (defaults to 0) for future sharing feature

- [ ] Implement query handler for RecipeCreated (AC: #5)
  - [ ] Create `src/queries/recipes.rs` in main binary
  - [ ] Implement on_recipe_created handler to project event to recipes table
  - [ ] Use event.timestamp for created_at field
  - [ ] Create subscribe_recipe_query subscription builder function

- [ ] Implement recipe list query with type filter (AC: #6)
  - [ ] Add get_user_recipes query function with optional recipe_type filter
  - [ ] Query recipes table with user_id and optional type filter
  - [ ] Return Vec<RecipeRow> with all recipe data

- [ ] Create recipe creation form route (AC: #4)
  - [ ] Create `src/routes/recipes/create.rs` with GET handler for form
  - [ ] Create `templates/pages/recipes/create.html` with Askama template
  - [ ] Add recipe type selector (dropdown or radio buttons)
  - [ ] Add conditional accepts_accompaniment checkbox (shown only for MainCourse)
  - [ ] Include all required fields with Tailwind styling

- [ ] Implement recipe creation POST handler (AC: #4)
  - [ ] Create POST /recipes route handler
  - [ ] Extract Form data using axum_extra::extract::Form
  - [ ] Generate metadata with user_id from JWT and new ULID request_id
  - [ ] Call Command::create_recipe with input and metadata
  - [ ] Return success template or error template on failure

- [ ] Create recipe list view route (AC: #6)
  - [ ] Create `src/routes/recipes/list.rs` with GET handler
  - [ ] Call get_user_recipes query with optional type filter from query params
  - [ ] Create `templates/pages/recipes/list.html` template showing recipes
  - [ ] Add type filter UI (tabs or dropdown) with Twinspark

- [ ] Write unit tests for command validation (AC: #7)
  - [ ] Create `tests/recipes_test.rs`
  - [ ] Test create_recipe with all four recipe types
  - [ ] Test validation failures (missing required fields)
  - [ ] Test accepts_accompaniment defaults to false for MainCourse
  - [ ] Use sqlx::migrate! and evento::sql_migrator for database setup

- [ ] Write integration tests for query projection (AC: #7)
  - [ ] Test RecipeCreated event projects to recipes table correctly
  - [ ] Use subscribe_recipe_query().unsafe_oneshot() for synchronous processing
  - [ ] Verify all event fields stored in projection
  - [ ] Test get_user_recipes query returns correct data

- [ ] Write E2E test for recipe creation flow (AC: #7)
  - [ ] Create Playwright test in `tests/e2e/recipe_creation.spec.ts`
  - [ ] Test full flow: login → create recipe form → submit → view in list
  - [ ] Verify conditional accepts_accompaniment field for MainCourse
  - [ ] Test all four recipe types can be created

## Dev Notes

### Architecture Patterns

**Event-Driven CQRS (per architecture.md):**
- Write path: Route → Command → evento::create → Write DB → Event emitted
- Read path: Event → Query Handler → Read DB → Route → Template
- Separate databases: evento.db (write), queries.db (read)
- Commands enforce business rules, queries optimize for reads

**Bounded Context Pattern (per CLAUDE.md):**
- Create `crates/imkitchen-recipe/` with command.rs, event.rs, aggregate.rs
- Recipe aggregate is the only aggregate in this bounded context
- NO dependencies on other bounded contexts (strict isolation)

**Validation Strategy (per CLAUDE.md):**
- Synchronous validation in command (validator crate)
- Command returns errors directly (no error events)
- Async validation would be deferred to command handler (not needed here)

**Testing Strategy (per CLAUDE.md):**
- Use sqlx::migrate! for database setup (NEVER direct SQL)
- Use subscribe_recipe_query().unsafe_oneshot() for synchronous event processing
- Validate using query functions (NEVER direct SELECT)

### Project Structure Alignment

**Files to Create:**
```
crates/imkitchen-recipe/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── command.rs
│   ├── event.rs
│   └── aggregate.rs

src/
├── queries/
│   └── recipes.rs
└── routes/
    └── recipes/
        ├── mod.rs
        ├── create.rs
        └── list.rs

migrations/queries/
└── {timestamp}_recipes.sql

templates/pages/recipes/
├── create.html
└── list.html

tests/
├── recipes_test.rs
└── e2e/
    └── recipe_creation.spec.ts
```

**Database Schema (from architecture.md, line 570):**
```sql
CREATE TABLE recipes (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    recipe_type TEXT NOT NULL,  -- 'Appetizer' | 'MainCourse' | 'Dessert' | 'Accompaniment'
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,  -- JSON array
    instructions TEXT NOT NULL,
    dietary_restrictions TEXT,  -- JSON array
    cuisine_type TEXT,
    complexity TEXT,
    advance_prep_text TEXT,
    accepts_accompaniment BOOLEAN DEFAULT 0,
    is_shared BOOLEAN DEFAULT 0,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE INDEX idx_recipes_owner ON recipes(owner_id);
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = 1;
```

### Technical Constraints

**Recipe Type Enum:**
- Must use PascalCase variants per CLAUDE.md: Appetizer, MainCourse, Dessert, Accompaniment
- Store as TEXT in database (not integer)
- Validate in command using validator crate

**Ingredients & Dietary Restrictions:**
- Store as JSON arrays in TEXT columns
- Use serde_json for serialization/deserialization
- Example: `["2 lbs chicken", "1 cup rice"]`

**accepts_accompaniment Field:**
- Only applicable to MainCourse type
- Defaults to false per PRD FR006
- UI shows checkbox only when MainCourse selected (Twinspark conditional)

**Conditional UI Logic:**
- Use Twinspark attributes for showing/hiding accepts_accompaniment checkbox
- Example: `ts-trigger="change" ts-action="..."`

### Mockup Reference

**Visual Reference:** `mockups/recipe-create.html` (per epics.md line 168)
- Recipe creation form with all 4 types
- Explicit field configuration
- Main course accepts_accompaniment toggle
- All fields visible in mockup should be implemented

### References

- [Source: docs/PRD.md#FR004-FR006] Recipe functional requirements
- [Source: docs/epics.md#Story-2.1] Story acceptance criteria
- [Source: docs/architecture.md#Recipe-Management] Bounded context structure
- [Source: docs/architecture.md#Data-Architecture] Database schema
- [Source: CLAUDE.md#Command-Guidelines] Event-driven patterns
- [Source: CLAUDE.md#Evento-Handler-Guidelines] Query handler patterns
- [Source: mockups/recipe-create.html] Visual design reference

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
