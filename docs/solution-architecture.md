# Solution Architecture Document

**Project:** imkitchen
**Date:** 2025-10-11
**Author:** Jonathan
**Architecture Style:** Event-Sourced Monolith with Server-Side Rendering
**Repository:** Monorepo (Rust workspace)

## Executive Summary

imkitchen is an intelligent meal planning platform built as a Progressive Web App using Rust with server-side rendering. The architecture leverages event sourcing and CQRS via evento to maintain full audit trails and support complex meal planning workflows. The system is structured as a monolithic Rust application with domain-driven design, where business logic resides in workspace crates organized by bounded context. The root binary orchestrates domain logic and serves HTML via Axum and Askama templates, with TwinSpark providing progressive enhancement for interactive behaviors.

**Key Architectural Characteristics:**
- **Server-rendered HTML**: No frontend/backend split - single Rust application
- **Event sourcing**: Full audit trail via evento (SQLite event store)
- **CQRS**: Domain crates maintain their own read models from event streams
- **Progressive enhancement**: TwinSpark for reactivity without heavy JavaScript
- **Offline-first PWA**: Service workers cache server-rendered pages
- **Docker/Kubernetes deployment**: Containerized for cloud-native orchestration

## 1. Technology Stack and Decisions

### 1.1 Technology and Library Decision Table

| Category | Technology | Version | Justification |
|----------|------------|---------|---------------|
| **Language** | Rust | 1.90+ | Type safety, performance, zero-cost abstractions, strong ecosystem |
| **HTTP Server** | Axum | 0.8+ | Async, Tower middleware, excellent Rust ergonomics |
| **Templates** | Askama | 0.14+ | Compile-time type-safe templates, zero runtime overhead |
| **Event Sourcing/CQRS** | evento | 1.3+ | SQLite-based ES/CQRS, aggregate pattern, subscriptions |
| **Database** | SQLite | 3.45+ | Embedded, event store + read models, zero-ops |
| **Query Builder** | SQLx | 0.8+ | Async, compile-time SQL verification DISABLED per requirements |
| **CLI Framework** | Clap | 4.5+ | Derive macros, subcommands, shell completions |
| **Configuration** | config | 0.15+ | Dynamic path-based config with environment variable overrides for secrets |
| **Validation** | validator | 0.20+ | Derive-based validation for forms/inputs |
| **i18n** | rust-i18n | 3.1.5 | Multi-language support (initial English-only) |
| **Observability** | OpenTelemetry | 0.31+ | Distributed tracing, metrics, logs |
| **CSS Framework** | Tailwind CSS | 4.1+ | Utility-first, customizable design tokens |
| **Progressive Enhancement** | TwinSpark | latest | Declarative HTML attributes for interactivity |
| **E2E Testing** | Playwright | 1.56+ (TypeScript) | Cross-browser automation, PWA support |
| **JWT** | jsonwebtoken | 9.3+ | Cookie-based auth tokens |
| **Password Hashing** | argon2 | 0.5+ | OWASP-recommended password hashing |
| **Email (SMTP)** | lettre | 0.11+ | Async SMTP client for password reset/notifications |
| **HTTP Client** | reqwest | 0.12+ | Async HTTP client for external APIs (Stripe, etc.) |
| **Stripe SDK** | async-stripe | 0.39+ | Payment processing for premium subscriptions |
| **MinIO SDK** | rust-s3 | 0.34+ | S3-compatible client for recipe images |
| **Web Push** | web-push | 0.10+ | VAPID-based push notifications (browser standard) |
| **Serialization** | serde | 1.0+ | JSON serialization for API responses |
| **Async Runtime** | tokio | 1.40+ | Async runtime for Axum/reqwest/lettre |
| **Error Handling** | thiserror | 1.0+ | Custom error types with derive macros |
| **Logging** | tracing | 0.1+ | Structured logging with OpenTelemetry integration |
| **Service Worker** | Workbox | 7.1+ (JS) | PWA offline caching strategy |
| **Docker** | Docker | 25.0+ | Container runtime |
| **Orchestration** | Kubernetes | 1.30+ | Container orchestration, scaling, deployment |

### 1.2 Technology Rationale

**Rust Monolith**: Type safety, performance, and zero-cost abstractions ideal for event-sourced systems with complex domain logic. Single binary simplifies deployment.

**evento (Event Sourcing)**: Full event log provides audit trail for meal plan changes, enables temporal queries, supports CQRS projections.

**Server-Side Rendering (Askama)**: Type-safe templates compiled at build time, zero runtime overhead, eliminates frontend/backend complexity.

**TwinSpark (Progressive Enhancement)**: Adds interactivity (AJAX, form handling) via HTML attributes without React/Vue complexity. Degrades gracefully.

**SQLite**: Embedded database eliminates external dependencies, simplifies deployment. Sufficient for 10K concurrent users (per NFRs). evento handles event store, read models in same database.

**Docker/Kubernetes**: Aligns with deployment preference, enables horizontal scaling, blue-green deployments, health checks.

**Tailwind CSS**: Rapid UI development, consistent design system, small production bundles via PurgeCSS.

## 2. Application Architecture

### 2.1 Architecture Pattern: Event-Sourced Monolith

**Pattern**: Event-sourced monolithic application with DDD bounded contexts

```
┌─────────────────────────────────────────────────────────────┐
│                     Root Binary (CLI)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ serve        │  │ migrate      │  │ (future cmds)│     │
│  └──────┬───────┘  └──────┬───────┘  └──────────────┘     │
│         │                  │                                 │
│  ┌──────▼──────────────────▼──────────────────────────┐    │
│  │          HTTP Server (Axum)                         │    │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐   │    │
│  │  │  Routes    │  │ Templates  │  │ Middleware │   │    │
│  │  │ (Handlers) │  │ (Askama)   │  │   (Auth)   │   │    │
│  │  └─────┬──────┘  └────────────┘  └────────────┘   │    │
│  └────────┼───────────────────────────────────────────┘    │
│           │                                                  │
│  ┌────────▼──────────────────────────────────────────┐     │
│  │          Domain Crates (Business Logic)            │     │
│  │ ┌──────────┐ ┌──────────┐ ┌──────────┐  ┌──────┐│     │
│  │ │  user    │ │  recipe  │ │meal_plan │  │ ... ││     │
│  │ │          │ │          │ │          │  │     ││     │
│  │ │ Events   │ │ Events   │ │ Events   │  │     ││     │
│  │ │ Cmds     │ │ Cmds     │ │ Cmds     │  │     ││     │
│  │ │ ReadModel│ │ ReadModel│ │ ReadModel│  │     ││     │
│  │ └─────┬────┘ └─────┬────┘ └─────┬────┘  └──┬───┘│     │
│  └───────┼────────────┼────────────┼───────────┼────┘     │
│          │            │            │           │           │
│  ┌───────▼────────────▼────────────▼───────────▼────┐     │
│  │              evento (Event Store)                 │     │
│  │  ┌──────────────┐      ┌──────────────┐          │     │
│  │  │ Event Stream │ ───> │ Subscriptions│          │     │
│  │  │   (SQLite)   │      │ (Read Models)│          │     │
│  │  └──────────────┘      └──────────────┘          │     │
│  └───────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

**Architecture Characteristics:**

1. **Single Process**: All domain logic in one Rust binary
2. **Domain Crates**: Each bounded context is a workspace crate (e.g., `crates/user`, `crates/recipe`)
3. **Event Sourcing**: All state changes captured as events via evento
4. **CQRS**: Commands write events, queries read from materialized views (read models)
5. **No API Layer**: Routes directly invoke domain crates and render HTML via Askama

### 2.2 Server-Side Rendering Strategy

**Pattern**: Full server-side rendering with progressive enhancement

**Request Flow:**
1. Browser sends HTTP GET/POST to Axum route
2. Route handler invokes domain crate command/query
3. Domain crate loads aggregate from evento event stream OR queries read model
4. Handler passes data to Askama template
5. Askama renders HTML server-side (compile-time type checking)
6. HTML returned with TwinSpark attributes for interactivity
7. Browser renders HTML, TwinSpark enhances with AJAX behaviors

**Page Types:**
- **Public pages** (landing, login): No auth, cached aggressively
- **Authenticated pages** (dashboard, recipes): Auth middleware, user-specific data
- **Dynamic pages** (meal calendar): Real-time data from read models
- **Form pages** (create recipe): POST handlers with validation, redirect on success

**No client-side routing** - traditional multi-page application with TwinSpark for smooth transitions.

### 2.3 Page Routing and Navigation

**Route Structure:**

```rust
// Public routes
GET  /                    → Landing page
GET  /login               → Login form
POST /login               → Login handler
GET  /register            → Registration form
POST /register            → Registration handler
GET  /password-reset      → Password reset request
POST /password-reset      → Send reset email
GET  /password-reset/:token → Reset form
POST /password-reset/:token → Update password

// Authenticated routes (require JWT cookie)
GET  /dashboard           → Today's meals + prep tasks
GET  /plan                → Meal calendar (week view)
POST /plan/generate       → Generate new meal plan
POST /plan/regenerate     → Regenerate meal plan
POST /plan/meal/:id/replace → Replace individual meal slot

GET  /recipes             → Recipe library (with filters)
GET  /recipes/new         → Create recipe form
POST /recipes             → Create recipe handler
GET  /recipes/:id         → Recipe detail
GET  /recipes/:id/edit    → Edit recipe form
PUT  /recipes/:id         → Update recipe handler
DELETE /recipes/:id       → Delete recipe handler
POST /recipes/:id/favorite → Toggle favorite
POST /recipes/:id/share   → Share to community

// Public community routes (SEO-friendly)
GET  /discover            → Community recipe feed (public, indexed)
GET  /discover/:id        → Community recipe detail (public, indexed)

// Authenticated community actions (require JWT)
POST /discover/:id/add    → Add community recipe to library
POST /discover/:id/rate   → Rate/review recipe

GET  /shopping            → Current week shopping list
GET  /shopping/:week      → Specific week shopping list
POST /shopping/:id/check  → Mark item collected

GET  /notifications       → Notification settings
POST /notifications       → Update notification preferences

GET  /profile             → User profile and settings
PUT  /profile             → Update profile
GET  /subscription        → Subscription management
POST /subscription/upgrade → Upgrade to premium (Stripe)

// Static assets
GET  /static/*            → CSS, JS, images, service worker
GET  /sw.js               → Service worker for PWA
GET  /manifest.json       → PWA manifest
```

**Navigation Pattern**: Traditional `<a>` links for full page loads. TwinSpark can be added selectively with `ts-req` attribute for AJAX navigation, but defaults to standard browser navigation (progressive enhancement).

### 2.4 Data Fetching Approach

**Command/Query Segregation (CQRS):**

**Commands** (writes):
- Submitted via HTML forms (POST/PUT/DELETE)
- Routed to domain crate command handler
- Command handler loads aggregate from event stream
- Business logic validates command
- New event(s) appended to event stream
- Redirect to success page (PRG pattern: Post/Redirect/Get)

**Queries** (reads):
- Executed in route handlers before template rendering
- Query domain crate read models (SQLite tables)
- Read models updated via evento subscriptions
- Data passed to Askama template
- Template renders HTML with data

**Example Flow - Create Recipe:**
```rust
POST /recipes
  ↓
Axum Handler receives form data
  ↓
Validate with validator crate
  ↓
// Create new Recipe aggregate with RecipeCreated event
let recipe_id = evento::create::<Recipe>()
    .data(&RecipeCreated { title, ingredients, ... })?
    .metadata(&user_id)?
    .commit(&executor)
    .await?;
  ↓
RecipeCreated event written to evento event stream
  ↓
evento subscription (background) updates recipe_read_model table
  ↓
Redirect to GET /recipes/:id
  ↓
Query recipe_read_model table (SQLx)
  ↓
Render recipe detail template (Askama)
```

**No client-side state** - all state in database, all rendering server-side.

## 3. Data Architecture

### 3.1 Database Schema

**Database**: Single SQLite database with two schema types:

1. **Event Store** (managed by evento):
   - `events` table and indexes created automatically by evento migrations
   - Schema details abstracted by evento library
   - Stores: Aggregate ID, event type, payload (JSON), timestamp, version

2. **Read Models** (materialized views - manual migrations):
   - `users` - User profiles and auth
   - `recipes` - Recipe library (denormalized)
   - `meal_plans` - Active meal plans
   - `shopping_lists` - Generated shopping lists
   - `notifications` - Prep reminders queue

**Note**: evento manages event store schema internally. No manual SQL needed for `events` table.

### 3.2 Data Models and Relationships

**Read Model Schemas** (simplified - full schemas in per-epic tech specs):

**users table:**
```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,           -- UUID
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,   -- Argon2
  created_at TEXT NOT NULL,
  -- Profile fields
  dietary_restrictions TEXT,     -- JSON array
  household_size INTEGER,
  skill_level TEXT,              -- beginner|intermediate|expert
  weeknight_availability TEXT,   -- JSON time range
  -- Subscription
  tier TEXT NOT NULL,            -- free|premium
  recipe_limit INTEGER,          -- 10 for free, NULL for premium
  -- Projections updated from UserCreated, ProfileUpdated, SubscriptionUpgraded events
);
```

**recipes table:**
```sql
CREATE TABLE recipes (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  ingredients TEXT NOT NULL,     -- JSON array of {name, qty, unit}
  instructions TEXT NOT NULL,    -- JSON array of step strings
  prep_time_min INTEGER,
  cook_time_min INTEGER,
  advance_prep_hours INTEGER,    -- NULL if no advance prep
  serving_size INTEGER,
  is_favorite BOOLEAN DEFAULT FALSE,
  is_shared BOOLEAN DEFAULT FALSE,
  complexity TEXT,               -- simple|moderate|complex
  cuisine TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_favorite ON recipes(user_id, is_favorite);
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = TRUE;
```

**meal_plans table:**
```sql
CREATE TABLE meal_plans (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  start_date TEXT NOT NULL,      -- ISO 8601 date (Monday)
  status TEXT NOT NULL,          -- active|archived
  rotation_state TEXT,           -- JSON: tracks which recipes used
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE meal_assignments (
  id TEXT PRIMARY KEY,
  meal_plan_id TEXT NOT NULL,
  date TEXT NOT NULL,            -- ISO 8601 date
  meal_type TEXT NOT NULL,       -- breakfast|lunch|dinner
  recipe_id TEXT NOT NULL,
  prep_required BOOLEAN,
  FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id),
  FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);
```

**shopping_lists table:**
```sql
CREATE TABLE shopping_lists (
  id TEXT PRIMARY KEY,
  meal_plan_id TEXT NOT NULL,
  week_start_date TEXT NOT NULL,
  generated_at TEXT NOT NULL,
  FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id)
);

CREATE TABLE shopping_list_items (
  id TEXT PRIMARY KEY,
  shopping_list_id TEXT NOT NULL,
  ingredient_name TEXT NOT NULL,
  quantity REAL NOT NULL,
  unit TEXT NOT NULL,
  category TEXT,                 -- produce|dairy|meat|pantry
  is_collected BOOLEAN DEFAULT FALSE,
  FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists(id)
);
```

**ratings table:**
```sql
CREATE TABLE ratings (
  id TEXT PRIMARY KEY,
  recipe_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  stars INTEGER NOT NULL CHECK(stars >= 1 AND stars <= 5),
  review_text TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (recipe_id) REFERENCES recipes(id),
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE(recipe_id, user_id)
);

CREATE INDEX idx_ratings_recipe ON ratings(recipe_id);
```

**Event-to-ReadModel Projections** (evento subscriptions):

**Example - Recipe Aggregate:**
```rust
// Domain events
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct RecipeCreated {
    title: String,
    ingredients: Vec<Ingredient>,
    // ...
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]
struct RecipeFavorited { favorited: bool }

// Aggregate
#[derive(Default, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
struct Recipe {
    title: String,
    ingredients: Vec<Ingredient>,
    is_favorite: bool,
    // ...
}

// Event handlers (rebuilds aggregate state from events)
#[evento::aggregator]
impl Recipe {
    async fn recipe_created(&mut self, event: EventDetails<RecipeCreated>) -> anyhow::Result<()> {
        self.title = event.data.title;
        self.ingredients = event.data.ingredients;
        Ok(())
    }

    async fn recipe_favorited(&mut self, event: EventDetails<RecipeFavorited>) -> anyhow::Result<()> {
        self.is_favorite = event.data.favorited;
        Ok(())
    }
}

// Read model projection (subscription handler)
#[evento::handler(Recipe)]
async fn project_recipe_to_read_model<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Insert into read model table
    sqlx::query!(
        "INSERT INTO recipes (id, user_id, title, ingredients, created_at) VALUES (?, ?, ?, ?, ?)",
        event.aggregator_id,
        event.metadata, // user_id stored in metadata
        event.data.title,
        serde_json::to_string(&event.data.ingredients)?,
        chrono::Utc::now().to_rfc3339()
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

**Subscription Setup** (in main.rs):
```rust
// Register read model projection subscriptions
evento::subscribe("recipe-projections")
    .aggregator::<Recipe>()
    .handler(project_recipe_to_read_model())
    .handler(project_recipe_favorited())
    .run(&executor)
    .await?;
```

**Projection Patterns**:
- `UserCreated` → Insert into `users` table (subscription handler)
- `RecipeCreated` → Insert into `recipes` table (subscription handler)
- `RecipeFavorited` → Update `is_favorite` in `recipes` table (subscription handler)
- `MealPlanGenerated` → Insert into `meal_plans` + `meal_assignments` tables (subscription handler)
- `MealReplaced` → Update `meal_assignments` + regenerate shopping list (subscription handler)
- `RecipeRated` → Insert/update `ratings` table (subscription handler)

### 3.3 Data Migrations Strategy

**Two Migration Systems:**

1. **evento Event Store Migrations** (automatic):
   - evento manages event store schema internally
   - Run via `evento::sql_migrator::new_migrator::<sqlx::Sqlite>().unwrap().run(&mut *conn, &Plan::apply_all()).await.unwrap();`
   - Creates `events` table and indexes automatically
   - No manual SQL files needed for event store

2. **Read Model Migrations** (SQLx):
   - Manual migrations for read model tables (users, recipes, meal_plans, etc.)
   - SQLx migration files in `migrations/` directory

**Migration Files**:
```
migrations/
├── 001_create_users_table.sql
├── 002_create_recipes_table.sql
├── 003_create_meal_plans_table.sql
├── 004_create_shopping_lists_table.sql
├── 005_create_ratings_table.sql
├── 006_create_notifications_table.sql
└── 007_create_push_subscriptions_table.sql
```

**CLI Command**: `imkitchen migrate`

**Execution Flow**:
```rust
async fn run_migrations(pool: &SqlitePool) -> Result<(), Error> {
    let mut conn = pool.acquire().await?;

    // 1. evento event store migrations (automatic)
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut *conn, &Plan::apply_all())
        .await
        .unwrap();

    // 2. Read model migrations (SQLx)
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    Ok(())
}
```

**Execution**: Run on startup in production (idempotent), manual in development.

**Rollback Strategy**: Manual rollback scripts for read model migrations. Event sourcing enables replaying projections from event stream.

## 4. API Design

### 4.1 API Structure

**No REST API** - This is a server-rendered application with HTML endpoints.

**Form-Based Mutations:**
- All writes via HTML forms (POST/PUT/DELETE)
- `application/x-www-form-urlencoded` or `multipart/form-data` (recipe images)
- Validation server-side with `validator` crate
- Errors rendered inline in forms (re-render with error messages)

**TwinSpark AJAX Enhancements:**
- Selected routes support `Accept: text/html` partial responses
- TwinSpark attributes on buttons/forms trigger AJAX requests
- Server returns HTML fragments (not JSON)
- Fragments swapped into DOM via TwinSpark

**Example - Replace Meal (AJAX):**
```html
<form ts-req="/plan/meal/123/replace"
      ts-req-method="POST"
      ts-target="#meal-slot-123"
      ts-swap="outerHTML">
  <select name="recipe_id">...</select>
  <button type="submit">Replace</button>
</form>
```

Server responds with HTML fragment:
```html
<div id="meal-slot-123" class="meal-slot">
  <h4>New Recipe Title</h4>
  <span class="prep-indicator">Prep Required</span>
</div>
```

**TwinSpark Attributes Used:**
- `ts-req`: URL to request (can be relative or absolute)
- `ts-req-method`: HTTP method (GET, POST, PUT, DELETE)
- `ts-target`: CSS selector for element to update
- `ts-swap`: Swap strategy (innerHTML, outerHTML, beforebegin, afterend, etc.)
- `ts-trigger`: Custom trigger events (default: submit for forms, click for buttons)

### 4.2 HTML Endpoints (Not REST)

**Endpoint Categories:**

1. **Page Endpoints** (GET): Return full HTML pages
2. **Form Handlers** (POST/PUT/DELETE): Process mutations, redirect or re-render
3. **Partial Endpoints** (GET with TwinSpark): Return HTML fragments for AJAX

**Response Patterns:**
- Success: 302 Redirect (PRG pattern) or 200 OK with HTML fragment (TwinSpark)
- Validation error: 422 Unprocessable Entity with form re-rendered with errors
- Auth error: 401 Unauthorized, redirect to /login
- Not found: 404 with error page

### 4.3 Form Actions and Mutations

**Form Validation Pattern:**
```rust
#[derive(Deserialize, Validate)]
struct CreateRecipeForm {
    #[validate(length(min = 3, max = 200))]
    title: String,
    #[validate(length(min = 1))]
    ingredients: String, // JSON string, validate after parse
    #[validate(range(min = 1, max = 999))]
    prep_time_min: u32,
    // ...
}

async fn create_recipe_handler(
    auth: Auth,
    Form(form): Form<CreateRecipeForm>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate
    form.validate()?;

    // 2. Parse ingredients JSON
    let ingredients: Vec<Ingredient> = serde_json::from_str(&form.ingredients)?;

    // 3. Invoke domain command
    let cmd = CreateRecipeCommand { ... };
    let recipe_id = recipe::create_recipe(cmd).await?;

    // 4. Redirect to recipe detail
    Ok(Redirect::to(&format!("/recipes/{}", recipe_id)))
}
```

**Error Handling:**
- Validation errors collected, form re-rendered with inline error messages
- Domain errors (e.g., "Recipe limit reached") displayed as flash messages
- Unexpected errors logged, user sees generic error page

## 5. Authentication and Authorization

### 5.1 Auth Strategy

**JWT Cookie-Based Authentication:**

1. **Registration**: POST /register → Password hashed with Argon2 → User aggregate created → UserCreated event
2. **Login**: POST /login → Verify password → Generate JWT → Set HTTP-only cookie
3. **Session**: JWT cookie included in all requests → Axum middleware validates → Extract user ID
4. **Logout**: POST /logout → Clear cookie

**JWT Claims:**
```rust
struct Claims {
    sub: String,      // User ID
    email: String,
    tier: String,     // free|premium
    exp: u64,         // Expiration timestamp
    iat: u64,         // Issued at
}
```

**JWT Configuration:**
- Algorithm: HS256
- Secret: Environment variable (32-byte random key)
- Expiration: 7 days
- Cookie: HTTP-only, Secure (HTTPS only), SameSite=Lax

### 5.2 Session Management

**Stateless Sessions**: No server-side session store, all state in JWT.

**Token Refresh**: No refresh token pattern in MVP. Users re-authenticate after 7 days.

**Security Measures:**
- CSRF protection via SameSite cookie attribute
- HTTP-only cookie prevents XSS access to token
- Secure flag enforces HTTPS in production
- Short-lived tokens reduce exposure window

### 5.3 Protected Routes

**Auth Middleware**:
```rust
async fn auth_middleware(
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = cookies.get("auth_token")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode_jwt(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
```

**Route Protection**: Apply middleware to all `/dashboard`, `/recipes`, `/plan`, `/shopping`, `/profile` routes.

**Public Routes**:
- Landing/Auth: `/`, `/login`, `/register`, `/password-reset`
- Community Discovery (SEO): `/discover`, `/discover/:id` (read-only, indexed by search engines)

**Authenticated Actions**: POST/PUT/DELETE on `/discover/:id/*` require JWT (add to library, rate/review).

### 5.4 Role-Based Access Control

**MVP Roles**: `free` and `premium` tiers (stored in JWT claims and user record).

**Authorization Checks**:
- Free tier: Enforce 10 recipe limit in `CreateRecipeCommand`
- Premium tier: Unrestricted recipe creation
- Domain logic: Recipe limit check in `user` crate aggregate logic

**Future RBAC**: Admin role for moderation, recipe visibility rules.

## 6. State Management

### 6.1 Server State (Event Sourcing)

**All application state** managed via evento event sourcing:

**Aggregates** (domain crates):
- `User` aggregate: Auth, profile, subscription
- `Recipe` aggregate: Recipe CRUD, favoriting, sharing
- `MealPlan` aggregate: Meal plan generation, rotation logic
- `ShoppingList` aggregate: Shopping list calculation
- `Notification` aggregate: Prep reminder scheduling

**Event Stream**: All state changes recorded as events in SQLite `events` table.

**State Reconstruction**: Load aggregate from event stream, replay events to current state.

**Projections**: evento subscriptions update read models (SQLite tables) for queries.

### 6.2 Client State (Minimal)

**No client-side state management** (no Redux, Zustand, etc.).

**Transient UI State**:
- Form inputs: Native HTML form state
- Dropdown open/closed: CSS :focus-within or TwinSpark attributes
- Modal open/closed: TwinSpark state attributes

**Persistence**: All meaningful state persisted server-side, retrieved on page load.

### 6.3 Form State

**Native HTML Forms**:
- Standard `<form>` elements with `<input>`, `<select>`, `<textarea>`
- Validation: Server-side with `validator` crate, errors displayed inline
- Client-side hints: HTML5 validation attributes (`required`, `minlength`, `pattern`)

**Progressive Enhancement**:
- Forms work without JavaScript (full page POST)
- TwinSpark intercepts form submissions for AJAX behavior
- Fallback to traditional POST on TwinSpark failure

### 6.4 Caching Strategy

**Server-Side Caching**:
- **HTTP Caching**: Cache-Control headers on static assets (1 year), public pages (5 min)
- **Read Model Caching**: Read models ARE the cache (materialized views from events)
- **Query Caching**: No additional layer - SQLite queries fast enough for MVP

**Client-Side Caching**:
- **Service Worker**: Workbox caches HTML pages, static assets for offline access
- **Cache Strategy**: Stale-while-revalidate for HTML, cache-first for static assets

**Invalidation**: Read models updated in real-time via evento subscriptions. No explicit cache invalidation needed.

## 7. UI/UX Architecture

### 7.1 Component Structure

**Askama Template Hierarchy**:

```
templates/
├── base.html                    # Base layout (DOCTYPE, <head>, nav, footer)
├── components/
│   ├── button.html              # Reusable button component
│   ├── recipe-card.html         # Recipe card (grid/list variants)
│   ├── meal-slot.html           # Calendar meal slot
│   ├── shopping-item.html       # Shopping list item with checkbox
│   ├── form-field.html          # Input with label and error display
│   ├── modal.html               # Modal container with backdrop
│   ├── toast.html               # Notification toast
│   └── nav-tabs.html            # Bottom tab navigation (mobile)
├── pages/
│   ├── landing.html             # Public landing page
│   ├── login.html               # Login form
│   ├── dashboard.html           # User dashboard (today's meals)
│   ├── meal-calendar.html       # Week view meal calendar
│   ├── recipe-list.html         # Recipe library with filters
│   ├── recipe-detail.html       # Recipe detail page
│   ├── recipe-form.html         # Create/edit recipe
│   ├── shopping-list.html       # Shopping list with checkboxes
│   ├── community-feed.html      # Community recipe discovery
│   └── profile.html             # User profile settings
└── partials/
    ├── recipe-grid.html         # Recipe grid fragment (AJAX)
    ├── meal-slot-content.html   # Meal slot HTML (AJAX replace)
    └── shopping-category.html   # Shopping category section
```

**Template Composition**:
```html
{% extends "base.html" %}

{% block title %}Dashboard{% endblock %}

{% block content %}
<div class="container">
  <h1>Today's Meals</h1>

  {% for meal in todays_meals %}
    {% include "components/meal-slot.html" %}
  {% endfor %}

  <a href="/plan/generate" class="btn-primary">Generate Meal Plan</a>
</div>
{% endblock %}
```

**Component Macros** (reusable logic):
```html
{# Define macro #}
{% macro recipe_card(recipe) %}
<div class="recipe-card">
  <img src="{{ recipe.image_url }}" alt="{{ recipe.title }}">
  <h3>{{ recipe.title }}</h3>
  <span class="complexity-{{ recipe.complexity }}">{{ recipe.complexity }}</span>
  <p>{{ recipe.prep_time_min }} min prep, {{ recipe.cook_time_min }} min cook</p>
</div>
{% endmacro %}

{# Use macro #}
{% for recipe in recipes %}
  {% call recipe_card(recipe) %}
{% endfor %}
```

### 7.2 Styling Approach

**Tailwind CSS** with custom configuration:

**Design Tokens** (tailwind.config.js):
```js
module.exports = {
  theme: {
    extend: {
      colors: {
        primary: { 500: '#2563eb', 600: '#1d4ed8' },
        secondary: { 500: '#f59e0b', 600: '#d97706' },
        success: { 500: '#10b981' },
        warning: { 500: '#f59e0b' },
        error: { 500: '#ef4444' },
      },
      spacing: {
        // 8px grid system
      },
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
      },
    },
  },
}
```

**Utility Classes**:
- Layout: `container`, `grid`, `flex`, responsive variants
- Typography: `text-lg`, `font-semibold`, heading sizes
- Spacing: `p-4`, `m-8`, `gap-6`
- Colors: `bg-primary-500`, `text-gray-900`, `border-gray-400`
- Interactive: `hover:bg-primary-600`, `focus:ring-2`

**Custom Components** (when needed):
```css
@layer components {
  .btn-primary {
    @apply bg-primary-500 text-white px-6 py-3 rounded-lg
           hover:bg-primary-600 focus:ring-2 focus:ring-offset-2;
  }

  .recipe-card {
    @apply bg-white rounded-xl shadow-sm overflow-hidden
           hover:shadow-md transition-shadow;
  }
}
```

### 7.3 Responsive Design

**Breakpoints**:
- Mobile: 0-767px (default styling)
- Tablet: 768px-1023px (`md:` prefix)
- Desktop: 1024px+ (`lg:` prefix)

**Mobile-First Approach**:
```html
<div class="flex flex-col md:flex-row lg:grid lg:grid-cols-3">
  <!-- Mobile: stack vertically -->
  <!-- Tablet: horizontal flex -->
  <!-- Desktop: 3-column grid -->
</div>
```

**Navigation Adaptation**:
- Mobile: Bottom tab bar (fixed position)
- Desktop: Left sidebar with icons + labels

**Layout Adaptation**:
- Calendar: Mobile (1 day per screen, swipe), Desktop (full week view)
- Recipe library: Mobile (list or 2-col grid), Desktop (4-col grid)

### 7.4 Accessibility

**WCAG 2.1 Level AA Compliance:**

1. **Semantic HTML**: `<nav>`, `<main>`, `<article>`, heading hierarchy
2. **ARIA Attributes**: `aria-label` on icon buttons, `aria-live` on toasts
3. **Keyboard Navigation**: All interactive elements focusable, visible focus rings
4. **Color Contrast**: 4.5:1 (normal text), 3:1 (large text), 7:1 (kitchen mode)
5. **Alt Text**: All images have descriptive `alt` attributes
6. **Form Labels**: Explicit `<label for="input-id">` associations
7. **Skip Links**: "Skip to main content" at top of page
8. **Touch Targets**: 44x44px minimum (Tailwind defaults)

**Kitchen Mode Toggle**:
- High contrast variant: 7:1 color ratios
- Large text: +4px on headings/body
- Activated via `/profile` settings

**Screen Reader Testing**: Manual testing with NVDA/VoiceOver required per sprint.

## 8. Performance Optimization

### 8.1 Server-Side Rendering Performance

**Compile-Time Optimization**:
- Askama templates compiled to Rust code at build time
- Zero runtime template parsing overhead
- Type checking catches errors before deployment

**Database Query Optimization**:
- Indexed queries on read models (user_id, recipe favorites, etc.)
- SQLite prepared statements cached
- Connection pooling via SQLx

**Response Caching**:
- Static assets: `Cache-Control: public, max-age=31536000, immutable`
- Public pages: `Cache-Control: public, max-age=300` (5 min)
- Authenticated pages: `Cache-Control: private, no-cache`

### 8.2 Static Asset Optimization

**CSS**:
- Tailwind CSS purged (only used classes in production bundle)
- Minified and compressed (gzip/brotli)
- Single CSS file, loaded in `<head>` (render-blocking acceptable for critical styles)

**JavaScript**:
- TwinSpark library (~5KB minified)
- Service worker (Workbox, ~10KB)
- Minimal custom JS for progressive enhancement
- Deferred loading (`<script defer>`)

**Images**:
- Recipe images stored in MinIO (S3-compatible)
- Served via CDN (future optimization)
- Lazy loading (`loading="lazy"` attribute)
- WebP format with JPEG fallback

### 8.3 PWA Offline Strategy

**Service Worker** (Workbox):

```js
// sw.js
import { precacheAndRoute } from 'workbox-precaching';
import { registerRoute } from 'workbox-routing';
import { StaleWhileRevalidate, CacheFirst } from 'workbox-strategies';

// Precache static assets at install
precacheAndRoute(self.__WB_MANIFEST);

// Cache HTML pages (stale-while-revalidate)
registerRoute(
  ({request}) => request.mode === 'navigate',
  new StaleWhileRevalidate({
    cacheName: 'pages',
    plugins: [
      new ExpirationPlugin({ maxEntries: 50 }),
    ],
  })
);

// Cache images (cache-first)
registerRoute(
  ({request}) => request.destination === 'image',
  new CacheFirst({
    cacheName: 'images',
    plugins: [
      new ExpirationPlugin({ maxEntries: 100, maxAgeSeconds: 30 * 24 * 60 * 60 }),
    ],
  })
);

// Cache API responses (network-first with fallback)
registerRoute(
  ({url}) => url.pathname.startsWith('/recipes'),
  new NetworkFirst({
    cacheName: 'api',
    plugins: [
      new ExpirationPlugin({ maxEntries: 100 }),
    ],
  })
);
```

**Offline Functionality**:
- Cached pages: Dashboard, recipe library, recipe details
- Cached data: User's favorite recipes, active meal plan
- Background sync: Queue mutations (create recipe, rate recipe) when offline, sync on reconnect

### 8.4 Database Performance

**SQLite Optimizations**:
- WAL mode enabled: `PRAGMA journal_mode=WAL`
- Synchronous=NORMAL: `PRAGMA synchronous=NORMAL`
- Foreign keys enabled: `PRAGMA foreign_keys=ON`
- Analyze statistics: `ANALYZE` on startup

**Read Model Indexes**: All foreign keys, user_id columns, frequently filtered columns.

**Connection Pooling**: SQLx pool (max 5 connections for MVP, sufficient for 10K users).

**Query Patterns**:
- Favor read models over event stream queries
- Use evento subscriptions for real-time projection updates
- Avoid N+1 queries (join/batch where possible)

## 9. SEO and PWA Configuration

### 9.1 SEO Strategy

**Public Community Discovery**:
- `/discover` and `/discover/:id` routes are public (no auth required)
- Server-rendered HTML with full recipe content for search engine crawling
- Meta tags: title, description, Open Graph, Twitter Cards
- Structured data: Recipe schema (JSON-LD) for rich snippets
- Sitemap: `/sitemap.xml` includes all public community recipes
- Robots.txt: Allow indexing of `/discover/*`, disallow `/dashboard`, `/profile`, etc.

**Example Recipe Meta Tags**:
```html
<head>
  <title>Chicken Tikka Masala - imkitchen</title>
  <meta name="description" content="Authentic Chicken Tikka Masala recipe with 24-hour marinade. Rated 4.8 stars by 47 home cooks.">

  <!-- Open Graph -->
  <meta property="og:title" content="Chicken Tikka Masala">
  <meta property="og:description" content="Authentic Chicken Tikka Masala recipe...">
  <meta property="og:image" content="https://imkitchen.app/recipes/123/image.jpg">
  <meta property="og:url" content="https://imkitchen.app/discover/123">

  <!-- Schema.org Recipe -->
  <script type="application/ld+json">
  {
    "@context": "https://schema.org",
    "@type": "Recipe",
    "name": "Chicken Tikka Masala",
    "author": { "@type": "Person", "name": "Chef Marcus" },
    "aggregateRating": {
      "@type": "AggregateRating",
      "ratingValue": "4.8",
      "reviewCount": "47"
    },
    "prepTime": "PT20M",
    "cookTime": "PT30M",
    "totalTime": "PT50M"
  }
  </script>
</head>
```

**SEO Benefits**:
- Search engines index community recipes
- Rich snippets in Google search results (recipe cards)
- Social sharing with preview cards
- Organic traffic drives user acquisition
- Guest users can browse, must register to add/rate

### 9.2 PWA Manifest

**manifest.json**:
```json
{
  "name": "imkitchen - Intelligent Meal Planning",
  "short_name": "imkitchen",
  "description": "Automated meal planning and cooking optimization",
  "start_url": "/dashboard",
  "display": "standalone",
  "background_color": "#ffffff",
  "theme_color": "#2563eb",
  "orientation": "portrait-primary",
  "icons": [
    {
      "src": "/static/icons/icon-192.png",
      "sizes": "192x192",
      "type": "image/png",
      "purpose": "any maskable"
    },
    {
      "src": "/static/icons/icon-512.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any maskable"
    }
  ],
  "categories": ["lifestyle", "food"],
  "screenshots": [
    {
      "src": "/static/screenshots/dashboard-mobile.png",
      "sizes": "750x1334",
      "type": "image/png",
      "platform": "narrow"
    }
  ]
}
```

### 9.3 Push Notifications

**Web Push API** (browser standard, no vendor):

**Setup Flow**:
1. User enables notifications in `/profile` settings
2. Browser requests notification permission
3. Browser generates push subscription (endpoint + keys)
4. POST subscription to `/api/push-subscribe`
5. Store subscription in `push_subscriptions` table

**Sending Notifications**:
1. Domain event: `PrepReminderScheduled` (scheduled 8 hours before meal)
2. Background worker queries `push_subscriptions` for user
3. Use `web-push` crate to send notification via VAPID
4. Notification payload: Title, body, icon, action buttons

**Notification Example**:
```json
{
  "title": "Prep Reminder",
  "body": "Marinate chicken tonight for Thursday's Chicken Tikka Masala",
  "icon": "/static/icons/icon-192.png",
  "badge": "/static/icons/badge-72.png",
  "actions": [
    {"action": "view", "title": "View Recipe"},
    {"action": "dismiss", "title": "Dismiss"}
  ],
  "data": {
    "recipe_id": "123",
    "url": "/recipes/123"
  }
}
```

**Service Worker Notification Handler**:
```js
self.addEventListener('notificationclick', (event) => {
  event.notification.close();
  if (event.action === 'view') {
    clients.openWindow(event.notification.data.url);
  }
});
```

## 10. Deployment Architecture

### 10.1 Docker Containerization

**Dockerfile** (multi-stage build):
```dockerfile
# Build stage
FROM rust:1.90 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/imkitchen /usr/local/bin/
COPY static /app/static
COPY templates /app/templates

WORKDIR /app

ENV DATABASE_URL=sqlite:///data/imkitchen.db
ENV PORT=8080

EXPOSE 8080

CMD ["imkitchen", "serve"]
```

**Image Size Optimization**:
- Multi-stage build (Rust toolchain not in runtime image)
- Slim Debian base (~80MB)
- Static binary (~20MB)
- Final image: ~100MB

### 10.2 Kubernetes Deployment

**Deployment YAML** (simplified):
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: imkitchen
spec:
  replicas: 3
  selector:
    matchLabels:
      app: imkitchen
  template:
    metadata:
      labels:
        app: imkitchen
    spec:
      containers:
      - name: imkitchen
        image: imkitchen:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          value: "sqlite:///data/imkitchen.db"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: imkitchen-secrets
              key: jwt-secret
        volumeMounts:
        - name: data
          mountPath: /data
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: imkitchen-data
---
apiVersion: v1
kind: Service
metadata:
  name: imkitchen
spec:
  selector:
    app: imkitchen
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

**Persistent Volume** for SQLite database:
- PVC for `/data` directory
- ReadWriteMany for multi-pod access (NFS or equivalent)
- Backup strategy: periodic snapshots of PVC

**Note**: SQLite with multiple pods requires shared filesystem (NFS) OR single writer with read replicas. For MVP, single pod sufficient; scale to 3 pods with shared NFS volume.

### 10.3 Scaling Considerations

**Horizontal Scaling** (Kubernetes):
- Stateless application layer (all state in database)
- SQLite on shared volume (NFS) supports multiple readers
- Alternative: Migrate to PostgreSQL for multi-writer scenario at scale

**Database Scaling**:
- MVP: Single SQLite instance (10K concurrent users feasible)
- Scale-up: Migrate to PostgreSQL with evento (supports Postgres backend)
- Sharding: Not needed at MVP scale

**Load Balancing**:
- Kubernetes Service (LoadBalancer type) distributes traffic
- Session affinity not required (stateless JWTs)

### 10.4 Dynamic Configuration

**Configuration Strategy** (via `config` crate 0.15+):

**Config File Path** (dynamic, specified at runtime):
- CLI flag: `--config /path/to/config.toml`
- Environment variable: `CONFIG_PATH=/path/to/config.toml`
- Default: `config/default.toml` (if no path specified)

**Configuration Priority** (highest to lowest):
1. **Environment variables** - Overrides for any config value (IMKITCHEN__JWT__SECRET, IMKITCHEN__DATABASE__URL, etc.)
2. **Config file** - All settings from path specified via CLI/env (can include secrets)
3. **Defaults** - Hardcoded fallbacks in code

**Example Configuration File**:
```toml
[server]
port = 3000
host = "0.0.0.0"

[database]
url = "sqlite:///data/imkitchen.db"
max_connections = 5

[jwt]
secret = "your-secret-key-here"  # Can be set here or overridden via IMKITCHEN__JWT__SECRET
expiration_days = 7

[smtp]
host = "smtp.example.com"
port = 587
username = "user@example.com"
password = "smtp-password"  # Can be set here or overridden via IMKITCHEN__SMTP__PASSWORD

[stripe]
secret_key = "sk_test_..."  # Can be set here or overridden via IMKITCHEN__STRIPE__SECRET_KEY

[minio]
endpoint = "http://minio:9000"
access_key = "minioadmin"
secret_key = "minioadmin"

[vapid]
public_key = "..."
private_key = "..."

[observability]
otel_endpoint = "http://localhost:4317"
log_level = "info"
```

**Configuration Values** (can be in config file or environment variables):
- `jwt.secret`: Secret key for JWT signing (32+ bytes)
- `database.url`: SQLite or Postgres connection string
- `smtp.*`: Email service configuration (host, port, username, password)
- `stripe.secret_key`: Stripe API key
- `minio.*`: MinIO/S3 configuration (endpoint, access_key, secret_key)
- `vapid.*`: Web Push VAPID keys (public_key, private_key)
- `observability.otel_endpoint`: OpenTelemetry collector endpoint

**Usage Examples**:
```bash
# 1. All configuration in config file (development)
imkitchen --config config/dev.toml serve

# 2. Config file with environment variable overrides (production)
export IMKITCHEN__JWT__SECRET=$(openssl rand -base64 32)
export IMKITCHEN__SMTP__PASSWORD="secure-password"
imkitchen --config /etc/imkitchen/production.toml serve

# 3. Use default config path
imkitchen serve

# 4. Config path via environment variable
export CONFIG_PATH=/opt/imkitchen/config.toml
imkitchen serve
```

**Security Recommendations**:
- **Development**: Store all config (including secrets) in config files for convenience
- **Production**: Two options:
  1. Store secrets in config file with restricted permissions (chmod 600)
  2. Store secrets in environment variables (Kubernetes Secrets, Docker secrets, etc.)
- **Environment variables override config file** - use this for sensitive production secrets
- **Best practice**: Never commit production secrets to version control

## 11. Component and Integration Overview

### 11.1 Domain Crate Structure

**Workspace Crates**:

```
crates/
├── shared_kernel/       # Shared types, events, traits
│   ├── lib.rs
│   ├── types.rs         # UserId, RecipeId, etc.
│   ├── events.rs        # Cross-domain events
│   └── traits.rs        # Common traits
├── user/                # User domain
│   ├── lib.rs
│   ├── aggregate.rs     # UserAggregate (evento)
│   ├── commands.rs      # RegisterUser, UpdateProfile, etc.
│   ├── events.rs        # UserCreated, ProfileUpdated, etc.
│   └── read_model.rs    # User query projections
├── recipe/              # Recipe domain
│   ├── lib.rs
│   ├── aggregate.rs     # RecipeAggregate
│   ├── commands.rs      # CreateRecipe, UpdateRecipe, etc.
│   ├── events.rs        # RecipeCreated, RecipeFavorited, etc.
│   └── read_model.rs    # Recipe query projections
├── meal_planning/       # Meal planning domain
│   ├── lib.rs
│   ├── aggregate.rs     # MealPlanAggregate
│   ├── commands.rs      # GenerateMealPlan, ReplaceMeal, etc.
│   ├── events.rs        # MealPlanGenerated, MealReplaced, etc.
│   ├── algorithm.rs     # Meal planning optimization algorithm
│   └── read_model.rs    # Meal plan query projections
├── shopping/            # Shopping list domain
│   ├── lib.rs
│   ├── aggregate.rs     # ShoppingListAggregate
│   ├── commands.rs      # GenerateShoppingList, MarkItemCollected
│   ├── events.rs        # ShoppingListGenerated, ItemCollected
│   └── read_model.rs    # Shopping list query projections
└── notifications/       # Notification domain
    ├── lib.rs
    ├── aggregate.rs     # NotificationAggregate
    ├── commands.rs      # ScheduleReminder, SendReminder
    ├── events.rs        # ReminderScheduled, ReminderSent
    └── scheduler.rs     # Background worker for reminders
```

### 11.2 Root Binary Structure

```
src/
├── main.rs              # CLI entry point (Clap commands)
├── server.rs            # Axum HTTP server setup
├── routes/              # HTTP route handlers
│   ├── mod.rs
│   ├── auth.rs          # Login, register, logout
│   ├── dashboard.rs     # Dashboard page
│   ├── recipes.rs       # Recipe CRUD routes
│   ├── meal_plan.rs     # Meal planning routes
│   ├── shopping.rs      # Shopping list routes
│   └── profile.rs       # Profile and settings
├── templates/           # Askama templates (see section 7.1)
├── middleware/          # Axum middleware
│   ├── auth.rs          # JWT validation
│   ├── logging.rs       # Request logging (tracing)
│   └── error.rs         # Error handling
├── static/              # Static assets
│   ├── css/
│   │   └── tailwind.css
│   ├── js/
│   │   ├── twinspark.js
│   │   └── sw.js        # Service worker
│   ├── icons/
│   └── manifest.json
└── config/              # Configuration files
    └── default.toml     # Default config (can include all settings)
```

### 11.3 Key Integrations

**External Service Integrations:**

1. **SMTP (lettre)**:
   - Use case: Password reset emails, notification emails
   - Configuration: SMTP host, port, credentials (env vars)
   - Error handling: Queue failed emails for retry (future enhancement)

2. **Stripe (async-stripe)**:
   - Use case: Premium subscription payments
   - Endpoints: Create checkout session, handle webhooks
   - Security: Webhook signature verification

3. **MinIO (rust-s3)**:
   - Use case: Recipe image storage
   - Operations: Upload, retrieve, delete images
   - Bucket: `imkitchen-recipes` (created on startup if missing)

4. **Web Push (web-push)**:
   - Use case: Prep reminder push notifications
   - VAPID keys: Generated once, stored in config
   - Subscription management: Store browser push subscriptions in database

**Inter-Domain Communication**:
- No direct crate-to-crate calls
- Root binary orchestrates via commands/queries
- Domain events published via evento (subscriptions for cross-domain reactions)

**Example - Meal Plan Generated → Shopping List**:
1. User POST /plan/generate
2. Route handler calls `meal_planning::generate_meal_plan(cmd)`
3. `MealPlanGenerated` event written to evento stream
4. evento subscription triggers `shopping::generate_shopping_list(meal_plan_id)`
5. `ShoppingListGenerated` event written
6. Read models updated via subscriptions

## 12. Architecture Decision Records

### ADR-001: Event Sourcing with evento

**Context**: Need full audit trail of meal plan changes, support undo/temporal queries, enable CQRS.

**Decision**: Use evento for event sourcing with SQLite backend.

**Rationale**:
- Full event log enables audit trail (who changed what when)
- CQRS projections optimize read performance (materialized views)
- Temporal queries support future features (meal plan history, analytics)
- SQLite simplifies deployment (embedded database)

**Trade-offs**:
- Increased complexity vs. CRUD
- Eventual consistency for read models
- Cannot delete events (GDPR requires anonymization approach)

**Alternatives Considered**:
- Traditional CRUD with audit log table
- PostgreSQL with evento (deferred to scale-up phase)

---

### ADR-002: Server-Side Rendering (Askama + TwinSpark)

**Context**: Need mobile-first PWA with offline support, avoid frontend framework complexity.

**Decision**: Server-rendered HTML with Askama templates, TwinSpark for progressive enhancement.

**Rationale**:
- Type-safe templates compiled at build time (zero runtime errors)
- No frontend/backend split reduces cognitive load
- Progressive enhancement ensures functionality without JavaScript
- TwinSpark provides AJAX behaviors without React/Vue complexity

**Trade-offs**:
- Less interactive than SPA
- Full page loads on navigation (mitigated by TwinSpark)
- Limited client-side state management

**Alternatives Considered**:
- Next.js (TypeScript ecosystem, but heavy)
- SvelteKit (SSR + SPA, but separate frontend)
- HTMX (similar to TwinSpark, chose TwinSpark for Rust community)

---

### ADR-003: SQLite for MVP

**Context**: Need embedded database for simplified deployment, support event store + read models.

**Decision**: SQLite with WAL mode, shared volume for multi-pod deployment.

**Rationale**:
- Zero-ops database (no external service)
- Sufficient for 10K concurrent users (per NFRs)
- evento supports SQLite natively
- Embedded database simplifies Docker/K8s deployment

**Trade-offs**:
- Multi-writer scaling limited (requires shared filesystem or single writer)
- No built-in replication (backup strategy needed)

**Migration Path**: Migrate to PostgreSQL at scale (evento supports Postgres).

**Alternatives Considered**:
- PostgreSQL (deferred to scale-up phase)
- MySQL (less Rust ecosystem support)

---

### ADR-004: Docker/Kubernetes Deployment

**Context**: User preference for containerized deployment, need horizontal scaling.

**Decision**: Docker containers orchestrated by Kubernetes.

**Rationale**:
- Docker ensures consistent runtime environment
- Kubernetes provides auto-scaling, health checks, rolling deployments
- Cloud-agnostic (runs on any K8s cluster)

**Trade-offs**:
- Increased operational complexity vs. PaaS (Vercel, Heroku)
- Requires K8s knowledge for team

**Alternatives Considered**:
- Heroku (simpler, but vendor lock-in)
- AWS ECS (AWS-specific, less portable)

---

### ADR-005: Monolithic Rust Application

**Context**: Level 3 project with 5 epics, single team, avoid microservices complexity.

**Decision**: Monolithic Rust application with DDD bounded contexts (workspace crates).

**Rationale**:
- Simpler deployment (single binary)
- Shared types and evento infrastructure
- Easier to refactor into microservices later if needed
- DDD crates provide logical boundaries without network overhead

**Trade-offs**:
- Cannot scale individual domains independently
- Larger binary size

**Alternatives Considered**:
- Microservices (premature for MVP)
- Modular monolith with shared database (chosen approach)

---

### ADR-006: Freemium Model with 10 Recipe Limit

**Context**: Business model requires premium conversion, need friction for free tier.

**Decision**: Free tier limited to 10 recipes, premium tier unlimited.

**Rationale**:
- 10 recipes sufficient for trial (7+ recipes needed for meal planning)
- Friction encourages upgrade without blocking core functionality
- Enforced in domain logic (`user` aggregate)

**Trade-offs**:
- May frustrate free users
- Requires clear upgrade prompts in UI

**Alternatives Considered**:
- Feature-based limits (advanced scheduling, community features)
- Time-based trial (30 days)

## 13. Implementation Guidance

### 13.1 Development Workflow

**TDD Approach** (enforced per requirements):

1. **Write Test**: Define test for new feature/bugfix
2. **Run Test**: Verify test fails (red)
3. **Write Code**: Implement feature to pass test
4. **Run Test**: Verify test passes (green)
5. **Refactor**: Clean up code, maintain passing tests

**Test Pyramid**:
- Unit tests: Domain aggregate logic (evento commands/events)
- Integration tests: HTTP routes, database projections
- E2E tests: Full user flows (Playwright)

**Coverage Goal**: 80% code coverage (per NFRs).

### 13.2 File Organization

**Root Structure**:
```
imkitchen/
├── Cargo.toml                # Workspace manifest
├── Cargo.lock
├── src/                      # Root binary source
│   ├── main.rs
│   ├── server.rs
│   └── routes/
├── crates/                   # Domain crates
│   ├── shared_kernel/
│   ├── user/
│   ├── recipe/
│   └── ...
├── templates/                # Askama templates
├── static/                   # Static assets (CSS, JS, images)
├── migrations/               # SQLx migrations
├── tests/                    # Integration tests (root level)
├── e2e/                      # Playwright tests (TypeScript)
├── config/                   # Configuration files
├── .github/                  # GitHub Actions CI/CD
├── Dockerfile
├── k8s/                      # Kubernetes manifests
└── README.md
```

### 13.3 Naming Conventions

**Rust Code**:
- Crates: `snake_case` (e.g., `meal_planning`)
- Modules: `snake_case` (e.g., `read_model.rs`)
- Structs: `PascalCase` (e.g., `UserAggregate`, `CreateRecipeCommand`)
- Functions: `snake_case` (e.g., `generate_meal_plan`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RECIPE_LIMIT`)

**Events**:
- Past tense: `UserCreated`, `RecipeFavorited`, `MealPlanGenerated`

**Commands**:
- Imperative: `CreateRecipe`, `GenerateMealPlan`, `UpdateProfile`

**Templates**:
- Kebab-case: `recipe-card.html`, `meal-calendar.html`

**Routes**:
- Kebab-case: `/meal-plan`, `/shopping-list`, `/password-reset`

**CSS Classes** (Tailwind + custom):
- Kebab-case: `.recipe-card`, `.btn-primary`, `.meal-slot`

### 13.4 Best Practices

**Domain Logic**:
- All business rules in domain crate aggregates (NOT in route handlers)
- Route handlers are thin: validate, call domain, render template
- No direct database access in root binary (use domain crate queries)

**Error Handling**:
- Domain errors: Custom error types with `thiserror`
- Route errors: Map to HTTP status codes (422, 401, 404, 500)
- User-facing errors: Flash messages, inline form errors

**Testing**:
- Domain logic: Pure functions, easy to unit test
- HTTP routes: Integration tests with in-memory database
- E2E: Playwright tests for critical user flows (onboarding, meal planning)

**Security**:
- All passwords hashed with Argon2 before storage
- JWT secrets in environment variables (never committed)
- CSRF protection via SameSite cookies
- Input validation with `validator` crate
- SQL injection prevention via SQLx parameterized queries

**Performance**:
- Database indexes on all foreign keys and filter columns
- Read model queries (no event stream traversal for reads)
- HTTP caching headers on static assets
- Service worker caching for offline access

## 14. Proposed Source Tree

```
imkitchen/
├── Cargo.toml                          # Workspace root manifest
├── Cargo.lock
├── README.md
├── Dockerfile
├── .dockerignore
├── .gitignore
├── rustfmt.toml
├── clippy.toml
│
├── src/                                # Root binary source
│   ├── main.rs                         # CLI entry point (Clap)
│   ├── server.rs                       # Axum server setup
│   ├── config.rs                       # Configuration loading (config crate)
│   ├── error.rs                        # Global error types
│   │
│   ├── routes/                         # HTTP route handlers
│   │   ├── mod.rs
│   │   ├── auth.rs                     # Login, register, logout
│   │   ├── dashboard.rs                # Dashboard page
│   │   ├── recipes.rs                  # Recipe CRUD
│   │   ├── meal_plan.rs                # Meal planning
│   │   ├── shopping.rs                 # Shopping list
│   │   ├── discover.rs                 # Community recipes
│   │   ├── profile.rs                  # User profile settings
│   │   ├── health.rs                   # /health, /ready endpoints
│   │   └── static_files.rs             # Static asset serving
│   │
│   └── middleware/                     # Axum middleware
│       ├── mod.rs
│       ├── auth.rs                     # JWT validation
│       ├── logging.rs                  # Request logging (tracing)
│       └── error_handler.rs            # Error response rendering
│
├── templates/                          # Askama templates
│   ├── base.html                       # Base layout (nav, footer)
│   │
│   ├── components/                     # Reusable components
│   │   ├── button.html
│   │   ├── recipe-card.html
│   │   ├── meal-slot.html
│   │   ├── shopping-item.html
│   │   ├── form-field.html
│   │   ├── modal.html
│   │   ├── toast.html
│   │   └── nav-tabs.html
│   │
│   ├── pages/                          # Full page templates
│   │   ├── landing.html
│   │   ├── login.html
│   │   ├── register.html
│   │   ├── dashboard.html
│   │   ├── meal-calendar.html
│   │   ├── recipe-list.html
│   │   ├── recipe-detail.html
│   │   ├── recipe-form.html
│   │   ├── shopping-list.html
│   │   ├── community-feed.html
│   │   ├── profile.html
│   │   └── error.html
│   │
│   └── partials/                       # AJAX fragment templates
│       ├── recipe-grid.html
│       ├── meal-slot-content.html
│       └── shopping-category.html
│
├── static/                             # Static assets
│   ├── css/
│   │   ├── tailwind.css                # Tailwind entry point
│   │   └── output.css                  # Compiled output (gitignored)
│   ├── js/
│   │   ├── twinspark.js                # TwinSpark library
│   │   ├── sw.js                       # Service worker
│   │   └── app.js                      # Minimal app JS (if needed)
│   ├── icons/
│   │   ├── icon-192.png
│   │   ├── icon-512.png
│   │   └── badge-72.png
│   ├── images/
│   │   └── placeholder-recipe.jpg
│   ├── screenshots/                    # PWA screenshots
│   │   └── dashboard-mobile.png
│   └── manifest.json                   # PWA manifest
│
├── crates/                             # Domain workspace crates
│   ├── shared_kernel/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs                # UserId, RecipeId, etc.
│   │   │   ├── events.rs               # Cross-domain events
│   │   │   └── traits.rs               # Common traits
│   │   └── tests/
│   │
│   ├── user/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── aggregate.rs            # UserAggregate (evento)
│   │   │   ├── commands.rs             # RegisterUser, UpdateProfile
│   │   │   ├── events.rs               # UserCreated, ProfileUpdated
│   │   │   ├── read_model.rs           # User query projections
│   │   │   └── error.rs                # Domain-specific errors
│   │   └── tests/
│   │       ├── aggregate_tests.rs
│   │       └── read_model_tests.rs
│   │
│   ├── recipe/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── aggregate.rs            # RecipeAggregate
│   │   │   ├── commands.rs             # CreateRecipe, UpdateRecipe
│   │   │   ├── events.rs               # RecipeCreated, RecipeFavorited
│   │   │   ├── read_model.rs           # Recipe queries
│   │   │   └── error.rs
│   │   └── tests/
│   │
│   ├── meal_planning/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── aggregate.rs            # MealPlanAggregate
│   │   │   ├── commands.rs             # GenerateMealPlan, ReplaceMeal
│   │   │   ├── events.rs               # MealPlanGenerated, MealReplaced
│   │   │   ├── algorithm.rs            # Meal planning optimization
│   │   │   ├── rotation.rs             # Recipe rotation logic
│   │   │   ├── read_model.rs           # Meal plan queries
│   │   │   └── error.rs
│   │   └── tests/
│   │       ├── algorithm_tests.rs
│   │       └── rotation_tests.rs
│   │
│   ├── shopping/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── aggregate.rs            # ShoppingListAggregate
│   │   │   ├── commands.rs             # GenerateShoppingList, MarkItemCollected
│   │   │   ├── events.rs               # ShoppingListGenerated, ItemCollected
│   │   │   ├── aggregation.rs          # Ingredient aggregation logic
│   │   │   ├── categorization.rs       # Category assignment (produce, dairy, etc.)
│   │   │   ├── read_model.rs           # Shopping list queries
│   │   │   └── error.rs
│   │   └── tests/
│   │
│   └── notifications/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── aggregate.rs            # NotificationAggregate
│       │   ├── commands.rs             # ScheduleReminder, SendReminder
│       │   ├── events.rs               # ReminderScheduled, ReminderSent
│       │   ├── scheduler.rs            # Background worker (tokio tasks)
│       │   ├── push.rs                 # Web Push API integration (web-push crate)
│       │   ├── read_model.rs           # Notification queries
│       │   └── error.rs
│       └── tests/
│
├── migrations/                         # SQLx read model migrations (evento handles event store)
│   ├── 001_create_users_table.sql
│   ├── 002_create_recipes_table.sql
│   ├── 003_create_meal_plans_table.sql
│   ├── 004_create_shopping_lists_table.sql
│   ├── 005_create_ratings_table.sql
│   ├── 006_create_notifications_table.sql
│   └── 007_create_push_subscriptions_table.sql
│
├── tests/                              # Integration tests (root level)
│   ├── common/
│   │   ├── mod.rs
│   │   └── fixtures.rs                 # Test data fixtures
│   ├── auth_tests.rs                   # Auth flow integration tests
│   ├── recipe_tests.rs                 # Recipe CRUD integration tests
│   ├── meal_plan_tests.rs              # Meal planning integration tests
│   └── shopping_tests.rs               # Shopping list integration tests
│
├── e2e/                                # Playwright E2E tests (TypeScript)
│   ├── package.json
│   ├── playwright.config.ts
│   ├── tests/
│   │   ├── onboarding.spec.ts          # New user onboarding flow
│   │   ├── meal-planning.spec.ts       # Generate meal plan flow
│   │   ├── recipe-management.spec.ts   # Create/edit recipe flow
│   │   ├── shopping.spec.ts            # Shopping list generation flow
│   │   └── community.spec.ts           # Community recipe discovery
│   └── fixtures/
│       └── test-data.ts                # E2E test data
│
├── config/                             # Configuration files
│   └── default.toml                    # Default config (can include all settings)
│
├── k8s/                                # Kubernetes manifests
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── ingress.yaml
│   ├── pvc.yaml                        # Persistent volume claim (SQLite)
│   ├── secrets.yaml.example            # Example secrets (not checked in)
│   └── configmap.yaml                  # Environment config
│
├── .github/                            # GitHub Actions CI/CD
│   └── workflows/
│       ├── ci.yml                      # Lint, test, build
│       └── deploy.yml                  # Deploy to K8s
│
├── docs/                               # Documentation
│   ├── PRD.md                          # Product Requirements Document
│   ├── solution-architecture.md        # This document
│   ├── epics.md                        # Epic breakdown
│   ├── ux-specification.md             # UX/UI specification
│   └── tech-spec-epic-*.md             # Per-epic tech specs
│
├── scripts/                            # Utility scripts
│   ├── build-docker.sh                 # Docker build script
│   ├── deploy-k8s.sh                   # K8s deployment script
│   └── generate-vapid.sh               # Generate VAPID keys for Web Push
│
└── tailwind.config.js                  # Tailwind CSS configuration
```

**Critical Folders:**

- **`crates/`**: Domain business logic with evento aggregates, commands, events, read models. Each crate is a DDD bounded context.
- **`src/routes/`**: HTTP route handlers that orchestrate domain crates and render Askama templates. No business logic here.
- **`templates/`**: Server-rendered HTML templates (Askama) with components, pages, and partials for AJAX.
- **`migrations/`**: SQLx database migrations for event store and read model schemas.
- **`tests/` + `e2e/`**: Comprehensive test coverage (unit, integration, E2E) enforcing TDD.

## 15. Testing Strategy

### 15.1 Unit Tests

**Domain Logic** (crates/*/tests/):
- Test aggregate command handlers (pure functions)
- Test event application (state transitions)
- Test business rules (recipe limit, rotation logic, etc.)
- Mock evento dependencies (in-memory event store for tests)

**Example - Recipe Aggregate Test**:
```rust
#[test]
fn test_create_recipe_enforces_limit() {
    let user_id = UserId::new();
    let aggregate = UserAggregate::new(user_id, Tier::Free);

    // Create 10 recipes (free tier limit)
    for i in 0..10 {
        let cmd = CreateRecipeCommand { title: format!("Recipe {}", i), ... };
        assert!(aggregate.create_recipe(cmd).is_ok());
    }

    // 11th recipe should fail
    let cmd = CreateRecipeCommand { title: "Recipe 11".to_string(), ... };
    let result = aggregate.create_recipe(cmd);
    assert!(matches!(result, Err(UserError::RecipeLimitReached)));
}
```

### 15.2 Integration Tests

**HTTP Routes** (tests/*.rs):
- Test full request/response cycle
- Use in-memory SQLite database
- Test authentication flows
- Test form validation and error rendering

**Example - Create Recipe Integration Test**:
```rust
#[tokio::test]
async fn test_create_recipe_endpoint() {
    let app = test_app().await;
    let client = reqwest::Client::new();

    // Login to get auth cookie
    let login_resp = client.post(&format!("{}/login", app.url))
        .form(&[("email", "test@example.com"), ("password", "password123")])
        .send()
        .await.unwrap();
    let cookie = login_resp.cookies().find(|c| c.name() == "auth_token").unwrap();

    // Create recipe
    let resp = client.post(&format!("{}/recipes", app.url))
        .header("Cookie", format!("auth_token={}", cookie.value()))
        .form(&[
            ("title", "Chicken Tikka Masala"),
            ("prep_time_min", "20"),
            ("cook_time_min", "30"),
            // ...
        ])
        .send()
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER); // 303 redirect
    assert!(resp.headers().get("Location").unwrap().to_str().unwrap().starts_with("/recipes/"));
}
```

### 15.3 E2E Tests

**Playwright Tests** (e2e/tests/*.spec.ts):
- Test critical user flows end-to-end
- Use real browser (Chromium, Firefox, WebKit)
- Test PWA functionality (offline mode, notifications)

**Example - Meal Planning E2E Test**:
```typescript
test('generate meal plan flow', async ({ page }) => {
  // 1. Login
  await page.goto('/login');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.click('button[type="submit"]');

  // 2. Navigate to dashboard
  await page.waitForURL('/dashboard');

  // 3. Click "Generate Meal Plan"
  await page.click('a[href="/plan/generate"]');

  // 4. Wait for meal plan calendar
  await page.waitForSelector('.meal-calendar');

  // 5. Verify 7 days rendered
  const days = await page.$$('.calendar-day');
  expect(days.length).toBe(7);

  // 6. Verify meals assigned
  const meals = await page.$$('.meal-slot');
  expect(meals.length).toBeGreaterThan(0);
});
```

### 15.4 Coverage Goals

**Coverage Target**: 80% code coverage (per NFRs)

**Coverage Tools**:
- Rust: `cargo-tarpaulin` (unit + integration tests)
- Playwright: Built-in code coverage for E2E

**CI Enforcement**: GitHub Actions fails build if coverage drops below 80%.

**Coverage Report**: HTML report generated, uploaded as artifact.

## 16. DevOps and CI/CD

### 16.1 CI Pipeline (GitHub Actions)

**Workflow** (.github/workflows/ci.yml):

```yaml
name: CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo tarpaulin --out Xml --all-features
      - uses: codecov/codecov-action@v3

  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release
      - run: ./target/release/imkitchen serve &
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - working-directory: e2e
        run: |
          npm ci
          npx playwright install --with-deps
          npx playwright test

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/build-push-action@v5
        with:
          context: .
          push: false
          tags: imkitchen:latest
```

### 16.2 CD Pipeline (GitHub Actions)

**Workflow** (.github/workflows/deploy.yml):

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: your-registry/imkitchen:${{ github.sha }}

      - name: Deploy to Kubernetes
        uses: azure/k8s-deploy@v4
        with:
          manifests: |
            k8s/deployment.yaml
            k8s/service.yaml
          images: your-registry/imkitchen:${{ github.sha }}
          kubectl-version: latest
```

### 16.3 Monitoring and Observability

**OpenTelemetry Integration**:
- Tracing: All HTTP requests, domain command executions
- Metrics: Request rates, latencies, error rates
- Logs: Structured JSON logs (tracing crate)

**Instrumentation Example**:
```rust
#[tracing::instrument(skip(user_id))]
async fn generate_meal_plan(user_id: UserId, cmd: GenerateMealPlanCommand) -> Result<MealPlanId, Error> {
    tracing::info!("Generating meal plan for user {}", user_id);

    // Domain logic...

    tracing::info!("Meal plan generated successfully");
    Ok(meal_plan_id)
}
```

**Collector**: OpenTelemetry Collector (OTEL_EXPORTER_OTLP_ENDPOINT env var)

**Backends** (future):
- Jaeger (tracing)
- Prometheus (metrics)
- Loki (logs)

### 16.4 Health Checks

**Health Endpoints**:
- `GET /health`: Liveness probe (returns 200 if process alive)
- `GET /ready`: Readiness probe (returns 200 if database connected, evento initialized)

**Kubernetes Configuration**:
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 30

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
```

## 17. Security

### 17.1 Authentication Security

**Password Hashing**: Argon2 (OWASP-recommended)
- Cost: Default secure parameters (memory=65536, iterations=3, parallelism=4)
- Salt: Randomly generated per password (argon2 crate handles)

**JWT Security**:
- Secret: 32-byte random key (generated once, stored in K8s secret)
- Algorithm: HS256
- Expiration: 7 days
- Cookie: HTTP-only, Secure (HTTPS only), SameSite=Lax

**CSRF Protection**: SameSite=Lax cookie attribute prevents CSRF attacks.

### 17.2 Input Validation

**Server-Side Validation** (validator crate):
- All form inputs validated before domain commands
- SQL injection: Prevented via SQLx parameterized queries
- XSS: Prevented via Askama auto-escaping

**Example Validation**:
```rust
#[derive(Deserialize, Validate)]
struct CreateRecipeForm {
    #[validate(length(min = 3, max = 200))]
    title: String,

    #[validate(range(min = 1, max = 999))]
    prep_time_min: u32,

    #[validate(email)]
    creator_email: Option<String>,
}
```

### 17.3 OWASP Compliance

**OWASP Top 10 Mitigations**:

1. **A01:2021 – Broken Access Control**:
   - Auth middleware enforces JWT validation on all protected routes
   - Authorization checks in domain logic (recipe ownership, freemium limits)

2. **A02:2021 – Cryptographic Failures**:
   - TLS 1.3 enforced (HTTPS only in production)
   - Sensitive data encrypted at rest (SQLite encryption via SQLCipher - future)
   - JWT secrets in environment variables (never committed)

3. **A03:2021 – Injection**:
   - SQLx parameterized queries prevent SQL injection
   - Askama auto-escaping prevents XSS
   - No user input passed to shell commands

4. **A04:2021 – Insecure Design**:
   - Event sourcing provides audit trail
   - Domain-driven design enforces business rules
   - TDD ensures security requirements tested

5. **A05:2021 – Security Misconfiguration**:
   - Security headers: CSP, X-Frame-Options, X-Content-Type-Options
   - Default deny (auth middleware required for protected routes)

6. **A06:2021 – Vulnerable and Outdated Components**:
   - Dependabot enabled (automated dependency updates)
   - Cargo audit in CI pipeline (fails on known vulnerabilities)

7. **A07:2021 – Identification and Authentication Failures**:
   - Argon2 password hashing (not bcrypt or plaintext)
   - JWT expiration enforced
   - No session fixation (stateless JWTs)

8. **A08:2021 – Software and Data Integrity Failures**:
   - Evento event store immutability (events never deleted/modified)
   - Docker image signed (future enhancement)

9. **A09:2021 – Security Logging and Monitoring Failures**:
   - OpenTelemetry tracing logs all auth attempts, domain commands
   - Failed login attempts logged
   - Rate limiting (future enhancement)

10. **A10:2021 – Server-Side Request Forgery**:
    - No user-controlled URLs in external requests
    - Stripe/MinIO endpoints hardcoded or validated

### 17.4 Data Privacy (GDPR)

**User Rights**:
- **Right to Access**: GET /profile returns user data as JSON (future export endpoint)
- **Right to Deletion**: DELETE /profile anonymizes user (events retained with anonymized IDs)
- **Right to Portability**: Data export as JSON (future enhancement)

**Anonymization Strategy**:
- Events cannot be deleted (event sourcing constraint)
- User deletion replaces PII with anonymized values (`user_<hash>`)
- Recipe/meal plan data retained for community (anonymized author)

### 17.5 Security Headers

**HTTP Headers** (Axum middleware):
```rust
// Content-Security-Policy
"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self';"

// Other headers
"X-Frame-Options: DENY"
"X-Content-Type-Options: nosniff"
"Referrer-Policy: no-referrer"
"Permissions-Policy: geolocation=(), microphone=(), camera=()"
```

---

## Specialist Sections

### Specialist Placeholder: Advanced DevOps

**Status**: Complex - Requires DevOps specialist agent

**Scope**: This architecture covers CI/CD basics (GitHub Actions, Docker, K8s deployment). Advanced concerns require DevOps specialist:

- **Infrastructure as Code**: Terraform/Pulumi for K8s cluster provisioning
- **Multi-Environment Strategy**: Dev, staging, production with namespaces
- **Secrets Management**: HashiCorp Vault integration
- **Database Backups**: Automated SQLite backups to S3, restore procedures
- **Blue-Green Deployments**: Zero-downtime deployment strategy
- **Horizontal Pod Autoscaling**: HPA configuration for traffic spikes
- **Monitoring Dashboards**: Grafana dashboards for metrics/logs

**Next Steps**: Engage DevOps specialist agent after this architecture is approved.

---

### Specialist Placeholder: Advanced Security

**Status**: Simple - Handled inline

**Security Architecture** (covered above):
- OWASP Top 10 mitigations
- JWT authentication with HTTP-only cookies
- Argon2 password hashing
- Input validation and SQL injection prevention
- CSRF protection via SameSite cookies
- GDPR compliance (data export/deletion)
- Security headers (CSP, X-Frame-Options, etc.)

**Future Enhancements** (post-MVP):
- Rate limiting (login attempts, API endpoints)
- 2FA for premium users
- SQLCipher for database encryption at rest
- Penetration testing (quarterly)
- SIEM integration

---

### Specialist Placeholder: Advanced Testing

**Status**: Simple - Handled inline

**Testing Strategy** (covered above):
- Unit tests: Domain aggregate logic (evento commands/events)
- Integration tests: HTTP routes, database projections
- E2E tests: Playwright for critical user flows
- Coverage: 80% goal (cargo-tarpaulin)

**Test Types**:
- TDD enforced (write test first, then code)
- Property-based testing for meal planning algorithm (future)
- Load testing (future): k6 or Locust for performance benchmarks
- Accessibility testing: axe-core integration in Playwright

---

## Appendix

### Architecture Patterns Summary

**Event Sourcing**: All state changes recorded as immutable events (evento).

**CQRS**: Commands write events, queries read from materialized views (read models).

**DDD**: Bounded contexts (domain crates) with aggregates, commands, events.

**Server-Side Rendering**: Askama templates compiled at build time, no client-side framework.

**Progressive Enhancement**: TwinSpark for AJAX behaviors, degrades gracefully without JS.

**Offline-First PWA**: Service workers cache pages/assets for offline access.

---

_Generated using BMad Method Solution Architecture workflow_
_Architecture Level: Expert (concise, decision-focused)_
_Date: 2025-10-11_
