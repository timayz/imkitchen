# Project Guidelines: Rust + Axum + Askama

A guide to structuring a web application so that the **type system does the testing for you**. The goal is not zero tests — it is zero *redundant* tests. Every invariant we can lift into a type is a class of bugs that cannot compile, and a class of tests we never write.

---

## Table of contents

1. [Philosophy](#1-philosophy)
2. [Project layout](#2-project-layout)
3. [Domain types: parse, don't validate](#3-domain-types-parse-dont-validate)
4. [Newtype IDs](#4-newtype-ids)
5. [Sum types for state modeling](#5-sum-types-for-state-modeling)
6. [Error handling: one `AppError` to rule them all](#6-error-handling-one-apperror-to-rule-them-all)
7. [Custom Axum extractors: validation at the boundary](#7-custom-axum-extractors-validation-at-the-boundary)
8. [Auth via typestate](#8-auth-via-typestate)
9. [SQLx integration](#9-sqlx-integration)
10. [Askama patterns](#10-askama-patterns)
11. [Forms and CSRF](#11-forms-and-csrf)
12. [Configuration](#12-configuration)
13. [Testing strategy — what's left to test](#13-testing-strategy--whats-left-to-test)
14. [Anti-patterns](#14-anti-patterns)
15. [Checklist for new features](#15-checklist-for-new-features)

---

## 1. Philosophy

Three rules, in order of priority:

1. **Make illegal states unrepresentable.** If the compiler rejects bad states, you don't need tests for them.
2. **Parse at the boundary, trust internally.** Raw strings/ints cross into the app exactly once. Everything downstream sees validated domain types.
3. **Push side effects to the edges.** A pure functional core is trivial to reason about; the shell at the edges (HTTP, DB, filesystem) is the only thing that needs integration testing.

When a feature feels hard to test, that is almost always a sign the *types are wrong*, not that you need more tests. Fix the types first.

---

## 2. Project layout

```
src/
├── main.rs              # Wiring only: load config, build router, bind port
├── app.rs               # Router construction, middleware stack
├── config.rs            # Config struct, env loading, validation
├── error.rs             # AppError + IntoResponse
├── domain/              # Pure types and business logic. NO axum, NO sqlx.
│   ├── mod.rs
│   ├── ids.rs           # Newtype IDs
│   ├── user.rs
│   ├── post.rs
│   └── slug.rs
├── extract/             # Custom Axum extractors
│   ├── mod.rs
│   ├── auth.rs
│   └── valid.rs
├── db/                  # SQLx queries. Returns domain types.
│   ├── mod.rs
│   ├── users.rs
│   └── posts.rs
├── web/                 # Handlers + templates
│   ├── mod.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── posts.rs
│   │   └── auth.rs
│   └── templates.rs     # Askama template structs
└── templates/           # .html files (askama source)
    ├── base.html
    ├── post.html
    └── ...
```

The `domain/` module has no dependencies on `axum`, `sqlx`, or any I/O. This is the rule that keeps the core testable by inspection rather than by running.

---

## 3. Domain types: parse, don't validate

Never pass `String` around to mean something more specific. Wrap it.

```rust
// src/domain/slug.rs
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SlugError {
    #[error("slug cannot be empty")]
    Empty,
    #[error("slug must be 100 characters or fewer")]
    TooLong,
    #[error("slug may only contain lowercase letters, digits, and '-'")]
    InvalidChars,
}

impl FromStr for Slug {
    type Err = SlugError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(SlugError::Empty);
        }
        if s.len() > 100 {
            return Err(SlugError::TooLong);
        }
        if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(SlugError::InvalidChars);
        }
        Ok(Self(s.to_owned()))
    }
}

impl Slug {
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn into_string(self) -> String { self.0 }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
```

**Rules for domain types:**

- Constructor is private; `FromStr` / `TryFrom` is the only way in.
- Error type is a specific enum, not `anyhow::Error`.
- `Display` yields the canonical serialized form.
- No `Default` impl unless "default" is genuinely meaningful.
- Tests for these live next to them and are *exhaustive for each error variant*. This is one of the few places where unit tests pay off hugely — one test of `Slug::from_str` replaces every validation test across every handler.

Other good candidates: `Email`, `Password` (hashed), `Username`, `NonEmptyString`, `Markdown`, `SafeHtml`, `MoneyCents`, `Percentage`.

---

## 4. Newtype IDs

Every entity gets its own ID type. Mixing them up becomes a compile error.

```rust
// src/domain/ids.rs
use uuid::Uuid;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash,
            serde::Serialize, serde::Deserialize,
            sqlx::Type,
        )]
        #[sqlx(transparent)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

id_type!(UserId);
id_type!(PostId);
id_type!(SessionId);
id_type!(CommentId);
```

Now `fn get_post(author: UserId, post: PostId)` cannot be called with swapped arguments. A whole category of "wrong ID passed" bugs is eliminated.

---

## 5. Sum types for state modeling

Any time you find yourself writing `Option<A>` and `Option<B>` on the same struct where they're related, that is a sum type trying to get out.

```rust
// Bad — allows nonsense states
struct RequestState {
    loading: bool,
    data: Option<Post>,
    error: Option<String>,
}
// Can be { loading: true, data: Some(..), error: Some(..) }. Nonsense.

// Good — only valid states exist
enum RequestState {
    Idle,
    Loading,
    Loaded(Post),
    Failed(String),
}
```

Exhaustive `match` means adding a new variant forces you to handle it everywhere. That is the compiler giving you a TODO list — a safer one than any test suite could produce.

**Guideline:** Prefer enums over booleans whenever the boolean's meaning is domain-specific. `is_published: bool` becomes `status: PostStatus { Draft, Published { at: DateTime<Utc> }, Archived }`. Notice how `Published` carries its timestamp — impossible to have a published post without a publication time.

---

## 6. Error handling: one `AppError` to rule them all

```rust
// src/error.rs
use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    // Internal errors are bucketed — we never expose detail to users.
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Template(#[from] askama::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Template(_) | Self::Other(_) => {
                tracing::error!(error = ?self, "internal error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        // In a real app, render an error template here instead of plain text.
        let body = match &self {
            Self::Database(_) | Self::Template(_) | Self::Other(_) => {
                "Something went wrong.".to_string()
            }
            other => other.to_string(),
        };

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

**Why this shape:**

- `#[from]` means `?` just works on `sqlx::Error`, `askama::Error`, `anyhow::Error`.
- Internal errors log full detail server-side but show a generic message to the client. No accidental info leakage.
- Handlers return `AppResult<T>` and you're done — Axum's `IntoResponse` does the rest.

---

## 7. Custom Axum extractors: validation at the boundary

This is where a lot of the magic lives. An extractor validates *once*, at the edge, and every handler downstream gets a proof-by-construction.

```rust
// src/extract/valid.rs
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use std::str::FromStr;
use crate::{domain::slug::Slug, error::AppError};

pub struct SlugPath(pub Slug);

impl<S: Send + Sync> FromRequestParts<S> for SlugPath {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(raw): Path<String> = Path::from_request_parts(parts, state)
            .await
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let slug = Slug::from_str(&raw)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        Ok(Self(slug))
    }
}
```

Usage:

```rust
async fn show_post(SlugPath(slug): SlugPath, State(db): State<PgPool>) -> AppResult<PostTemplate> {
    // `slug` is provably valid. No `if slug.is_valid()` in sight.
}
```

For forms, use `axum::Form<T>` with a `TryFrom` wrapper:

```rust
#[derive(serde::Deserialize)]
pub struct RawNewPost {
    pub title: String,
    pub slug: String,
    pub body: String,
}

pub struct NewPost {
    pub title: NonEmptyString,
    pub slug: Slug,
    pub body: Markdown,
}

impl TryFrom<RawNewPost> for NewPost {
    type Error = AppError;
    fn try_from(raw: RawNewPost) -> Result<Self, Self::Error> {
        Ok(Self {
            title: raw.title.parse().map_err(|e: _| AppError::Validation(e.to_string()))?,
            slug: raw.slug.parse().map_err(|e: SlugError| AppError::Validation(e.to_string()))?,
            body: raw.body.parse().map_err(|e: _| AppError::Validation(e.to_string()))?,
        })
    }
}
```

Handler:

```rust
async fn create_post(
    Form(raw): Form<RawNewPost>,
    State(db): State<PgPool>,
) -> AppResult<Redirect> {
    let new_post: NewPost = raw.try_into()?;
    // From here down, every field is valid. No defensive code.
}
```

---

## 8. Auth via typestate

The key idea: if a handler *requires* auth, encode that in its signature. Don't do `if session.is_authenticated()` in the handler body.

```rust
// src/extract/auth.rs
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    response::Redirect,
};
use crate::{domain::user::User, db::Db};

pub struct RequireAuth(pub User);

impl FromRequestParts<Db> for RequireAuth {
    type Rejection = Redirect;

    async fn from_request_parts(parts: &mut Parts, db: &Db) -> Result<Self, Self::Rejection> {
        let session_id = extract_session_cookie(parts).ok_or_else(|| Redirect::to("/login"))?;
        let user = db.user_by_session(session_id).await
            .map_err(|_| Redirect::to("/login"))?
            .ok_or_else(|| Redirect::to("/login"))?;
        Ok(Self(user))
    }
}

// Optional version for handlers that adapt to logged-in vs. anonymous
pub struct MaybeAuth(pub Option<User>);

impl FromRequestParts<Db> for MaybeAuth { /* similar, never rejects */ }
```

Now:

```rust
// Can only run for logged-in users — compiler-enforced
async fn create_post(RequireAuth(user): RequireAuth, ...) -> AppResult<Redirect> { ... }

// Adapts to either state
async fn show_post(MaybeAuth(viewer): MaybeAuth, ...) -> AppResult<PostTemplate> { ... }
```

You never need a test called `test_create_post_redirects_when_not_logged_in` — that behavior is *structurally* guaranteed by the extractor. One test of the extractor covers every handler that uses it.

**For role-based access**, build more specific extractors:

```rust
pub struct RequireAdmin(pub User);

impl FromRequestParts<Db> for RequireAdmin {
    type Rejection = AppError;
    async fn from_request_parts(parts: &mut Parts, db: &Db) -> Result<Self, Self::Rejection> {
        let RequireAuth(user) = RequireAuth::from_request_parts(parts, db).await
            .map_err(|_| AppError::Unauthorized)?;
        if !user.is_admin() { return Err(AppError::Forbidden); }
        Ok(Self(user))
    }
}
```

An admin handler signature proves the user is an admin. You cannot accidentally expose an admin-only handler by forgetting a check.

---

## 9. SQLx integration

SQLx's `query!` and `query_as!` macros check SQL against your real database at compile time. Combined with our newtypes (which derive `sqlx::Type`), the type safety extends all the way to the rows.

```rust
// src/db/posts.rs
use crate::domain::{ids::{PostId, UserId}, post::Post, slug::Slug};
use sqlx::PgPool;

pub async fn by_slug(db: &PgPool, slug: &Slug) -> sqlx::Result<Option<Post>> {
    sqlx::query_as!(
        Post,
        r#"
        SELECT
            id as "id: PostId",
            author_id as "author_id: UserId",
            slug as "slug!: String",
            title,
            body,
            published_at
        FROM posts
        WHERE slug = $1
        "#,
        slug.as_str()
    )
    .fetch_optional(db)
    .await
}
```

The `"id: PostId"` syntax tells SQLx to decode the column into our newtype. A typo in a column name fails at `cargo build`, not at runtime.

**Guideline:** every function in `db/` takes domain types as input and returns domain types as output. Raw strings and UUIDs never leak into the rest of the app.

---

## 10. Askama patterns

Askama checks templates at compile time. This is huge — a typo in `{{ autor }}` is a build error, not a 500 at 3am.

**Template struct = handler contract.** The struct fields are exactly what the template needs.

```rust
// src/web/templates.rs
use askama::Template;
use crate::domain::{post::Post, user::User};

#[derive(Template)]
#[template(path = "post/show.html")]
pub struct PostShowTemplate<'a> {
    pub post: &'a Post,
    pub viewer: Option<&'a User>,
    pub csrf_token: &'a str,
}

#[derive(Template)]
#[template(path = "post/list.html")]
pub struct PostListTemplate<'a> {
    pub posts: &'a [Post],
    pub viewer: Option<&'a User>,
}
```

**Render directly from handlers.** Askama templates implement `IntoResponse` (with the `with-axum` feature enabled):

```toml
# Cargo.toml
askama = { version = "0.12", features = ["with-axum"] }
askama_axum = "0.4"
```

```rust
async fn show_post(
    SlugPath(slug): SlugPath,
    MaybeAuth(viewer): MaybeAuth,
    State(db): State<PgPool>,
) -> AppResult<PostShowTemplate<'static>> {
    let post = db::posts::by_slug(&db, &slug).await?
        .ok_or(AppError::NotFound)?;
    Ok(PostShowTemplate {
        post: Box::leak(Box::new(post)),      // or restructure to own the data
        viewer: viewer.as_ref(),
        csrf_token: "...",
    })
}
```

In practice you'll usually own data in the template struct rather than borrow — simpler lifetimes:

```rust
#[derive(Template)]
#[template(path = "post/show.html")]
pub struct PostShowTemplate {
    pub post: Post,
    pub viewer: Option<User>,
    pub csrf_token: String,
}
```

**Custom filters** for display concerns keep templates clean:

```rust
// src/web/filters.rs
pub fn markdown(s: &str) -> askama::Result<String> {
    Ok(pulldown_cmark::html::push_html(/* ... */))
}

pub fn relative_time(dt: &chrono::DateTime<chrono::Utc>) -> askama::Result<String> {
    // "3 hours ago"
}
```

Usage in template: `{{ post.body|markdown|safe }}`.

**Guideline:** never put business logic in templates. If a template needs a derived value, compute it in Rust and add it to the template struct. Templates are for layout.

---

## 11. Forms and CSRF

Use a typed form wrapper plus a CSRF-aware extractor.

```rust
pub struct CsrfProtected<T>(pub T);

impl<T, S> FromRequest<S> for CsrfProtected<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Verify CSRF token from form field or header against session
        // Then deserialize body
    }
}
```

Handlers that mutate state take `CsrfProtected<T>`. Forgetting CSRF on a mutating endpoint becomes a compile-level convention: `POST` handlers don't accept bare `Form<T>`.

---

## 12. Configuration

Load once at startup, into a strongly-typed struct. Fail fast on missing values.

```rust
// src/config.rs
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: std::net::SocketAddr,
    pub session_secret: SessionSecret,   // newtype, not String
    pub environment: Environment,        // enum: Development | Production
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")?,
            bind_addr: std::env::var("BIND_ADDR")?.parse()?,
            session_secret: std::env::var("SESSION_SECRET")?.parse()?,
            environment: std::env::var("ENVIRONMENT")?.parse()?,
        })
    }
}
```

No global `static` config, no `OnceCell`. Pass `Arc<Config>` via `State`. Testable, and no hidden dependencies.

---

## 13. Testing strategy — what's left to test

With the patterns above, your tests collapse into a small set of high-leverage buckets:

### Unit tests (fast, run on every save)

- **Domain type parsers.** Exhaustive: every error variant, boundary conditions. One file per type.
- **Pure business logic.** Functions in `domain/` that compute things. No mocking needed because no I/O.
- **Custom filters.** One test per filter.

### Integration tests (run in CI, against a real Postgres)

- **One happy-path test per route.** Route exists, returns 200, renders the expected template.
- **One auth test per protected route.** Anonymous gets redirected/403.
- **DB queries.** One test per query against a real schema — catches SQL drift. SQLx macros already catch syntax and types; this catches semantics.
- **Form submission round-trip.** POST a form, expect a redirect, follow it, see the new state.

### What you *don't* need tests for

- Validation logic inside handlers → the extractor or domain type already guarantees it.
- Template field existence → `cargo build` catches it.
- "What if the user is logged out" for every handler → the extractor is tested once.
- ID mix-ups → won't compile.
- Nonsense state combinations → won't compile.

### Property tests where they pay

- Round-trip serialization (`Slug -> String -> Slug`).
- Invariants of pure business logic (e.g., `apply_discount` never produces negative money).

Use `proptest` or `quickcheck`. One property test typically replaces a dozen example-based tests.

---

## 14. Anti-patterns

Avoid these — they undo the leverage of everything above.

### `String` creeping through the app

If you're holding a `String` three layers deep and it represents something specific (an email, a slug, a currency code), wrap it at the boundary.

### `anyhow::Error` in domain code

`anyhow` is fine at the top of `main.rs` and in `AppError::Other`. It is *not* fine as the error type of a domain function — it discards the information you need to respond appropriately.

### `unwrap()` outside of `main.rs` and tests

Every `unwrap` is an unhandled state. Use `?` and propagate, or match and handle.

### Validation inside handlers

```rust
// Bad
async fn create_post(Form(f): Form<Raw>) -> AppResult<Redirect> {
    if f.title.is_empty() { return Err(AppError::Validation("...".into())); }
    if f.slug.len() > 100 { return Err(AppError::Validation("...".into())); }
    // ... 40 more lines of checks
}

// Good
async fn create_post(Form(raw): Form<Raw>) -> AppResult<Redirect> {
    let new_post: NewPost = raw.try_into()?;
    // Validation is done. Move on.
}
```

### `Option<T>` where a sum type is clearer

If two fields are `Option` and their presence is correlated, that's a sum type.

### Logic in templates

Anything beyond `if`, `for`, and simple field access belongs in Rust. Templates that compute are templates that break silently.

### Global mutable state

No `lazy_static!` mutable things, no `static mut`. Pass state via `State(...)` extractors. Your tests will thank you.

### Reaching for mocks

If testing a function requires a mock, the function is probably doing too much. Split the pure logic from the I/O, test the pure part directly, and integration-test the I/O part once.

---

## 15. Checklist for new features

When adding a feature, walk this list:

- [ ] Are all IDs newtypes?
- [ ] Do all inputs have a dedicated domain type (not `String`)?
- [ ] Is the state modeled as a sum type where possible?
- [ ] Are handler signatures proving their preconditions (auth, validation)?
- [ ] Does the Askama template struct match exactly what the template uses?
- [ ] Do DB queries return domain types, not raw rows?
- [ ] Are errors typed, not `anyhow` all the way down?
- [ ] Is the happy path covered by one integration test?
- [ ] Is the auth/authorization path covered by one integration test?
- [ ] Are domain types covered by exhaustive unit tests?

If yes to all of these, you can ship with confidence and a test suite a fraction of what a loosely-typed equivalent would need.

---

## Further reading

- "Parse, don't validate" — Alexis King.
- "Making illegal states unrepresentable" — Yaron Minsky.
- "Type-Driven Development" — Edwin Brady.
- Rust API Guidelines (Rust project): naming, trait impls, documentation conventions.
- The `axum` examples repo: excellent source for extractor patterns.

---

*Living document. Update it when you find a pattern that pays off, and delete guidance that stops paying off.*
