# Epic Technical Specification: Recipe Management & Import System

Date: 2025-11-03
Author: Jonathan
Epic ID: 2
Status: Draft

---

## Overview

Epic 2 establishes the recipe management foundation for imkitchen by implementing a complete recipe lifecycle system supporting four explicit recipe types (Appetizer, Main Course, Dessert, Accompaniment) with evento-driven CQRS architecture. This epic enables users to manually create, edit, delete, and bulk import recipes via validated JSON files, while introducing the freemium favorite system (10-recipe limit for free tier) that drives premium conversion and the recipe snapshot mechanism that ensures meal plan historical accuracy despite recipe modifications or deletions.

The epic delivers both immediate user value (recipe library building through manual and bulk import) and critical technical infrastructure (snapshot isolation, freemium access control patterns) required for subsequent meal planning algorithm implementation in Epic 3.

## Objectives and Scope

**In Scope:**
- Recipe CRUD operations (create, read, update, soft delete) for all four recipe types
- evento-based Recipe aggregate in `imkitchen-recipe` bounded context crate
- Recipe projection tables in read database (queries.db) for efficient querying
- Recipe favorites system with freemium tier enforcement (free: 10 max, premium: unlimited)
- Bulk JSON recipe import with streaming parser (max 10MB per file, 20 files per batch)
- Real-time import progress tracking via Twinspark polling
- Malicious content detection and validation against JSON Schema v1.0
- Duplicate recipe detection (name matching + fuzzy ingredient similarity)
- Publicly accessible versioned JSON schema documentation (/api/schema/recipe/v1.0)
- Recipe snapshot system storing complete recipe copies for meal plan isolation
- Cascade deletion of favorites when recipe deleted (silent, no notifications)
- Soft delete pattern for recipes with deleted_at timestamp

**Out of Scope (Deferred to Later Epics):**
- Recipe sharing and community visibility (Epic 5)
- Recipe rating and review system (Epic 5)
- Meal plan generation algorithm integration (Epic 3)
- Shopping list generation from recipes (Epic 4)
- Recipe search and filtering UI enhancements beyond basic type filtering
- Recipe versioning and edit history tracking
- Nutrition information and macro tracking
- Recipe scaling for serving size adjustment

## System Architecture Alignment

**Bounded Context:** `crates/imkitchen-recipe/`
- Implements Recipe aggregate root with evento pattern
- Commands: create_recipe, update_recipe, delete_recipe, favorite_recipe, unfavorite_recipe, import_recipes
- Events: RecipeCreated, RecipeUpdated, RecipeDeleted, RecipeFavorited, RecipeUnfavorited
- Aggregate maintains minimal state: recipe_type, owner_id, name, favorite_count, deleted_at

**Database Architecture:**
- **Write DB (evento.db):** Event store managed exclusively by evento
- **Read DB (queries.db):** Projections in `recipes`, `recipe_favorites`, `meal_plan_recipe_snapshots` tables
- **Validation DB (validation.db):** NOT used in Epic 2 (no unique constraints requiring async validation)

**Route Handlers (Main Binary):**
- `src/routes/recipes/create.rs` - Recipe creation form and handler
- `src/routes/recipes/edit.rs` - Recipe editing with ownership validation
- `src/routes/recipes/list.rs` - User's recipe library with type filtering
- `src/routes/recipes/favorite.rs` - Favorite toggle with freemium checks
- `src/routes/recipes/import.rs` - Streaming JSON import with progress tracking
- `src/routes/schema.rs` - Public JSON schema documentation endpoints

**Query Handlers (Main Binary):**
- `src/queries/recipes.rs` - Recipe projection handlers and query functions
- `src/queries/snapshots.rs` - Snapshot creation and retrieval functions

**Access Control Integration:**
- Uses existing `src/access_control.rs` service for freemium favorite limit enforcement
- Checks: is_premium_active OR premium_bypass OR global_premium_bypass
- Free tier: 10 max favorites, Premium tier: unlimited

**Architecture Constraints:**
- All commands complete in <10 seconds (graceful shutdown limit)
- Query handlers must be idempotent (event replay support)
- NEVER use projections in commands (eventual consistency risk)
- Streaming JSON parser required for 10MB file support (memory efficiency)
- Soft delete pattern required (preserve event history, enable data recovery)

## Detailed Design

### Services and Modules

| Module | Responsibility | Inputs | Outputs | Owner |
|--------|---------------|--------|---------|-------|
| `imkitchen-recipe` (crate) | Recipe domain logic, aggregate root, commands | CreateRecipeInput, UpdateRecipeInput, etc. | Events (RecipeCreated, etc.), Result<String> | Recipe bounded context |
| `src/routes/recipes/create.rs` | Recipe creation form/handler | HTTP Form (CreateRecipeForm) | HTML (recipe form/success) | Main binary |
| `src/routes/recipes/edit.rs` | Recipe editing with ownership check | HTTP Form + recipe_id | HTML (edit form/success) | Main binary |
| `src/routes/recipes/list.rs` | Recipe library display with filters | Query params (type filter) | HTML (recipe list) | Main binary |
| `src/routes/recipes/favorite.rs` | Favorite toggle with tier checks | recipe_id, user_id | HTML partial (button state/upgrade modal) | Main binary |
| `src/routes/recipes/import.rs` | Streaming JSON import handler | Multipart files (JSON) | HTML partial (progress/summary) | Main binary |
| `src/routes/schema.rs` | JSON schema API/docs | None (public) | JSON schema / HTML docs | Main binary |
| `src/queries/recipes.rs` | Recipe projection handlers/queries | Events, query params | Projections, query results | Main binary |
| `src/queries/snapshots.rs` | Snapshot creation/retrieval | recipe_id, meal_plan_id | Snapshot records | Main binary |
| `src/access_control.rs` (existing) | Freemium tier enforcement | user_id, feature check | bool (allowed/denied) | Main binary |

### Data Models and Contracts

**Recipe Aggregate (evento):**
```rust
#[derive(Default, Encode, Decode, Clone)]
pub struct Recipe {
    pub owner_id: String,
    pub recipe_type: RecipeType,
    pub name: String,
    pub favorite_count: u32,
    pub deleted_at: Option<i64>,
}

#[derive(Encode, Decode, Clone)]
pub enum RecipeType {
    Appetizer,
    MainCourse,
    Dessert,
    Accompaniment,
}
```

**Recipe Projection (Read DB):**
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
    accepts_accompaniment BOOLEAN DEFAULT 0,  -- Main courses only
    is_shared BOOLEAN DEFAULT 0,
    deleted_at INTEGER,  -- Soft delete timestamp
    created_at INTEGER NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE INDEX idx_recipes_owner ON recipes(owner_id);
CREATE INDEX idx_recipes_type ON recipes(recipe_type);
CREATE INDEX idx_recipes_deleted ON recipes(deleted_at);
```

**Recipe Favorites Projection:**
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
```

**Recipe Snapshots (for meal plans):**
```sql
CREATE TABLE meal_plan_recipe_snapshots (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    day_index INTEGER NOT NULL CHECK (day_index >= 0 AND day_index <= 6),
    meal_slot TEXT NOT NULL,  -- 'appetizer' | 'main' | 'dessert' | 'accompaniment'
    original_recipe_id TEXT,  -- May be null if recipe deleted
    recipe_type TEXT NOT NULL,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,  -- JSON array
    instructions TEXT NOT NULL,
    dietary_restrictions TEXT,  -- JSON array
    cuisine_type TEXT,
    snapshot_at INTEGER NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_snapshots_meal_plan ON meal_plan_recipe_snapshots(meal_plan_id);
```

**Event Metadata (all events):**
```rust
#[derive(Encode, Decode, Clone)]
pub struct EventMetadata {
    pub user_id: Option<String>,
    pub request_id: String,  // ULID for request tracing
}
```

### APIs and Interfaces

**Recipe Creation:**
- **Route:** `POST /recipes`
- **Input:** `CreateRecipeForm` (multipart/form-data)
  - name: String (required)
  - recipe_type: RecipeType enum (required)
  - ingredients: String (textarea, JSON array on backend)
  - instructions: String (textarea, required)
  - dietary_restrictions: Vec<String> (multi-select)
  - cuisine_type: String (optional)
  - complexity: String (optional)
  - advance_prep_text: String (optional)
  - accepts_accompaniment: bool (Main courses only, default false)
- **Output:** HTML (success template with recipe_id or error form)
- **Command:** `recipe_command.create_recipe(input, metadata)` → Result<String>
- **Events:** RecipeCreated

**Recipe Editing:**
- **Route:** `GET /recipes/{id}/edit` (form), `POST /recipes/{id}/edit` (submit)
- **Input:** Same as creation + recipe_id
- **Output:** HTML (edit form pre-populated or success)
- **Command:** `recipe_command.update_recipe(input, metadata)` → Result<()>
- **Events:** RecipeUpdated
- **Validation:** User must own recipe (owner_id == user_id)

**Recipe Deletion:**
- **Route:** `POST /recipes/{id}/delete`
- **Input:** recipe_id
- **Output:** HTML (confirmation modal → success redirect)
- **Command:** `recipe_command.delete_recipe(recipe_id, metadata)` → Result<()>
- **Events:** RecipeDeleted (sets deleted_at timestamp)
- **Side Effect:** Cascade delete favorites via ON DELETE CASCADE FK

**Recipe Favorite Toggle:**
- **Route:** `POST /recipes/{id}/favorite`
- **Input:** recipe_id
- **Output:** HTML partial (updated button state OR upgrade modal)
- **Commands:**
  - `recipe_command.favorite_recipe(input, metadata)` → Result<()>
  - `recipe_command.unfavorite_recipe(input, metadata)` → Result<()>
- **Events:** RecipeFavorited, RecipeUnfavorited
- **Access Control:** Check favorite count before favoriting; free tier max 10

**Recipe Import:**
- **Route:** `POST /recipes/import`
- **Input:** Multipart files (max 20 JSON files, 10MB each)
- **Output:** HTML partial (import_id + progress template)
- **Processing:** Streaming JSON parser, async tokio tasks
- **Command:** `recipe_command.import_recipes(batch, metadata)` → Result<ImportResult>
- **Events:** Multiple RecipeCreated events
- **Validation:** JSON Schema v1.0, malicious content scan, duplicate detection
- **Progress Tracking:** In-memory HashMap<import_id, ImportProgress>

**Import Progress Polling:**
- **Route:** `GET /recipes/import/progress/{import_id}`
- **Input:** import_id
- **Output:** HTML partial (progress counts OR summary when complete)
- **Twinspark:** Polls every 1 second until status=completed

**JSON Schema API:**
- **Route:** `GET /api/schema/recipe/v1.0`
- **Input:** None (public, no auth)
- **Output:** JSON (recipe-v1.0.json schema)
- **Content-Type:** application/json
- **CORS:** Enabled for third-party tools

**Schema Documentation:**
- **Route:** `GET /docs/schema`
- **Input:** None (public)
- **Output:** HTML (schema explanation with examples)

### Workflows and Sequencing

**Recipe Creation Flow:**
1. User navigates to `/recipes/new`
2. GET handler renders recipe creation form (Askama template)
3. User fills form, selects recipe_type, optional accepts_accompaniment (Main only)
4. POST handler extracts Form data, creates CreateRecipeInput
5. Route handler calls `recipe_command.create_recipe(input, metadata)`
6. Command validates input, emits RecipeCreated event via evento::create
7. Query handler (on_recipe_created) inserts into recipes projection
8. Success template returned with recipe_id

**Recipe Import Flow with Progress:**
1. User uploads JSON files to `/recipes/import` (drag-drop or file picker)
2. POST handler validates files (count, size, content-type)
3. Generate import_id (ULID)
4. Spawn tokio task for each file (streaming parser)
5. Return import-progress.html partial with Twinspark polling
6. Browser polls `/recipes/import/progress/{import_id}` every 1s
7. Parser validates each recipe, checks duplicates, scans for malicious content
8. For valid recipes: call create_recipe command → emit RecipeCreated events
9. Update ImportProgress in-memory storage after each recipe
10. Progress partial returns updated counts (imported/failed/remaining)
11. When all files processed, return import-summary.html (polling stops)
12. Summary shows success count, failed count, detailed error list

**Recipe Favorite Flow with Freemium Check:**
1. User clicks favorite button on recipe card
2. POST `/recipes/{id}/favorite` with Twinspark ts-req
3. Route handler queries user's current favorite count
4. Access control check: `access_control.can_add_favorite(user_id)`
   - Premium/bypass: unlimited, proceed
   - Free tier: count >= 10? Return upgrade-modal.html, else proceed
5. If allowed: call `recipe_command.favorite_recipe(input, metadata)`
6. Command emits RecipeFavorited event
7. Query handler inserts into recipe_favorites table
8. Return updated button partial (filled heart icon)

**Recipe Deletion with Cascade:**
1. User clicks delete button → confirmation modal
2. POST `/recipes/{id}/delete` with Twinspark confirmation
3. Route handler validates ownership
4. Call `recipe_command.delete_recipe(recipe_id, metadata)`
5. Command emits RecipeDeleted event (sets deleted_at timestamp)
6. Query handler (on_recipe_deleted) updates recipes table: SET deleted_at = timestamp
7. Database ON DELETE CASCADE automatically removes recipe_favorites entries
8. Redirect to recipe list (recipe no longer visible)

**Snapshot Creation Flow (Called by Meal Plan Generation):**
1. Meal plan generation algorithm selects recipe_id for slot
2. Call `create_recipe_snapshot(recipe_id, meal_plan_id, day_index, meal_slot)`
3. Query current recipe data from recipes projection
4. Insert into meal_plan_recipe_snapshots with all fields copied
5. Return snapshot_id
6. Snapshot preserved even if original recipe modified or deleted

## Non-Functional Requirements

### Performance

**Target Metrics:**
- Recipe creation: <500ms P95 (evento event + projection update)
- Recipe list query: <200ms P95 (indexed query on owner_id)
- Recipe import: <30s per file (streaming parser, 10MB max)
- Import progress polling: <100ms P95 (in-memory HashMap lookup)
- Favorite toggle: <300ms P95 (count query + evento event)
- Snapshot creation: <200ms P95 (single INSERT operation)

**Optimization Strategies:**
- Database indexes on owner_id, recipe_type, deleted_at for fast filtering
- Composite PK on recipe_favorites (user_id, recipe_id) for fast lookups
- Streaming JSON parser prevents 200MB memory spikes (10MB constant)
- In-memory ImportProgress storage (no database I/O for progress)
- Soft delete with index enables fast "not deleted" filtering

**Performance Testing:**
- Benchmark recipe creation with 100 concurrent users (target <500ms P95)
- Test import with 20 files × 10MB (100 recipes each) → <10 minutes total
- Profile favorite count query with 1000+ favorited recipes → <50ms
- Test snapshot creation during batch meal plan generation (21 recipes) → <5s total

### Security

**Authentication & Authorization:**
- All recipe routes protected by JWT middleware (except schema API)
- Ownership validation: only owner can edit/delete recipes
- Favorite toggle validates user_id from JWT token
- Schema API public (no authentication required)

**Input Validation:**
- validator crate constraints on all CreateRecipeInput fields
- Email format, string lengths, required fields enforced
- Recipe_type enum validation (only 4 allowed values)
- accepts_accompaniment conditional validation (Main courses only)

**File Upload Security (OWASP Compliance):**
- Max file size: 10MB enforced at upload
- Content-type validation: application/json only
- Malicious content scan: detect <script>, <iframe>, eval, onclick patterns
- Streaming parser with 30s timeout per file (DoS prevention)
- JSON schema validation against recipe-v1.0.json
- Reject oversized payloads (> 10MB after decompression)
- Log all malicious content attempts for security monitoring

**XSS Prevention:**
- Askama escapes all template variables by default
- NEVER use `safe` filter in recipe display templates
- CSP headers restrict inline scripts

**CSRF Protection:**
- SameSite=Strict cookie attribute on JWT
- Twinspark forms include CSRF tokens (Axum middleware)

**Data Protection:**
- Soft delete preserves recipe data for audit/recovery
- Deleted recipes excluded from all user-facing queries (WHERE deleted_at IS NULL)
- Snapshot system prevents data loss from recipe deletion

### Reliability/Availability

**Error Handling:**
- Commands return Result<T, anyhow::Error> for graceful error propagation
- Route handlers display user-friendly error messages in templates
- Import errors collected per-recipe (don't fail entire batch)
- Invalid recipes skipped with detailed error messages in summary

**Idempotency:**
- Query handlers idempotent (can replay events from beginning)
- Projection updates use INSERT OR REPLACE for recipe favorites
- Snapshot creation checks for existing snapshot before inserting

**Graceful Degradation:**
- Import continues if individual files fail (partial success)
- Progress tracking lost on server restart (in-memory storage)
- Snapshot references preserved even if original recipe deleted

**Data Integrity:**
- Soft delete ensures event history preserved
- ON DELETE CASCADE prevents orphaned favorites
- Foreign key constraints enforce referential integrity
- Transaction boundaries around multi-step operations

### Observability

**Logging (tracing crate):**
- `info!` - Recipe created/updated/deleted, import started/completed
- `warn!` - Validation failures, duplicate recipes blocked, malicious content detected
- `error!` - Command failures, database errors, import errors
- `debug!` - Event processing, projection updates, query executions
- Structured logging with user_id, recipe_id, request_id (ULID) in all logs

**Metrics (Required Signals):**
- Recipe creation rate (recipes/minute)
- Import success rate (successful imports / total attempts)
- Import processing time (P50, P95, P99)
- Favorite count by tier (free vs premium)
- Duplicate detection rate (blocked duplicates / total imports)
- Malicious content detection rate (blocked files / total files)
- Recipe deletion rate (deletions/day)
- Snapshot creation rate (snapshots/meal plan generation)

**Tracing:**
- Request_id (ULID) in EventMetadata traces all events from single user action
- Span tracing for import workflow (file upload → parse → validate → store)
- Query handler spans for projection updates

**Alerts (Future):**
- High import failure rate (>20% over 1 hour)
- Malicious content detection spike
- Database connection pool exhaustion
- Import processing time P95 > 60s

## Dependencies and Integrations

### Rust Dependencies (workspace)

**Core Framework:**
- `evento = "1.5"` (features: sqlite) - Event sourcing and CQRS
- `axum = "0.8"` - Web framework
- `axum-extra = "0.12"` (features: form, query, cookie) - Form extraction
- `tower = "0.5"` - Middleware
- `tokio = "1"` (features: full) - Async runtime

**Database:**
- `sqlx = "0.8"` (features: runtime-tokio-rustls, sqlite) - SQL queries and migrations

**Templating:**
- `askama = "0.14"` - Type-safe HTML templates
- `askama_web = "0.14"` - Askama Axum integration

**Validation & Serialization:**
- `validator = "0.20"` (features: derive) - Input validation
- `serde = "1"` (features: derive) - Serialization
- `serde_json = "1"` - JSON parsing for import
- `bincode = "2"` - Event serialization for evento

**Security:**
- `jsonwebtoken = "10.1"` - JWT validation
- `argon2 = "0.5"` - Password hashing (existing, not used in Epic 2)

**Utilities:**
- `ulid = "1.2"` - Request ID generation
- `anyhow = "1"` - Error handling
- `tracing = "0.1"` - Structured logging
- `tracing-subscriber = "0.3"` - Log output
- `chrono = "0.4"` - Timestamp handling

**Static Assets:**
- `rust-embed = "8.8"` - Embed Twinspark.js, CSS

### External Integrations

**Database Files:**
- `evento.db` - Event store (write DB) managed by evento
- `queries.db` - Projections (read DB) for recipes, favorites, snapshots
- `validation.db` - NOT used in Epic 2

**Static Assets:**
- `static/schemas/recipe-v1.0.json` - JSON Schema Draft 2020-12
- `static/js/twinspark.js` - UI reactivity library
- `static/css/output.css` - Tailwind CSS compiled

**Frontend Dependencies (optional build time):**
- Node.js 22+ for Tailwind CLI (if rebuilding CSS)
- Playwright 1.56+ for E2E tests

### Internal Integrations

**Bounded Context Crates:**
- `imkitchen-recipe` (this epic) - Recipe domain logic
- `imkitchen-user` (Epic 1) - User authentication, access control service

**Access Control Service:**
- Located: `src/access_control.rs`
- Used by: favorite_recipe command
- Method: `can_add_favorite(user_id: &str) -> Result<bool>`
- Checks: is_premium_active OR premium_bypass OR global_premium_bypass

**User Profile Integration:**
- Recipe owner_id references users.id (FK constraint)
- User suspension hides their shared recipes (future Epic 5)

**Meal Plan Integration (Future - Epic 3):**
- Meal plan generation will call `create_recipe_snapshot()`
- Snapshots isolate meal plans from recipe changes/deletions

## Acceptance Criteria (Authoritative)

### Story 2.1: Recipe Creation (Four Types)

**AC-2.1.1:** Recipe aggregate with RecipeCreated event including: recipe_type (enum), name, ingredients, instructions, dietary_restrictions, cuisine_type, complexity, advance_prep_text
**AC-2.1.2:** Main courses include accepts_accompaniment field (defaults to false)
**AC-2.1.3:** Recipe creation command validates required fields using validator crate
**AC-2.1.4:** Recipe creation form with type selector and conditional fields (accompaniment field shown only for Main Course)
**AC-2.1.5:** Recipe projection table stores all recipe data for querying
**AC-2.1.6:** User can view their recipe list filtered by type
**AC-2.1.7:** Tests verify recipe creation for all four types with validation

### Story 2.2: Recipe Editing and Deletion

**AC-2.2.1:** RecipeUpdated event stores changed fields with evento::save pattern
**AC-2.2.2:** RecipeDeleted event marks recipe as deleted with soft delete timestamp
**AC-2.2.3:** Recipe edit form pre-populated with current data
**AC-2.2.4:** Deletion requires confirmation modal
**AC-2.2.5:** Deleted recipes removed from user's favorites automatically
**AC-2.2.6:** Deleted shared recipes hidden from community immediately
**AC-2.2.7:** Query handlers update projections for edit/delete events
**AC-2.2.8:** Tests verify edit, delete, and cascade deletion of favorites

### Story 2.3: Recipe Favorites System

**AC-2.3.1:** RecipeFavorited and RecipeUnfavorited events store user-recipe relationship
**AC-2.3.2:** Free tier users limited to maximum 10 favorited recipes
**AC-2.3.3:** Attempting to exceed 10 favorites shows upgrade modal (no unfavoriting option)
**AC-2.3.4:** Premium tier users have unlimited favorites
**AC-2.3.5:** Recipe cards show favorite button with toggle state
**AC-2.3.6:** User profile displays favorited recipes list
**AC-2.3.7:** When recipe owner deletes recipe, all favorites automatically removed (no notifications)
**AC-2.3.8:** Query projection tracks favorite_count per recipe for community sorting
**AC-2.3.9:** Tests verify favorite limits, premium bypass, and cascade deletion

### Story 2.4: Recipe JSON Import - File Upload & Validation

**AC-2.4.1:** Recipe import route accepts multiple JSON files (max 10MB per file, 20 files per batch)
**AC-2.4.2:** Drag-and-drop UI with file picker fallback
**AC-2.4.3:** Validation against recipe schema: all required AND optional fields must be present and valid
**AC-2.4.4:** Malicious content detection (script injection, oversized payloads)
**AC-2.4.5:** Streaming parser for large files to prevent memory issues
**AC-2.4.6:** Invalid recipes skipped with detailed error messages collected
**AC-2.4.7:** Duplicate detection blocks recipes with matching name or similar ingredients
**AC-2.4.8:** Imported recipes stored as private (not shared) by default
**AC-2.4.9:** Tests verify validation, malicious content rejection, and duplicate blocking

### Story 2.5: Recipe JSON Import - Real-Time Progress & Summary

**AC-2.5.1:** Real-time progress display: "Imported X recipes, Y failed, Z remaining..."
**AC-2.5.2:** Twinspark polling updates progress without blocking UI
**AC-2.5.3:** Summary report after completion: success count, failed count, duplicate count
**AC-2.5.4:** Detailed error list for failed recipes (missing fields, validation errors)
**AC-2.5.5:** Success message with link to recipe library when complete
**AC-2.5.6:** Progress state cleared on page reload (no persistent history)
**AC-2.5.7:** Tests verify progress updates and summary accuracy

### Story 2.6: JSON Schema Documentation

**AC-2.6.1:** JSON schema document created with all recipe fields and types
**AC-2.6.2:** Schema versioned (v1.0) and published at public URL (/api/schema/recipe/v1.0)
**AC-2.6.3:** Documentation page explains schema fields, required vs optional, and example JSON
**AC-2.6.4:** Schema matches HTML form validation exactly
**AC-2.6.5:** Schema includes all four recipe types with type-specific fields
**AC-2.6.6:** Tests verify schema endpoint accessibility

### Story 2.7: Recipe Snapshot System for Meal Plans

**AC-2.7.1:** Meal plan generation creates complete snapshot/copy of each referenced recipe
**AC-2.7.2:** Snapshots stored in meal_plan_recipes table with week_id foreign key
**AC-2.7.3:** Snapshots include all recipe fields: name, ingredients, instructions, dietary_restrictions, etc.
**AC-2.7.4:** Calendar displays recipe data from snapshot, not original recipe
**AC-2.7.5:** Recipe modifications by owner don't affect existing meal plan snapshots
**AC-2.7.6:** Recipe deletion by owner doesn't affect existing meal plan snapshots
**AC-2.7.7:** Tests verify snapshot creation, isolation from original recipe changes

## Traceability Mapping

| AC ID | Spec Section | Component/API | Test Strategy |
|-------|-------------|---------------|---------------|
| 2.1.1 | Data Models - Recipe Aggregate | RecipeCreated event in imkitchen-recipe/event.rs | Unit test: verify event contains all fields |
| 2.1.2 | Data Models - Recipe Aggregate | accepts_accompaniment field in Recipe struct | Unit test: verify default false, editable for MainCourse |
| 2.1.3 | APIs - Recipe Creation | CreateRecipeInput validation | Unit test: validator constraints, required field errors |
| 2.1.4 | APIs - Recipe Creation | POST /recipes route handler | E2E test: form submission, conditional field visibility |
| 2.1.5 | Data Models - Recipe Projection | recipes table schema | Integration test: verify projection populated after event |
| 2.1.6 | APIs - Recipe List | GET /recipes with type filter | Integration test: query with type=MainCourse, verify results |
| 2.1.7 | - | All recipe creation components | Integration test: create all 4 types, verify projections |
| 2.2.1 | Data Models - Recipe Events | RecipeUpdated event, evento::save | Unit test: update fields, verify event emitted |
| 2.2.2 | Data Models - Recipe Events | RecipeDeleted event with deleted_at | Unit test: verify soft delete timestamp set |
| 2.2.3 | APIs - Recipe Editing | GET /recipes/{id}/edit | Integration test: form pre-populated with current data |
| 2.2.4 | Workflows - Recipe Deletion | Confirmation modal template | E2E test: modal appears, cancel/confirm behavior |
| 2.2.5 | Workflows - Recipe Deletion | ON DELETE CASCADE FK | Integration test: delete recipe, verify favorites removed |
| 2.2.6 | Data Models - Recipe Projection | deleted_at column, WHERE filter | Integration test: delete recipe, verify hidden from community |
| 2.2.7 | Services - Query Handlers | on_recipe_updated, on_recipe_deleted | Integration test: emit events, verify projections updated |
| 2.2.8 | - | All editing/deletion components | Integration test: edit/delete/cascade tests combined |
| 2.3.1 | Data Models - Recipe Events | RecipeFavorited, RecipeUnfavorited | Unit test: verify events with user_id/recipe_id |
| 2.3.2 | APIs - Recipe Favorite | favorite_recipe command with count check | Integration test: free tier, favorite 10, verify 11th blocked |
| 2.3.3 | APIs - Recipe Favorite | Upgrade modal template | E2E test: attempt 11th favorite, verify modal shown |
| 2.3.4 | APIs - Recipe Favorite | Access control service bypass | Integration test: premium tier, favorite >10, verify allowed |
| 2.3.5 | APIs - Recipe Favorite | Favorite button partial template | E2E test: click toggle, verify button state changes |
| 2.3.6 | APIs - Recipe List | User profile favorite list query | Integration test: query favorited recipes, verify list |
| 2.3.7 | Workflows - Recipe Deletion | ON DELETE CASCADE FK | Integration test: favorite recipe, delete it, verify cascade |
| 2.3.8 | Data Models - Recipe Projection | favorite_count field in recipes | Integration test: favorite/unfavorite, verify count updates |
| 2.3.9 | - | All favorites components | Integration test: limits/bypass/cascade combined |
| 2.4.1 | APIs - Recipe Import | POST /recipes/import file validation | Integration test: upload 20 files × 10MB, verify accepted |
| 2.4.2 | APIs - Recipe Import | Import form with drag-drop | E2E test: drag files, verify upload triggered |
| 2.4.3 | APIs - Recipe Import | JSON Schema validation | Unit test: validate against recipe-v1.0.json schema |
| 2.4.4 | Security - File Upload | Malicious content scanner | Unit test: inject <script>, verify rejected |
| 2.4.5 | APIs - Recipe Import | Streaming JSON parser (tokio) | Performance test: 10MB file, verify constant memory |
| 2.4.6 | Workflows - Recipe Import | Error collection per-recipe | Integration test: batch with invalid recipes, verify errors listed |
| 2.4.7 | Workflows - Recipe Import | Duplicate detection function | Unit test: name match, fuzzy ingredient similarity |
| 2.4.8 | Data Models - Recipe Projection | is_shared=false default | Integration test: import recipe, verify is_shared field |
| 2.4.9 | - | All import validation components | Integration test: validation/malicious/duplicate combined |
| 2.5.1 | Workflows - Import Progress | ImportProgress struct, in-memory storage | Integration test: import in progress, verify counts updated |
| 2.5.2 | APIs - Import Progress | GET /recipes/import/progress/{id} polling | E2E test: Twinspark polling, verify updates every 1s |
| 2.5.3 | APIs - Import Progress | Import summary template | Integration test: import complete, verify summary displayed |
| 2.5.4 | Workflows - Import Progress | Error list in ImportProgress | Integration test: failed recipes, verify error details shown |
| 2.5.5 | APIs - Import Progress | Success template with library link | E2E test: complete import, verify link to /recipes |
| 2.5.6 | Services - Import Storage | In-memory HashMap (no persistence) | Integration test: page reload, verify progress cleared |
| 2.5.7 | - | All progress tracking components | Integration test: progress updates/summary combined |
| 2.6.1 | Data Models - JSON Schema | recipe-v1.0.json schema file | Unit test: validate schema against JSON Schema Draft 2020-12 |
| 2.6.2 | APIs - JSON Schema | GET /api/schema/recipe/v1.0 | Integration test: endpoint returns 200, valid JSON |
| 2.6.3 | APIs - JSON Schema | GET /docs/schema documentation page | E2E test: page displays fields, examples visible |
| 2.6.4 | Data Models - JSON Schema | Schema constraints match validator | Unit test: compare schema vs CreateRecipeInput validation |
| 2.6.5 | Data Models - JSON Schema | RecipeType enum, conditional fields | Unit test: verify 4 types, accepts_accompaniment conditional |
| 2.6.6 | - | Schema endpoint components | Integration test: public access, no auth required |
| 2.7.1 | Workflows - Snapshot Creation | create_recipe_snapshot function | Integration test: generate meal plan, verify snapshots created |
| 2.7.2 | Data Models - Snapshot Projection | meal_plan_recipe_snapshots table | Integration test: verify FK to meal_plans, day_index/meal_slot |
| 2.7.3 | Data Models - Snapshot Projection | Snapshot includes all recipe fields | Integration test: compare snapshot vs original recipe fields |
| 2.7.4 | Services - Query Handlers | Calendar queries use snapshots | Integration test: query calendar, verify data from snapshots |
| 2.7.5 | Workflows - Snapshot Isolation | Snapshot unaffected by recipe updates | Integration test: update recipe, verify snapshot unchanged |
| 2.7.6 | Workflows - Snapshot Isolation | Snapshot preserved after recipe deletion | Integration test: delete recipe, verify snapshot intact |
| 2.7.7 | - | All snapshot components | Integration test: isolation from updates/deletes combined |

## Risks, Assumptions, Open Questions

### Risks

**Risk-1: Streaming Parser Memory Overhead**
- **Description:** Streaming JSON parser may still spike memory with many concurrent imports (20 files × 10MB)
- **Impact:** High - Server OOM crash during peak import usage
- **Mitigation:** Implement concurrent import limit (max 5 simultaneous files per user), use tokio semaphore
- **Owner:** Import route implementation (Story 2.4)

**Risk-2: Duplicate Detection Performance**
- **Description:** Fuzzy ingredient matching (80% similarity) may be slow with large recipe database (>10,000 recipes)
- **Impact:** Medium - Import processing time exceeds 30s timeout
- **Mitigation:** Implement incremental duplicate checking (check against user's own recipes first, then community), cache recent duplicate checks
- **Owner:** Import command implementation (Story 2.4)

**Risk-3: In-Memory Progress Tracking Loss**
- **Description:** Server restart during import loses progress tracking, user sees stale "Importing..." UI
- **Impact:** Low - User can refresh page and restart import, no data loss
- **Mitigation:** Add timestamp to ImportProgress, expire entries after 10 minutes, show "Import expired" message
- **Owner:** Import progress implementation (Story 2.5)

**Risk-4: JSON Schema Versioning Complexity**
- **Description:** Future schema changes (v1.1, v2.0) require backward compatibility handling
- **Impact:** Medium - Breaking changes may invalidate existing third-party tools
- **Mitigation:** Document versioning policy (semantic versioning), maintain all versions indefinitely, add deprecation notices
- **Owner:** Schema documentation implementation (Story 2.6)

### Assumptions

**Assumption-1:** Users will primarily import recipes from personal backups, not large community exports (batch size assumption)
**Assumption-2:** Fuzzy ingredient matching with 80% threshold provides acceptable duplicate detection accuracy
**Assumption-3:** In-memory progress tracking acceptable for MVP; persistent tracking deferred to post-launch
**Assumption-4:** Snapshot storage overhead acceptable (estimates: 50 snapshots per user, 2KB each = 100KB per user)
**Assumption-5:** Malicious content scan (script tag detection) sufficient for MVP; advanced payload analysis deferred
**Assumption-6:** Soft delete adequate for Epic 2; hard delete (GDPR compliance) deferred to Epic 5

### Open Questions

**Question-1:** Should duplicate detection be configurable per-user (strict vs lenient)?
- **Answer Needed By:** Story 2.4 implementation
- **Proposed Resolution:** Start with fixed 80% threshold, add user preference in Epic 5 based on feedback

**Question-2:** Should import progress be visible to other admins (multi-user admin panel)?
- **Answer Needed By:** Story 2.5 implementation
- **Proposed Resolution:** Epic 2 scope: single-user progress only; multi-admin view deferred to Epic 5

**Question-3:** Should snapshots deduplicate identical recipes across meal plans (storage optimization)?
- **Answer Needed By:** Story 2.7 implementation
- **Proposed Resolution:** Epic 2 scope: no deduplication (simple INSERT for each snapshot); optimize in Epic 3 if storage becomes issue

**Question-4:** Should recipe edit history be tracked (audit trail)?
- **Answer Needed By:** Architecture decision (impacts evento event structure)
- **Proposed Resolution:** Epic 2 scope: no edit history (only current state in projection); add versioning in post-MVP if requested

## Test Strategy Summary

### Test Levels

**Unit Tests (Rust `#[test]`):**
- Recipe aggregate handlers (recipe_created, recipe_updated, recipe_deleted)
- Command validation logic (CreateRecipeInput, UpdateRecipeInput)
- Event serialization/deserialization (bincode)
- Duplicate detection function (name match, fuzzy ingredient similarity)
- Malicious content scanner (script injection patterns)
- JSON Schema validation against recipe-v1.0.json

**Integration Tests (`tests/` folder):**
- Recipe creation flow: command → event → projection → query
- Recipe editing with ownership validation
- Recipe deletion with cascade to favorites
- Favorite system with freemium limits (10 max for free tier)
- Recipe import with streaming parser (valid/invalid/malicious files)
- Import progress tracking (in-memory storage updates)
- Snapshot creation and isolation from recipe changes/deletions
- All tests use `sqlx::migrate!` and `evento::sql_migrator` for database setup (DRY principle)

**E2E Tests (Playwright):**
- Recipe creation form with conditional fields (Main Course → accepts_accompaniment)
- Recipe deletion confirmation modal (cancel/confirm behavior)
- Favorite button toggle with Twinspark (button state changes)
- Upgrade modal triggered at 11th favorite (free tier)
- Recipe import with drag-and-drop UI
- Import progress polling (Twinspark every 1s until complete)
- Schema documentation page (fields, examples visible)

### Coverage of Acceptance Criteria

- **Story 2.1 (7 ACs):** 7 tests (100% coverage) - unit + integration + E2E
- **Story 2.2 (8 ACs):** 8 tests (100% coverage) - unit + integration + E2E
- **Story 2.3 (9 ACs):** 9 tests (100% coverage) - unit + integration + E2E
- **Story 2.4 (9 ACs):** 9 tests (100% coverage) - unit + integration
- **Story 2.5 (7 ACs):** 7 tests (100% coverage) - integration + E2E
- **Story 2.6 (6 ACs):** 6 tests (100% coverage) - unit + integration + E2E
- **Story 2.7 (7 ACs):** 7 tests (100% coverage) - integration

**Total:** 53 acceptance criteria → 53 tests (1:1 mapping minimum)

### Test Frameworks

- **Rust Unit/Integration:** Built-in `#[test]`, `#[tokio::test]` for async
- **E2E:** Playwright 1.56+ (TypeScript)
- **Performance:** `cargo bench` for streaming parser, duplicate detection
- **Database:** In-memory SQLite (`:memory:`) for tests, cleanup via DROP TABLE

### Edge Cases and Negative Tests

1. **Recipe creation with empty ingredients** → validation error
2. **Recipe edit by non-owner** → ownership error (401 Unauthorized)
3. **Recipe deletion already deleted** → idempotent (no error)
4. **Favorite recipe already favorited** → idempotent (no error)
5. **Import with 0 valid recipes** → summary shows 0 success, detailed errors
6. **Import with 21 files** → rejected at upload (max 20 files)
7. **Import with 11MB file** → rejected at upload (max 10MB per file)
8. **Import with malformed JSON** → parsing error, listed in summary
9. **Import with script injection** → blocked, logged as security event
10. **Progress polling for expired import_id** → "Import expired" message
11. **Snapshot creation for deleted recipe** → error handled gracefully
12. **Cascade delete when recipe has 100+ favorites** → all removed (ON DELETE CASCADE performance test)

### Performance Benchmarks

- Import processing: 100 recipes in <30 seconds
- Duplicate detection: <100ms per recipe with 10,000 existing recipes
- Favorite count query: <50ms with 1,000 favorited recipes
- Snapshot creation: <200ms per snapshot (21 snapshots in <5s)
- Streaming parser: 10MB file with constant memory (<50MB resident)
