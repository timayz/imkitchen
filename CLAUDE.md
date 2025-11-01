# Rust Event-Driven Web Server Coding Standards

This document outlines the coding standards and best practices for writing event-driven Rust web server applications. These guidelines ensure code quality, maintainability, security, and performance across asynchronous web projects.

## Dependencies

- **Rust**: 1.90+
- **axum**: 0.8+ - Web server framework
- **axum-extra**: 0.12+ - Additional utilities for Axum (features: form, query)
- **askama**: 0.14+ - Template engine
- **askama_web**: 0.14+ - Askama Axum integration
- **twinspark**: UI reactivity library
- **evento**: 1.5+ - Event-driven architecture (feature: sqlite)
- **sqlx**: 0.8+ - Async SQL toolkit
- **sqlite3**: Database engine
- **validator**: 0.20+ - Input validation
- **ulid**: 1.2+ - ULID generation for request IDs

All dependencies must be managed using workspace dependencies in the root `Cargo.toml` to ensure version consistency across all crates.

## CLI and Configuration

### CLI Commands

Every application must implement the following CLI commands:

- **serve** - Start the web server
- **migrate** - Run database migrations (must create database if it doesn't exist)
- **reset** - Drop the database and run migrate command

### Configuration Files

Configuration must be managed using TOML files in the `config/` directory:

- **config/default.toml** - Default configuration for all environments (committed to git)
- **config/dev.toml** - Local development overrides (added to .gitignore, never committed)

### Configuration Rules

- **Never use .env files** - always use TOML configuration files
- **Always add config/dev.toml to .gitignore** - local development settings should not be committed
- **Always commit config/default.toml** - default settings should be version controlled
- **Always create database if it doesn't exist** - the migrate command must handle database creation

### Migration Files

- **Always name migration files using the format `{current_timestamp}_{table_name}.sql`** - use only the table name, the timestamp changes for each migration. Examples: `20231025143052_users.sql`, `20231026104523_users.sql`
- **Timestamp must be in the format `YYYYMMDDHHmmss`** - ensures migrations run in chronological order

## Project Architecture

Projects should be structured as Rust workspaces following DDD (Domain-Driven Design) principles:

```
project-root/
├── Cargo.toml                 # Workspace definition
├── config/
│   ├── default.toml          # Default configuration (committed)
│   └── dev.toml              # Local dev config (.gitignore)
├── src/
│   ├── main.rs               # CLI entry point
│   ├── server.rs             # Web server application (serve command)
│   ├── migrate.rs            # Database migrations (migrate and reset commands)
│   ├── routes/
│   │   ├── auth/
│   │   │   └── login.rs      # Axum route handler
│   │   ├── home.rs
│   │   └── profile.rs        # Axum route handler
│   └── queries/
│       └── profile.rs        # Query and subscription (filename matches template name)
├── templates/
│   ├── pages/
│   │   ├── auth/
│   │   │   └── login.html
│   │   ├── home.html
│   │   └── profile.html      # Matches queries/profile.rs
│   ├── partials/             # Partial response templates
│   │   ├── auth/
│   │   │   └── login-pending.html
│   │   └── profile-pending.html
│   ├── components/           # Reusable HTML components
│   └── email/                # Email templates
└── crates/                   # Bounded contexts
    ├── myapp-user/
    │   ├── Cargo.toml
    │   ├── command.rs        # All commands for this context
    │   ├── event.rs          # All events for this context
    │   ├── aggregate.rs      # Domain model (aggregate root)
    │   └── domain/           # Subdomain folder
    │       └── ...
    └── myapp-order/
        ├── Cargo.toml
        ├── command.rs
        ├── event.rs
        ├── aggregate.rs
        └── domain/
            └── ...
```

Each bounded context in the `crates/` folder encapsulates a specific domain with its own commands, events, and aggregate root.

### Naming Conventions

- **Bounded context crate names must be prefixed with the root binary name** - use `{app-name}-{domain}` format (e.g., if your app is `myapp`, use `myapp-user`, `myapp-order`, `myapp-product`)
- **Do not use "context" prefix or suffix** - avoid `user-context`, `context-user`, etc.

## Database Guidelines

Applications must use separate databases for write and read operations following CQRS pattern:

- **Write database** - Used exclusively by Evento for storing events and aggregate state
- **Read database** - Used for storing projections and querying data

### Database Rules

- **Always use write database for Evento** - all event sourcing operations go through the write database
- **Always use read database for projections** - query handlers build and query from the read database only
- **Never query Evento data from read operations** - read database contains projections, not event store data
- **Never write projections to write database** - keep write and read databases strictly separated

## Aggregate Guidelines

Keep aggregates as small as possible. Aggregates must contain only fields that are used in commands or command handlers.

## Command Guidelines

All commands for a bounded context must be defined as methods of a `Command` struct in the `command.rs` file. The `Command` struct must have an `Evento` field.

### Command Rules

- **Always use input struct as command argument** - command methods must accept a single input struct parameter as the first argument, followed by metadata as the second argument. This ensures better maintainability and validation
- **CRITICAL: Always use consistent data in commands, never bypass this rule** - commands must ONLY use Evento to retrieve consistent data OR validation tables (in command handlers). Never use projections, never query read databases, never use cached data. Projections are eventually consistent and using them in commands will lead to data corruption and race conditions. This rule has no exceptions
- **Never use projections** in command implementations, as projections are not consistent data
- **Always use Evento or validation tables** to retrieve consistent data when implementing commands
- **Never use commands/events from other domains** - maintain strict bounded context isolation
- **Use validator** for command input validation
- **Always defer long/async validation** to the command handler
- **Always use SubscriptionBuilder.unsafe_oneshot to process events synchronously** when testing commands that have deferred validation
- **Always handle commands in less than 10 seconds** - due to graceful shutdown and subscribe start delay being set at 10s
- **Always use evento::create** to create aggregator while inserting event and **evento::save** to insert event to existing aggregator
- **SQL tables may be used for consistency validation in command handlers only** - validation tables (not projection tables) can be queried in handlers
- **Never generate timestamps in commands** - timestamps are handled by Evento automatically
- **Never use direct database operations in tests** - never SELECT/INSERT/UPDATE/DELETE directly from database in tests. Always use `evento::load` to validate command results
- **Always include metadata with events** - events must have metadata containing:
  - `user_id` (optional) - the ID of the user who triggered the action
  - `request_id` (ULID) - a unique identifier to track all events generated by a single user action, enabling tracing of the complete event chain from one request

### Complete Example: Command with Async Validation + Polling

This example demonstrates a complete flow with async validation, server-side rendering, and polling.

**Events (crates/myapp-user/event.rs):**
```rust
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone)]
pub struct EventMetadata {
    pub user_id: Option<String>,
    pub request_id: String,  // ULID
}

#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserCreated {
    pub name: String,
    pub email: String,
}

#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserCreationSucceeded {
    pub name: String,
    pub email: String,
}

#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserCreationFailed {
    pub error: String,
}
```

**Aggregate (crates/myapp-user/aggregate.rs):**
```rust
use bincode::{Decode, Encode};
use evento::EventDetails;

#[derive(Default, Encode, Decode, Clone)]
pub struct User {
    pub status: Option<String>,
}

#[evento::aggregator]
impl User {
    async fn user_created(&mut self, _event: EventDetails<UserCreated, EventMetadata>) -> anyhow::Result<()> {
        self.status = Some("pending".to_string());
        Ok(())
    }

    async fn user_creation_succeeded(&mut self, _event: EventDetails<UserCreationSucceeded, EventMetadata>) -> anyhow::Result<()> {
        self.status = Some("success".to_string());
        Ok(())
    }

    async fn user_creation_failed(&mut self, _event: EventDetails<UserCreationFailed, EventMetadata>) -> anyhow::Result<()> {
        self.status = Some("failed".to_string());
        Ok(())
    }
}
```

**Command (crates/myapp-user/command.rs):**
```rust
use evento::Executor;
use sqlx::SqlitePool;

pub struct CreateUserInput {
    pub name: String,
    pub email: String,
}

pub struct Command<E: Executor> {
    evento: E,
    validation_pool: SqlitePool,
}

impl<E: Executor> Command<E> {
    pub fn new(evento: E, validation_pool: SqlitePool) -> Self {
        Self { evento, validation_pool }
    }

    pub async fn create_user(
        &self,
        input: CreateUserInput,
        metadata: EventMetadata,
    ) -> anyhow::Result<String> {
        // Validation is deferred to command handler
        let user_id = evento::create::<User>()
            .data(&UserCreated {
                name: input.name,
                email: input.email
            })?
            .metadata(&metadata)?
            .commit(&self.evento)
            .await?;

        Ok(user_id)
    }
}
```

**Command handler with async validation (crates/myapp-user/command.rs):**
```rust
use evento::{Context, EventDetails, Executor};

#[evento::handler(User)]
async fn on_user_created<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserCreated, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    // Async validation: Check email uniqueness in validation table
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_emails WHERE email = ?)"
    )
    .bind(&event.data.email)
    .fetch_one(&pool)
    .await?;

    if exists {
        // Emit error event instead of returning error
        evento::save::<User>(&event.aggregator_id)
            .data(&UserCreationFailed {
                error: "Email already exists".to_string(),
            })?
            .metadata(&event.metadata)?
            .commit(context.executor())
            .await?;

        return Ok(()); // Return Ok to acknowledge event processing
    }

    // Insert into validation table
    sqlx::query("INSERT INTO user_emails (user_id, email) VALUES (?, ?)")
        .bind(&event.aggregator_id)
        .bind(&event.data.email)
        .execute(&pool)
        .await?;

    // Emit success event
    evento::save::<User>(&event.aggregator_id)
        .data(&UserCreationSucceeded {
            name: event.data.name.clone(),
            email: event.data.email.clone(),
        })?
        .metadata(&event.metadata)?
        .commit(context.executor())
        .await?;

    Ok(())
}

pub fn subscribe_user_command(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("user-command")
        .data(pool)
        .handler(on_user_created())
        .skip::<User, UserCreationSucceeded>()
        .skip::<User, UserCreationFailed>()
}
```

**Query handlers (src/queries/users.rs):**
```rust
use evento::{Context, EventDetails, Executor};
use sqlx::SqlitePool;

#[evento::handler(User)]
async fn on_user_creation_succeeded<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserCreationSucceeded, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    // Insert into query table (projection)
    sqlx::query("INSERT INTO users (id, name, email, status, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&event.aggregator_id)
        .bind(&event.data.name)
        .bind(&event.data.email)
        .bind("success")
        .bind(event.timestamp)
        .execute(&pool)
        .await?;

    Ok(())
}

#[evento::handler(User)]
async fn on_user_creation_failed<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserCreationFailed, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    // Insert error into query table (projection)
    sqlx::query("INSERT INTO users (id, status, error, created_at) VALUES (?, ?, ?, ?)")
        .bind(&event.aggregator_id)
        .bind("failed")
        .bind(&event.data.error)
        .bind(event.timestamp)
        .execute(&pool)
        .await?;

    Ok(())
}

pub fn subscribe_user_query(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("user-query")
        .data(pool)
        .handler(on_user_creation_succeeded())
        .handler(on_user_creation_failed())
        .skip::<User, UserCreated>()
}

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

pub async fn get_user(pool: &SqlitePool, user_id: &str) -> anyhow::Result<Option<UserRow>> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, name, email, status, error FROM users WHERE id = ?"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}
```

**Axum route handlers (src/routes/users.rs):**
```rust
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::Form;
use ulid::Ulid;

#[derive(Template, WebTemplate)]
#[template(path = "pages/user-form.html")]
struct UserFormTemplate {
    error: Option<String>,
}

#[derive(Template, WebTemplate)]
#[template(path = "partials/user-pending.html")]
struct UserPendingTemplate {
    user_id: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "partials/user-success.html")]
struct UserSuccessTemplate {
    user: UserRow,
}

#[derive(serde::Deserialize)]
pub struct CreateUserForm {
    name: String,
    email: String,
}

pub async fn get_form() -> impl IntoResponse {
    UserFormTemplate { error: None }
}

pub async fn post_form(
    State(state): State<AppState>,
    Form(form): Form<CreateUserForm>,
) -> impl IntoResponse {
    let input = CreateUserInput {
        name: form.name,
        email: form.email,
    };

    let metadata = EventMetadata {
        user_id: None,  // Or extract from auth session
        request_id: Ulid::new().to_string(),
    };

    let result = state.command.create_user(input, metadata).await;

    match result {
        Ok(user_id) => UserPendingTemplate { user_id }.into_response(),
        Err(e) => UserFormTemplate {
            error: Some(e.to_string()),
        }
        .into_response(),
    }
}

pub async fn get_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match get_user(&state.pool, &user_id).await {
        Ok(Some(user)) => {
            match user.status.as_str() {
                "success" => UserSuccessTemplate { user }.into_response(),
                "failed" => UserFormTemplate {
                    error: user.error,
                }
                .into_response(),
                _ => UserPendingTemplate { user_id }.into_response(),
            }
        }
        Ok(None) => UserPendingTemplate { user_id }.into_response(),
        Err(e) => UserFormTemplate {
            error: Some(e.to_string()),
        }
        .into_response(),
    }
}
```

**Form template (templates/pages/user-form.html):**
```html
<!DOCTYPE html>
<html>
<head>
    <title>Create User</title>
    <script src="/twinspark.js"></script>
</head>
<body>
    <div id="user-form">
        {% if let Some(msg) = error %}
        <div class="bg-red-100 text-red-800 p-4 mb-4">{{ msg }}</div>
        {% endif %}

        <form ts-req="/users"
              ts-req-method="POST"
              ts-target="#user-form">
            <div class="mb-4">
                <label class="block mb-2">Name</label>
                <input type="text" name="name" required class="border p-2 w-full">
            </div>

            <div class="mb-4">
                <label class="block mb-2">Email</label>
                <input type="email" name="email" required class="border p-2 w-full">
            </div>

            <button type="submit" class="bg-blue-500 text-white px-4 py-2">
                Create User
            </button>
        </form>
    </div>
</body>
</html>
```

**Pending template with polling (templates/partials/user-pending.html):**
```html
<div id="user-form"
     ts-req="/users/{{ user_id }}/status"
     ts-trigger="load delay 1s">
    <div class="bg-yellow-100 text-yellow-800 p-4">
        Creating user, please wait...
    </div>
</div>
```

**Success template (templates/partials/user-success.html):**
```html
<div id="user-form" class="bg-green-100 text-green-800 p-4">
    <p>User created successfully!</p>
    <p>Name: {{ user.name }}</p>
    <p>Email: {{ user.email }}</p>
</div>
```

**Router setup (src/main.rs):**
```rust
use axum::{routing::{get, post}, Router};

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(users::get_form).post(users::post_form))
        .route("/users/{id}/status", get(users::get_user_status))
}
```

**Flow:**
1. User submits form → `post_form` handler
2. Command is executed, emits `UserCreated` event, returns immediately with `user_id`
3. Pending template is returned with polling setup
4. Command handler processes `UserCreated` event:
   - If validation passes: Emits `UserCreationSucceeded` event
   - If validation fails: Emits `UserCreationFailed` event with error message
5. Query handler processes success/failure events and updates projection
6. Browser polls `/users/{id}/status` every 1 second
7. Status endpoint checks projection:
   - If status is "success": Returns success template (polling stops)
   - If status is "failed": Returns form template with error message (polling stops)
   - If no record yet: Returns pending template (polling continues)

## Query Guidelines

All queries must be defined in the root binary application, not in the bounded context crates.

### Query Rules

- **Never try to access Evento data** - queries have their own tables (projections)
- **Query handlers must be idempotent** - handlers can run more than once for the same event (e.g., when resetting cursor position to the beginning)
- **Always handle events in less than 10 seconds** - due to graceful shutdown and subscribe start delay being set at 10s
- **Each query should have one Evento subscription** that builds it for one HTML view only, containing only the data required by that view
- **Always use SubscriptionBuilder.unsafe_oneshot to process events synchronously** when testing query handlers
- **Always use `event.timestamp` for timestamp fields** - use it for `created_at`, `updated_at`, and any other time-related fields in projections
- **Never perform heavy calculations in queries** - all computation, aggregation, and data transformation must be done in subscription handlers that update projections. Query functions should only retrieve already-computed data from projections
- **Never use direct database operations in tests** - never SELECT/INSERT/UPDATE/DELETE directly from database in tests. Always use implemented query functions to validate query output

## Evento Handler Guidelines

Event handlers process events from the event stream and update projections or perform validation.

### Evento Handler Rules

- **Subscriptions can handle multiple aggregates** - a single subscription is not restricted to one aggregate type. You can handle events from multiple different aggregates in the same subscription (e.g., User, Order, Product events in one subscription)
- **Never have duplicate handlers** - cannot have duplicate handlers for the same Aggregate/Event combination in a single subscription. Each unique Aggregate/Event pair can only have one handler
- **Use `.skip::<Aggregate, Event>()` for specific events you don't want to handle** - Evento is strict and will error if an event has no handler. Use `.skip()` to explicitly skip specific events you don't need to process (e.g., `.skip::<Todo, TodoDeleted>()`)
- **Skip is optional** - only use skip when you have specific events you intentionally don't want to handle
- **Always handle one event per handler** - each handler function should handle exactly one event type
- **Handler order doesn't matter** - handlers are matched by event type, not by order
- **Never use tokio::spawn to run subscriptions** - subscriptions handle their own async execution internally, spawning them is unnecessary and can cause issues
- **Always make subscription builders reusable between main.rs and tests (DRY)** - subscription functions should return a SubscriptionBuilder that can be used with `.run(...)` in main.rs and `.unsafe_oneshot(...)` in tests. Example: `my_subscribe().run(...)` in production, `my_subscribe().unsafe_oneshot(...)` in tests

### Subscription Examples

**❌ Bad: Missing handler, will error when TodoDeleted event occurs**
```rust
pub fn subscribe_todo_query(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("todo-query")
        .data(pool)
        .handler(on_todo_created())
        .handler(on_todo_completed())
        .handler(on_todo_uncompleted())
        // BAD: No handler for TodoDeleted, will error when that event occurs
}
```

**✅ Good: Using skip for specific event you don't need**
```rust
pub fn subscribe_todo_query(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("todo-query")
        .data(pool)
        .handler(on_todo_created())
        .handler(on_todo_completed())
        .handler(on_todo_uncompleted())
        .skip::<Todo, TodoDeleted>() // GOOD: Explicitly skip TodoDeleted event
}
```

**❌ Bad: Duplicate handlers**
```rust
pub fn subscribe_user_query(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("user-query")
        .data(pool)
        .handler(on_user_created())
        .handler(on_user_created()) // BAD: Duplicate handler for UserCreated
}
```

**✅ Good: All events handled**
```rust
pub fn subscribe_user_query(
    pool: SqlitePool,
) -> evento::SubscriptionBuilder {
    evento::subscribe("user-query")
        .data(pool)
        .handler(on_user_created())
        .handler(on_user_email_changed())
        .handler(on_user_deleted())
        // GOOD: All User events have handlers, no skip needed
}
```

**Using SubscriptionBuilder in main.rs:**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let executor = evento::SqliteExecutor::new("evento.db").await?;
    let pool = SqlitePool::connect("query.db").await?;

    // Use .run() in production
    subscribe_user_query(pool).run(&executor).await?;

    Ok(())
}
```

**Using SubscriptionBuilder in tests:**
```rust
#[tokio::test]
async fn test_user_query() -> anyhow::Result<()> {
    let executor = evento::SqliteExecutor::new(":memory:").await?;
    let pool = SqlitePool::connect(":memory:").await?;

    // Use .unsafe_oneshot() in tests for synchronous event processing
    subscribe_user_query(pool.clone()).unsafe_oneshot(&executor).await?;

    // Now test your query...

    Ok(())
}
```

## SQLx Guidelines

- **Never use compile-time checked queries** - avoid `query!` and `query_as!` macros in favor of runtime queries

## Axum Guidelines

- **Always use latest route parameter format** - use `{id}` instead of `:id` for route parameters (Axum 0.8+)
- **Always use axum-extra Form/Query extractors when dealing with HTML forms** - use `axum_extra::extract::Form` and `axum_extra::extract::Query` for form data and query parameters instead of the core axum extractors

### Axum Route Examples

**❌ Bad: Old route parameter format**
```rust
fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/:id", get(get_user))           // BAD: Old :id format
        .route("/posts/:post_id/comments/:id", get(get_comment))  // BAD
}
```

**✅ Good: Latest route parameter format**
```rust
fn routes() -> Router<AppState> {
    Router::new()
        .route("/users/{id}", get(get_user))          // GOOD: New {id} format
        .route("/posts/{post_id}/comments/{id}", get(get_comment))  // GOOD
}
```

## Server-Side Rendering

Always use Askama for HTML templating.

### Server-Side Rendering Rules

- **Always use Twinspark for UI reactivity** - avoid using JavaScript as much as possible
- **Always do request polling after actions** - after submit, button clicks, etc., as all actions are async and should wait for projection updates. Use Twinspark attribute `ts-trigger="load delay 1s"` for polling
- **Always render HTML with status 200** - do not use REST API patterns
- **Always use Tailwind 4.1+ classes for styling** - avoid using CSS as much as possible. Tailwind 4.1+ does not require tailwind.config.js
- **Always define `ts-target` when using `ts-req`** - specify where the response HTML should be placed to avoid unexpected behavior

## Askama Guidelines

Askama is a type-safe template engine for Rust. For Axum integration, use `askama_web` with the `axum-0.8` feature.

### Template Structure

Templates must derive both `Template` and `WebTemplate`.

### Template Syntax

#### Variables

Access using dot notation: `{{ name }}` or `{{ user.name }}`

Can use constants from Rust code: `{{ crate::MAX_NB_USERS }}`

#### Control Structures

**For loops:**
```
{% for user in users %}
  <li>{{ user.name|e }}</li>
{% endfor %}
```

**If/Else statements:**
```
{% if users.len() == 0 %}
  No users
{% else if users.len() == 1 %}
  1 user
{% endif %}
```

**Match blocks for enum handling:**
```
{% match item %}
  {% when Some with (val) %}
    Found {{ val }}
  {% when None %}
{% endmatch %}
```

**If let for Option/Result:**
```
{% if let Some(user) = user %}
  {{ user.name }}
{% endif %}
```

#### Filters

Applied with pipe symbol: `{{ name|escape }}`

Can chain filters: `{{ name|lower|capitalize }}`

Common filters: `escape` (or `e`), `lower`, `upper`, `capitalize`, `trim`, `safe`

#### Template Inheritance

Base template defines blocks:
```
{% extends "base.html" %}
{% block content %}
  <h1>Index</h1>
{% endblock %}
```

#### Comments

Use `{# comment #}` for template comments

#### Whitespace Control

Use `-` to strip whitespace: `{{- name -}}` or `{%- if -%}`

### Askama Rules

- **Always derive both `Template` and `WebTemplate`** - required for Axum integration
- **Never use `safe` filter unless absolutely necessary** - XSS risk
- **Always use `escape` filter for user-generated content** - default behavior
- **Keep template logic simple** - complex logic belongs in handlers
- **Use template inheritance for layouts** - avoid duplication
- **Return templates directly from handlers** - they implement `IntoResponse`

### TwinSpark API Reference

#### HTML Updates Directives

**Core**
- `ts-req` - Make a request for an HTML
- `ts-target` - Replace another part of a page with incoming HTML
- `ts-req-selector` - Select only a part of a response
- `ts-swap` - Select a strategy for HTML replacement
  - `replace` (default) - Replace target element with an incoming element
  - `inner` - Replaces target's children with an incoming element
  - `prepend` - Inserts incoming element as a first child of the target
  - `append` - Inserts incoming element as a last child of the target
  - `beforebegin` - Inserts incoming element before target
  - `afterend` - Inserts incoming element after target
  - `morph` - Morphs incoming element into target
  - `morph-all` - Same as morph, but does not skip document.activeElement when changing elements
  - `skip` - Just skip that response, sometimes useful for operations with side-effects
- `ts-swap-push` - "Push" HTML from server to a client
- `ts-trigger` - Specify event which triggers the request
  - Syntax: `<event-type> [modifier...]`
  - Modifiers:
    - `delay:<ms>` - Delay invocation by specified amount of milliseconds
    - `once` - Remove listener after first invocation
    - `changed` - Trigger only if element's value has changed since last invocation (or checked for checkboxes and radios)
  - Standard DOM events: `click`, `submit`, `mouseover`, `change`, `blur`, etc.
  - Additional event types:
    - `load` - Trigger on document load or when element appears on screen
    - `scroll` - Trigger when window is scrolled
    - `windowScroll` - Trigger when the target element is scrolled
    - `outside` - Trigger on click outside of the element
    - `remove` - Trigger when the element is removed (uses MutationObserver)
    - `childrenChange` - Trigger when children are added to or removed from the element (uses MutationObserver)
    - `empty` - childrenChange, but element left with no children
    - `notempty` - childrenChange, but element has at least 1 child
    - `visible` - At least 1% of the element appeared in the viewport (uses IntersectionObserver)
    - `invisible` - Element moved off the screen (uses IntersectionObserver)
    - `closeby` - At least 1% of the element is closer than window.innerHeight / 2 to the viewport (uses IntersectionObserver)
    - `away` - Inverse of closeby (uses IntersectionObserver)

**Additional**
- `ts-req-method` - Is it GET or POST?
- `ts-req-strategy` - How to deal with multiple requests being generated
  - `first` - Prevent triggering new requests until the active one finishes (useful for forms)
  - `last` - Abort active request when a new one is triggered
  - `queue` (default) - Send requests as they are triggered
- `ts-req-history` - Change URL after request
- `ts-data` - Additional data for request
- `ts-json` - As ts-data, but for JSON requests
- `ts-req-batch` - Combine multiple requests into a single one

#### Actions Directives

- `ts-action` - Run actions using pipeline syntax: `some-command arg1 arg2, other-command arg3 arg4`
  - Pipelines are asynchronous and wait for promises to resolve
  - If a command returns exactly `false`, pipeline stops
  - Multiple pipelines can be joined by `;` and are executed in order
  - Arguments with spaces can be enclosed in quotes: `target "parent p"`
  - Escaping works: `log '\' is a quote'`
  - Built-in commands:
    - `stop` - Calls .stopPropagation() on triggering event
    - `prevent` - Calls .preventDefault() on triggering event
    - `delay N` - Delays pipeline execution by N ms, or seconds with syntax Ns
    - `target SEL` - Selects another element, identified by SEL (supports ts-trigger modifiers)
    - `remove [SEL]` - Removes target element, or element identified by SEL
    - `class+ CLS` - Adds class CLS to a target element
    - `class CLS` - Alias for class+
    - `class- CLS` - Removes class CLS from a target element
    - `class^ CLS` - Toggles class CLS on a target element
    - `classtoggle CLS` - Alias for class^
    - `text [VALUE]` - Returns .innerText, or sets it if VALUE is passed
    - `html [VALUE]` - Returns .innerHTML, or sets it if VALUE is passed
    - `attr name [VALUE]` - Returns VALUE of attribute name, or sets it if VALUE is passed
    - `log [...]` - Logs all passed arguments and pipeline input if passed
    - `not cmd [...]` - Inverts result of calling cmd
    - `wait EVENTNAME` - Waits until event EVENTNAME happens on target element, only once
    - `on EVENTNAME` - Adds an event listener on EVENTNAME (needs ts-trigger="load" to execute)
    - `req [METHOD] URL` - Execute a request like ts-req, adds pipeline input as input=INPUT if present
- `ts-trigger` - Specify event which triggers actions
- `ts-req-before` - Actions to run before request
- `ts-req-after` - Actions to run after request

#### Events

- `ts-ready` - When HTML is "activated"
- `ts-trigger` - Event generated by ts-trigger
- `ts-req-before` - Before request
- `ts-req-after` - After request
- `ts-req-error` - On request errors
- `ts-pushstate` - When a new entry is pushed to browser history
- `ts-replacestate` - When a browser history entry is replaced
- `visible` - When 1% of element appears on screen
- `invisible` - When element was visible and now less than 1% of it is
- `closeby` - When 1% of element is closer to viewport than half of window height
- `away` - Antonym to closeby
- `remove` - When an element is removed (depends on a trigger subscribing)
- `empty` - When element becomes childless
- `notempty` - When element had hierarchy changes and has children
- `childrenChange` - Combination of empty and notempty

#### Headers

**Request**
- `accept` - TwinSpark requests are always text/html+partial
- `ts-url` - Current page URL
- `ts-origin` - Identifier of an element which made request
- `ts-target` - Identifier of a target element

**Response**
- `ts-swap` - Override HTML swap strategy
- `ts-swap-push` - "Push" some HTML, replace: selector to <= selector from
- `ts-history` - New browser history URL
- `ts-title` - New page title in case of history push
- `ts-location` - Redirect to target URL


## Email Guidelines

Always use Askama for HTML and text email templates.

## Logging

Always use the `tracing` crate for structured logging throughout the application.

### Logging Rules

- **Always use tracing log levels extensively** - log as much as possible to enable easy debugging
- **Always use the right log level** for each situation:
  - `error!` - For errors that need immediate attention (failures, panics, critical issues)
  - `warn!` - For potentially problematic situations that don't stop execution
  - `info!` - For important business logic flow and significant state changes
  - `debug!` - For detailed diagnostic information useful during development
  - `trace!` - For very detailed debugging information (loop iterations, data transformations)
- **Always include context in logs** - use structured fields to provide relevant information
- **Always log command execution** - log when commands start and complete
- **Always log event handling** - log when events are processed in query handlers
- **Always log external calls** - log database queries, HTTP requests, and external service calls
- **Always log validation failures** - include what failed and why
- **Never log sensitive data** - avoid logging passwords, tokens, or personal information

## Code Quality

After completing any task, the following commands must pass without errors:

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --all`

### Code Quality Rules

- **Never bypass clippy warnings using #[allow(...)]** - all clippy warnings must be fixed, not suppressed. If a clippy warning appears, refactor the code to address the underlying issue

## Testing Guidelines

Always use Test-Driven Development (TDD).

### Testing Rules

- **Always create tests in the `tests` folder** of the workspace - never in `src`
- **Test filenames in root binary should always be prefixed with the route's first parent name**
- **Always test application features, not external library features** - focus on your code's behavior
- **E2E testing should only be done for critical application features**
- **Always use Playwright for E2E testing**
- **Always use migrations for database setup** - use `sqlx::migrate!` or `evento::sql_migrator` to set up database schema in tests. Never create tables directly using `sqlx::query`
- **Always apply DRY (Don't Repeat Yourself) when creating database setup for tests** - create reusable helper functions for database initialization, connection pool setup, and cleanup. Never duplicate database setup code across test files

## Git Guidelines

Always use Conventional Commits for commit messages and GitHub PR titles.

### Commit Message Format

Use the format: `type(scope): description`

**Types:**
- `feat` - A new feature that affects user actions **in the web UI only**
- `fix` - A bug fix that affects user actions **in the web UI only**
- `chore` - Changes that don't affect web UI user actions (CLI commands, refactoring, dependencies, tooling, internal improvements, etc.)
- `docs` - Documentation only changes
- `test` - Adding or updating tests
- `perf` - Performance improvements
- `ci` - CI/CD configuration changes

### Commit Rules

- **Use `feat` or `fix` only for web UI changes** - changes that directly affect what users can do in the web interface
- **User actions = web UI interactions only** - CLI commands, developer tools, and internal APIs are NOT user actions
- **Use `chore` for everything else** - CLI commands, refactoring, dependency updates, internal improvements, code cleanup, etc.
- **Always write clear, descriptive commit messages** - focus on the "why" rather than the "what"
- **Use present tense** - "add feature" not "added feature"
- **Keep the first line under 72 characters** - detailed explanation can go in the commit body

### Examples

**✅ Good: Web UI user-facing changes**
```
feat(auth): add password reset functionality
fix(todos): prevent duplicate todo creation on double-click
feat(profile): add user avatar upload
fix(checkout): correct tax calculation for EU countries
feat(dashboard): add filtering options to user list
```

**✅ Good: Non web UI changes (use chore)**
```
chore(cli): add migrate command
chore(cli): add database backup command
chore(deps): update axum to 0.8.1
chore(refactor): extract user validation to separate module
chore(types): improve error type definitions
chore(db): optimize user query indexes
chore(api): add internal health check endpoint
```

**❌ Bad: Incorrect type usage**
```
feat(cli): add seed command          // Should be chore (CLI is not web UI)
feat(deps): update dependencies      // Should be chore
fix(refactor): clean up code         // Should be chore
chore(login): add login page         // Should be feat (web UI feature)
feat(api): add admin API endpoint    // Should be chore (internal API, not web UI)
```

### Pull Request Titles

- **PR titles must follow the same Conventional Commits format** - use the same type prefixes
- **PR title should summarize all commits** - if multiple types are present, use the most significant one
- **Keep PR scope focused** - ideally one type per PR

### Examples

**✅ Good PR Titles**
```
feat(auth): implement OAuth2 login flow
fix(api): resolve race condition in order processing
chore(tests): migrate to new testing framework
```

**❌ Bad PR Titles**
```
Update stuff                      // Not descriptive, no type
Added new feature                 // No type prefix
fix: various bug fixes           // Too vague, no scope
```

