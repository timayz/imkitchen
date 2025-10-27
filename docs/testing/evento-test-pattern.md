# Evento Test Pattern Guide

## Overview

This guide explains the recommended testing pattern for routes that use **evento** (event sourcing framework) in the imkitchen project. Understanding this pattern is critical for writing correct integration tests that verify both evento event emission and read model projection.

---

## The Problem: Asynchronous Event Processing

In production, evento processes events asynchronously:

1. **Route handler emits event** → Event saved to event store
2. **Projection subscribes to events** → Runs asynchronously in background
3. **Read model updated** → Database tables updated after projection completes

This asynchronous behavior creates a **race condition in tests**: if you query the read model immediately after the route handler returns, the projection may not have run yet, causing test assertions to fail.

---

## The Solution: `unsafe_oneshot`

Evento provides a special test-only method called **`unsafe_oneshot`** that processes all pending events **synchronously** before returning control to the test.

### Production Code (uses `run`)

```rust
// src/main.rs - Production setup
#[tokio::main]
async fn main() {
    let pool = create_pool().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Projections run asynchronously in background
    user_projection(pool.clone())
        .run(&executor)  // <-- Asynchronous processing
        .await
        .unwrap();

    meal_planning::meal_plan_projection(pool.clone())
        .run(&executor)  // <-- Asynchronous processing
        .await
        .unwrap();

    // Start Axum server...
}
```

### Test Code (uses `unsafe_oneshot`)

```rust
// tests/integration_test.rs - Test setup
#[tokio::test]
async fn test_route_with_evento_projection() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // Call route handler (emits evento event)
    let app = create_test_app(pool.clone(), executor.clone());
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Process projections SYNCHRONOUSLY using unsafe_oneshot
    user_projection(pool.clone())
        .unsafe_oneshot(&executor)  // <-- Synchronous processing (test only)
        .await
        .unwrap();

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)  // <-- Synchronous processing (test only)
        .await
        .unwrap();

    // NOW it's safe to assert read model updates
    let meal_plans = query_meal_plans(&pool, user_id).await;
    assert!(!meal_plans.is_empty(), "Meal plans should exist in read model");
}
```

---

## Why `unsafe_oneshot`?

### Name Breakdown

- **`unsafe`**: Signals that this method should **NEVER** be used in production code
  - Processes events synchronously (blocks event queue)
  - Bypasses evento's async event processing guarantees
  - Only safe in isolated test environments with in-memory databases

- **`oneshot`**: Processes all pending events **once** and returns
  - Not a continuous subscription (unlike `run` which loops forever)
  - Drains the event queue, then stops

### Benefits for Testing

1. **Deterministic timing**: Events processed before test assertions
2. **No race conditions**: Read models guaranteed to be up-to-date
3. **Synchronous flow**: Test logic is easy to follow (no sleeps or retries)

---

## Complete Example: Multi-Week Meal Plan Generation Test

```rust
use axum::{body::Body, extract::Request, http::StatusCode};
use evento::migrator::{Migrate, Plan};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[tokio::test]
async fn test_generate_multi_week_meal_plan() {
    // =====================
    // STEP 1: Setup Test Database
    // =====================
    let pool = create_test_db().await;  // In-memory SQLite
    let executor: evento::Sqlite = pool.clone().into();

    // Create test user using evento
    let user_id = "test_user_1";
    let event_data = user::events::UserCreated {
        email: "test@example.com".to_string(),
        password_hash: "hash".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    let generated_id = evento::create::<user::UserAggregate>()
        .data(&event_data)?
        .metadata(&true)?
        .commit(&executor)
        .await?;

    // Process user projection synchronously (CRITICAL)
    user::user_projection(pool.clone())
        .unsafe_oneshot(&executor)  // <-- Must run BEFORE route test
        .await
        .unwrap();

    // Update user ID for test consistency
    sqlx::query("UPDATE users SET id = ?1 WHERE id = ?2")
        .bind(user_id)
        .bind(&generated_id)
        .execute(&pool)
        .await?;

    // Create test recipes
    create_test_recipes(&pool, user_id, 10).await.unwrap();

    // =====================
    // STEP 2: Call Route Handler
    // =====================
    let app = create_test_app(pool.clone(), executor.clone(), user_id.to_string());

    let request = Request::builder()
        .method("POST")
        .uri("/plan/generate-multi-week")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Route should return 200 OK");

    // Parse response JSON
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body["generation_batch_id"].is_string());
    assert!(body["first_week"].is_object());

    // =====================
    // STEP 3: Process Evento Projections Synchronously
    // =====================
    // CRITICAL: This ensures the MultiWeekMealPlanGenerated event
    // emitted by the route handler has been processed and the
    // read model (meal_plans table) has been updated.
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)  // <-- Synchronous event processing
        .await
        .unwrap();

    // =====================
    // STEP 4: Assert Read Model Updated
    // =====================
    // NOW it's safe to query the read model
    let meal_plan_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM meal_plans WHERE user_id = ?1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(meal_plan_count > 0, "Meal plans should exist in read model");

    let meal_assignment_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM meal_assignments WHERE user_id = ?1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        meal_assignment_count, 105,
        "Should have 105 meal assignments (5 weeks × 21 meals)"
    );
}
```

---

## Common Pitfalls and Solutions

### ❌ Pitfall 1: Forgetting `unsafe_oneshot`

```rust
// BROKEN TEST - Race condition
#[tokio::test]
async fn test_broken() {
    let app = create_test_app(pool.clone(), executor.clone(), user_id);
    let response = app.oneshot(request).await.unwrap();

    // Event emitted but projection NOT processed yet!

    // This assertion WILL FAIL because read model not updated
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meal_plans")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 5);  // <-- FAILS: count is still 0
}
```

**Solution:** Always call `unsafe_oneshot` after route handlers that emit events.

---

### ❌ Pitfall 2: Using `unsafe_oneshot` in Production

```rust
// DANGER - NEVER DO THIS IN PRODUCTION
#[tokio::main]
async fn main() {
    let pool = create_pool().await;
    let executor: evento::Sqlite = pool.clone().into();

    // WRONG: This blocks the event queue synchronously
    user_projection(pool.clone())
        .unsafe_oneshot(&executor)  // <-- NEVER in production!
        .await
        .unwrap();
}
```

**Why is this dangerous?**
- Blocks the entire event processing queue
- Prevents concurrent event handling
- Defeats evento's async event sourcing architecture

**Solution:** Use `.run(&executor)` in production (asynchronous).

---

### ❌ Pitfall 3: Wrong Order (Projection Before Event)

```rust
// BROKEN TEST - Projection runs before event emitted
#[tokio::test]
async fn test_broken_order() {
    let pool = create_test_db().await;
    let executor: evento::Sqlite = pool.clone().into();

    // WRONG ORDER: Running projection before route emits event
    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Route emits event AFTER projection already ran
    let response = app.oneshot(request).await.unwrap();

    // Assertion fails because projection already finished
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meal_plans")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 5);  // <-- FAILS: event not processed
}
```

**Solution:** Call `unsafe_oneshot` **AFTER** the route handler that emits events.

---

## When to Use `unsafe_oneshot`

### ✅ Use in Integration Tests

```rust
#[tokio::test]
async fn test_integration() {
    // Route emits evento event
    let response = app.oneshot(request).await.unwrap();

    // Process projection synchronously
    projection.unsafe_oneshot(&executor).await.unwrap();

    // Assert read model updated
    assert!(!query_result.is_empty());
}
```

### ❌ NEVER Use in Production

```rust
// src/main.rs - Production code
#[tokio::main]
async fn main() {
    // Use .run() for async processing in production
    projection.run(&executor).await.unwrap();
}
```

### ❌ NEVER Use in Unit Tests

Unit tests should mock evento and test handlers in isolation:

```rust
#[tokio::test]
async fn test_unit() {
    // Mock evento executor
    let mock_executor = MockExecutor::new();

    // Test handler logic without real evento
    let result = handler(mock_executor).await;

    assert!(result.is_ok());
}
```

---

## Test Helper Pattern

Create a reusable test helper that handles evento setup:

```rust
// tests/common/mod.rs
pub async fn setup_test_db() -> (SqlitePool, evento::Sqlite) {
    let pool = create_pool(":memory:", 1).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    let executor: evento::Sqlite = pool.clone().into();

    (pool, executor)
}

pub async fn process_events(pool: SqlitePool, executor: &evento::Sqlite) {
    // Process all projections synchronously
    user::user_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    meal_planning::meal_plan_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();

    recipe::recipe_projection(pool.clone())
        .unsafe_oneshot(executor)
        .await
        .unwrap();
}
```

**Usage:**

```rust
#[tokio::test]
async fn test_with_helper() {
    let (pool, executor) = setup_test_db().await;

    // Call route
    let response = app.oneshot(request).await.unwrap();

    // Process all events
    process_events(pool.clone(), &executor).await;

    // Assert results
    assert!(!query_result.is_empty());
}
```

---

## Summary

| Aspect | Production | Tests |
|--------|-----------|-------|
| **Event Processing** | Asynchronous (`.run()`) | Synchronous (`.unsafe_oneshot()`) |
| **Use Case** | Background event handling | Deterministic test assertions |
| **Safety** | Safe for concurrent processing | Only safe in isolated test environments |
| **Order** | N/A (runs continuously) | **MUST** call after route emits events |

**Golden Rule for Tests:**
1. Call route handler (emits evento event)
2. Call `.unsafe_oneshot()` on all relevant projections
3. Assert read model updates

---

## Additional Resources

- **Evento Documentation:** https://docs.rs/evento
- **Integration Test Examples:** See `/tests/week_navigation_integration_tests.rs`
- **Test Helper Module:** See `/tests/common/mod.rs`
- **Technical Specification:** See `/docs/tech-spec-epic-8.md` (Section: Test Strategy Summary)
