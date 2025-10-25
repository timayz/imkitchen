# Solution Architecture - imkitchen

**Project:** imkitchen | **Date:** 2025-10-11 | **Author:** Jonathan
**Style:** Event-Sourced Monolith with SSR | **Repo:** Rust Monorepo

## Executive Summary

Event-sourced meal planning PWA in Rust with SSR. evento (SQLite) for ES/CQRS, Axum HTTP server, Askama templates, TwinSpark progressive enhancement, Docker/K8s deployment.

**Characteristics:** Server-rendered HTML, event sourcing, CQRS, progressive enhancement, offline-first PWA, Docker/K8s.

## 1. Technology Stack

| Category | Technology | Version | Justification |
|----------|------------|---------|---------------|
| **Language** | Rust | 1.90+ | Type safety, performance, zero-cost abstractions |
| **HTTP Server** | Axum | 0.8+ | Async, Tower middleware, ergonomics |
| **Templates** | Askama | 0.14+ | Compile-time type-safe, zero runtime overhead |
| **Event Sourcing** | evento | 1.3+ | SQLite ES/CQRS, aggregates, subscriptions |
| **Database** | SQLite | 3.45+ | Embedded, event store + read models |
| **Query Builder** | SQLx | 0.8+ | Async, compile-time SQL verification DISABLED |
| **CLI** | Clap | 4.5+ | Derive macros, subcommands |
| **Config** | config | 0.15+ | Dynamic path, env var overrides |
| **Validation** | validator | 0.20+ | Derive-based validation |
| **i18n** | rust-i18n | 3.1.5 | Multi-language (initial English) |
| **Observability** | OpenTelemetry | 0.31+ | Tracing, metrics, logs |
| **CSS** | Tailwind CSS | 4.1+ | Utility-first, customizable |
| **Progressive Enhancement** | TwinSpark | latest | Declarative HTML attributes |
| **E2E Testing** | Playwright | 1.56+ | Cross-browser automation |
| **JWT** | jsonwebtoken | 9.3+ | Cookie auth tokens |
| **Password** | argon2 | 0.5+ | OWASP-recommended |
| **Email** | lettre | 0.11+ | Async SMTP |
| **HTTP Client** | reqwest | 0.12+ | Async HTTP |
| **Stripe** | async-stripe | 0.39+ | Payments |
| **MinIO** | rust-s3 | 0.34+ | S3 images |
| **Web Push** | web-push | 0.10+ | VAPID notifications |
| **Serialization** | serde | 1.0+ | JSON |
| **Async** | tokio | 1.40+ | Runtime |
| **Errors** | thiserror | 1.0+ | Custom error types |
| **Logging** | tracing | 0.1+ | Structured logging |
| **Service Worker** | Workbox | 7.1+ | PWA offline |
| **Container** | Docker | 25.0+ | Runtime |
| **Orchestration** | Kubernetes | 1.30+ | Scaling, deployment |

## 2. Architecture Pattern

**Event-Sourced Monolith**: Single Rust binary, domain crates per bounded context, evento event store, CQRS read models.

**Request Flow:** HTTP → Axum → Domain crate command/query → evento/read model → Askama template → HTML + TwinSpark → Browser

### Meal Planning Model v2.0

**Structure:** 7 days × 3 courses (appetizer, main_course, dessert) = 21 assignments
**Recipe Types:** Required `recipe_type` field (appetizer|main_course|dessert)
**Algorithm:** Course-type matching, rotation logic, user constraints

### Page Routing

```
GET  /                       Landing
GET/POST /login              Auth
GET/POST /register           Registration
GET/POST /password-reset     Reset flow
GET  /dashboard              Today's meals + prep
GET  /plan                   Week calendar
POST /plan/generate          Generate plan
POST /plan/meal/:id/replace  Replace meal
GET  /recipes                Library with filters
GET/POST /recipes/new        Create recipe
GET  /recipes/:id            Detail
GET/PUT /recipes/:id/edit    Edit
DELETE /recipes/:id          Delete
POST /recipes/:id/favorite   Toggle favorite
POST /recipes/:id/share      Share to community
GET  /discover               Community feed (public, SEO)
GET  /discover/:id           Community recipe (public, SEO)
POST /discover/:id/add       Add to library (auth)
POST /discover/:id/rate      Rate/review (auth)
GET  /shopping               Shopping list
POST /shopping/:id/check     Mark collected
GET/POST /notifications      Settings
GET/PUT /profile             Profile
GET/POST /subscription       Upgrade
GET  /static/*               Assets
GET  /sw.js                  Service worker
GET  /manifest.json          PWA manifest
```

### Data Fetching (CQRS)

**Commands:** Form POST → Domain command → evento event stream → Redirect (PRG)
**Queries:** Route handler → Page-specific read models (SQLite) → Askama template

**Page-Specific Read Models:**
- Dashboard: `dashboard_meals`, `dashboard_prep_tasks`, `dashboard_metrics`
- Calendar: `calendar_view`
- Recipe Library: `recipe_list`, `recipe_filter_counts`, `recipe_collections`
- Recipe Detail: `recipe_detail`, `recipe_ratings`
- Shopping: `shopping_list_view`, `shopping_list_summary`

**Form Consistency:** Use `evento::load` for edit forms (trusted state), read models for display-only.

## 3. Data Architecture

### Database Schema

**Single SQLite Database:**
1. **Event Store** (evento-managed): `events` table (automatic)
2. **Read Models** (manual migrations):
   - `users` - Auth and profile
   - `recipes` - Recipe data with `recipe_type` index
   - `meal_plans`, `meal_assignments` - Plans with `course_type` (appetizer|main_course|dessert)
   - `shopping_lists`, `shopping_list_items` - Shopping data
   - `ratings` - Recipe reviews
   - Page-specific: `recipe_list`, `recipe_detail`, `recipe_filter_counts`, `calendar_view`, etc.

**Week Convention:** Monday start (ISO 8601), next-week-only planning.

### Key Schemas

```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  dietary_restrictions TEXT,
  household_size INTEGER,
  skill_level TEXT,
  weeknight_availability TEXT,
  tier TEXT NOT NULL,
  recipe_limit INTEGER
);

CREATE TABLE recipes (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  ingredients TEXT NOT NULL,
  instructions TEXT NOT NULL,
  prep_time_min INTEGER,
  cook_time_min INTEGER,
  advance_prep_hours INTEGER,
  serving_size INTEGER,
  recipe_type TEXT NOT NULL,
  is_favorite BOOLEAN DEFAULT FALSE,
  is_shared BOOLEAN DEFAULT FALSE,
  complexity TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX idx_recipes_type ON recipes(recipe_type);

CREATE TABLE meal_assignments (
  id TEXT PRIMARY KEY,
  meal_plan_id TEXT NOT NULL,
  date TEXT NOT NULL,
  course_type TEXT NOT NULL,
  recipe_id TEXT NOT NULL,
  prep_required BOOLEAN
);
```

### Projections (evento subscriptions)

**Pattern:** Domain events → Multiple page-specific read models

```rust
// Example: RecipeCreated updates 3 read models
#[evento::handler(Recipe)]
async fn project_recipe_to_list_view(...) { /* Insert into recipe_list */ }

#[evento::handler(Recipe)]
async fn project_recipe_to_detail_view(...) { /* Insert into recipe_detail */ }

#[evento::handler(Recipe)]
async fn update_recipe_filter_counts(...) { /* Update recipe_filter_counts */ }

// Subscription setup
evento::subscribe("recipe-list-projections")
    .aggregator::<Recipe>()
    .handler(project_recipe_to_list_view)
    .run(&executor).await?;
```

**Mapping:**
- `UserCreated` → `users`
- `RecipeCreated` → `recipe_list`, `recipe_detail`, `recipe_filter_counts`
- `RecipeFavorited` → Update `recipe_list`, `recipe_detail`, `recipe_filter_counts`
- `MealPlanGenerated` → `calendar_view`, `dashboard_meals`, `shopping_list_view`
- `MealReplaced` → Update `calendar_view`, recalc `shopping_list_view`

### Migrations

**Two Systems:**
1. evento migrations (automatic): `evento::sql_migrator::new_migrator::<sqlx::Sqlite>().run(...)`
2. SQLx migrations (manual): `sqlx::migrate!("./migrations").run(pool)`

## 4. API Design

**No REST API** - Server-rendered HTML with forms.

**Form Mutations:** `application/x-www-form-urlencoded`, validator crate validation, PRG pattern.

**TwinSpark AJAX:**
```html
<form ts-req="/plan/meal/123/replace"
      ts-req-method="POST"
      ts-target="#meal-slot-123"
      ts-swap="outerHTML">
  <select name="recipe_id">...</select>
  <button type="submit">Replace</button>
</form>
```

**Response:** 302 redirect (success), 422 with errors (validation), 401 (auth), 404 (not found).

## 5. Authentication

**JWT Cookie-Based:**
1. Registration: Argon2 hash → UserCreated event
2. Login: Verify password → JWT → HTTP-only cookie
3. Session: JWT cookie → Middleware validates → Extract user ID
4. Logout: Clear cookie

**JWT Claims:**
```rust
struct Claims {
    sub: String,      // User ID
    email: String,
    tier: String,     // free|premium
    exp: u64,
    iat: u64,
}
```

**Config:** HS256, 32-byte secret (env var), 7-day expiry, HTTP-only, Secure, SameSite=Lax.

**Authorization:** Free tier 10 recipe limit (domain logic), premium unlimited.

## 6. State Management

**Server State:** evento aggregates (User, Recipe, MealPlan, ShoppingList, Notification).
**Client State:** Minimal - native form state, TwinSpark attributes.
**Caching:** HTTP headers (static assets 1yr, public pages 5min), read models ARE cache, Workbox service worker (stale-while-revalidate).

## 7. UI/UX

### Template Structure

```
templates/
├── base.html
├── components/
│   ├── button.html, recipe-card.html, meal-slot.html, shopping-item.html
│   ├── form-field.html, modal.html, toast.html, nav-tabs.html
├── pages/
│   ├── landing.html, login.html, dashboard.html, meal-calendar.html
│   ├── recipe-list.html, recipe-detail.html, recipe-form.html
│   ├── shopping-list.html, community-feed.html, profile.html
└── partials/
    ├── recipe-grid.html, meal-slot-content.html, shopping-category.html
```

### Styling

**Tailwind CSS:** Custom config with design tokens, utility classes, PurgeCSS for production.

**Breakpoints:** Mobile (<768px), Tablet (768-1023px), Desktop (1024px+), mobile-first approach.

**Accessibility:** WCAG 2.1 AA - semantic HTML, ARIA, keyboard nav, 4.5:1 contrast (7:1 kitchen mode), alt text, form labels, 44px touch targets.

## 8. Performance

**SSR:** Compile-time templates (zero runtime), indexed read models, SQLx connection pool.

**Caching:** Static 1yr, public 5min, private no-cache.

**Assets:** Tailwind purged/minified, TwinSpark 5KB, deferred JS, lazy images, WebP with JPEG fallback.

**PWA:** Workbox precache static assets, stale-while-revalidate HTML, cache-first images, background sync mutations.

**Database:** SQLite with WAL mode, optimized PRAGMAs:
```rust
PRAGMA journal_mode = WAL
PRAGMA busy_timeout = 5000
PRAGMA synchronous = NORMAL
PRAGMA cache_size = -20000
PRAGMA foreign_keys = true
PRAGMA temp_store = memory
```

**Connection Pools:**
- Write pool: 1 connection (prevents SQLITE_BUSY)
- Read pool: Multiple (concurrent reads)
- Use `BEGIN IMMEDIATE` for write transactions

## 9. SEO & PWA

**SEO:** `/discover` routes public (no auth), meta tags, Open Graph, Schema.org Recipe JSON-LD, sitemap, robots.txt.

**manifest.json:**
```json
{
  "name": "imkitchen - Intelligent Meal Planning",
  "short_name": "imkitchen",
  "start_url": "/dashboard",
  "display": "standalone",
  "theme_color": "#2563eb",
  "icons": [{"src": "/static/icons/icon-192.png", "sizes": "192x192"}]
}
```

**Web Push:** VAPID-based, store subscriptions in DB, send via `web-push` crate, notification payload with actions.

## 10. Deployment

### Docker

```dockerfile
FROM rust:1.90 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3
COPY --from=builder /app/target/release/imkitchen /usr/local/bin/
COPY static /app/static
COPY templates /app/templates
WORKDIR /app
ENV DATABASE_URL=sqlite:///data/imkitchen.db PORT=8080
EXPOSE 8080
CMD ["imkitchen", "serve"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: imkitchen
        image: imkitchen:latest
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
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: imkitchen-data
```

**Scaling:** Stateless app, SQLite on NFS (MVP), migrate to PostgreSQL at scale.

### Configuration

**Dynamic Path:** `--config /path/to/config.toml` or `CONFIG_PATH` env var.

**Priority:** 1. Env vars (override), 2. Config file, 3. Defaults

**Example config.toml:**
```toml
[server]
port = 3000
host = "0.0.0.0"

[database]
url = "sqlite:///data/imkitchen.db"

[jwt]
secret = "your-secret"  # Or override via IMKITCHEN__JWT__SECRET
expiration_days = 7

[smtp]
host = "smtp.example.com"
port = 587
username = "user"
password = "pass"  # Or IMKITCHEN__SMTP__PASSWORD

[stripe]
secret_key = "sk_test_..."

[minio]
endpoint = "http://minio:9000"

[vapid]
public_key = "..."
private_key = "..."
```

## 11. Component Overview

### Domain Crates

```
crates/
├── shared_kernel/       # Types, events, traits
├── user/                # UserAggregate, RegisterUser, UserCreated
├── recipe/              # RecipeAggregate, CreateRecipe, RecipeCreated
├── meal_planning/       # MealPlanAggregate, algorithm, rotation
├── shopping/            # ShoppingListAggregate, aggregation, categorization
└── notifications/       # NotificationAggregate, scheduler, push
```

### Root Binary

```
src/
├── main.rs              # CLI (Clap)
├── server.rs            # Axum setup
├── config.rs, db.rs, error.rs
├── routes/              # auth, dashboard, recipes, meal_plan, shopping, discover, profile
├── middleware/          # auth, logging, error_handler
templates/, static/, migrations/, tests/, e2e/, config/, k8s/
```

### Key Integrations

- **SMTP (lettre):** Password reset emails
- **Stripe (async-stripe):** Checkout sessions, webhooks
- **MinIO (rust-s3):** Recipe image storage
- **Web Push (web-push):** Prep reminders

**Inter-Domain:** No direct crate calls, root binary orchestrates, evento subscriptions for cross-domain reactions.

## 12. Architecture Decisions

**ADR-001: evento for ES/CQRS** - Audit trail, temporal queries, read model optimization. Trade-off: complexity, eventual consistency.

**ADR-002: SSR (Askama + TwinSpark)** - Type safety, no frontend split, progressive enhancement. Trade-off: less interactive than SPA.

**ADR-003: SQLite** - Zero-ops, 10K user capacity, evento native. Trade-off: multi-writer scaling requires NFS. Migration path: PostgreSQL.

**ADR-004: Docker/K8s** - Consistent runtime, auto-scaling, cloud-agnostic. Trade-off: operational complexity.

**ADR-005: Monolith** - Simpler deployment, shared infrastructure, DDD boundaries. Trade-off: cannot scale domains independently.

**ADR-006: Freemium 10 Recipe Limit** - Trial sufficient, upgrade friction. Enforced in domain logic.

## 13. Testing Strategy

**TDD Enforced:** Write test → Red → Implement → Green → Refactor

**Test Pyramid:**
- **Unit:** Domain aggregates (evento commands/events)
- **Integration:** HTTP routes, projections (use `unsafe_oneshot` for sync)
- **E2E:** Playwright critical flows

**Projection Testing:**
```rust
evento::subscribe("recipe-projections")
    .aggregator::<Recipe>()
    .handler(project_recipe_to_list_view)
    .unsafe_oneshot(&executor)  // Sync processing for tests
    .await?;
```

**Coverage:** 80% goal (cargo-tarpaulin), CI enforced.

## 14. DevOps & CI/CD

**CI (.github/workflows/ci.yml):**
- Lint: `cargo fmt --check`, `cargo clippy`
- Test: `cargo test`, `cargo tarpaulin` (80% coverage)
- E2E: Build release, run server, Playwright tests
- Build: Docker image

**CD (.github/workflows/deploy.yml):**
- Build/push Docker to registry
- Deploy to K8s with rolling updates

**Observability:** OpenTelemetry tracing/metrics/logs, collector endpoint via env var.

**Health Checks:** `/health` (liveness), `/ready` (readiness with DB check).

## 15. Security

**Auth:** Argon2 hashing, JWT HS256 7-day expiry, HTTP-only Secure SameSite=Lax cookies.

**Validation:** Server-side validator crate, SQLx parameterized queries (no injection), Askama auto-escape (no XSS).

**OWASP Top 10:**
- A01: Auth middleware, domain authorization checks
- A02: TLS 1.3, JWT secrets in env vars
- A03: Parameterized queries, auto-escaping
- A04: Event sourcing audit, DDD, TDD
- A05: Security headers (CSP, X-Frame-Options, X-Content-Type-Options)
- A06: Dependabot, cargo audit in CI
- A07: Argon2, JWT expiration
- A08: Immutable event store
- A09: OpenTelemetry logging, failed login logs
- A10: No user-controlled URLs

**GDPR:** Export (future), deletion = anonymization (events retained with `user_<hash>`).

**Headers:**
```
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; ...
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
Referrer-Policy: no-referrer
```

## 16. Source Tree

```
imkitchen/
├── Cargo.toml, Cargo.lock, Dockerfile, README.md
├── src/
│   ├── main.rs, server.rs, config.rs, db.rs, error.rs
│   ├── routes/ (auth, dashboard, recipes, meal_plan, shopping, discover, profile, health)
│   └── middleware/ (auth, logging, error_handler)
├── templates/
│   ├── base.html
│   ├── components/ (button, recipe-card, meal-slot, form-field, modal, toast, nav-tabs)
│   ├── pages/ (landing, login, dashboard, meal-calendar, recipe-list/detail/form, shopping, community, profile)
│   └── partials/ (recipe-grid, meal-slot-content, shopping-category)
├── static/
│   ├── css/tailwind.css, js/ (twinspark, sw, app), icons/, manifest.json
├── crates/
│   ├── shared_kernel/ (types, events, traits)
│   ├── user/ (aggregate, commands, events, read_model)
│   ├── recipe/ (aggregate, commands, events, read_model)
│   ├── meal_planning/ (aggregate, algorithm, rotation, read_model)
│   ├── shopping/ (aggregate, aggregation, categorization, read_model)
│   └── notifications/ (aggregate, scheduler, push, read_model)
├── migrations/ (001-007 SQL files)
├── tests/ (integration tests)
├── e2e/ (Playwright tests)
├── config/default.toml
├── k8s/ (deployment, service, ingress, pvc, secrets.example, configmap)
├── .github/workflows/ (ci.yml, deploy.yml)
└── docs/ (PRD, this doc, epics, ux-spec, tech-specs)
```

## 17. Implementation Guidance

**Best Practices:**
- Business rules in domain aggregates, not handlers
- Route handlers: validate → domain call → render
- Errors: thiserror domain errors, map to HTTP status
- All passwords Argon2, JWT secrets in env vars
- Database indexes on FKs and filters
- HTTP caching on static assets
- Service worker for offline

**Naming:**
- Crates/modules: `snake_case`
- Structs: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Events: Past tense (`UserCreated`)
- Commands: Imperative (`CreateRecipe`)
- Templates/routes/CSS: `kebab-case`

---

_Architecture Level: Expert (concise, decision-focused)_
_Date: 2025-10-11_
