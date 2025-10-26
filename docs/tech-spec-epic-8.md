# Technical Specification: Enhanced Meal Planning - Backend Routes & Handlers

Date: 2025-10-26
Author: Jonathan
Epic ID: Epic 8
Status: Draft

---

## Overview

This technical specification covers Epic 8: Enhanced Meal Planning - Backend Routes & Handlers, which delivers the HTTP API layer for multi-week meal planning functionality. Building upon the algorithm implementation from Epic 7, this epic creates RESTful routes using Axum that expose meal plan generation, week navigation, regeneration, and user preference management to the frontend.

The routes serve as the integration point between the pure domain logic (Epic 7 algorithm) and the server-rendered frontend (Epic 9 TwinSpark + Askama), implementing authentication middleware, request validation, evento event emission, and HTML/JSON response generation following the event-sourced architecture patterns established in Epic 6.

## Objectives and Scope

**In Scope:**
- Multi-week meal plan generation route (POST /plan/generate-multi-week)
- Week detail navigation route (GET /plan/week/:week_id)
- Single week regeneration route (POST /plan/week/:week_id/regenerate)
- All future weeks regeneration route (POST /plan/regenerate-all-future)
- User meal planning preferences update route (PUT /profile/meal-planning-preferences)
- Request validation and error handling for all routes
- Integration tests verifying request/response contracts
- API documentation for frontend integration
- Performance testing (P95 <500ms for all routes)

**Out of Scope:**
- Algorithm implementation (completed in Epic 7)
- Frontend UI components and TwinSpark interactions (Epic 9: Frontend UX Implementation)
- Shopping list display routes (Epic 4 + Epic 9)
- Notification system routes for prep reminders (Epic 4)
- Authentication system implementation (Epic 1, already exists)
- Database schema migrations (completed in Epic 6)
- Recipe management routes (Epic 2, already exists)

## System Architecture Alignment

This epic aligns with the event-sourced monolith architecture using Axum for HTTP routing, evento for CQRS event handling, and SQLite for read model queries. The route handlers follow the established pattern: validate request → load data from read models → call domain logic → emit evento events → return response.

**Referenced Architecture Components:**
- **HTTP Server:** Axum with Extension extractors for shared state (database pool, evento executor)
- **Domain Crate:** `crates/meal_planning/src/algorithm.rs` (Epic 7) - Called by route handlers
- **Event Store:** evento SQLite - Event sourcing for `MultiWeekMealPlanGenerated`, `SingleWeekRegenerated`, `AllFutureWeeksRegenerated` events
- **Read Models:** `meal_plans`, `meal_assignments`, `shopping_lists`, `users` tables
- **Authentication:** JWT cookie-based middleware (Epic 1) - `user_id` extracted from claims
- **Template Engine:** Askama for HTML response rendering (Epic 9 integration point)

**Architecture Constraints Respected:**
- TDD enforced: Write integration tests first, implement routes to pass
- Tailwind CSS 4.1+ syntax for HTML templates (Epic 9 responsibility)
- Test pattern: `unsafe_oneshot` for evento subscriptions in tests (synchronous processing)
- Performance target: <500ms P95 for all routes (measured via integration tests + production metrics)
- No vendor lock-in: Axum handlers remain framework-agnostic via dependency injection

**Design Decisions from Architecture Document (Section 7):**
- POST /plan/generate-multi-week returns first week data + navigation links (section 7.1)
- Week regeneration requires week NOT locked (is_locked == false) (section 7.1)
- Regenerate all future weeks requires confirmation parameter to prevent accidental data loss (section 7.1)
- Route responses include algorithm reasoning for transparency (section 1.5, transparency principle)
- Error responses user-friendly with actionable guidance (section 7.1)

## Detailed Design

### Services and Modules

| Module/Function | Responsibility | Inputs | Outputs | Owner |
|-----------------|----------------|--------|---------|-------|
| `POST /plan/generate-multi-week` | Trigger multi-week meal plan generation for authenticated user | JWT claims (user_id), preferences from DB | JSON/HTML with first week + nav links | Axum route handler |
| `GET /plan/week/:week_id` | Display specific week's meal plan calendar | JWT claims (user_id), path param (week_id) | JSON/HTML with week meals + shopping list link | Axum route handler |
| `POST /plan/week/:week_id/regenerate` | Regenerate individual future week | JWT claims (user_id), path param (week_id) | JSON/HTML with updated week data | Axum route handler |
| `POST /plan/regenerate-all-future` | Regenerate all future weeks (preserves current) | JWT claims (user_id), confirmation param | JSON/HTML with count of regenerated weeks | Axum route handler |
| `PUT /profile/meal-planning-preferences` | Update user's meal planning preferences | JWT claims (user_id), JSON body (preferences) | JSON/HTML with updated preferences | Axum route handler |
| `AuthMiddleware` | Extract user_id from JWT cookie, verify authentication | HTTP request with cookie | Extension(UserId) or 401 Unauthorized | Axum middleware |
| `ValidationMiddleware` | Validate request payloads against schemas | HTTP request body | Validated data or 400 Bad Request | Axum extractor |

### Data Models and Contracts

**Request/Response Contracts:**

```rust
// POST /plan/generate-multi-week
// Request: No body, user_id from JWT
// Response: JSON
{
  "generation_batch_id": "uuid",
  "max_weeks_possible": 5,
  "current_week_index": 0,
  "first_week": {
    "id": "week-uuid",
    "start_date": "2025-10-28",
    "end_date": "2025-11-03",
    "status": "future",
    "is_locked": false,
    "meal_assignments": [
      {
        "id": "assignment-uuid",
        "date": "2025-10-28",
        "course_type": "main_course",
        "recipe": {
          "id": "recipe-uuid",
          "title": "Chicken Tikka Masala",
          "prep_time_min": 20,
          "cook_time_min": 30,
          "complexity": "moderate"
        },
        "accompaniment": {
          "id": "accompaniment-uuid",
          "title": "Basmati Rice",
          "category": "rice"
        },
        "prep_required": true,
        "algorithm_reasoning": "Saturday: Weekend allows longer prep time"
      }
      // ... 20 more assignments
    ],
    "shopping_list_id": "shopping-uuid"
  },
  "navigation": {
    "next_week_id": "week-uuid-2",
    "week_links": [
      { "week_id": "uuid-1", "start_date": "2025-10-28", "is_current": false },
      { "week_id": "uuid-2", "start_date": "2025-11-04", "is_current": false }
    ]
  }
}

// Error responses
{
  "error": "InsufficientRecipes",
  "message": "You need at least 7 favorite recipes in each category to generate a meal plan.",
  "details": {
    "appetizers": 5,
    "main_courses": 3,
    "desserts": 7
  },
  "action": {
    "label": "Add More Recipes",
    "url": "/recipes/new"
  }
}
```

```rust
// GET /plan/week/:week_id
// Request: Path param week_id, user_id from JWT
// Response: JSON
{
  "week": {
    "id": "week-uuid",
    "start_date": "2025-10-28",
    "end_date": "2025-11-03",
    "status": "current",
    "is_locked": true,
    "meal_assignments": [ /* full list */ ],
    "shopping_list": {
      "id": "shopping-uuid",
      "categories": [
        {
          "name": "Produce",
          "items": [
            {
              "ingredient_name": "Tomatoes",
              "quantity": 6,
              "unit": "whole",
              "from_recipe_ids": ["recipe-uuid-1", "recipe-uuid-2"]
            }
          ]
        }
      ]
    }
  },
  "navigation": {
    "previous_week_id": "week-uuid-prev",
    "next_week_id": "week-uuid-next"
  }
}

// Error responses
{
  "error": "WeekNotFound",
  "message": "Week not found or does not belong to you.",
  "status": 404
}

{
  "error": "Forbidden",
  "message": "This week belongs to a different user.",
  "status": 403
}
```

```rust
// POST /plan/week/:week_id/regenerate
// Request: Path param week_id, user_id from JWT
// Response: JSON
{
  "week": {
    "id": "week-uuid",
    "start_date": "2025-11-04",
    "status": "future",
    "is_locked": false,
    "meal_assignments": [ /* regenerated list */ ],
    "shopping_list_id": "new-shopping-uuid"
  },
  "message": "Week regenerated successfully. Shopping list updated."
}

// Error responses
{
  "error": "WeekLocked",
  "message": "Cannot regenerate current week. It is locked to prevent disrupting in-progress meals.",
  "status": 403
}

{
  "error": "WeekAlreadyStarted",
  "message": "Cannot regenerate a week that has already started.",
  "status": 400
}
```

```rust
// POST /plan/regenerate-all-future
// Request: JSON body with confirmation
{
  "confirmation": true
}

// Response: JSON
{
  "regenerated_weeks": 4,
  "preserved_current_week_id": "week-uuid-current",
  "first_future_week": {
    "id": "week-uuid-next",
    "start_date": "2025-11-04",
    "meal_assignments": [ /* list */ ]
  },
  "message": "All 4 future weeks regenerated successfully. Current week preserved."
}

// Error responses
{
  "error": "ConfirmationRequired",
  "message": "This action requires confirmation. Include { \"confirmation\": true } in request body.",
  "status": 400
}
```

```rust
// PUT /profile/meal-planning-preferences
// Request: JSON body
{
  "max_prep_time_weeknight": 30,
  "max_prep_time_weekend": 90,
  "avoid_consecutive_complex": true,
  "cuisine_variety_weight": 0.7
}

// Response: JSON
{
  "preferences": {
    "max_prep_time_weeknight": 30,
    "max_prep_time_weekend": 90,
    "avoid_consecutive_complex": true,
    "cuisine_variety_weight": 0.7
  },
  "message": "Meal planning preferences updated. Changes will apply to your next meal plan generation."
}

// Error responses
{
  "error": "ValidationFailed",
  "message": "Invalid preferences provided.",
  "details": {
    "max_prep_time_weeknight": "Must be greater than 0",
    "cuisine_variety_weight": "Must be between 0.0 and 1.0"
  },
  "status": 400
}
```

### APIs and Interfaces

**Route Specifications:**

```rust
// Story 8.1: Multi-Week Generation Route
#[post("/plan/generate-multi-week")]
async fn generate_multi_week_meal_plan(
    Extension(user_id): Extension<UserId>,
    Extension(db): Extension<DatabasePool>,
    Extension(executor): Extension<EventoExecutor>,
) -> Result<Json<MultiWeekResponse>, ApiError>;

// Story 8.2: Week Navigation Route
#[get("/plan/week/:week_id")]
async fn get_week_detail(
    Extension(user_id): Extension<UserId>,
    Path(week_id): Path<String>,
    Extension(db): Extension<DatabasePool>,
) -> Result<Json<WeekDetailResponse>, ApiError>;

// Story 8.3: Week Regeneration Route
#[post("/plan/week/:week_id/regenerate")]
async fn regenerate_week(
    Extension(user_id): Extension<UserId>,
    Path(week_id): Path<String>,
    Extension(db): Extension<DatabasePool>,
    Extension(executor): Extension<EventoExecutor>,
) -> Result<Json<WeekResponse>, ApiError>;

// Story 8.4: Regenerate All Future Weeks Route
#[post("/plan/regenerate-all-future")]
async fn regenerate_all_future_weeks(
    Extension(user_id): Extension<UserId>,
    Json(payload): Json<RegenerateAllPayload>,
    Extension(db): Extension<DatabasePool>,
    Extension(executor): Extension<EventoExecutor>,
) -> Result<Json<RegenerateAllResponse>, ApiError>;

// Story 8.5: User Preferences Update Route
#[put("/profile/meal-planning-preferences")]
async fn update_meal_planning_preferences(
    Extension(user_id): Extension<UserId>,
    Json(payload): Json<MealPlanningPreferences>,
    Extension(db): Extension<DatabasePool>,
    Extension(executor): Extension<EventoExecutor>,
) -> Result<Json<PreferencesResponse>, ApiError>;
```

**Error Handling:**

```rust
pub enum ApiError {
    InsufficientRecipes { appetizers: usize, main_courses: usize, desserts: usize },
    AlgorithmTimeout,
    WeekNotFound,
    Forbidden,
    WeekLocked,
    WeekAlreadyStarted,
    ConfirmationRequired,
    ValidationFailed(HashMap<String, String>),
    Unauthorized,
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details, action) = match self {
            ApiError::InsufficientRecipes { appetizers, main_courses, desserts } => (
                StatusCode::BAD_REQUEST,
                "InsufficientRecipes",
                "You need at least 7 favorite recipes in each category to generate a meal plan.".to_string(),
                Some(json!({ "appetizers": appetizers, "main_courses": main_courses, "desserts": desserts })),
                Some(json!({ "label": "Add More Recipes", "url": "/recipes/new" })),
            ),
            ApiError::WeekLocked => (
                StatusCode::FORBIDDEN,
                "WeekLocked",
                "Cannot regenerate current week. It is locked to prevent disrupting in-progress meals.".to_string(),
                None,
                None,
            ),
            // ... other variants
        };

        let body = json!({
            "error": error_code,
            "message": message,
            "details": details,
            "action": action,
        });

        (status, Json(body)).into_response()
    }
}
```

### Workflows and Sequencing

**Multi-Week Generation Request Flow:**

```
1. Frontend: User clicks "Generate Meal Plan" button (TwinSpark ts-req="/plan/generate-multi-week" ts-req-method="POST")
   ↓
2. Axum: POST /plan/generate-multi-week receives request
   ↓
3. AuthMiddleware: Extract user_id from JWT cookie
   → If invalid/missing JWT: Return 401 Unauthorized
   → If valid: Extension(UserId) injected
   ↓
4. Route Handler: Load user's favorite recipes from read model (meal_planning_read_model table)
   SQL: SELECT * FROM recipes WHERE user_id = ? AND is_favorite = true
   ↓
5. Route Handler: Load user's meal planning preferences from users table
   SQL: SELECT max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight, dietary_restrictions FROM users WHERE id = ?
   ↓
6. Route Handler: Call Epic 7 algorithm
   generate_multi_week_meal_plans(user_id, favorite_recipes, preferences).await
   → Algorithm returns Result<MultiWeekMealPlan, Error>
   ↓
7. Error Handling:
   If Error::InsufficientRecipes → Return 400 with helpful message + "Add Recipe" action
   If Error::AlgorithmTimeout → Return 500 with retry message
   If Error::NoCompatibleRecipes → Return 400 with constraint relaxation suggestion
   ↓
8. Route Handler: Emit MultiWeekMealPlanGenerated evento event
   executor.emit(MultiWeekMealPlanGenerated {
       generation_batch_id,
       user_id,
       weeks: Vec<WeekMealPlanData>,
       rotation_state,
       generated_at,
   }).await
   ↓
9. Evento Projection: Subscribe to MultiWeekMealPlanGenerated event
   Insert weeks into meal_plans table
   Insert assignments into meal_assignments table
   Insert shopping lists into shopping_lists table
   (Uses unsafe_oneshot in tests for synchronous processing)
   ↓
10. Route Handler: Build JSON response
   Extract first week from generated_weeks
   Build navigation links for all weeks
   Return 200 OK with JSON body
   ↓
11. Frontend (Epic 9): Receive response, render week calendar via Askama template
   TwinSpark swaps content into target element (ts-target="#meal-calendar")
```

**Week Regeneration Request Flow:**

```
1. Frontend: User clicks "Regenerate This Week" button on future week
   TwinSpark: ts-req="/plan/week/:week_id/regenerate" ts-req-method="POST"
   ↓
2. Axum: POST /plan/week/:week_id/regenerate receives request
   ↓
3. AuthMiddleware: Extract user_id from JWT cookie
   ↓
4. Route Handler: Load week from read model
   SQL: SELECT * FROM meal_plans WHERE id = ? AND user_id = ?
   ↓
5. Authorization Check:
   If week.user_id != user_id → Return 403 Forbidden
   If week.is_locked == true → Return 403 WeekLocked
   If week.status == "past" OR week.status == "current" → Return 400 WeekAlreadyStarted
   ↓
6. Route Handler: Load current rotation_state for user's meal plan batch
   SQL: SELECT * FROM meal_plan_rotation_state WHERE generation_batch_id = ? AND user_id = ?
   ↓
7. Route Handler: Call Epic 7 algorithm
   generate_single_week(recipes, preferences, &mut rotation_state, week_start_date)
   → Returns Result<WeekMealPlan, Error>
   ↓
8. Route Handler: Emit SingleWeekRegenerated evento event
   executor.emit(SingleWeekRegenerated {
       week_id,
       week_start_date,
       meal_assignments,
       updated_rotation_state,
   }).await
   ↓
9. Evento Projection: Update meal_assignments for week, regenerate shopping list
   DELETE FROM meal_assignments WHERE meal_plan_id = ?
   INSERT INTO meal_assignments (...)
   UPDATE shopping_lists WHERE meal_plan_id = ?
   ↓
10. Route Handler: Return 200 OK with regenerated week data JSON
   ↓
11. Frontend (Epic 9): TwinSpark swaps updated week calendar
```

**Regenerate All Future Weeks Flow:**

```
1. Frontend: User clicks "Regenerate All Future Weeks" → Confirmation modal appears
   Modal: "This will regenerate X future weeks. Continue?" [Cancel] [Confirm]
   ↓
2. Frontend: User clicks Confirm → POST request with confirmation: true
   TwinSpark: ts-req="/plan/regenerate-all-future" ts-req-method="POST" ts-data='{"confirmation": true}'
   ↓
3. Axum: POST /plan/regenerate-all-future receives request
   ↓
4. AuthMiddleware: Extract user_id from JWT cookie
   ↓
5. Route Handler: Validate confirmation parameter
   If payload.confirmation != true → Return 400 ConfirmationRequired
   ↓
6. Route Handler: Identify current week (locked) and future weeks
   SQL: SELECT * FROM meal_plans WHERE user_id = ? AND status IN ('current', 'future') ORDER BY start_date
   ↓
7. Route Handler: Preserve current week (is_locked == true), regenerate all future weeks
   For each future_week:
       generate_single_week(recipes, preferences, &mut rotation_state, week_start_date)
   ↓
8. Route Handler: Emit AllFutureWeeksRegenerated evento event
   executor.emit(AllFutureWeeksRegenerated {
       generation_batch_id,
       user_id,
       weeks: Vec<WeekMealPlanData>,
       preserved_current_week_id,
   }).await
   ↓
9. Evento Projection: Bulk update meal_assignments and shopping_lists for all future weeks
   DELETE FROM meal_assignments WHERE meal_plan_id IN (future_week_ids)
   INSERT INTO meal_assignments (...) (batch insert)
   UPDATE shopping_lists WHERE meal_plan_id IN (future_week_ids)
   ↓
10. Route Handler: Return 200 OK with count of regenerated weeks + first future week data
   ↓
11. Frontend (Epic 9): TwinSpark reloads entire meal calendar or shows success toast
```

## Non-Functional Requirements

### Performance

**Target Metrics:**
- **Multi-week generation route:** <500ms P95 (excluding algorithm execution, which is <5s from Epic 7)
- **Week detail route:** <100ms P95 (read-only query)
- **Week regeneration route:** <500ms P95 (single week regeneration + DB write)
- **Regenerate all future route:** <2000ms P95 (multiple weeks + bulk DB writes)
- **Preferences update route:** <100ms P95 (simple UPDATE query)

**Performance Testing:**
- Integration tests measure route response times with realistic data (50 recipes, 5 weeks)
- Load testing with 100 concurrent requests (k6 or Apache JMeter)
- P50, P95, P99 latencies tracked in CI/CD metrics
- Regression tests fail if P95 exceeds thresholds

**Optimization Strategies:**
- Database connection pooling (min 5, max 20 connections)
- Batch recipe loading (single query instead of N+1)
- Read model queries optimized with indexes (Epic 6 migrations)
- Evento event batching where appropriate (reduce DB round-trips)
- Response streaming for large JSON payloads (chunked transfer encoding)

### Security

**Authentication:**
- All routes protected by JWT cookie middleware (Epic 1 implementation)
- `user_id` extracted from JWT claims, verified against signature
- Expired tokens return 401 Unauthorized with redirect to /login
- No bearer token in Authorization header (cookie-only for CSRF protection)

**Authorization:**
- Week detail route: Verify `week.user_id == user_id` before returning data (prevent cross-user access)
- Week regeneration route: Same authorization check + lock status verification
- Preferences update route: Only authenticated user can update own preferences

**Input Validation:**
- Path parameters validated: `week_id` must be valid UUID format
- JSON payloads validated against schemas (use `validator` crate)
  - `max_prep_time_weeknight` > 0
  - `max_prep_time_weekend` > 0
  - `cuisine_variety_weight` between 0.0 and 1.0
- SQL injection prevention: Use parameterized queries with SQLx
- XSS prevention: Askama templates auto-escape HTML output (Epic 9)

**Error Messages:**
- No stack traces in production responses (log internally, return generic message)
- Avoid leaking sensitive data in errors (don't reveal user counts, recipe titles of other users)
- Error responses user-friendly: "Week not found" instead of "SQL query returned 0 rows"

**Rate Limiting:**
- Meal plan generation limited to 5 requests per user per hour (prevent abuse)
- Regeneration limited to 10 requests per user per hour
- Rate limit responses: 429 Too Many Requests with Retry-After header

**Dependency Security:**
- All dependencies audited via `cargo audit` in CI/CD
- Axum, SQLx, evento, serde, chrono kept up-to-date with security patches

### Reliability/Availability

**Graceful Degradation:**
- If algorithm times out, return 500 with retry message (don't crash server)
- If database connection pool exhausted, queue request with timeout (don't fail immediately)
- If evento event emission fails, log error and return 500 (don't corrupt read models)

**Error Recovery:**
- Database transactions ensure atomicity (all evento projections succeed or roll back)
- Failed evento events logged to dead-letter queue for manual retry
- Route handlers idempotent where possible (regenerate produces same result if retried)

**Edge Cases Handled:**
- Empty favorite recipes list → InsufficientRecipes error with clear counts
- Week ID not found → 404 WeekNotFound error
- Week belongs to different user → 403 Forbidden error
- Week locked (current week) → 403 WeekLocked error
- Missing confirmation parameter → 400 ConfirmationRequired error

**Health Checks:**
- `/health` endpoint returns 200 OK if server alive (liveness probe)
- `/ready` endpoint returns 200 OK if database connectable (readiness probe)
- Kubernetes/Docker health checks use these endpoints

**Testing for Reliability:**
- Integration tests cover all error scenarios (Story 8.6)
- Chaos engineering: Simulate database failures, slow queries (optional)
- Load testing confirms graceful degradation under stress

### Observability

**Logging:**
- Use `tracing` crate for structured logging
- Log levels:
  - `DEBUG`: Request details (user_id, week_id, payload)
  - `INFO`: Route execution (meal plan generated, week regenerated)
  - `WARN`: Degraded behavior (algorithm timeout, rate limit hit)
  - `ERROR`: Failures (database errors, evento emission failures)

**Log Examples:**
```rust
tracing::info!(user_id = %user_id, "Multi-week meal plan generation requested");
tracing::debug!(week_id = %week_id, user_id = %user_id, "Loading week detail");
tracing::warn!(user_id = %user_id, error = %err, "Algorithm timeout during generation");
tracing::error!(user_id = %user_id, error = %err, "Failed to emit MultiWeekMealPlanGenerated event");
```

**Metrics (Prometheus format):**
- `http_requests_total{route, method, status}` (counter) - Total HTTP requests
- `http_request_duration_seconds{route, method}` (histogram) - Request latency
- `meal_plan_generation_requests_total{outcome}` (counter) - Success vs error
- `week_regeneration_requests_total{week_status}` (counter) - Future vs locked attempts
- `evento_event_emission_duration_seconds{event_type}` (histogram) - Event emission time

**Tracing (OpenTelemetry):**
- Span: `POST /plan/generate-multi-week` with attributes: user_id, recipe_count, weeks_generated
- Span: `POST /plan/week/:week_id/regenerate` with attributes: user_id, week_id, week_status
- Span: `database_query` with attributes: query_name, duration_ms
- Span: `evento_emit` with attributes: event_type, batch_id

**Error Context:**
- All errors include structured context for debugging
- `ApiError` variants carry rich data (recipe counts, week_id, user_id)
- Error logs include full request context (headers, payload, user session)

## Dependencies and Integrations

**Rust Crate Dependencies:**

| Dependency | Version | Purpose | Story Reference |
|------------|---------|---------|-----------------|
| `axum` | 0.7+ | HTTP routing framework, extractors, middleware | All stories |
| `tokio` | 1.40+ | Async runtime for Axum handlers | All stories |
| `sqlx` | 0.8+ | Database connection pooling, read model queries | All stories |
| `evento` | 1.5+ | Event sourcing framework, event emission | Stories 8.1, 8.3, 8.4, 8.5 |
| `serde` | 1.0+ | JSON serialization/deserialization | All stories |
| `serde_json` | 1.0+ | JSON response building | All stories |
| `validator` | 0.18+ | Request payload validation | Story 8.5 |
| `chrono` | 0.4+ | Date/time handling for week calculations | All stories |
| `uuid` | 1.10+ | Week ID, batch ID validation | All stories |
| `tracing` | 0.1+ | Structured logging | All stories |
| `thiserror` | 1.0+ | Custom error type definitions | All stories |

**Internal Module Dependencies:**

- `crates/meal_planning/src/algorithm.rs`: Epic 7 algorithm functions called by route handlers
- `crates/meal_planning/src/rotation.rs`: RotationState loading for regeneration (Epic 6 Story 6.5)
- `crates/recipe`: Recipe read model queries (Epic 2)
- `crates/user`: User preferences read model queries (Epic 1 + Epic 6 Story 6.4)
- `crates/shared_kernel`: Common types (UserId, RecipeId, Date), error types

**Database Integration:**

- **Read Models** (queries via SQLx):
  - `recipes` table: Load favorite recipes for meal plan generation
  - `users` table: Load meal planning preferences
  - `meal_plans` table: Load week data for navigation and regeneration
  - `meal_assignments` table: Load meal assignments for week detail
  - `meal_plan_rotation_state` table: Load rotation state for regeneration
  - `shopping_lists` + `shopping_items` tables: Load shopping list for week detail

- **Write Operations** (via evento projections):
  - Route handlers NEVER write directly to read models
  - All writes happen through evento event emission → projection handlers
  - Ensures event sourcing integrity and audit trail

**Event Store Integration:**

- Route handlers emit events:
  - `MultiWeekMealPlanGenerated` (Story 8.1)
  - `SingleWeekRegenerated` (Story 8.3)
  - `AllFutureWeeksRegenerated` (Story 8.4)
  - `UserMealPlanningPreferencesUpdated` (Story 8.5)
- Projections (Epic 6 Story 6.6) subscribe to events, update read models
- Integration tests use `unsafe_oneshot` for synchronous projection processing

**Frontend Integration (Epic 9):**

- Routes return JSON responses for TwinSpark consumption
- HTML responses rendered via Askama templates (Epic 9 responsibility)
- TwinSpark attributes in templates:
  - `ts-req="/plan/generate-multi-week"` on "Generate Meal Plan" button
  - `ts-target="#meal-calendar"` to swap calendar content
  - `ts-swap="innerHTML"` for content replacement
  - `ts-req-method="POST"` for non-GET requests

**No External Service Dependencies:**
- Routes are self-contained within monolith
- No third-party API calls (payment, email handled separately in Epic 1)

## Acceptance Criteria (Authoritative)

**Story 8.1: Create Multi-Week Generation Route**
1. Route `POST /plan/generate-multi-week` created
2. Route protected by authentication middleware (JWT cookie)
3. Handler extracts `user_id` from JWT claims
4. Handler loads user's favorite recipes from database (read model query)
5. Handler loads user's meal planning preferences from users table
6. Handler calls `generate_multi_week_meal_plans` algorithm (Epic 7)
7. Handler commits `MultiWeekMealPlanGenerated` event to evento
8. Handler returns JSON with first week data + navigation links
9. Error: `InsufficientRecipes` returns 400 with helpful message + "Add Recipe" action
10. Error: `AlgorithmTimeout` returns 500 with retry message
11. Integration test: POST generates meal plan, verifies JSON response structure

**Story 8.2: Create Week Navigation Route**
1. Route `GET /plan/week/:week_id` created
2. Route protected by authentication middleware
3. Handler verifies week belongs to authenticated user (authorization check)
4. Handler loads week data from `meal_plans` + `meal_assignments` tables
5. Handler loads shopping list for week from `shopping_lists` table
6. Handler returns JSON/HTML with week calendar data
7. Returns 404 if `week_id` not found
8. Returns 403 if week belongs to different user
9. Integration test: GET with valid week_id returns correct data
10. Integration test: GET with invalid week_id returns 404

**Story 8.3: Create Week Regeneration Route**
1. Route `POST /plan/week/:week_id/regenerate` created
2. Route verifies week belongs to user and is not locked (authorization + validation)
3. Handler loads current rotation_state for meal plan batch
4. Handler generates new meal assignments for week (calls `generate_single_week`)
5. Handler commits `SingleWeekRegenerated` event to evento
6. Handler regenerates shopping list for week (projection handles this)
7. Returns 403 if week is locked (`is_locked == true` or `status == "current"`)
8. Returns 400 if week already started (`status == "past"`)
9. Integration test: POST regenerates future week successfully
10. Integration test: POST on locked week returns 403

**Story 8.4: Create Regenerate All Future Weeks Route**
1. Route `POST /plan/regenerate-all-future` created
2. Requires confirmation parameter (prevent accidental regeneration)
3. Handler identifies current week (locked) and preserves it
4. Handler regenerates all future weeks (`status == "future"`)
5. Handler resets rotation state but preserves current week's main courses
6. Handler commits `AllFutureWeeksRegenerated` event to evento
7. Handler regenerates shopping lists for all future weeks (projection handles this)
8. Returns count of regenerated weeks + first future week data
9. Integration test: POST with confirmation regenerates all future weeks
10. Integration test: POST without confirmation returns 400

**Story 8.5: Create User Preferences Update Route**
1. Route `PUT /profile/meal-planning-preferences` created
2. Handler validates input (`max_prep_time_weeknight` > 0, `cuisine_variety_weight` 0.0-1.0)
3. Handler commits `UserMealPlanningPreferencesUpdated` event to evento
4. Handler returns updated preferences in response
5. Returns 400 if validation fails with field-specific error messages
6. Integration test: PUT updates preferences successfully
7. Integration test: PUT with invalid data returns 400 with validation errors

**Story 8.6: Write Route Integration Tests and API Documentation**
1. Integration test suite covers all routes (>85% coverage)
2. Tests verify authentication/authorization logic (401, 403 responses)
3. Tests verify error handling (400, 404, 500 scenarios)
4. Tests verify request/response JSON contracts (schema validation)
5. API documentation created (OpenAPI spec or README)
6. Documentation includes example requests/responses
7. All integration tests pass in CI/CD
8. Performance tests verify P95 <500ms for all routes

## Traceability Mapping

| AC # | Spec Section | Component/API | Test Idea |
|------|--------------|---------------|-----------|
| 8.1.1 | APIs and Interfaces | POST /plan/generate-multi-week | Integration: POST request with valid JWT returns 200 |
| 8.1.2 | Security | AuthMiddleware | Integration: POST without JWT returns 401 |
| 8.1.3 | Workflows | Extract user_id from JWT | Unit: Mock JWT, verify user_id extraction |
| 8.1.4 | Dependencies | Load favorite recipes | Integration: Query recipes table, verify count |
| 8.1.5 | Dependencies | Load preferences | Integration: Query users table, verify fields |
| 8.1.6 | Workflows | Call algorithm | Integration: Mock algorithm, verify called with correct params |
| 8.1.7 | Dependencies | Emit evento event | Integration: Verify MultiWeekMealPlanGenerated emitted |
| 8.1.8 | APIs and Interfaces | Return JSON response | Integration: Parse JSON, verify structure matches schema |
| 8.1.9 | APIs and Interfaces | InsufficientRecipes error | Integration: POST with <7 recipes returns 400 with action |
| 8.1.10 | APIs and Interfaces | AlgorithmTimeout error | Integration: Mock timeout, verify 500 response |
| 8.2.1 | APIs and Interfaces | GET /plan/week/:week_id | Integration: GET request with valid week_id returns 200 |
| 8.2.3 | Security | Authorization check | Integration: GET week of different user returns 403 |
| 8.2.4 | Dependencies | Load week data | Integration: Query meal_plans + meal_assignments tables |
| 8.2.5 | Dependencies | Load shopping list | Integration: Query shopping_lists table |
| 8.2.7 | APIs and Interfaces | 404 error | Integration: GET with invalid UUID returns 404 |
| 8.2.8 | APIs and Interfaces | 403 error | Integration: GET with other user's week_id returns 403 |
| 8.3.1 | APIs and Interfaces | POST /plan/week/:week_id/regenerate | Integration: POST with valid week_id returns 200 |
| 8.3.2 | Security | Week lock validation | Integration: POST on locked week returns 403 |
| 8.3.3 | Dependencies | Load rotation state | Integration: Query meal_plan_rotation_state table |
| 8.3.4 | Workflows | Generate single week | Integration: Verify generate_single_week called |
| 8.3.5 | Dependencies | Emit SingleWeekRegenerated | Integration: Verify event emitted with unsafe_oneshot |
| 8.3.7 | APIs and Interfaces | WeekLocked error | Integration: POST on current week returns 403 |
| 8.3.8 | APIs and Interfaces | WeekAlreadyStarted error | Integration: POST on past week returns 400 |
| 8.4.1 | APIs and Interfaces | POST /plan/regenerate-all-future | Integration: POST with confirmation returns 200 |
| 8.4.2 | Security | Confirmation parameter | Integration: POST without confirmation returns 400 |
| 8.4.3 | Workflows | Preserve current week | Integration: Verify current week not regenerated |
| 8.4.4 | Workflows | Regenerate future weeks | Integration: Verify all future weeks updated |
| 8.4.6 | Dependencies | Emit AllFutureWeeksRegenerated | Integration: Verify event emitted |
| 8.4.8 | APIs and Interfaces | Return count | Integration: Verify response contains regenerated_weeks count |
| 8.5.1 | APIs and Interfaces | PUT /profile/meal-planning-preferences | Integration: PUT with valid JSON returns 200 |
| 8.5.2 | Security | Input validation | Integration: PUT with invalid data returns 400 |
| 8.5.3 | Dependencies | Emit UserMealPlanningPreferencesUpdated | Integration: Verify event emitted |
| 8.5.5 | APIs and Interfaces | Validation errors | Integration: Verify 400 response includes field errors |
| 8.6.1 | Test Strategy | Integration test coverage | CI: Verify >85% route handler coverage |
| 8.6.2 | Test Strategy | Auth/authz tests | Integration: Test 401 and 403 scenarios |
| 8.6.3 | Test Strategy | Error handling | Integration: Test all 400, 404, 500 error paths |
| 8.6.4 | Test Strategy | JSON contract validation | Integration: JSON schema validation on responses |
| 8.6.8 | Performance | P95 latency | Load test: Verify all routes <500ms P95 |

## Risks, Assumptions, Open Questions

**Risks:**

1. **Risk: Database Connection Pool Exhaustion Under Load**
   - Impact: Routes timeout or fail when connection pool (max 20) exhausted
   - Likelihood: Medium (if meal plan generation spikes during peak hours)
   - Mitigation: Monitor pool utilization metrics, implement request queuing with timeout, scale horizontally if needed

2. **Risk: Evento Event Projection Lag**
   - Impact: Read models not immediately consistent after route returns (eventual consistency delay)
   - Likelihood: Low (projections typically <100ms)
   - Mitigation: Document eventual consistency in API docs, add polling or WebSocket for real-time updates (Epic 9)

3. **Risk: Large JSON Responses Exceed Network Limits**
   - Impact: 5-week meal plan with full recipe details may exceed 1MB, slow response times
   - Likelihood: Low (typical response ~50KB)
   - Mitigation: Implement pagination for week navigation, lazy-load recipe details on demand

4. **Risk: Rate Limiting Too Restrictive**
   - Impact: Legitimate users hit limits during testing/experimentation
   - Likelihood: Medium (users may regenerate plans frequently)
   - Mitigation: Start with generous limits (5 generations/hour), adjust based on analytics, allow premium users higher limits

**Assumptions:**

1. **Assumption: Epic 7 Algorithm Performance Meets Targets**
   - Validation: Epic 7 benchmarks confirm <5s for 5-week generation
   - Impact if False: Route timeouts exceed 500ms target, poor UX
   - Mitigation: If Epic 7 benchmarks fail, optimize algorithm before Epic 8 begins

2. **Assumption: JWT Cookie Authentication Sufficient**
   - Assumption: Cookie-based auth prevents CSRF, no need for bearer tokens
   - Impact if False: Security vulnerabilities require rework
   - Mitigation: Epic 1 already implemented and tested, assumption validated

3. **Assumption: TwinSpark Handles JSON Responses**
   - Assumption: Epic 9 TwinSpark integration can consume JSON and render via Askama templates
   - Impact if False: Route responses need HTML rendering in Epic 8
   - Mitigation: Coordinate with Epic 9, return dual JSON/HTML support if needed

4. **Assumption: Askama Templates Escape HTML Output**
   - Assumption: XSS prevention automatic via Askama (Epic 9)
   - Impact if False: XSS vulnerabilities in rendered HTML
   - Mitigation: Epic 9 validates template escaping, Epic 8 returns JSON only (no HTML generation)

**Open Questions:**

1. **Question: Should Routes Return HTML or JSON?**
   - Context: Epic 9 TwinSpark integration may prefer server-rendered HTML vs JSON + client-side templating
   - Decision Needed By: Story 8.1 implementation
   - Recommendation: Return JSON by default, Epic 9 handles Askama template rendering via TwinSpark ts-req responses

2. **Question: How to Handle Concurrent Regeneration Requests?**
   - Context: User clicks "Regenerate" multiple times rapidly (double-click)
   - Decision Needed By: Story 8.3 implementation
   - Recommendation: Implement idempotency keys or debounce on frontend (Epic 9), backend ignores duplicate requests within 5 seconds

3. **Question: Should API Documentation Use OpenAPI or Markdown?**
   - Context: Story 8.6 requires API documentation
   - Decision Needed By: Story 8.6 implementation
   - Recommendation: Use OpenAPI 3.0 spec for machine-readable docs, generate Markdown from spec for human-readable reference

4. **Question: Performance Target Includes Algorithm Execution Time?**
   - Context: Multi-week generation route P95 <500ms, but algorithm takes 3-5s (Epic 7)
   - Decision Needed By: Story 8.1 implementation
   - Recommendation: P95 <500ms for route overhead only (loading data, emitting events), algorithm time excluded from metric

## Test Strategy Summary

**Test-Driven Development (TDD) Enforced:**
- Write failing integration test → Implement route handler to pass → Refactor → Repeat
- All stories begin with integration test creation before implementation

**Test Pyramid:**

1. **Integration Tests (70% of tests)**
   - **Scope:** Full HTTP request/response cycle with database and evento
   - **Framework:** Axum test client (`axum::test::RequestBuilder`)
   - **Coverage:** All routes, all error scenarios, authentication/authorization
   - **Examples:**
     - `test_generate_multi_week_with_valid_jwt()`
     - `test_generate_multi_week_without_jwt_returns_401()`
     - `test_get_week_detail_with_invalid_week_id_returns_404()`
     - `test_regenerate_locked_week_returns_403()`
     - `test_update_preferences_with_invalid_data_returns_400()`

2. **Unit Tests (20% of tests)**
   - **Scope:** Individual helper functions (validation, error conversion)
   - **Framework:** Built-in Rust `#[test]`
   - **Coverage:** Input validation, error handling, JSON serialization
   - **Examples:**
     - `test_validate_preferences_accepts_valid_data()`
     - `test_validate_preferences_rejects_negative_prep_time()`
     - `test_api_error_into_response_formatting()`

3. **Performance Tests (10% of tests)**
   - **Scope:** Route latency under realistic load
   - **Framework:** Integration tests with timing, k6 for load testing
   - **Coverage:** P95 latency for all routes
   - **Examples:**
     - `test_generate_multi_week_latency_under_500ms()`
     - `test_week_detail_latency_under_100ms()`
     - Load test: 100 concurrent requests, verify P95 thresholds

**Test Data Management:**
- **Fixtures:** Helper functions create test users, recipes, meal plans
- **Database:** In-memory SQLite for integration tests (fast, isolated)
- **Evento:** Use `unsafe_oneshot` for synchronous projection processing in tests
- **Cleanup:** Each test runs in transaction, rolled back after completion

**Test Organization:**
```
crates/api/
├── src/
│   ├── routes/
│   │   ├── meal_planning.rs    // Route handlers
│   │   └── user_preferences.rs
│   └── middleware/
│       └── auth.rs
└── tests/
    ├── integration/
    │   ├── test_meal_planning_routes.rs
    │   ├── test_week_navigation.rs
    │   ├── test_regeneration.rs
    │   └── test_preferences.rs
    └── performance/
        └── route_latency_tests.rs
```

**Coverage Targets:**
- **Overall:** >85% line coverage for `crates/api/src/routes/` (measured via `cargo-tarpaulin`)
- **Critical Routes:** 100% coverage for multi-week generation, week regeneration
- **CI Enforcement:** Coverage report in CI, build fails if <85%

**Test Execution:**
- **Local Development:** `cargo test` (runs all unit + integration tests)
- **Performance:** `cargo test --release -- --ignored` (performance tests excluded from default run)
- **CI/CD:** GitHub Actions runs `cargo test`, `cargo tarpaulin`, load tests on PR

**Test Pattern for Evento Integration:**
```rust
#[tokio::test]
async fn test_generate_multi_week_emits_event() {
    // Setup test database and evento executor
    let executor = evento::Executor::new(/* test db */);
    let db_pool = setup_test_db().await;

    // Create test user and recipes
    let user_id = create_test_user(&db_pool).await;
    create_test_recipes(&db_pool, user_id, 10).await;

    // Call route handler
    let app = app_with_test_state(db_pool.clone(), executor.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/plan/generate-multi-week")
                .header("Cookie", format!("session={}", valid_jwt(user_id)))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::OK);

    // Subscribe to events with unsafe_oneshot for sync processing
    evento::subscribe("test-projections")
        .aggregator::<MealPlan>()
        .handler(project_multi_week_meal_plan_generated)
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Verify read model updated
    let meal_plans = query_meal_plans(&db_pool, user_id).await;
    assert!(!meal_plans.is_empty());
}
```

**Regression Testing:**
- Integration tests added for every bug found in production
- Performance regression tests in CI track latency over time

**Edge Case Test Examples:**
- Empty favorite recipes (should return InsufficientRecipes error)
- Week ID not found (should return 404)
- Week belongs to different user (should return 403)
- Locked week regeneration attempt (should return 403)
- Missing confirmation parameter (should return 400)
- Invalid UUID format in path (should return 400)
- Database connection failure (should return 500 with retry message)
- Evento event emission failure (should return 500, log error)
