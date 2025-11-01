# Decision Architecture

## Executive Summary

imkitchen uses an event-driven CQRS architecture built on Rust/Axum with evento for event sourcing and SQLite for storage. The system is organized into three bounded contexts (user, recipe, mealplan) following DDD principles, with server-side rendering via Askama templates and Twinspark for UI reactivity. The architecture prioritizes performance (<5s meal plan generation, <3s page loads), security (JWT auth, OWASP standards), and portability (no vendor lock-in, configurable SMTP). A pure Rust in-memory algorithm handles meal plan generation with dietary filtering and accompaniment pairing, while a centralized Access Control Service enforces freemium tier restrictions throughout the application.

## Project Initialization

**Manual Setup Required** - No starter template used. Follow project structure defined below.

First implementation story (Story 1.1) should:
1. Create workspace Cargo.toml with all dependencies
2. Set up CLI commands (serve, migrate, reset)
3. Create config/ directory with default.toml
4. Initialize git repository with proper .gitignore
5. Set up migration directories (migrations/queries/, migrations/validation/)

## Decision Summary

| Category | Decision | Version | Affects Epics | Rationale |
| -------- | -------- | ------- | ------------- | --------- |
| Language | Rust | 1.90+ | All | Performance, safety, CLAUDE.md standard |
| Web Framework | Axum | 0.8.6 | All | Modern async, excellent performance, Tower integration |
| Templating | Askama | 0.14.0 | 4, 5, 6 | Type-safe templates, SSR for SEO |
| UI Reactivity | Twinspark | Latest | 2, 4, 5 | Server-driven, minimal JS, CLAUDE.md standard |
| Event Sourcing | evento | 1.5.0 | 1, 2, 3 | Event-driven architecture, CQRS, CLAUDE.md standard |
| Database | SQLite | (via sqlx 0.8.2) | All | Simple, portable, separate write/read/validation DBs |
| Styling | Tailwind CSS | 4.1.0 | All | Utility-first, no config file needed in 4.1+ |
| Validation | validator | 0.20.0 | 1, 2 | Input validation for commands |
| Request IDs | ulid | 1.2.0 | All | Unique IDs for request tracking in metadata |
| CLI | clap | 4.5.23 | 1 | Command-line interface (serve, migrate, reset) |
| Configuration | config | 0.15.0 | 1 | TOML-based configuration system |
| Observability | opentelemetry | 0.31.0 | All | Structured logging, tracing |
| Email | lettre | 0.11.14 | 6 | Configurable SMTP for admin notifications |
| Testing | Playwright | 1.56.0 | All | E2E testing for critical user flows |
| Meal Plan Algorithm | Pure Rust in-memory | N/A | 3 | Predictable <5s performance, full control |
| Recipe Snapshots | Separate table with FKs | N/A | 3, 4 | Event size optimization, query performance |
| Access Control | Centralized service | N/A | 4, 5 | Consistent freemium enforcement |
| File Upload | Streaming parser (tokio) | N/A | 2 | Memory efficiency, 10MB file support |
| Notifications | Hybrid (in-app + email) | N/A | 6 | MVP-friendly, Web Push deferred |
| SEO/PWA | SSR + service worker | N/A | 6 | SEO optimization, offline capability |

## Project Structure

```
imkitchen/
├── Cargo.toml                          # Workspace definition
├── config/
│   ├── default.toml                    # Committed config
│   └── dev.toml                        # .gitignore (SMTP passwords, etc.)
├── src/
│   ├── main.rs                         # CLI (serve, migrate, reset)
│   ├── lib.rs                          # Shared app types
│   ├── server.rs                       # Axum server setup
│   ├── access_control.rs               # Freemium enforcement service
│   ├── email.rs                        # Email service (lettre)
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── auth/
│   │   │   ├── mod.rs
│   │   │   ├── login.rs
│   │   │   ├── register.rs
│   │   │   └── profile.rs
│   │   ├── recipes/
│   │   │   ├── mod.rs
│   │   │   ├── create.rs
│   │   │   ├── import.rs              # Streaming JSON import
│   │   │   ├── favorite.rs
│   │   │   ├── community.rs
│   │   │   └── rate.rs
│   │   ├── mealplan/
│   │   │   ├── mod.rs
│   │   │   ├── generate.rs
│   │   │   └── calendar.rs
│   │   ├── shopping.rs                # Shopping list generation
│   │   ├── dashboard.rs
│   │   ├── landing.rs                 # SEO landing page
│   │   ├── contact.rs
│   │   └── admin/
│   │       ├── mod.rs
│   │       ├── users.rs
│   │       └── contact_inbox.rs
│   └── queries/
│       ├── mod.rs
│       ├── users.rs                   # User projections & queries
│       ├── recipes.rs                 # Recipe projections & queries
│       ├── mealplans.rs               # MealPlan projections & queries
│       └── snapshots.rs               # Recipe snapshot queries
├── crates/
│   ├── imkitchen-user/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── command.rs             # User commands
│   │   │   ├── event.rs               # User events
│   │   │   └── aggregate.rs           # User aggregate root
│   ├── imkitchen-recipe/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── command.rs             # Recipe commands
│   │   │   ├── event.rs               # Recipe events
│   │   │   └── aggregate.rs           # Recipe aggregate root
│   └── imkitchen-mealplan/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── command.rs             # MealPlan commands
│       │   ├── event.rs               # MealPlan events
│       │   ├── aggregate.rs           # MealPlan aggregate root
│       │   └── generator.rs           # Pure Rust generation algorithm
├── migrations/
│   ├── queries/                       # Read database migrations
│   │   ├── 20250101000000_users.sql
│   │   ├── 20250101000001_user_profiles.sql
│   │   ├── 20250101000002_recipes.sql
│   │   ├── 20250101000003_recipe_favorites.sql
│   │   ├── 20250101000004_recipe_ratings.sql
│   │   ├── 20250101000005_meal_plans.sql
│   │   ├── 20250101000006_meal_plan_recipe_snapshots.sql
│   │   ├── 20250101000007_shopping_lists.sql
│   │   └── 20250101000008_contact_messages.sql
│   └── validation/                    # Validation database migrations
│       └── 20250101000000_user_emails.sql
├── templates/
│   ├── base.html                      # Base template with SEO meta tags
│   ├── pages/
│   │   ├── landing.html               # SEO-optimized landing page
│   │   ├── dashboard.html
│   │   ├── auth/
│   │   │   ├── login.html
│   │   │   ├── register.html
│   │   │   └── profile.html
│   │   ├── recipes/
│   │   │   ├── create.html
│   │   │   ├── import.html
│   │   │   ├── list.html
│   │   │   ├── detail.html
│   │   │   └── community.html
│   │   ├── mealplan/
│   │   │   └── calendar.html
│   │   ├── shopping.html
│   │   └── admin/
│   │       ├── users.html
│   │       └── contact_inbox.html
│   ├── partials/                      # Twinspark partial responses
│   │   ├── recipes/
│   │   │   ├── import-progress.html
│   │   │   └── import-summary.html
│   │   └── mealplan/
│   │       └── generation-pending.html
│   └── components/                    # Reusable components
│       ├── recipe-card.html
│       └── meal-card.html
├── static/
│   ├── css/
│   │   ├── input.css                  # Tailwind input
│   │   └── output.css                 # Compiled CSS
│   ├── js/
│   │   ├── twinspark.js
│   │   └── service-worker.js          # PWA offline support
│   ├── manifest.json                  # PWA manifest
│   └── icons/
└── tests/
    ├── auth_test.rs
    ├── recipes_test.rs
    ├── mealplan_test.rs
    ├── import_test.rs
    └── e2e/                           # Playwright tests
        └── user_flows.spec.ts
```

## Epic to Architecture Mapping

| Epic | Bounded Context | Key Components | Database Tables |
|------|----------------|----------------|-----------------|
| Epic 1: Foundation & User Management | `imkitchen-user` | User aggregate, auth commands, admin panel routes | users, user_profiles, user_emails (validation) |
| Epic 2: Recipe Management & Import | `imkitchen-recipe` | Recipe aggregate, import handler, streaming parser | recipes, recipe_favorites, recipe_ratings |
| Epic 3: Meal Planning Engine | `imkitchen-mealplan` | MealPlan aggregate, generation algorithm | meal_plans, meal_plan_recipe_snapshots |
| Epic 4: Calendar & Shopping | Main binary (routes/queries) | Calendar routes, shopping list generator | meal_plans (read), shopping_lists |
| Epic 5: Community & Freemium | `imkitchen-recipe` + `src/access_control.rs` | Recipe sharing, ratings, access control service | recipes (is_shared), recipe_ratings, access control logic |
| Epic 6: Notifications & Landing | Main binary | Email service, landing page routes, reminder job | contact_messages, pending_reminders |

## Technology Stack Details

### Core Technologies

**Runtime & Language:**
- Rust 1.90+ with 2021 edition
- Tokio 1.42+ async runtime
- Standard library for core algorithms

**Web Server:**
- Axum 0.8.6 web framework
- Tower 0.5+ for middleware
- Hyper 1.5+ HTTP server
- axum-extra 0.12+ for Form/Query extractors

**Event Sourcing & CQRS:**
- evento 1.5.0 with SQLite feature
- Separate databases: write (evento), read (queries), validation
- Event-driven command/query separation

**Data Layer:**
- SQLite 3.x via sqlx 0.8.2
- Runtime query checking (no compile-time macros)
- Migration support via sqlx::migrate!

**Templating & UI:**
- Askama 0.14.0 + askama_web 0.14.0
- Twinspark (latest) for reactivity
- Tailwind CSS 4.1.0 for styling
- Server-side rendering (SSR)

**Authentication & Security:**
- JWT cookie-based authentication
- jsonwebtoken 9.3+ for token generation/validation
- argon2 0.6+ for password hashing
- HTTP-only cookies for token storage

**Validation & Serialization:**
- validator 0.20.0 for input validation
- serde 1.0+ with derive macros
- serde_json 1.0+ for JSON handling

**Utilities:**
- ulid 1.2.0 for request IDs
- chrono 0.4+ for date/time handling
- config 0.15.0 for TOML configuration
- clap 4.5.23 for CLI parsing

**Observability:**
- tracing 0.1+ for structured logging
- tracing-subscriber 0.3+ for log output
- opentelemetry 0.31.0 for telemetry

**Email:**
- lettre 0.11.14 for SMTP
- Configurable via TOML (smtp_host, smtp_port, credentials)

**Testing:**
- Rust built-in test framework
- Playwright 1.56.0 (TypeScript) for E2E tests

### Integration Points

**Write Path (Commands):**
```
User Request → Axum Route Handler → Command (in bounded context crate)
  → evento::create/save → Write DB (evento.db)
  → Event emitted → Command/Query Handlers subscribed
```

**Read Path (Queries):**
```
User Request → Axum Route Handler → Query Function
  → Read DB (queries.db) → Askama Template → HTML Response
```

**Validation Path:**
```
Command with async validation → Command Handler checks validation DB
  → Emit success/failure event → Query handler updates projection
```

**Database Connections:**
- Write DB: `evento.db` - evento manages this exclusively
- Read DB: `queries.db` - query handlers write, route handlers read
- Validation DB: `validation.db` - command handlers read/write for uniqueness checks

**Event Flow:**
1. Commands emit events to evento
2. evento stores events in write DB
3. Subscriptions (command handlers, query handlers) process events
4. Query handlers update projections in read DB
5. Route handlers query read DB for user responses

## Implementation Patterns

These patterns ensure consistent implementation across all AI agents:

### Command Pattern (per CLAUDE.md)

**Structure:**
```rust
// crates/imkitchen-user/src/command.rs
pub struct Command<E: Executor> {
    evento: E,
    validation_pool: SqlitePool,  // For async validation
}

impl<E: Executor> Command<E> {
    // MUST use input struct as first argument, metadata as second
    pub async fn register_user(
        &self,
        input: RegisterUserInput,
        metadata: EventMetadata,
    ) -> anyhow::Result<String> {
        // Validation in command (sync only)
        input.validate()?;

        // Emit event
        let user_id = evento::create::<User>()
            .data(&UserRegistered { ... })?
            .metadata(&metadata)?
            .commit(&self.evento)
            .await?;

        Ok(user_id)
    }
}
```

**Rules:**
- Commands ONLY use evento or validation tables for data
- NEVER use projections in commands (they're eventually consistent)
- ALL async validation deferred to command handlers
- Commands complete in <10 seconds (due to graceful shutdown)

### Query Pattern (per CLAUDE.md)

**Structure:**
```rust
// src/queries/users.rs
pub async fn get_user_profile(
    pool: &SqlitePool,
    user_id: &str,
) -> anyhow::Result<Option<UserProfile>> {
    sqlx::query_as::<_, UserProfile>(
        "SELECT id, email, dietary_restrictions FROM user_profiles WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
```

**Rules:**
- Queries ONLY access read DB (queries.db)
- NEVER try to access evento data directly
- Query handlers MUST be idempotent (can replay events)
- Use `event.timestamp` for all time fields in projections

### Event Handler Pattern (per CLAUDE.md)

**Structure:**
```rust
// src/queries/users.rs
#[evento::handler(User)]
async fn on_user_registered<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserRegistered, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    sqlx::query("INSERT INTO users (id, email, created_at) VALUES (?, ?, ?)")
        .bind(&event.aggregator_id)
        .bind(&event.data.email)
        .bind(event.timestamp)
        .execute(&pool)
        .await?;

    Ok(())
}

pub fn subscribe_user_query(pool: SqlitePool) -> evento::SubscriptionBuilder {
    evento::subscribe("user-query")
        .data(pool)
        .handler(on_user_registered())
        .skip::<User, UserDeleted>()  // Skip events we don't handle
}
```

**Rules:**
- One subscription per query view (don't combine unrelated projections)
- Handlers complete in <10 seconds
- Use `.skip::<Aggregate, Event>()` for events you don't need
- Make subscription builders reusable (same function for main.rs and tests)

## Consistency Rules

### Naming Conventions

**Files & Modules:**
- File names: `snake_case.rs` (e.g., `meal_plan.rs`, `user_profile.rs`)
- Module names: `snake_case` matching file names
- Test files: `{feature}_test.rs` in `tests/` folder (e.g., `auth_test.rs`)

**Routes (Axum 0.8+):**
- Route parameters: `{id}` format (NOT `:id`)
- Examples: `/users/{id}`, `/recipes/{recipe_id}/rate`

**Database:**
- Table names: `snake_case` plural (e.g., `users`, `meal_plans`, `recipe_favorites`)
- Column names: `snake_case` (e.g., `user_id`, `created_at`, `is_admin`)
- Foreign keys: `{table}_id` (e.g., `user_id`, `recipe_id`)
- Timestamps: `INTEGER` storing Unix timestamps from `event.timestamp`

**Rust Code:**
- Structs: `PascalCase` (e.g., `UserProfile`, `MealPlanGenerator`)
- Enums: `PascalCase` with `PascalCase` variants (e.g., `RecipeType::MainCourse`)
- Functions: `snake_case` (e.g., `generate_meal_plan`, `get_user_profile`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_FAVORITES_FREE_TIER`)

### Code Organization

**Bounded Context Crates:**
- One aggregate root per crate
- Files: `command.rs`, `event.rs`, `aggregate.rs`, `lib.rs`
- ALL commands in `command.rs` (not split across files)
- ALL events in `event.rs` (not split across files)

**Route Handlers:**
- Group by feature area: `routes/auth/`, `routes/recipes/`, `routes/mealplan/`
- One file per route: `routes/recipes/create.rs`, `routes/recipes/import.rs`
- Co-locate related routes in same module

**Query Functions:**
- Group by aggregate: `queries/users.rs`, `queries/recipes.rs`, `queries/mealplans.rs`
- Include both query functions AND query handlers in same file
- Subscription builders in same file as handlers

**Tests:**
- Integration tests in `tests/` folder (NOT `src/`)
- Test file naming: `{feature}_test.rs` (e.g., `auth_test.rs`, `import_test.rs`)
- Use `sqlx::migrate!` and `evento::sql_migrator` for database setup
- NEVER use direct SQL for test setup (always use migrations)

### Error Handling

**Strategy:** Use `anyhow::Result` for all fallible operations

**Command Errors:**
```rust
// Return errors from commands - don't emit error events
pub async fn create_recipe(&self, input: CreateRecipeInput) -> anyhow::Result<String> {
    input.validate()?;  // Validation errors bubble up

    let recipe_id = evento::create::<Recipe>()
        .data(&RecipeCreated { ... })?
        .commit(&self.evento)
        .await?;  // Database errors bubble up

    Ok(recipe_id)
}
```

**Route Handler Errors:**
```rust
// Display user-friendly error messages in templates
pub async fn create_recipe_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateRecipeForm>,
) -> impl IntoResponse {
    match state.command.create_recipe(input, metadata).await {
        Ok(recipe_id) => RecipeCreatedTemplate { recipe_id }.into_response(),
        Err(e) => RecipeFormTemplate {
            error: Some(format!("Failed to create recipe: {}", e))
        }.into_response(),
    }
}
```

**Query Handler Errors:**
```rust
// Log errors but don't fail subscription
#[evento::handler(Recipe)]
async fn on_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated, EventMetadata>,
) -> anyhow::Result<()> {
    // If this fails, evento will retry
    let pool = context.extract::<SqlitePool>();
    sqlx::query("INSERT INTO recipes (...) VALUES (...)")
        .execute(&pool)
        .await?;

    Ok(())
}
```

### Logging Strategy

**Use `tracing` crate extensively:**

```rust
use tracing::{info, warn, error, debug};

// Log command execution
#[tracing::instrument(skip(self))]
pub async fn register_user(&self, input: RegisterUserInput) -> anyhow::Result<String> {
    info!("Registering new user: {}", input.email);
    // ... command logic
    info!("User registered successfully: {}", user_id);
    Ok(user_id)
}

// Log validation failures
if exists {
    warn!("Registration failed: email already exists: {}", input.email);
    return Err(anyhow!("Email already registered"));
}

// Log event handling
#[tracing::instrument(skip(context, event))]
async fn on_user_registered<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserRegistered, EventMetadata>,
) -> anyhow::Result<()> {
    debug!("Processing UserRegistered event: {}", event.aggregator_id);
    // ... projection logic
    Ok(())
}
```

**Log Levels:**
- `error!` - Failures, panics, critical issues
- `warn!` - Potentially problematic situations
- `info!` - Important business logic flow, state changes
- `debug!` - Detailed diagnostic information
- `trace!` - Very detailed (loop iterations, data transformations)

## Data Architecture

### Database Separation

**Write DB (`evento.db`):**
- Managed exclusively by evento
- Stores all events and aggregate state
- Never query directly - use evento API

**Read DB (`queries.db`):**
- Projections for all queries
- Updated by query handlers
- Queried by route handlers

**Validation DB (`validation.db`):**
- Unique constraint validation (e.g., email uniqueness)
- Used by command handlers for async validation
- Example: `user_emails` table with unique index

### Core Tables (Read DB)

**users**
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    hashed_password TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    is_suspended BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
);
```

**user_profiles**
```sql
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY,
    dietary_restrictions TEXT,  -- JSON array
    cuisine_variety_weight REAL NOT NULL DEFAULT 0.7,
    household_size INTEGER,
    is_premium_active BOOLEAN NOT NULL DEFAULT 0,
    premium_bypass BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**recipes**
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
```

**recipe_favorites**
```sql
CREATE TABLE recipe_favorites (
    user_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    favorited_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, recipe_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);
```

**recipe_ratings**
```sql
CREATE TABLE recipe_ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review_text TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**meal_plans**
```sql
CREATE TABLE meal_plans (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,  -- ISO date (e.g., "2025-11-03")
    week_number INTEGER NOT NULL,  -- 1-5
    is_current_week BOOLEAN NOT NULL DEFAULT 0,
    generated_at INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**meal_plan_recipe_snapshots**
```sql
CREATE TABLE meal_plan_recipe_snapshots (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    day_index INTEGER NOT NULL CHECK (day_index >= 0 AND day_index <= 6),
    meal_slot TEXT NOT NULL,  -- 'appetizer' | 'main' | 'dessert' | 'accompaniment'
    original_recipe_id TEXT,  -- Reference (may be deleted)
    recipe_type TEXT NOT NULL,
    name TEXT NOT NULL,
    ingredients TEXT NOT NULL,
    instructions TEXT NOT NULL,
    dietary_restrictions TEXT,
    cuisine_type TEXT,
    snapshot_at INTEGER NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
);
```

**contact_messages**
```sql
CREATE TABLE contact_messages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',  -- 'new' | 'read' | 'resolved'
    created_at INTEGER NOT NULL
);
```

## API Contracts

### HTTP Routes

**Authentication:**
- `GET /auth/register` - Registration form
- `POST /auth/register` - Create account
- `GET /auth/login` - Login form
- `POST /auth/login` - Authenticate user
- `GET /auth/profile` - View/edit profile
- `POST /auth/profile` - Update profile

**Recipes:**
- `GET /recipes` - List user's recipes
- `GET /recipes/new` - Recipe creation form
- `POST /recipes` - Create recipe
- `GET /recipes/{id}` - Recipe details
- `POST /recipes/{id}/edit` - Update recipe
- `POST /recipes/{id}/delete` - Delete recipe
- `POST /recipes/{id}/favorite` - Toggle favorite
- `POST /recipes/{id}/share` - Toggle sharing
- `GET /recipes/import` - Import form
- `POST /recipes/import` - Upload JSON files (returns import_id)
- `GET /recipes/import/progress/{import_id}` - Real-time progress (Twinspark polling) - Returns partial HTML with imported/failed/remaining counts
- `GET /recipes/community` - Browse shared recipes
- `POST /recipes/{id}/rate` - Rate/review recipe

**Meal Plans:**
- `GET /mealplan` - Calendar view
- `POST /mealplan/generate` - Generate meal plan
- `POST /mealplan/regenerate` - Regenerate future weeks
- `GET /mealplan/week/{week_number}` - View specific week

**Shopping:**
- `GET /shopping/{week_number}` - Shopping list for week

**Dashboard:**
- `GET /` - Landing page (unauthenticated) or Dashboard (authenticated)
- `GET /dashboard` - Dashboard with nearest day

**Admin:**
- `GET /admin/users` - User management
- `POST /admin/users/{id}/suspend` - Suspend user
- `POST /admin/users/{id}/activate` - Activate user
- `POST /admin/users/{id}/premium-bypass` - Toggle bypass
- `GET /admin/contact` - Contact inbox
- `POST /admin/contact/{id}/mark-read` - Mark message read

**Contact:**
- `GET /contact` - Contact form
- `POST /contact` - Submit message

### Response Format

**HTML Responses (SSR):**
- All routes return rendered HTML via Askama templates
- Error states return same template with error message
- Success states may redirect or return updated template

**Twinspark Partial Responses:**
- Progress updates return partial HTML (e.g., `import-progress.html`)
- Polling endpoints return updated fragments

**No JSON API** - All data exchange via HTML forms and server-side rendering

## Security Architecture

### Authentication

**JWT Cookie-Based:**
- Secure HTTP-only cookies
- Token contains: `user_id`, `is_admin`, `exp` (expiration)
- Token lifetime: 7 days
- Refresh on every request (sliding window)

**Password Hashing:**
- Argon2id algorithm (OWASP recommended)
- Salt automatically generated per password
- Cost parameters: time_cost=2, mem_cost=19456, parallelism=1

**Route Protection:**
- Middleware extracts JWT from cookie
- Validates signature and expiration
- Populates `Extension<User>` for protected routes
- Redirects to `/auth/login` if not authenticated

### OWASP Compliance (NFR007)

**Input Validation:**
- All user input validated with `validator` crate
- Email format validation
- Password complexity: min 8 chars, uppercase, lowercase, number
- SQL injection prevented via sqlx parameterized queries

**XSS Prevention:**
- Askama escapes all template variables by default
- NEVER use `safe` filter unless absolutely necessary
- CSP headers restrict inline scripts

**CSRF Protection:**
- SameSite=Strict cookie attribute
- Double-submit cookie pattern for state-changing operations

**File Upload Security:**
- Max file size: 10MB enforced at upload
- Content-type validation: application/json only
- Malicious content detection: scan for script tags, eval patterns
- Streaming parser with timeout (30s per file)

### GDPR Compliance (NFR008)

**Data Encryption:**
- Passwords hashed with Argon2id (never stored plaintext)
- Database files encrypted at rest (deployment-specific)
- TLS 1.3 for data in transit (deployment-specific)

**User Rights:**
- Account deletion: soft delete with data preservation
- Data export: not in MVP (deferred to post-MVP)
- Consent: registration implies consent for essential features

## Performance Considerations

### Meal Plan Generation (<5s P95)

**Algorithm Optimization:**
- Load user's favorited recipes into memory (typically <100 recipes)
- Filter by dietary restrictions in single pass
- Use Vec for storage, not HashMap (better cache locality)
- Pre-compute cuisine frequency map
- Weighted random selection with pre-allocated RNG

**Performance Testing:**
- Benchmark with 10, 50, 100, 200 favorited recipes
- Target: P95 < 5s for 5-week generation with 100 recipes
- Profile with `cargo flamegraph` to identify bottlenecks

### Page Load Times (<3s Mobile)

**Optimizations:**
- Minimal CSS (Tailwind 4.1 produces small bundle)
- Twinspark library is tiny (<10KB)
- No heavy JavaScript frameworks
- Lazy load recipe images
- SQLite queries indexed properly

**PWA Offline Caching:**
- Service worker caches CSS, JS, fonts
- Recipe pages cached for offline viewing
- Network-first strategy for dynamic content

### Database Performance

**Indexing Strategy:**
```sql
CREATE INDEX idx_recipes_owner ON recipes(owner_id);
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = 1;
CREATE INDEX idx_recipe_favorites_user ON recipe_favorites(user_id);
CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_number);
CREATE INDEX idx_snapshots_meal_plan ON meal_plan_recipe_snapshots(meal_plan_id);
```

**Query Optimization:**
- Use `EXPLAIN QUERY PLAN` to verify index usage
- Avoid N+1 queries - fetch related data in single query
- Use joins instead of multiple round-trips

## Deployment Architecture

### Deployment Model

**Single Binary Deployment:**
- Compile release binary: `cargo build --release`
- Binary includes: web server, CLI, migrations
- Self-contained, no external runtime dependencies

**Database Files:**
- `evento.db` - Event store (write DB)
- `queries.db` - Projections (read DB)
- `validation.db` - Validation constraints
- All SQLite files on same filesystem

**Static Assets:**
- `static/` directory served by Axum
- CDN optional for production (not required)

### Configuration

**Environment-Specific:**
- `config/default.toml` - Committed defaults
- `config/dev.toml` - Local overrides (.gitignored)
- `config/prod.toml` - Production overrides (.gitignored)

**SMTP Configuration:**
```toml
[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "admin@imkitchen.app"
smtp_password = ""  # Set in dev.toml or prod.toml
from_address = "noreply@imkitchen.app"
admin_emails = ["admin@imkitchen.app"]
```

**Premium Bypass:**
```toml
[access_control]
global_premium_bypass = false  # Set to true for dev/staging
```

### Deployment Options

**Option 1: VPS (e.g., DigitalOcean, Linode)**
- Install Rust binary
- SystemD service for auto-restart
- Nginx reverse proxy with TLS
- Let's Encrypt for SSL certificates

**Option 2: Container (Docker)**
- Multi-stage build for small image
- Mount config/db volumes
- Expose port 3000

**Option 3: Platform (Fly.io, Railway)**
- Push binary to platform
- Configure environment variables
- Auto-scaling based on load

## Development Environment

### Prerequisites

**Required:**
- Rust 1.90+ (`rustup install stable`)
- Node.js 22+ (for Tailwind CLI and Playwright)
- SQLite 3.x (usually pre-installed)

**Optional:**
- Playwright 1.56+ (for E2E tests)
- `cargo-watch` for auto-rebuild
- `cargo-flamegraph` for profiling

### Setup Commands

```bash
# Clone repository
git clone https://github.com/yourusername/imkitchen.git
cd imkitchen

# Copy configuration template
cp config/default.toml config/dev.toml
# Edit config/dev.toml with your SMTP credentials

# Run migrations (creates databases)
cargo run -- migrate

# Start development server
cargo run -- serve

# In separate terminal: Watch and rebuild CSS
npx tailwindcss -i static/css/input.css -o static/css/output.css --watch

# Run tests
cargo test

# Run E2E tests (requires Playwright installed)
npm install
npx playwright test

# Check code quality
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

### Local Development Workflow

1. Run migrations: `cargo run -- migrate`
2. Start server: `cargo run -- serve` (accessible at http://localhost:3000)
3. Watch CSS: `npx tailwindcss -i static/css/input.css -o static/css/output.css --watch`
4. Make changes, server auto-reloads (or use `cargo-watch`)
5. Run tests frequently: `cargo test`
6. Format before commit: `cargo fmt --all`

## Architecture Decision Records (ADRs)

### ADR-001: Event-Driven Architecture with CQRS

**Decision:** Use evento for event sourcing with separate write/read/validation databases

**Rationale:**
- CLAUDE.md standard requires evento + CQRS
- Separate databases enable independent scaling
- Event sourcing provides audit trail and time travel
- Commands enforce business rules, queries optimize for reads

**Consequences:**
- Eventual consistency between commands and queries
- Additional complexity vs traditional CRUD
- Requires careful event schema design

---

### ADR-002: Pure Rust Meal Planning Algorithm

**Decision:** Implement generation algorithm in pure Rust, in-memory, with deterministic performance

**Rationale:**
- <5s P95 requirement demands predictable performance
- Complex constraints (dietary, cuisine variety, pairing) need algorithmic control
- In-memory processing with <100 recipes is feasible
- Easier to test and profile than SQL-based approach

**Consequences:**
- Algorithm lives in application code (not database)
- Need to load favorited recipes into memory
- Simpler performance profiling and optimization

---

### ADR-003: Recipe Snapshots in Separate Table

**Decision:** Store recipe snapshots in separate table, not embedded in events

**Rationale:**
- Events stay lightweight (only reference snapshot IDs)
- Query performance better with separate indexed table
- Can deduplicate identical snapshots across weeks
- Easier to query historical meal plans

**Consequences:**
- Two-phase write (snapshot table + event)
- Need to manage snapshot lifecycle
- Calendar queries join with snapshots table

---

### ADR-004: Centralized Access Control Service

**Decision:** Create `AccessControlService` with methods like `can_view_week()`, `can_add_favorite()`

**Rationale:**
- Single source of truth for freemium logic
- Fine-grained control (not just route-level)
- Easy to test tier restrictions in isolation
- Consistent enforcement across features

**Consequences:**
- Must remember to call service in all relevant code
- Service becomes critical dependency
- Changes to access logic centralized

---

### ADR-005: Streaming Parser for Recipe Import

**Decision:** Use tokio tasks with streaming JSON parser for 10MB file uploads

**Rationale:**
- Memory efficiency (constant memory, not 200MB spike)
- Can yield progress updates during parse
- Detect oversized payloads early
- Handles 20 concurrent files safely

**Consequences:**
- Slightly more complex than loading entire file
- Need to implement progress tracking
- Must handle partial failures gracefully

---

### ADR-006: Hybrid Notifications (In-App + Email)

**Decision:** Store reminders in database, show when user opens app, optional email

**Rationale:**
- No Web Push infrastructure needed for MVP
- Email has universal support
- In-app reminders work without permissions
- Can add Web Push post-MVP based on demand

**Consequences:**
- Not as immediate as push notifications
- Requires user to open app to see reminder
- Email may end up in spam

---

### ADR-007: SSR + Progressive Enhancement for PWA

**Decision:** Askama SSR with service worker for offline, no client-side rendering framework

**Rationale:**
- Perfect SEO (full HTML on first load)
- Matches CLAUDE.md standard (Askama + Twinspark)
- Service worker enables offline capability
- Minimal JavaScript, better performance

**Consequences:**
- Need to write service worker manually
- Less interactivity than SPA
- Page transitions require server round-trip

---

### ADR-008: Configurable SMTP with lettre

**Decision:** Use lettre crate with TOML-based SMTP configuration

**Rationale:**
- No vendor lock-in (any SMTP provider works)
- Simple integration, no API dependencies
- Portable across environments
- Meets NFR010 (avoid vendor lock-in)

**Consequences:**
- Users must configure SMTP credentials
- Deliverability depends on SMTP provider
- No built-in email tracking/analytics

---

_Generated by BMAD Decision Architecture Workflow v1.3_
_Date: 2025-10-31_
_For: Jonathan_
