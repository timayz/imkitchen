# Decision Architecture

## Executive Summary

imkitchen is an event-sourced CQRS application written in Rust on Axum, with evento 2 for event sourcing and SQLite for storage. A single binary (`src/`) mounts a set of web crates (`web/*`) over a set of domain crates (`crates/*`): identity, billing, notification, audience, and the `core` crate that holds the three product aggregates (`recipe`, `shopping`, `contact`). Pages are server-rendered with Askama and made reactive with twinspark.js partial swaps.

The product has no calendar. There is no meal plan, no weeks, no day slots, no "today" dashboard and no scheduler. The central object is the user's **recipe list**, modelled by the `Shopping` aggregate (one per user, aggregate id = user id). Generation fills the list from the user's candidate pool (own recipes plus saved community favorites), the kitchen page walks the list recipe by recipe with a per-recipe cooking cursor (`RecipeStatus`), and the groceries page shows the merged, household-scaled ingredient list of everything in the list. All list commands are synchronous: the kitchen and groceries pages read the aggregate directly, so there is no read-model lag after a command.

Persisted bitcode shapes are frozen by `events.lock`; this is why legacy `MealPlan` event declarations still exist in `crates/types/src/mealplan.rs` even though nothing writes them anymore.

## Project Initialization

The workspace already exists. New work follows the structure below; there is no starter template. Use the `Makefile` targets (`make dev`, `make migrate`, `make reset`, `make css`, `make lint`, `make test`) and `config/dev.local.toml` for local overrides.

## Decision Summary

Versions are the workspace versions pinned in the root `Cargo.toml`.

| Category | Decision | Version | Affects | Rationale |
| -------- | -------- | ------- | ------- | --------- |
| Language | Rust, edition 2024 | stable | All | Performance, safety |
| Web Framework | Axum | 0.8 | All | Async, Tower integration, `{param}` routes |
| Templating | Askama | 0.16 | All web crates | Type-safe server-side templates |
| UI Reactivity | twinspark.js | vendored in `static/js` | All web crates | Server-driven partial swaps, minimal JS |
| Event Sourcing | evento (`sqlite`, `rw` features) | 2.0.0-alpha.29 | All domain crates | Aggregates, projections, subscriptions, snapshots |
| Shape lock | evento-lock | 2.0.0-alpha.29 | All domain crates | Freezes persisted bitcode shapes (`events.lock`) |
| Database | SQLite via sqlx | 0.9 | All | Single file, read pool + single-connection write pool |
| Migrations | sqlx_migrator + sea-query | 0.19 / 1.0.1 | `crates/db`, `crates/audience` | Code-defined migrations, one module per table |
| Serialization of events/snapshots | bitcode | 0.6 | All domain crates | Compact positional encoding (hence the shape lock) |
| Styling | Tailwind CSS CLI | see `tailwind.css` / `Makefile` | All web crates | Utility-first, compiled to `static/css/main.css` |
| Validation | validator | 0.21 | Commands | Input validation on command inputs |
| CLI | clap | 4.6 | Binary | `serve`, `migrate`, `reset` |
| Configuration | config (TOML) | 0.15 | Binary, `web/shared` | `config/default.toml` + `--config` override |
| Auth | jsonwebtoken + argon2 | 11.0 / 0.6 | `web/shared`, `crates/identity` | JWT cookie, Argon2 password hashes |
| Billing | async-stripe | 1.0.0-rc.6 | `crates/billing`, `web/settings`, `web/public` | Premium subscriptions |
| Scheduled jobs | tokio-cron-scheduler | 0.15 | `crates/billing` | Subscription renewals only (no meal-plan scheduler) |
| Email | lettre | 0.11 | `crates/notification` | Configurable SMTP |
| i18n | rust-i18n | 4.2 | `web/shared`, `crates/notification` | `locales/en.json`, `locales/fr.json` |
| Static assets | rust-embed | 8.12 | `web/shared` | Assets embedded in the binary, served at `/static` |
| Observability | tracing + tracing-subscriber | 0.1 / 0.3 | All | Structured (optionally JSON) logs |
| Random picking | rand | 0.10 | `crates/core/shopping` | Shuffling the candidate pool |
| E2E tests | Playwright (nix dev shell) | see `flake.nix` | All | Browser tests and store screenshots |

## Project Structure

```
imkitchen/
├── Cargo.toml                       # Workspace: crates/*, web/*, root binary `imkitchen`
├── config/
│   ├── default.toml                 # Committed defaults
│   └── dev.local.toml               # Local overrides (used by `make dev`)
├── events.lock                      # Frozen persisted shapes (evento-lock)
├── src/                             # Binary: CLI + server wiring only
│   ├── main.rs                      # clap CLI: serve | migrate | reset
│   ├── lib.rs / db.rs               # SQLite pool builders (read pool, write pool, CLI pool)
│   └── cli/
│       ├── server.rs                # Builds evento executor, starts every subscription, mounts routers
│       └── migrate.rs               # Runs crates/db + crates/audience migrators
├── crates/
│   ├── types/                       # evento aggregates and events (imkitchen-types)
│   │   └── src/{recipe,favorite,recipe_share,shopping,meal_preferences,contact,user_profile}.rs
│   │       mealplan.rs              # Legacy MealPlan declarations: frozen, never written
│   ├── core/                        # imkitchen-core: State, Core, Error and the product modules
│   │   └── src/
│   │       ├── lib.rs               # State { executor, read_db, write_db }, Core { recipe, shopping, contact }
│   │       ├── command.rs           # core::Error / core::Result + user!/not_found!/forbidden!/server! macros
│   │       ├── recipe/              # Recipe aggregate (root/), favorites, read models (query/), sagas
│   │       ├── shopping/            # The user's recipe list
│   │       │   ├── root/            # Shopping projection + commands (generate, add, remove, status, toggle)
│   │       │   │   ├── pick.rs      # Randomizer over the candidate pool
│   │       │   │   ├── merge.rs     # Ingredient merge + household scaling
│   │       │   │   └── state.rs     # ShoppingState: synchronous read of the list
│   │       │   ├── pool.rs          # "mealplan-command" subscription → meal_plan_recipe candidate pool
│   │       │   └── subscription.rs  # "shopping" subscription → shopping_recipe (ingredients, household size)
│   │       └── contact/             # Contact aggregate + admin read models
│   ├── identity/                    # User aggregate, login/admin views, password reset, meal preferences, profile
│   ├── billing/                     # Subscription + Invoice aggregates, Stripe, renewal scheduler
│   ├── notification/                # Email subscriptions (user, contact, billing) + lettre service
│   ├── audience/                    # First-party visit measurement on its own evento instance/DB
│   └── db/                          # sqlx_migrator migrations m0001..m0015, one sea-query module per table
├── web/
│   ├── shared/                      # AppState, Config, auth extractors, Template extractor, assets, middleware
│   ├── kitchen/                     # `/` and `/kitchen/...`: list, generation, cooking screens
│   ├── grocery/                     # `/groceries`, `/groceries/toggle`; `/menu` legacy redirects
│   ├── recipe/                      # `/recipes/...`, `/r/{slug}`, `/cooks/{username}`
│   ├── settings/                    # `/settings/{general,billing,account}`, `/invoices/{id}`
│   ├── public/                      # Landing/legal pages, auth, contact, upgrade, PWA assets, health
│   ├── admin/                       # `/admin/...` users, invoices, audience, contact inbox, batch import
│   └── demo/                        # `/demo/...` read-only tour on fixture data
├── templates/                       # Askama templates shared by every web crate
│   ├── _base.html, _user.html, _public.html, _admin.html, _settings.html
│   ├── index.html, kitchen.html, cooking.html, groceries.html, onboarding-*.html
│   ├── recipes-*.html, settings-*.html, admin-*.html, login/register/reset-password*.html
│   └── partials/                    # twinspark fragments (kitchen-dish, kitchen-generate-modal, cooking-screen, ...)
├── static/
│   ├── css/main.css                 # Compiled Tailwind output (from ./tailwind.css)
│   ├── js/{twinspark.js,sw-register.js,sw-source.js,pwa-install.js}
│   └── icons/, screenshots/         # PWA icons and store screenshots
├── locales/{en,fr}.json
├── tests/
│   ├── events_lock.rs               # Fails when a persisted shape changes
│   ├── e2e/, fixtures/, screenshots/  # Playwright
├── helm/, Dockerfile, compose.yml   # Deployment
└── docs/
```

## Epic to Architecture Mapping

| Area | Crates | Key Components | Database Tables |
|------|--------|----------------|-----------------|
| Foundation & identity | `crates/identity`, `web/public`, `web/settings`, `web/shared` | `User` aggregate, login/admin views, password reset, JWT cookie auth, meal preferences, user profile | `user`, `user_login`, `user_admin`, `user_global_stat`, `notification_recipient` |
| Recipes & import | `crates/core::recipe`, `web/recipe`, `web/admin` | `Recipe` aggregate, favorites, share sagas, thumbnails, FTS, ZIP batch import | `recipe_user` (+FTS), `recipe_owner`, `recipe_thumbnail`, `recipe_user_stat`, `origin_framing` |
| Recipe list & generation | `crates/core::shopping`, `web/kitchen` | `Shopping` aggregate, `generate`/`add_recipe`/`remove_recipe`, randomizer, ingredient merge | `meal_plan_recipe` (candidate pool), `shopping_recipe` |
| Kitchen & groceries | `web/kitchen`, `web/grocery` | Cooking cursor (`RecipeStatus`), step navigation, aisle-grouped groceries, check/uncheck | none beyond the `Shopping` aggregate and the two tables above |
| Community & premium | `crates/core::recipe`, `crates/billing`, `web/public`, `web/settings` | Share to community, saved favorites, Stripe subscription, invoices, renewal scheduler | `recipe_user.is_shared`, `user_subscription`, `user_invoice_user` |
| Notifications, public pages & admin | `crates/notification`, `crates/audience`, `web/public`, `web/admin` | Email subscriptions, contact inbox, audience stats, sitemap, PWA assets | `contact_admin` (+FTS), `contact_global_stat`, `audience_daily_stat` (audience DB) |

## Technology Stack Details

### Core Technologies

**Runtime & Language:**
- Rust, edition 2024, single workspace
- Tokio 1.52 async runtime

**Web Server:**
- Axum 0.8 with `macros` and `multipart`
- axum-extra 0.12 (cookies, forms, query, typed headers)
- tower-http 0.7: request body limits, brotli/gzip compression, tracing
- Custom middleware in `web/shared/src/middleware`: cache-control headers and HTML minification (minify-html, oxc, lightningcss)

**Event Sourcing & CQRS:**
- evento 2.0.0-alpha.29 with the `sqlite` and `rw` features
- One `evento::Evento` executor over an `RwSqlite` pair (read pool + write pool) with a 100 ms stability margin
- Aggregates are declared with `#[evento::aggregate]` enums in `crates/types`; projections with `#[evento::projection(name = "...")]`; event handlers with `#[evento::handler]`; read-model subscriptions with `#[evento::subscription]`
- Snapshots are bitcode-encoded; every projection has an explicit `.revision(n)`
- evento-lock freezes event, nested type and view shapes in `events.lock`

**Data Layer:**
- SQLite via sqlx 0.9 (runtime queries, no compile-time macros), statements built with sea-query 1.0.1 and bound with sea-query-sqlx
- One application database file (`sqlite:imkitchen.db` by default), opened twice: a read-only pool (`database.max_connections`) and a single-connection write pool in WAL mode with `wal_autocheckpoint = 0` so Litestream owns checkpointing
- Optional separate audience database (`[audience] database_url`) with its own evento instance
- Migrations are Rust code in `crates/db` (sqlx_migrator 0.19): evento's own schema migrations first, then `m0001`..`m0015`

**Templating & UI:**
- Askama 0.16; one `templates/` directory shared by all web crates
- `Template` extractor (`web/shared/src/template.rs`) injects the preferred language and the demo flag into every render; rust-i18n filters translate strings from `locales/`
- twinspark.js for partial swaps (`ts-req`, `ts-target`, `ts-swap`, `ts-trigger`)
- Tailwind CSS compiled by the CLI (`make css`) into `static/css/main.css`
- Assets embedded with rust-embed and served under `/static` by `AssetsService`
- PWA: `/manifest.json`, `/sw.js` (Workbox `injectManifest` from `static/js/sw-source.js`), install prompt script

**Authentication & Security:**
- JWT in an HTTP-only cookie (`jsonwebtoken` 11, `[jwt]` config: audience, issuer, secret, `expiration_days`, 14 by default)
- Argon2 0.6 password hashes
- Extractors in `web/shared/src/auth.rs`: `AuthToken`, `AuthUser` (user-facing pages), `RequireChef`, `AuthAdmin`
- Roles: `User`, `Chef`, `Admin`; account state `Active` / `Suspended`

**Validation & Serialization:**
- validator 0.21 (`#[derive(Validate)]` on command inputs such as `GenerateList`)
- serde / serde_json for forms, JSON columns and the `/groceries/toggle` JSON body
- bitcode 0.6 for events, snapshots and BLOB columns such as `shopping_recipe.ingredients`

**Billing:**
- async-stripe 1.0.0-rc.6 (customers, payment intents, setup intents, webhooks)
- tokio-cron-scheduler 0.15 runs the subscription renewal job every minute, only when `[premium]` is configured

**Email & i18n:**
- lettre 0.11 over SMTP (`[email]` config)
- rust-i18n 4.2 with `en` and `fr` locales

**Observability:**
- tracing with per-handler `#[tracing::instrument]` spans
- tracing-subscriber with env filter; JSON output when `monitoring.log_json = true`

**Testing:**
- `cargo test --workspace` (unit tests live next to the code, for example `merge.rs`, `pick.rs`, kitchen `next_status`, grocery `balanced_split`)
- `tests/events_lock.rs` guards persisted shapes
- Playwright for e2e and screenshot capture (`npm test`, `npm run test:e2e`)

### Integration Points

**Write Path (Commands):**
```
HTTP request → Axum handler (web/*) → Module command (crates/*)
  → load projection from evento → validate → .write()?.event(&E).requested_by(user).commit(executor)
  → event appended to the SQLite event log (write pool)
```

**Read Path (Queries):**
```
HTTP request → Axum handler → read model query (read pool, sea-query)
  → Askama template → HTML (full page or twinspark partial)
```

**Aggregate Read Path (no lag):**
```
HTTP request → Axum handler → Module::load / Module::state (evento projection + snapshot)
  → Askama template
```
Used wherever the page must reflect the command that just ran: the kitchen (`shopping.state()`), the groceries page, `meal_preferences.load()`, `identity.find_account()`.

**Projection Path:**
```
Event log → evento subscription (key such as "recipe-query", "shopping", "mealplan-command")
  → handler writes a read-model table (write pool)
```

**Event Flow:**
1. A command loads the aggregate's projection, checks invariants and commits one event.
2. evento stores the event in the application database.
3. Every subscription started in `src/cli/server.rs` receives the event in order and updates its read model, sends an email, or runs a saga.
4. Handlers read either the read-model tables (lists, search, candidate pool) or the projection directly (list state, preferences).

## Implementation Patterns

### Aggregate and Event Pattern

Events are declared once, in `crates/types`, as enum variants of an `#[evento::aggregate]`:

```rust
// crates/types/src/shopping.rs
#[evento::aggregate]
pub enum Shopping {
    Checked { ingredient: String },
    Unchecked { ingredient: String },
    RecipeAdded { recipe_id: String, recipe_ids: Vec<String>, ingredients: Vec<Ingredient> },
    RecipeRemoved { recipe_id: String, recipe_ids: Vec<String>, ingredients: Vec<Ingredient> },
    ListGenerated { recipe_ids: Vec<String>, ingredients: Vec<Ingredient> },
    RecipeStatusChanged { recipe_id: String, status: RecipeStatus },
    // Legacy variants (Generated, RecipeSetGenerated) stay declared, never written.
}
```

**Rules:**
- Never change the shape of an existing variant or of a type nested in one: bitcode is positional. Add a new variant instead (see `MealPreferences::RecipeTypesChanged`).
- Legacy variants that are no longer written stay declared with a doc comment saying so.
- Run `EVENTO_LOCK=update cargo test -p imkitchen --test events_lock` after adding an event, type or view, and commit `events.lock`.

### Projection Pattern

```rust
// crates/core/src/shopping/root/mod.rs
#[evento::projection(name = "imkitchen-core/shopping/Shopping", Encode, Decode)]
pub struct Shopping {
    pub user_id: String,
    pub checked: HashSet<String>,
    pub ingredients: HashSet<String>,
    pub recipes: Vec<String>,
    pub statuses: HashMap<String, RecipeStatus>,
    pub generated_at: u64,
}

pub fn create_projection<E: Executor>() -> Projection<E, Shopping> {
    Projection::new::<shopping::Shopping>()
        .revision(3)
        .handler(handle_checked())
        .handler(handle_list_generated())
        // ...
        .strict()
}

#[evento::handler]
async fn handle_list_generated(event: Event<ListGenerated>, data: &mut Shopping) -> anyhow::Result<()> {
    data.user_id = event.metadata.requested_by()?;
    data.generated_at = event.timestamp;
    data.recipes = event.data.recipe_ids;
    data.ingredients = event.data.ingredients.iter().map(|i| i.key()).collect();
    data.checked.clear();
    data.statuses.clear();
    Ok(())
}
```

**Rules:**
- Bump `.revision(n)` whenever the struct's field set or order changes; old snapshots are then rebuilt from events instead of failing to decode.
- `.strict()` makes an unhandled event a hard error, so every variant of the aggregate (including legacy ones) needs a handler.
- `impl ProjectionAggregate` returns the aggregate id; for `Shopping` and `MealPreferences` that is the user id.

### Command Pattern

Commands are methods on a per-aggregate `Module<E: Executor>` that derefs to `core::State` (`executor`, `read_db`, `write_db`). The input struct is the first argument, the requesting user id the last.

```rust
// crates/core/src/shopping/root/status.rs
pub struct ChangeRecipeStatus { pub recipe_id: String, pub status: RecipeStatus }

impl<E: Executor> super::Module<E> {
    pub async fn change_recipe_status(&self, input: ChangeRecipeStatus, request_by: impl Into<String>) -> crate::Result<()> {
        let request_by = request_by.into();
        let shopping = match self.load(&request_by).await? {
            Some(shopping) if shopping.recipes.contains(&input.recipe_id) => shopping,
            _ => crate::not_found!("recipe"),
        };
        shopping.write()?
            .event(&RecipeStatusChanged { recipe_id: input.recipe_id, status: input.status })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?;
        Ok(())
    }
}
```

**Rules:**
- Commands return `imkitchen_core::Result` and raise domain failures with the `user!`, `not_found!`, `forbidden!` and `server!` macros; the web layer maps these to toasts, 404 and 403 pages.
- A command decides from the projection it just loaded (optimistic concurrency through `aggregate_version`). It may read read-model tables that are maintained by recipe events (`shopping_recipe`, `meal_plan_recipe`) because those are inputs, not invariants.
- Commands are synchronous from the caller's point of view: when `commit` returns, `load()` and `state()` already reflect the event.

### Subscription (Read Model) Pattern

```rust
// crates/core/src/shopping/subscription.rs
pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("shopping")
        .handler(handle_recipe_created())
        .handler(handle_recipe_imported())
        .handler(handle_recipe_deleted())
        .handler(handle_recipe_ingredients_changed())
        .handler(handle_recipe_basic_information_changed())
}

#[evento::subscription]
async fn handle_recipe_ingredients_changed<E: Executor>(
    context: &Context<'_, E>,
    event: Event<imkitchen_types::recipe::IngredientsChanged>,
) -> anyhow::Result<()> {
    let pool = context.extract::<sqlx::SqlitePool>();
    // sea-query UPDATE shopping_recipe SET ingredients = bitcode(...) WHERE id = event.aggregate_id
    Ok(())
}
```

**Rules:**
- One subscription key per read model. Keys are persisted cursors: renaming one restarts the replay from the beginning, which is why the candidate-pool subscription keeps its historical key `"mealplan-command"`.
- Handlers must be idempotent (events are replayed on rebuild) and use `event.timestamp` for time columns.
- Every subscription is started in `src/cli/server.rs` and shut down gracefully on SIGTERM.
- Dropping a read model is a migration that drops the table and deletes the subscriber row (see `m0015::ForgetSubscribers`).

### Web Handler Pattern

```rust
// web/kitchen/src/lib.rs
pub async fn remove_recipe_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let preferences = imkitchen_web_shared::try_response!(anyhow:
        app.identity.meal_preferences.load(&user.id), template);
    imkitchen_web_shared::try_response!(
        app.core.shopping.remove_recipe(&id, preferences.household_size, &user.id), template);
    Redirect::to("/").into_response()
}
```

**Rules:**
- Handlers take the `Template` extractor and one of the auth extractors; `try_response!` turns a `core::Error` into a `partials/toast-error.html` response and `try_page_response!` into a full error page.
- Mutations that change the whole page redirect to `/`; the triggering element uses `ts-target="body"` so twinspark swaps the redirected page in. Mutations that change one region return a partial (`partials/*.html`).
- Each web crate exposes `routes() -> axum::Router<AppState>`; the binary merges them flat (no `nest`).

## Consistency Rules

### Naming Conventions

**Files & Modules:**
- `snake_case.rs`; one command per file under `root/` (`generate.rs`, `remove.rs`), one read model per file under `query/`
- Table modules in `crates/db/src/<table>.rs` with the `sea_query::Iden` enum and a `mN` submodule per migration that touches the table

**Routes (Axum 0.8):**
- Path parameters use `{id}` (never `:id`)
- Page: `GET /thing`; modal or partial: `GET /thing/modal`; action: `POST /thing/action`; status polling: `GET /thing/{id}/status`

**Database:**
- Table names are `snake_case` singular (`user`, `recipe_user`, `shopping_recipe`, `meal_plan_recipe`)
- Columns `snake_case`; ids are 26-char ULID strings; timestamps are `INTEGER` unix seconds from `event.timestamp`
- Read-model rows that mirror an aggregate carry `cursor` and `aggregate_version`

**Rust:**
- Aggregates/events: `PascalCase` enum variants in `crates/types`
- Commands: `verb_noun` methods (`add_recipe`, `change_recipe_status`), input structs `VerbNoun` (`GenerateList`, `ChangeRecipeStatus`, `ToggleInput`)
- Subscription keys: kebab-case strings (`"recipe-query"`, `"shopping"`, `"notification-billing"`)

### Code Organization

**Domain crates:**
- `crates/types` only declares aggregates, events and shared value types; no I/O
- `crates/core` groups each aggregate as `root/` (projection + commands), `query/` (read models), optional `saga/` and `favorite/`
- `identity`, `billing`, `notification`, `audience` follow the same `Module(State)` shape

**Web crates:**
- `web/shared` owns `AppState`, config, auth, the `Template` extractor, i18n, assets and middleware
- Feature crates own their handlers and `askama::Template` structs; templates live in the shared `templates/` tree
- `web/demo` renders the same templates from `fixtures.rs` with every mutation replaced by the sign-up modal

**Tests:**
- Unit tests next to the code; `tests/events_lock.rs` at the workspace root; Playwright under `tests/e2e`

### Error Handling

- Domain: `imkitchen_core::Error { Validate, Forbidden, NotFound, User, Server }` with `From` impls for sqlx, evento, argon2 and time errors
- Web: `try_response!` renders `partials/toast-error.html` (sent with `ts-swap: skip` when no fallback template is given) with a generic message for `Server`, "Forbidden" for `Forbidden`, and the error text for `User`, `Validate` and `NotFound`; `try_page_response!` renders the 404 template for a missing resource and the error pages otherwise
- Subscriptions: return `anyhow::Error`; evento logs and retries, so handlers must be safe to re-run

### Logging Strategy

- `#[tracing::instrument(skip_all, fields(user = user.id))]` on handlers
- `tracing::info!` for lifecycle (startup, subscriptions, shutdown), `warn!` for recoverable issues, `error!` for failed jobs
- Log level and JSON output come from `[monitoring]` in the config

## Data Architecture

### Databases

**Application database (`[database] url`, default `sqlite:imkitchen.db`):**
- Holds the evento event log and snapshots and every read-model table
- Opened by the server as a read-only pool and a single-connection write pool
- The write pool disables automatic WAL checkpoints so Litestream can replicate; `migrate` runs a `TRUNCATE` checkpoint and a gated `VACUUM`

**Audience database (`[audience] database_url`, optional):**
- Separate evento instance for landing-page visit beacons and the `audience_daily_stat` rollup
- Migrated automatically on `serve`

There is no separate validation database. Uniqueness checks (email, username) run against the `user` table, which has unique indexes and is written by the identity module on registration.

### Migrations

`crates/db/src/lib.rs` chains evento's schema migrations and `m0001`..`m0015`. Notable steps:

| Migration | Effect |
|-----------|--------|
| m0001 | Creates every table: user views, contact views, `recipe_user` (+FTS), `recipe_thumbnail`, `recipe_user_stat`, `meal_plan_recipe`, `shopping_recipe`, `notification_recipient`, and the since-dropped `meal_plan_slot`, `shopping_list`, `shopping_slot` |
| m0005 | Adds `recipe_user.slug`, creates `recipe_owner` for the share saga |
| m0006 | Creates `origin_framing`; adds `shopping_recipe.household_size` backfilled from `recipe_user` |
| m0013 | Adds `aggregate_version` to the mirrored read models |
| m0015 | Drops `meal_plan_slot`, `shopping_slot`, `shopping_list` and forgets the `mealplan-slot` and `shopping-list` subscribers (calendar removal) |

### Core Tables

**Recipe list inputs (maintained from `Recipe` and `Favorite` events):**

`meal_plan_recipe` — candidate pool for generation, one row per (recipe, user): the user's own recipes plus community recipes they saved. Maintained by `shopping::pool::subscription()` (`"mealplan-command"`).

| Column | Notes |
|--------|-------|
| `id`, `user_id` | Composite primary key |
| `recipe_type` | `Appetizer`, `MainCourse`, `Dessert`, `Accompaniment`, `Beverage`, `Condiment` |
| `name`, `prep_time`, `cook_time`, `advance_prep` | Display fields |
| `accepts_accompaniment` | Only mains that accept one trigger accompaniment picking |
| `dietary_restrictions` | JSON array, filtered with `json_each` |

`shopping_recipe` — ingredients and authored household size per recipe. Maintained by `shopping::subscription()` (`"shopping"`).

| Column | Notes |
|--------|-------|
| `id` | Recipe id, primary key |
| `user_id` | Owner |
| `ingredients` | BLOB, bitcode `Vec<Ingredient>` |
| `household_size` | Authored servings, default 4 |

**Recipe read models:** `recipe_user` (full recipe view with slug, times, ingredients, instructions, dietary restrictions, `is_shared`, thumbnail version, difficulty score, blur placeholder, plus an FTS table), `recipe_owner`, `recipe_thumbnail`, `recipe_user_stat`, `origin_framing` (per-domain iframe verdict).

**Identity read models:** `user` (email, password hash, username, role, state), `user_login`, `user_admin` (+FTS), `user_global_stat`, `notification_recipient`.

**Billing read models:** `user_subscription`, `user_invoice_user`.

**Contact read models:** `contact_admin` (+FTS), `contact_global_stat`.

### Aggregates and Snapshotted Views

| Aggregate (events in `crates/types`) | Projection (view) | Aggregate id |
|--------------------------------------|-------------------|--------------|
| `Shopping` | `imkitchen-core/shopping/Shopping` rev 3 | user id |
| `Recipe` | `imkitchen-core/recipe/Recipe` rev 3 | recipe id |
| `RecipeShare` | `imkitchen-core/recipe/RecipeShareState` rev 1 | user id |
| `Favorite` | `imkitchen-core/recipe/favorite/Favorite` rev 1 | `evento::hash_ids([recipe_id, user_id])` |
| `MealPreferences` | `imkitchen-identity/meal_preferences/MealPreferences` rev 2 | user id |
| `User` | `imkitchen-identity/User` rev 2 | user id |
| `UserProfile`, `Password` | `.../UserProfile` rev 1, `.../Password` rev 1 | user id / reset id |
| `Subscription`, `Invoice` | `imkitchen-billing/subscription/Subscription` rev 1 | user id / invoice id |
| `Contact` | `imkitchen-core/contact/Contact` rev 1 | contact id |
| `MealPlan` (legacy) | none; `DaysGenerated` and `SlotRecipeStatusChanged` stay declared for decoding | — |

### The Recipe List (`Shopping`)

- `recipes: Vec<String>` is the list in order; `statuses` holds each recipe's `RecipeStatus { Idle, Cooking(u8), Completed }` (missing means `Idle`); `ingredients` holds the merged ingredient keys and `checked` the ones ticked on the groceries page.
- `RecipeAdded` / `RecipeRemoved` / `ListGenerated` carry the full new `recipe_ids` and merged `ingredients`, so a projection rebuild never has to recompute them.
- `ListGenerated` replaces the list and clears `checked` and `statuses`. `RecipeRemoved` drops the recipe's status. `Checked`/`Unchecked` toggle one ingredient key.
- `Module::state(user_id, household_size)` loads the projection and recomputes the merged, household-scaled ingredient list from `shopping_recipe` synchronously; this is what the kitchen and groceries pages render.

## Data Flows

### Generation

1. `POST /kitchen/generate` with `count` (1..=30 mains) from the generate modal.
2. The handler loads `MealPreferences` (household size, dietary restrictions, cuisine variety weight, enabled optional recipe types) and calls `shopping.generate(GenerateList { count, household_size, randomize })`.
3. `pick::random` selects up to 35 random `MainCourse` rows from `meal_plan_recipe` matching every dietary restriction, shuffles them and truncates to `ceil(len * cuisine_variety_weight)`; `generate` then truncates to `count`. Without `randomize` a plain `sample_recipes` (up to 7) is used. No mains at all is a user error.
4. For each enabled optional type, in the order appetizer, accompaniment, dessert, beverage, condiment, `optional_pool` picks random recipes of that type and `generate` keeps up to `ceil(count / 2)`. Accompaniments are skipped unless a picked main accepts one. Ids are deduplicated; a flat list never repeats a recipe.
5. `merge::merge_ingredients` loads each recipe's `(household_size, ingredients)` from `shopping_recipe`, scales quantities with `scale_quantity` (serving target = `max(recipe size, household size)`, so quantities never scale below the authored size, rounding up) and sums duplicates by `Ingredient::key()`.
6. One `ListGenerated { recipe_ids, ingredients }` event is committed; checks and statuses reset.
7. The handler redirects to `/`; twinspark swaps the fresh kitchen page into `<body>`. No polling, because the page reads the aggregate.

### Adding and Removing Recipes

- `POST /recipes/{id}/add-to-shopping` → `shopping.add_recipe(id, household_size, user)`: idempotent, requires a `shopping_recipe` row (so shared recipes can be added), commits `RecipeAdded` with the recomputed set.
- `POST /kitchen/recipe/{id}/remove` → `shopping.remove_recipe(...)`: commits `RecipeRemoved`, then redirects to `/` so the kitchen can move its focus to the next recipe.

### Kitchen and Cooking Status

1. `GET /` with a session renders the kitchen from `shopping.state()`. An empty list renders onboarding (`onboarding-recipe.html` when the pool has no mains, `onboarding-menu.html` otherwise, using `sample_recipes` counts).
2. Entries are built from `recipe.filter_by_ids` plus the list order and statuses. The focused recipe is the first non-completed entry; "Prep ahead" lists other non-completed entries with an advance-prep note.
3. `POST /kitchen/{recipe_id}/select-dish` returns the `partials/kitchen-dish.html` fragment for another recipe in the list.
4. `GET /kitchen/{recipe_id}/cook` renders the full cooking screen; `POST /kitchen/{recipe_id}/step/{prev|next}` computes the next `RecipeStatus` in memory (`Idle` = ingredient screen, `Cooking(n)` = instruction `n`, `Completed` = last step), commits `RecipeStatusChanged`, and returns `partials/cooking-screen.html` from the new status without re-reading the aggregate.
5. Imported recipes whose origin allows framing show the original page in an iframe (`origin_framing` cache maintained by the `recipe-saga-embeddable` subscription); an imported recipe with no parsed steps and a non-embeddable origin redirects to the original.

### Groceries

1. `GET /groceries` calls `shopping.state(user, household_size)` and groups `ingredients` by `IngredientCategory` into aisle sections with per-aisle and overall progress; aisles are split into two balanced desktop columns.
2. `POST /groceries/toggle` with JSON `{ "name": "<ingredient key>" }` calls `shopping.toggle`, which commits `Checked` or `Unchecked` depending on the current `checked` set.
3. Checks are kept per ingredient key; keys that leave the list after a removal are simply ignored when counting progress and are cleared by the next generation.

## API Contracts

### HTTP Routes

All routes are merged flat into one Axum router in `src/cli/server.rs`. `{param}` is an Axum 0.8 path parameter.

**Kitchen (`web/kitchen`):**
- `GET /` - Landing page for visitors; kitchen (recipe list + focused recipe) or onboarding for signed-in users
- `GET /kitchen/generate` - Generate modal (partial)
- `POST /kitchen/generate` - Replace the list (`count` form field), redirect to `/`
- `POST /kitchen/recipe/{id}/remove` - Remove a recipe from the list, redirect to `/`
- `POST /kitchen/{recipe_id}/select-dish` - Focus another recipe (partial `kitchen-dish`)
- `GET /kitchen/{recipe_id}/cook` - Cooking screen
- `POST /kitchen/{recipe_id}/step/{direction}` - Move the cooking cursor (`prev` | `next`), returns the cooking screen partial
- `GET /kitchen/{legacy}` - Permanent redirect to `/` for calendar-era bookmarks

**Groceries (`web/grocery`):**
- `GET /groceries` - Aisle-grouped ingredient list for the whole recipe list
- `POST /groceries/toggle` - JSON `{ "name" }`, check/uncheck one ingredient
- `GET /menu`, `GET /menu/{legacy}` - Permanent redirect to `/`

**Recipes (`web/recipe`):**
- `GET /recipes` - User's recipes (search, filters)
- `POST /recipes/create` - Create a draft recipe
- `GET|POST /recipes/{id}/edit` - Edit form / save
- `GET /recipes/_edit/ingredient-row`, `GET /recipes/_edit/instruction-row` - Form row partials
- `GET|POST /recipes/import` - Import form / upload; `GET /recipes/import/{id}/status` - Progress partial
- `GET /recipes/{id}/delete` (modal), `POST /recipes/{id}/delete`, `GET /recipes/{id}/delete/status`
- `POST /recipes/{id}/save`, `POST /recipes/{id}/unsave` - Favorite a community recipe
- `POST /recipes/{id}/add-to-shopping` - Add to the recipe list
- `GET /recipes/{id}/share-to-community`, `GET /recipes/{id}/make-private`
- `POST /recipes/share-all`, `POST /recipes/make-all-private`
- `POST /recipes/{id}/thumbnail`, `GET /recipes/{id}/thumbnail/{device}/image.webp`
- `GET /recipes/{id}` - Redirect to the slug URL
- `GET /r/{slug}` - Recipe detail; `GET /r/{slug}/similar` - Similar recipes partial
- `GET /cooks/{username}` - A chef's public recipes

**Settings (`web/settings`):**
- `GET|POST /settings/general`, `POST /settings/general/profile`, `POST /settings/general/set-username`
- `GET /settings/billing`, `POST /settings/billing/check`, `POST /settings/billing/payment-method`
- `GET|POST /settings/billing/cancel`, `GET|POST /settings/billing/update-payment`
- `GET|POST /settings/account`
- `GET /invoices/{id}`

**Public (`web/public`):**
- `GET|POST /register`, `GET|POST /login`, `GET /logout`
- `GET|POST /reset-password`, `GET|POST /reset-password/new/{id}`
- `GET|POST /contact`
- `GET|POST /upgrade`, `GET /upgrade/modal`, `GET /upgrade/order-summary`
- `GET /about`, `/help`, `/terms`, `/policy`, `/legal`
- `POST /audience/visit` - Visit beacon (only when `[audience]` is configured)
- `GET /manifest.json`, `/sw.js`, `/robots.txt`, `/sitemap.xml`
- `GET /health`, `/ready`, `/_test-error` - Mounted with the read pool as state

**Admin (`web/admin`, `AuthAdmin`):**
- `GET /admin/users`, `GET /admin/users/{id}/edit`, `POST /admin/users/{id}`, `POST /admin/users/{id}/suspend`, `/activate`, `/toggle-premium`
- `GET /admin/invoices`, `GET /admin/invoices/{id}`
- `GET /admin/audience`
- `GET /admin/contact`, `POST /admin/contact/{id}/mark-read-and-reply`, `/resolve`, `/reopen`
- `GET /admin/recipes/import`, `POST /admin/recipes/import` (ZIP, 50 MB body limit), `GET /admin/recipes/import/{id}/status`

**Demo (`web/demo`, no login):**
- `GET /demo`, `/demo/kitchen`, `/demo/kitchen/{recipe_id}/cook`, `/demo/menu`, `/demo/recipes`, `/demo/recipes/{id}`, `/demo/r/{slug}`, `/demo/r/{slug}/similar`, `/demo/cooks/{username}`, `/demo/groceries`, `/demo/signup`

### Response Format

- Every route returns HTML rendered by Askama, either a full page or a `partials/*.html` fragment for twinspark.
- Full-page mutations redirect to the page to show; the triggering element carries `ts-target="body"`.
- Errors render toast partials or the 403/404/500 templates; there is no JSON API apart from the JSON request body of `/groceries/toggle` and the audience beacon.
- The global request body limit is 1 MB; the admin ZIP upload route is merged after that layer with its own 50 MB limit.

## Security Architecture

### Authentication

- JWT stored in an HTTP-only cookie built by `web/shared/src/auth.rs::build_cookie`; lifetime is `[jwt] expiration_days` (14 by default) and the kitchen page re-issues the cookie on each render (sliding window)
- Passwords hashed with Argon2 (`argon2` 0.6)
- `AuthUser` loads the login view (`user_login`) and refuses suspended accounts; `AuthAdmin` requires the `Admin` role; `RequireChef` requires `Login::is_chef()` and renders the 403 page otherwise
- Password reset is its own `Password` aggregate with an expiring reset id, emailed by the `notification-user` subscription

### Input Validation

- Command inputs derive `validator::Validate` (for example `GenerateList.count` in 1..=30)
- Forms are parsed by axum-extra `Form`; all SQL goes through sea-query bound parameters, with `sqlx::AssertSqlSafe` only wrapping builder output
- Askama escapes by default; the `askama.toml` escaper config treats `json`/`js` templates as text

### Uploads

- Recipe thumbnails are resized to WebP per device size by the recipe module
- Admin batch import accepts a ZIP up to 50 MB and tracks progress in memory (`AdminImportJobs`), polled by the status route

### Privacy

- Ad consent is an event on the `User` aggregate (`AdConsentGranted` / `AdConsentRevoked`); analytics tags load with Consent Mode defaults denied
- The audience beacon stores device, browser, OS, country, timezone and referrer only, in a separate database

## Performance Considerations

### Generation

- Generation is a handful of indexed SQLite queries on `meal_plan_recipe` (`user_id`, `user_id + recipe_type` indexes) plus one `shopping_recipe` lookup; the random sample is capped at 35 rows per course type, so cost is independent of library size
- The merged ingredient list is computed in memory and stored in the event, so rendering never recomputes across recipes it does not need

### Page Loads

- Pages that follow a command read the aggregate snapshot instead of waiting for a projection
- HTML is minified and compressed (brotli/gzip); assets are embedded and cache-controlled; CSS is one compiled Tailwind file
- The sitemap is pre-rendered in memory in identity, gzip and brotli encodings and rebuilt by a background task
- The PWA service worker (Workbox) precaches the shell and provides `offline.html`

### Database

- Single writer connection avoids `SQLITE_BUSY`; the read pool is sized by `database.max_connections`
- `mmap_size`, `cache_size`, `temp_store = memory` and `journal_size_limit` are set on every connection in `src/db.rs`
- Read models keep purpose-built indexes (recipe sitemap indexes added in m0014, FTS tables for recipes, users and contacts)

## Deployment Architecture

### Deployment Model

- One statically built binary (`cargo build --release --bin imkitchen`) in a `FROM scratch` image, running as uid 10001 on port 3000
- `imkitchen migrate --config …` runs as an init container; `imkitchen serve --config …` is the app (see `compose.yml` and `helm/`)
- SQLite files live on a persistent volume; Litestream replication is assumed for the application database (the write pool leaves checkpointing to it)

### Configuration

- `config/default.toml` is the committed baseline; pass `--config path` to override. Sections: `[server]` (url, host, port, optional `region` used as the evento routing key), `[database]`, `[jwt]`, `[root]` (bootstrap admin), `[email]`, `[stripe]`, optional `[premium]`, `[analytics]`, `[audience]`, `[monitoring]`
- Removing `[premium]` disables billing UI and the renewal scheduler; removing `[audience]` disables the beacon and the second database

### Local Stack

- `compose.yml` provides Traefik with local TLS certificates (`make cert`), MailDev for SMTP, and optional `standalone` profiles for running the built image
- `make dev` runs `cargo watch -x "run -- --config config/dev.local.toml serve"`

## Development Environment

### Prerequisites

- The nix dev shell in `flake.nix` provides the Rust toolchain, Tailwind CLI, Playwright with Chromium, and the Android SDK for the native shell
- Without nix: stable Rust, Node 18+, the Tailwind CLI and Playwright

### Setup Commands

```bash
# Migrate (creates imkitchen.db) and run the server with local config
make migrate
make dev

# Rebuild CSS once or watch
make css
make css-watch

# Rust checks
make lint        # cargo clippy --workspace --all-targets --all-features -- -D warnings
make fmt
make test        # cargo test --workspace

# After adding an event, nested type or view
EVENTO_LOCK=update cargo test -p imkitchen --test events_lock

# Browser tests / service worker build
npm run test:e2e
npm run build:sw
```

### Local Development Workflow

1. `make migrate` once, then `make dev` (http://localhost:3000, or https://imkitchen.localhost through Traefik after `make up`)
2. `make css-watch` in another terminal when touching templates
3. Add events in `crates/types`, handlers and commands in the owning crate, routes in the web crate, templates under `templates/`
4. If a projection struct changes shape, bump its `.revision(n)`; if an event or nested type must change, add a new variant instead
5. `make check` before committing

## Architecture Decision Records (ADRs)

### ADR-001: Event-Driven Architecture with CQRS

**Decision:** Use evento 2 for event sourcing; one SQLite database with a read pool and a single-connection write pool; read models maintained by subscriptions.

**Rationale:**
- Full audit trail of user actions; projections can be rebuilt
- Commands enforce invariants on the aggregate; reads use purpose-built tables
- A single file keeps deployment simple and Litestream-replicable

**Consequences:**
- Read models are eventually consistent; pages that must be immediately consistent read the aggregate (see ADR-004)
- Event shapes are a public contract (see ADR-002)

---

### ADR-002: Frozen Persisted Shapes (`events.lock`)

**Decision:** Encode events and snapshots with bitcode and guard every persisted shape with evento-lock in `events.lock` and `tests/events_lock.rs`.

**Rationale:**
- bitcode is positional: adding or removing a field makes every stored occurrence undecodable
- A failing test is cheaper than a corrupted event log

**Consequences:**
- Event and nested-type lines are append-only; evolving a concept means adding a variant (`MealPreferences::RecipeTypesChanged`)
- Projection changes require a `.revision(n)` bump, which invalidates snapshots and replays events
- Legacy declarations (`MealPlan`, `Shopping::Generated`, `Shopping::RecipeSetGenerated`) stay in the code for as long as the event log contains them

---

### ADR-003: The Recipe List Replaces the Calendar Meal Plan

**Decision:** Remove weeks, day slots and the scheduler. Model the user's current cooking intent as one ordered list of recipes in the `Shopping` aggregate, generated on demand and edited one recipe at a time.

**Rationale:**
- Users cook from a list, not a timetable; a date-bound plan went stale as soon as a day was skipped
- One aggregate per user keeps list, ingredient checks and cooking statuses consistent in a single event stream
- Generation becomes a pure selection problem over the candidate pool with no time dimension

**Consequences:**
- m0015 dropped `meal_plan_slot`, `shopping_slot`, `shopping_list`; `meal_plan_recipe` survives as the candidate pool and keeps the `"mealplan-command"` subscriber key for cursor continuity
- `/kitchen/{date}` and `/menu/{date}` bookmarks permanently redirect to `/`
- Events carry the full new list and merged ingredients so rebuilds stay cheap

---

### ADR-004: Synchronous List Reads from the Aggregate

**Decision:** The kitchen and groceries pages call `shopping.state()` (projection + `shopping_recipe` lookup) instead of a dedicated read-model table.

**Rationale:**
- A generate, add, remove or toggle must be visible on the very next render; polling a lagging projection produced flicker and stale lists
- The per-user list is small, so recomputing the merged ingredients per request is cheap

**Consequences:**
- No `shopping_list` table or subscription to maintain (dropped in m0015)
- The ingredient scaling rule (`scale_quantity`) is applied at read time with the user's current household size

---

### ADR-005: Candidate Pool and Per-Recipe Ingredient Tables

**Decision:** Keep two small recipe-event-driven tables for generation: `meal_plan_recipe` (what can be picked, with the fields the picker filters on) and `shopping_recipe` (authored household size and bitcode ingredients).

**Rationale:**
- Generation needs random, dietary-filtered sampling per course type, which SQLite does well with `ORDER BY random()` and `json_each`
- Merging needs each recipe's authored serving size; storing it next to the ingredients avoids joining the full recipe view

**Consequences:**
- Both tables are rebuilt from the event log if their subscription cursor is reset
- Saved community recipes are copied into the user's pool on `Favorite::Saved` and removed on `Unsaved`

---

### ADR-006: Per-Recipe Cooking Cursor

**Decision:** Store cooking progress as `RecipeStatus { Idle, Cooking(u8), Completed }` per recipe id inside the `Shopping` aggregate (`RecipeStatusChanged`).

**Rationale:**
- Progress belongs to the list entry, not to a date; completed recipes stay in the list, marked, until removed or regenerated
- Step navigation is a pure function (`next_status`) over the instruction count, testable without I/O

**Consequences:**
- Generation resets every status; removal drops the recipe's status
- The cooking screen partial is rendered from the new status in memory right after the command

---

### ADR-007: SSR + twinspark + PWA

**Decision:** Askama server rendering with twinspark partial swaps and a Workbox service worker; no client-side framework.

**Rationale:**
- Full HTML on first load for SEO (recipe pages, chef pages, sitemap)
- Redirect-then-swap keeps mutation handling in one place on the server
- Offline shell and install prompt with minimal JavaScript

**Consequences:**
- Every interaction is a server round trip; partials must be designed per region
- Demo mode reuses the same templates with mutations swapped for a sign-up modal

---

### ADR-008: Configurable SMTP with lettre; Stripe for Billing

**Decision:** Send email through SMTP configured in `[email]`; run premium subscriptions through Stripe with a renewal cron job inside the binary.

**Rationale:**
- No vendor lock-in for email; local MailDev for development
- Stripe handles payment methods and intents; the only scheduled job left in the system is the renewal check

**Consequences:**
- Deployments must provide SMTP credentials and Stripe keys; omitting `[premium]` turns billing off entirely

---

_Updated for the recipe-list architecture (calendar removed, evento 2)._
