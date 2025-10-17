# Action Plan: Recipe Rotation System Critical Fixes

**Generated:** 2025-10-17
**Story:** 3.3 - Recipe Rotation System
**Status:** Post-Review Improvements
**Estimated Total Effort:** 15-20 hours

---

## Executive Summary

This action plan addresses the 4 critical and 8 major issues identified in the code review of the Recipe Rotation System. Tasks are prioritized by risk impact and organized into 3 phases for systematic implementation.

**Phase 1 (Critical):** Data integrity and consistency - 8-12 hours
**Phase 2 (Major):** Error handling and validation - 4-6 hours
**Phase 3 (Enhancement):** Testing and optimization - 3-4 hours

---

## Phase 1: Critical Data Integrity Fixes (Priority: HIGH)

### Task 1.1: Add Database Transactions to Projection Handlers

**Issue:** Critical Issue #2 - Data consistency between JSON and table storage
**Impact:** HIGH - Data corruption risk on failures
**Effort:** 4-6 hours
**Files:** `crates/meal_planning/src/read_model.rs`

#### Implementation Steps:

1. **Update `meal_plan_generated_handler` (lines 322-377)**

```rust
#[evento::handler(MealPlanAggregate)]
pub async fn meal_plan_generated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Idempotency check FIRST
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM meal_plans WHERE id = ?1"
    )
    .bind(&event.aggregator_id)
    .fetch_optional(&pool)
    .await?;

    if exists.is_some() {
        return Ok(()); // Already processed
    }

    // START TRANSACTION
    let mut tx = pool.begin().await?;

    // Archive existing active meal plans
    sqlx::query(
        r#"
        UPDATE meal_plans
        SET status = 'archived'
        WHERE user_id = ?1 AND status = 'active'
        "#,
    )
    .bind(&event.data.user_id)
    .execute(&mut *tx)
    .await?;

    // Insert meal plan
    sqlx::query(
        r#"
        INSERT INTO meal_plans (id, user_id, start_date, status, rotation_state, created_at)
        VALUES (?1, ?2, ?3, 'active', ?4, ?5)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.start_date)
    .bind(&event.data.rotation_state_json)
    .bind(&event.data.generated_at)
    .execute(&mut *tx)
    .await?;

    // Insert meal assignments
    for assignment in &event.data.meal_assignments {
        let assignment_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(assignment_id)
        .bind(&event.aggregator_id)
        .bind(&assignment.date)
        .bind(assignment.meal_type.as_str())
        .bind(&assignment.recipe_id)
        .bind(assignment.prep_required)
        .execute(&mut *tx)
        .await?;
    }

    // COMMIT TRANSACTION
    tx.commit().await?;
    Ok(())
}
```

2. **Update `recipe_used_in_rotation_handler` (lines 383-430)**

```rust
#[evento::handler(MealPlanAggregate)]
pub async fn recipe_used_in_rotation_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeUsedInRotation>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Get user_id from meal plan
    let user_id: Option<(String,)> = sqlx::query_as(
        "SELECT user_id FROM meal_plans WHERE id = ?1"
    )
    .bind(&event.aggregator_id)
    .fetch_optional(&pool)
    .await?;

    let user_id = match user_id {
        Some((uid,)) => uid,
        None => {
            // Meal plan not found - likely event ordering issue
            // This is acceptable as evento will retry
            return Ok(());
        }
    };

    // Insert with idempotency via ON CONFLICT
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO recipe_rotation_state (id, user_id, cycle_number, recipe_id, used_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(user_id, cycle_number, recipe_id) DO NOTHING
        "#,
    )
    .bind(id)
    .bind(&user_id)
    .bind(event.data.cycle_number as i64)
    .bind(&event.data.recipe_id)
    .bind(&event.data.used_at)
    .execute(&pool)
    .await?;

    Ok(())
}
```

3. **Update `rotation_cycle_reset_handler` (lines 438-454)**

```rust
#[evento::handler(MealPlanAggregate)]
pub async fn rotation_cycle_reset_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RotationCycleReset>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Delete all rotation state entries for the old cycle
    // This is idempotent - safe to replay
    sqlx::query(
        r#"
        DELETE FROM recipe_rotation_state
        WHERE user_id = ?1 AND cycle_number = ?2
        "#,
    )
    .bind(&event.data.user_id)
    .bind(event.data.old_cycle_number as i64)
    .execute(&pool)
    .await?;

    Ok(())
}
```

#### Testing:

```rust
// Add to crates/meal_planning/tests/projection_tests.rs (NEW FILE)
#[sqlx::test]
async fn test_meal_plan_generated_transaction_rollback(pool: SqlitePool) {
    // Simulate failure midway through projection
    // Verify NO partial data written to database
}

#[sqlx::test]
async fn test_meal_plan_generated_idempotency(pool: SqlitePool) {
    // Emit same event twice
    // Verify only one meal plan created
}
```

#### Acceptance Criteria:
- [ ] All projection handlers use transactions
- [ ] Idempotency checks prevent duplicate processing
- [ ] Tests verify transaction rollback on failure
- [ ] No partial data written on error

---

### Task 1.2: Fix Silent Error Handling in Aggregate

**Issue:** Critical Issue #1 - Silent failure in rotation_cycle_reset handler
**Impact:** HIGH - State corruption on malformed JSON
**Effort:** 1-2 hours
**Files:** `crates/meal_planning/src/aggregate.rs`

#### Implementation Steps:

1. **Update `rotation_cycle_reset` (lines 106-121)**

```rust
async fn rotation_cycle_reset(
    &mut self,
    event: evento::EventDetails<RotationCycleReset>,
) -> anyhow::Result<()> {
    use crate::rotation::RotationState;

    // Parse rotation state with explicit error handling
    let mut rotation_state = RotationState::from_json(&self.rotation_state_json)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse rotation state for meal_plan_id={}: {}",
                self.meal_plan_id,
                e
            )
        })?;

    // Reset the cycle
    rotation_state.reset_cycle();
    rotation_state.total_favorite_count = event.data.favorite_count;

    // Update aggregate state
    self.rotation_state_json = rotation_state.to_json()?;

    Ok(())
}
```

2. **Add validation to `meal_plan_generated` (lines 72-85)**

```rust
async fn meal_plan_generated(
    &mut self,
    event: evento::EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    // Validate rotation state JSON is parseable
    let _rotation_state = RotationState::from_json(&event.data.rotation_state_json)
        .map_err(|e| {
            anyhow::anyhow!("Invalid rotation state in MealPlanGenerated event: {}", e)
        })?;

    self.meal_plan_id = event.aggregator_id.clone();
    self.user_id = event.data.user_id;
    self.start_date = event.data.start_date;
    self.meal_assignments = event.data.meal_assignments;
    self.rotation_state_json = event.data.rotation_state_json;
    self.created_at = event.data.generated_at.clone();
    self.status = MealPlanStatus::Active.as_str().to_string();
    self.archived_at = None;
    Ok(())
}
```

#### Testing:

```rust
// Add to crates/meal_planning/tests/aggregate_tests.rs (NEW FILE)
#[tokio::test]
async fn test_rotation_cycle_reset_invalid_json() {
    let mut aggregate = MealPlanAggregate::default();
    aggregate.rotation_state_json = "{invalid json}".to_string();

    let event = create_rotation_cycle_reset_event();
    let result = aggregate.rotation_cycle_reset(event).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse"));
}
```

#### Acceptance Criteria:
- [ ] All JSON parsing uses explicit error handling
- [ ] Errors include context (meal_plan_id, etc.)
- [ ] Tests verify error propagation
- [ ] No silent failures in aggregate handlers

---

### Task 1.3: Emit RotationCycleReset Event

**Issue:** Major Issue #5 - Missing event emission when cycle resets
**Impact:** HIGH - No audit trail for cycle resets
**Effort:** 2-3 hours
**Files:** `src/routes/meal_plan.rs`

#### Implementation Steps:

1. **Add cycle reset detection (after line 175)**

```rust
// File: src/routes/meal_plan.rs
// After line 175 (after algorithm generates meal plan)

let (meal_assignments, updated_rotation_state) = MealPlanningAlgorithm::generate(
    &start_date,
    recipes_for_planning,
    constraints,
    rotation_state.clone(), // Clone to compare later
    None,
)?;

// Detect if cycle was reset during generation
let cycle_reset_occurred = updated_rotation_state.cycle_number > rotation_state.cycle_number;
let old_cycle_number = rotation_state.cycle_number;
```

2. **Emit RotationCycleReset event (after RecipeUsedInRotation events, around line 265)**

```rust
// After emitting all RecipeUsedInRotation events

// If cycle was reset during generation, emit RotationCycleReset event
if cycle_reset_occurred {
    use meal_planning::events::RotationCycleReset;

    let reset_event = RotationCycleReset {
        user_id: auth.user_id.clone(),
        old_cycle_number,
        new_cycle_number: updated_rotation_state.cycle_number,
        favorite_count: recipes_for_planning.len(),
        reset_at: now.clone(),
    };

    tracing::info!(
        "Rotation cycle reset: {} -> {} for user {}",
        old_cycle_number,
        updated_rotation_state.cycle_number,
        auth.user_id
    );

    evento::save::<meal_planning::MealPlanAggregate>(&meal_plan_id)
        .data(&reset_event)
        .map_err(|e| {
            tracing::error!("Failed to encode RotationCycleReset event: {:?}", e);
            anyhow::anyhow!("Failed to encode reset event: {}", e)
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!("Failed to encode metadata: {:?}", e);
            anyhow::anyhow!("Failed to encode metadata: {}", e)
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!("Failed to commit reset event: {:?}", e);
            anyhow::anyhow!("Failed to commit reset event: {}", e)
        })?;
}
```

#### Testing:

```rust
// Add to integration test
#[sqlx::test]
async fn test_rotation_cycle_reset_event_emitted(pool: SqlitePool) {
    // Generate meal plans until all favorites used
    // Verify RotationCycleReset event emitted
    // Verify recipe_rotation_state table cleared for old cycle
}
```

#### Acceptance Criteria:
- [ ] RotationCycleReset event emitted when cycle resets
- [ ] Event contains old and new cycle numbers
- [ ] Event logged with tracing::info
- [ ] Projection handler clears old cycle data
- [ ] Tests verify event emission

---

### Task 1.4: Add Race Condition Protection

**Issue:** Critical Issue #4 - Concurrent meal plan generation
**Impact:** HIGH - Duplicate plans, incorrect rotation state
**Effort:** 2-4 hours
**Files:** `src/routes/meal_plan.rs`

#### Implementation Steps:

**Option 1: Database-Level Lock (Recommended)**

```rust
// File: src/routes/meal_plan.rs
// At start of post_generate_meal_plan (after line 117)

pub async fn post_generate_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Acquire advisory lock for this user's meal plan generation
    // This prevents concurrent generation for the same user
    let lock_acquired: (bool,) = sqlx::query_as(
        "SELECT pg_try_advisory_lock(?1::bigint)" // PostgreSQL
        // OR for SQLite: Use application-level semaphore
    )
    .bind(hash_user_id_to_i64(&auth.user_id))
    .fetch_one(&state.db_pool)
    .await?;

    if !lock_acquired.0 {
        return Err(AppError::ConcurrentGenerationInProgress);
    }

    // Ensure lock is released on function exit
    let _lock_guard = MealPlanGenerationLock {
        user_id: auth.user_id.clone(),
        pool: state.db_pool.clone(),
    };

    // ... rest of function
}

// Lock guard to ensure release
struct MealPlanGenerationLock {
    user_id: String,
    pool: SqlitePool,
}

impl Drop for MealPlanGenerationLock {
    fn drop(&mut self) {
        // Release advisory lock
        // Note: In production, use tokio spawn to handle async drop
    }
}
```

**Option 2: Application-Level Lock (Simpler for SQLite)**

```rust
// File: src/lib.rs - Add to AppState
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct AppState {
    pub db_pool: SqlitePool,
    pub evento_executor: evento::Sqlite,
    pub generation_locks: Arc<Mutex<HashMap<String, ()>>>, // NEW
}

// File: src/routes/meal_plan.rs
pub async fn post_generate_meal_plan(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // Try to acquire lock for this user
    let mut locks = state.generation_locks.lock().await;

    if locks.contains_key(&auth.user_id) {
        return Err(AppError::ConcurrentGenerationInProgress);
    }

    // Insert lock
    locks.insert(auth.user_id.clone(), ());
    drop(locks); // Release mutex

    // Ensure lock is removed on exit
    let _guard = GenerationLockGuard {
        user_id: auth.user_id.clone(),
        locks: state.generation_locks.clone(),
    };

    // ... rest of function
}

struct GenerationLockGuard {
    user_id: String,
    locks: Arc<Mutex<HashMap<String, ()>>>,
}

impl Drop for GenerationLockGuard {
    fn drop(&mut self) {
        let user_id = self.user_id.clone();
        let locks = self.locks.clone();
        tokio::spawn(async move {
            locks.lock().await.remove(&user_id);
        });
    }
}
```

3. **Add error variant to AppError**

```rust
// File: src/error.rs
#[derive(Debug)]
pub enum AppError {
    // ... existing variants

    #[error("Meal plan generation already in progress. Please wait.")]
    ConcurrentGenerationInProgress,
}
```

#### Testing:

```rust
// Add integration test
#[tokio::test]
async fn test_concurrent_generation_prevented() {
    // Spawn two concurrent generation requests for same user
    // Verify only one succeeds
    // Verify second returns ConcurrentGenerationInProgress error
}
```

#### Acceptance Criteria:
- [ ] Concurrent generation for same user prevented
- [ ] Lock automatically released on success or failure
- [ ] Clear error message to user
- [ ] Tests verify lock behavior
- [ ] No deadlocks possible

---

## Phase 2: Major Error Handling & Validation (Priority: MEDIUM)

### Task 2.1: Add Validation to RotationState Constructor

**Issue:** Major Issue #1 - No validation in constructor
**Impact:** MEDIUM - Logic errors with zero favorites
**Effort:** 1-2 hours
**Files:** `crates/meal_planning/src/rotation.rs`

#### Implementation Steps:

```rust
// File: crates/meal_planning/src/rotation.rs
// Update constructor (lines 30-38)

impl RotationState {
    /// Create new rotation state with specified favorite count
    ///
    /// # Errors
    /// Returns error if total_favorite_count is 0
    pub fn with_favorite_count(total_favorite_count: usize) -> Result<Self, String> {
        if total_favorite_count == 0 {
            return Err("total_favorite_count must be greater than 0".to_string());
        }

        Ok(RotationState {
            cycle_number: 1,
            cycle_started_at: chrono::Utc::now().to_rfc3339(),
            used_recipe_ids: HashSet::new(),
            total_favorite_count,
        })
    }

    /// Reset cycle with overflow protection
    pub fn reset_cycle(&mut self) {
        self.cycle_number = self.cycle_number.saturating_add(1);
        self.cycle_started_at = chrono::Utc::now().to_rfc3339();
        self.used_recipe_ids.clear();
    }
}
```

#### Update Callers:

```rust
// File: src/routes/meal_plan.rs (line 158)
let mut rotation_state = match previous_meal_plan {
    Some(plan) => RotationState::from_json(&plan.rotation_state).unwrap_or_default(),
    None => RotationState::with_favorite_count(recipes_for_planning.len())
        .map_err(|e| AppError::InsufficientRecipes {
            current: 0,
            required: 1
        })?,
};
```

#### Testing:

```rust
#[test]
fn test_rotation_state_rejects_zero_favorites() {
    let result = RotationState::with_favorite_count(0);
    assert!(result.is_err());
}

#[test]
fn test_cycle_number_saturating_add() {
    let mut state = RotationState::with_favorite_count(5).unwrap();
    state.cycle_number = u32::MAX;
    state.reset_cycle();
    assert_eq!(state.cycle_number, u32::MAX); // Doesn't overflow
}
```

#### Acceptance Criteria:
- [ ] Constructor returns Result<T, E>
- [ ] Validates total_favorite_count > 0
- [ ] Cycle number uses saturating_add
- [ ] Tests verify validation
- [ ] All callers updated

---

### Task 2.2: Replace unwrap() Calls

**Issue:** Major Issue #2 - Multiple unwrap() in production
**Impact:** MEDIUM - Potential panics
**Effort:** 2-3 hours
**Files:** `src/routes/meal_plan.rs`

#### Implementation Steps:

1. **Fix template rendering (lines 87, 97)**

```rust
// Before:
Ok(Html(template.render().unwrap()))

// After:
Ok(Html(template.render().map_err(|e| {
    tracing::error!("Failed to render meal calendar template: {:?}", e);
    AppError::TemplateRenderError(e.to_string())
})?))
```

2. **Fix JSON parsing (lines 132-135)**

```rust
// Before:
let ingredients: Vec<serde_json::Value> =
    serde_json::from_str(&r.ingredients).unwrap_or_default();

// After:
let ingredients: Vec<serde_json::Value> =
    serde_json::from_str(&r.ingredients)
        .map_err(|e| {
            tracing::warn!("Failed to parse ingredients for recipe {}: {}", r.id, e);
            e
        })
        .unwrap_or_default();
```

3. **Add AppError variant**

```rust
// File: src/error.rs
#[error("Failed to render template: {0}")]
TemplateRenderError(String),
```

#### Acceptance Criteria:
- [ ] No unwrap() or expect() in production routes
- [ ] All failures logged with context
- [ ] Proper error types returned
- [ ] User sees meaningful error messages

---

### Task 2.3: Update meal_replaced Handler

**Issue:** Major Issue #7 - meal_replaced doesn't update rotation
**Impact:** MEDIUM - Rotation tracking incomplete
**Effort:** 2-3 hours
**Files:** `crates/meal_planning/src/aggregate.rs`, `src/routes/meal_plan.rs`

#### Implementation Steps:

1. **Update aggregate handler (lines 136-153)**

```rust
async fn meal_replaced(
    &mut self,
    event: evento::EventDetails<MealReplaced>,
) -> anyhow::Result<()> {
    use crate::rotation::RotationState;

    // Find and update the meal assignment
    if let Some(assignment) = self
        .meal_assignments
        .iter_mut()
        .find(|a| a.date == event.data.date && a.meal_type == event.data.meal_type)
    {
        let old_recipe_id = assignment.recipe_id.clone();
        assignment.recipe_id = event.data.new_recipe_id.clone();

        // Update rotation state
        let mut rotation_state = RotationState::from_json(&self.rotation_state_json)?;

        // Mark new recipe as used
        rotation_state.mark_recipe_used(event.data.new_recipe_id.clone());

        // Remove old recipe from used set (make it available again)
        rotation_state.used_recipe_ids.remove(&old_recipe_id);

        // Save updated rotation state
        self.rotation_state_json = rotation_state.to_json()?;
    }

    Ok(())
}
```

2. **Update MealReplaced event to include cycle_number**

```rust
// File: crates/meal_planning/src/events.rs
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealReplaced {
    pub date: String,
    pub meal_type: String,
    pub old_recipe_id: String, // NEW: track what was replaced
    pub new_recipe_id: String,
    pub cycle_number: u32, // NEW: rotation cycle
    pub replaced_at: String,
}
```

#### Testing:

```rust
#[tokio::test]
async fn test_meal_replaced_updates_rotation() {
    let mut aggregate = create_meal_plan_aggregate();

    // Replace recipe
    let event = create_meal_replaced_event("recipe_1", "recipe_2");
    aggregate.meal_replaced(event).await.unwrap();

    // Verify rotation state updated
    let rotation = RotationState::from_json(&aggregate.rotation_state_json).unwrap();
    assert!(rotation.is_recipe_used("recipe_2"));
    assert!(!rotation.is_recipe_used("recipe_1"));
}
```

#### Acceptance Criteria:
- [ ] meal_replaced updates rotation state
- [ ] Old recipe marked available
- [ ] New recipe marked used
- [ ] Tests verify rotation updates
- [ ] Event includes rotation context

---

## Phase 3: Testing & Documentation (Priority: LOW)

### Task 3.1: Add Database Integration Tests

**Issue:** Major Issue #8 - No real database integration tests
**Impact:** MEDIUM - Untested projection behavior
**Effort:** 4-6 hours
**Files:** `crates/meal_planning/tests/` (new files)

#### Implementation Steps:

1. **Create `projection_integration_tests.rs`**

```rust
// File: crates/meal_planning/tests/projection_integration_tests.rs

use sqlx::SqlitePool;
use meal_planning::*;

#[sqlx::test]
async fn test_meal_plan_generated_creates_plan_and_assignments(pool: SqlitePool) {
    // Setup: Run migrations
    // Create evento executor
    // Emit MealPlanGenerated event
    // Run projection handler

    // Verify meal_plans table has row
    // Verify meal_assignments table has 21 rows
    // Verify rotation_state JSON is correct
}

#[sqlx::test]
async fn test_recipe_used_in_rotation_creates_state_rows(pool: SqlitePool) {
    // Emit RecipeUsedInRotation events
    // Run projection handler

    // Verify recipe_rotation_state has correct rows
    // Verify unique constraint works
    // Verify idempotency (emit same event twice)
}

#[sqlx::test]
async fn test_rotation_cycle_reset_clears_old_cycle(pool: SqlitePool) {
    // Setup: Create rotation state for cycle 1
    // Emit RotationCycleReset event for cycle 2
    // Run projection handler

    // Verify cycle 1 rows deleted
    // Verify cycle 2 can start fresh
}

#[sqlx::test]
async fn test_concurrent_projection_handlers_safe(pool: SqlitePool) {
    // Emit multiple events concurrently
    // Run projection handlers in parallel

    // Verify no deadlocks
    // Verify correct final state
    // Verify no race conditions
}

#[sqlx::test]
async fn test_projection_handler_idempotency(pool: SqlitePool) {
    // Emit same event multiple times
    // Run projection handler each time

    // Verify only one row created
    // Verify no errors on replay
}
```

2. **Create `end_to_end_rotation_tests.rs`**

```rust
// File: crates/meal_planning/tests/end_to_end_rotation_tests.rs

#[sqlx::test]
async fn test_full_rotation_cycle_e2e(pool: SqlitePool) {
    // 1. Generate initial meal plan (7 recipes used)
    // 2. Verify recipe_rotation_state has 7 rows
    // 3. Generate second plan (7 more recipes, 14 total)
    // 4. Generate third plan (cycle resets, 7 recipes reused)
    // 5. Verify RotationCycleReset event emitted
    // 6. Verify old cycle data cleared
    // 7. Verify new cycle starts fresh
}

#[sqlx::test]
async fn test_query_methods_return_correct_data(pool: SqlitePool) {
    // Setup: Create meal plans and rotation state

    // Test query_rotation_state
    // Test query_available_recipes_for_rotation
    // Test query_replacement_candidates
    // Test query_rotation_progress

    // Verify all queries return expected data
}
```

3. **Setup test infrastructure**

```rust
// File: crates/meal_planning/tests/common/mod.rs (NEW)

use sqlx::SqlitePool;
use meal_planning::*;

pub async fn setup_test_db() -> SqlitePool {
    // Create in-memory SQLite database
    // Run migrations
    // Return pool
}

pub async fn create_test_evento_executor(pool: SqlitePool) -> evento::Sqlite {
    // Setup evento with test database
}

pub fn create_test_meal_plan_event() -> MealPlanGenerated {
    // Factory for test events
}

pub fn create_test_recipes(count: usize) -> Vec<RecipeForPlanning> {
    // Factory for test recipes
}
```

#### Acceptance Criteria:
- [ ] At least 8 database integration tests added
- [ ] Tests use real SQLite database
- [ ] Tests use real evento executor
- [ ] All projection handlers tested
- [ ] All query methods tested
- [ ] Concurrency scenarios tested
- [ ] All tests pass

---

### Task 3.2: Add Missing Edge Case Tests

**Issue:** Minor Issue #5 - Missing negative tests
**Impact:** LOW - Untested error paths
**Effort:** 1-2 hours
**Files:** `crates/meal_planning/tests/rotation_integration_tests.rs`

#### Implementation Steps:

```rust
// Add to existing rotation_integration_tests.rs

#[test]
fn test_rotation_state_invalid_json_deserialization() {
    let invalid_json = "{broken json}";
    let result = RotationState::from_json(invalid_json);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("JSON"));
}

#[test]
fn test_rotation_state_corrupted_data() {
    let corrupted = r#"{"cycle_number": -1, "used_recipe_ids": null}"#;
    let result = RotationState::from_json(corrupted);
    // Should handle gracefully
}

#[test]
fn test_rotation_state_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let state = Arc::new(std::sync::Mutex::new(
        RotationState::with_favorite_count(100).unwrap()
    ));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let state = state.clone();
            thread::spawn(move || {
                let mut s = state.lock().unwrap();
                s.mark_recipe_used(format!("recipe_{}", i));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let final_state = state.lock().unwrap();
    assert_eq!(final_state.used_count(), 10);
}

#[test]
fn test_favorite_count_decrease_edge_case() {
    let mut state = RotationState::with_favorite_count(20).unwrap();

    // Use 15 recipes
    for i in 0..15 {
        state.mark_recipe_used(format!("recipe_{}", i));
    }

    // User un-favorites 10 recipes (now only 10 favorites)
    state.total_favorite_count = 10;

    // Should trigger reset since used (15) > total (10)
    assert!(state.should_reset_cycle());
}
```

#### Acceptance Criteria:
- [ ] Tests for invalid JSON
- [ ] Tests for corrupted state
- [ ] Tests for thread safety
- [ ] Tests for favorite count decrease
- [ ] All negative tests pass

---

### Task 3.3: Add Foreign Key Cascade

**Issue:** Minor Issue #7 - No cascade handling
**Impact:** LOW - Orphaned records on recipe deletion
**Effort:** 30 minutes
**Files:** `migrations/03_recipe_rotation_state.sql`

#### Implementation Steps:

```sql
-- File: migrations/03_recipe_rotation_state.sql
-- Update foreign key constraint (line 13)

CREATE TABLE IF NOT EXISTS recipe_rotation_state (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL, -- RFC3339 formatted timestamp

    -- Foreign keys with proper cascade behavior
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,

    -- Unique constraint: one entry per (user, cycle, recipe)
    UNIQUE (user_id, cycle_number, recipe_id)
);
```

**Note:** This requires creating a new migration file since existing migration already applied.

```sql
-- File: migrations/04_add_cascade_to_rotation_state.sql (NEW)

-- Drop existing foreign keys
-- SQLite doesn't support DROP CONSTRAINT, so recreate table

-- Create new table with cascades
CREATE TABLE IF NOT EXISTS recipe_rotation_state_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    UNIQUE (user_id, cycle_number, recipe_id)
);

-- Copy data from old table
INSERT INTO recipe_rotation_state_new
SELECT * FROM recipe_rotation_state;

-- Drop old table
DROP TABLE recipe_rotation_state;

-- Rename new table
ALTER TABLE recipe_rotation_state_new RENAME TO recipe_rotation_state;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_cycle
    ON recipe_rotation_state(user_id, cycle_number);
CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_id
    ON recipe_rotation_state(user_id);
```

#### Testing:

```rust
#[sqlx::test]
async fn test_recipe_deletion_cascades_to_rotation_state(pool: SqlitePool) {
    // Insert recipe and rotation state
    // Delete recipe
    // Verify rotation state row also deleted
}
```

#### Acceptance Criteria:
- [ ] Migration adds CASCADE to foreign keys
- [ ] Existing data preserved
- [ ] Tests verify cascade behavior
- [ ] No orphaned records

---

## Implementation Order & Timeline

### Week 1: Critical Fixes (HIGH Priority)
**Days 1-2:** Task 1.1 - Database Transactions (4-6 hours)
**Day 3:** Task 1.2 - Fix Silent Errors (1-2 hours)
**Day 3:** Task 1.3 - Emit Cycle Reset Event (2-3 hours)
**Day 4:** Task 1.4 - Race Condition Protection (2-4 hours)

**Checkpoint:** All critical data integrity issues resolved

### Week 2: Major Fixes (MEDIUM Priority)
**Day 5:** Task 2.1 - Validation (1-2 hours)
**Day 5:** Task 2.2 - Remove unwrap() (2-3 hours)
**Day 6:** Task 2.3 - Update meal_replaced (2-3 hours)

**Checkpoint:** All error handling improved

### Week 3: Testing & Polish (LOW Priority)
**Days 7-8:** Task 3.1 - Integration Tests (4-6 hours)
**Day 9:** Task 3.2 - Edge Case Tests (1-2 hours)
**Day 9:** Task 3.3 - Foreign Key Cascade (30 min)

**Final Checkpoint:** Production-ready

---

## Success Metrics

### Code Quality Metrics
- [ ] Zero unwrap() or expect() in production code
- [ ] Zero silent error handling (if let Ok without else)
- [ ] All projection handlers use transactions
- [ ] All constructors validate inputs
- [ ] 100% of critical paths have error handling

### Testing Metrics
- [ ] +15 integration tests added (8 projection + 7 edge cases)
- [ ] Database integration test coverage >80%
- [ ] All race conditions have tests
- [ ] All error paths have negative tests

### Production Readiness
- [ ] No known critical or major issues
- [ ] All 4 critical issues resolved
- [ ] All 8 major issues resolved or accepted
- [ ] Code review grade improved from B+ to A
- [ ] Ready for production deployment

---

## Rollback Plan

If issues arise during implementation:

1. **Database Migrations:** Each migration has rollback script in separate file
2. **Code Changes:** Use git branches for each task, can revert individually
3. **Testing:** Run full test suite after each task before proceeding
4. **Deployment:** Deploy in phases (Phase 1, then Phase 2, then Phase 3)

---

## Documentation Updates

After completing all tasks, update:

1. **Story 3.3 Documentation** (`docs/stories/story-3.3.md`)
   - Add "Post-Review Fixes" section
   - Document all changes made
   - Update completion notes

2. **Technical Spec** (`docs/tech-spec-epic-3.md`)
   - Update rotation system implementation details
   - Add notes about transaction handling
   - Document concurrency protection

3. **Architecture Docs** (`docs/solution-architecture.md`)
   - Add patterns used (advisory locks, transactions)
   - Document idempotency strategies

4. **README** (if applicable)
   - Update with any new requirements
   - Document concurrency limitations

---

## Questions & Decisions Log

### Q1: Should we use database advisory locks or application-level locks?
**Decision:** Application-level locks (HashMap<String, ()>) for SQLite
**Rationale:** Simpler to implement, adequate for single-server deployment
**Future:** Migrate to database-level locks when scaling horizontally

### Q2: Should RotationState constructor return Result or panic?
**Decision:** Return Result<T, E>
**Rationale:** Allows graceful error handling, better user experience
**Impact:** All callers must handle errors explicitly

### Q3: Should we create new migration or modify existing?
**Decision:** Create new migration (04_add_cascade_to_rotation_state.sql)
**Rationale:** Existing migration already applied in databases
**Impact:** Requires table recreation for SQLite

### Q4: Should meal_replaced remove old recipe from rotation?
**Decision:** Yes, make old recipe available again
**Rationale:** User might want to use it later in the cycle
**Impact:** Requires tracking both old and new recipe IDs in event

---

## Communication Plan

### Daily Standups
- Report progress on current task
- Escalate blockers immediately
- Share learnings with team

### Pull Request Strategy
- One PR per task for easier review
- Include tests in same PR as implementation
- Link to this action plan in PR description

### Stakeholder Updates
- End of Week 1: Critical fixes complete (email update)
- End of Week 2: Major fixes complete (demo session)
- End of Week 3: Full review and production readiness (team presentation)

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Transaction deadlocks | Low | High | Add timeout and retry logic |
| Migration fails on prod data | Low | High | Test migration on production backup first |
| Concurrency locks cause deadlock | Medium | High | Implement lock timeout (30 seconds) |
| Integration tests flaky | Medium | Medium | Use deterministic test data, avoid timing |
| Breaking changes to API | Low | High | Maintain backward compatibility |

---

## Appendix: Code Review Grade Improvement

**Current Grade:** B+
**Target Grade:** A

**Improvements Required:**
1. ✅ Fix 4 critical issues → Improves to A-
2. ✅ Fix 8 major issues → Improves to A
3. ✅ Add integration tests → Improves to A+

**Grade Breakdown:**
- Code Quality: A- → A (remove unwrap, fix errors)
- Architecture: A- → A (add transactions, fix consistency)
- Testing: B+ → A (add integration tests)
- Completeness: B → A- (fix meal_replaced, emit events)
- Security: A- → A (no changes needed)
- Performance: B+ → A- (add concurrency protection)

**Overall: B+ → A**

---

## Conclusion

This action plan provides a systematic approach to resolving all critical and major issues identified in the code review. By following this plan, the Recipe Rotation System will achieve production-ready quality with an "A" grade.

**Estimated Total Effort:** 15-20 hours
**Timeline:** 2-3 weeks (part-time)
**Outcome:** Production-ready rotation system with comprehensive testing and robust error handling

**Next Steps:**
1. Review and approve this action plan
2. Create GitHub issues for each task
3. Begin Phase 1 implementation
4. Track progress using TodoWrite tool
5. Update documentation as tasks complete

---

**Action Plan Version:** 1.0
**Created:** 2025-10-17
**Last Updated:** 2025-10-17
**Status:** Ready for Implementation
