# Story 8.1: Create Multi-Week Generation Route

Status: Approved

## Story

As a backend developer,
I want to create a POST route that generates multi-week meal plans,
so that authenticated users can trigger intelligent meal plan generation via HTTP API.

## Acceptance Criteria

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

## Tasks / Subtasks

- [ ] Define route handler function signature (AC: 1, 2, 3)
  - [ ] Create `generate_multi_week_meal_plan` function in `crates/api/src/routes/meal_planning.rs`
  - [ ] Add function signature with Axum extractors: `Extension(user_id)`, `Extension(db)`, `Extension(executor)`
  - [ ] Add `#[post("/plan/generate-multi-week")]` attribute
  - [ ] Ensure route is protected by authentication middleware (JWT cookie validation)

- [ ] Load user's favorite recipes from database (AC: 4)
  - [ ] Write SQL query: `SELECT * FROM recipes WHERE user_id = ? AND is_favorite = true`
  - [ ] Execute query using SQLx with database pool from Extension
  - [ ] Parse results into `Vec<Recipe>` domain model
  - [ ] Handle empty result set → return `InsufficientRecipes` error if < 7 recipes

- [ ] Load user's meal planning preferences (AC: 5)
  - [ ] Write SQL query: `SELECT max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight, dietary_restrictions FROM users WHERE id = ?`
  - [ ] Parse results into `MealPlanningPreferences` domain model
  - [ ] Apply defaults if fields are NULL

- [ ] Call Epic 7 algorithm (AC: 6)
  - [ ] Import `generate_multi_week_meal_plans` from `crates/meal_planning/src/algorithm.rs`
  - [ ] Call algorithm with user_id, favorite recipes, and preferences
  - [ ] Handle `Result<MultiWeekMealPlan, Error>` return type
  - [ ] Handle `Error::InsufficientRecipes` → return 400 with category-specific counts
  - [ ] Handle `Error::AlgorithmTimeout` → return 500 with retry message
  - [ ] Handle `Error::NoCompatibleRecipes` → return 400 with constraint relaxation suggestion

- [ ] Emit MultiWeekMealPlanGenerated evento event (AC: 7)
  - [ ] Build `MultiWeekMealPlanGenerated` event struct with generated data
  - [ ] Include: generation_batch_id, user_id, weeks (Vec<WeekMealPlanData>), rotation_state, generated_at timestamp
  - [ ] Call `executor.emit(event).await`
  - [ ] Handle event emission errors → return 500 with internal error message
  - [ ] Log event emission success with structured tracing

- [ ] Build JSON response (AC: 8)
  - [ ] Extract first week from generated weeks
  - [ ] Build navigation links for all weeks (week_id, start_date, is_current)
  - [ ] Construct `MultiWeekResponse` struct with first_week data, generation_batch_id, max_weeks_possible, current_week_index, navigation
  - [ ] Serialize to JSON using serde_json
  - [ ] Return `Ok(Json(response))` with 200 status

- [ ] Implement error handling (AC: 9, 10)
  - [ ] Create `ApiError` enum with variants: InsufficientRecipes, AlgorithmTimeout, InternalServerError
  - [ ] Implement `IntoResponse` for `ApiError` to convert to HTTP responses
  - [ ] InsufficientRecipes: Return 400 with JSON body including category counts and "Add More Recipes" action
  - [ ] AlgorithmTimeout: Return 500 with JSON body including retry message
  - [ ] Include user-friendly error messages and actionable guidance

- [ ] Add structured logging and tracing (Observability)
  - [ ] Log request start: `tracing::info!(user_id = %user_id, "Multi-week meal plan generation requested")`
  - [ ] Log algorithm execution: `tracing::debug!(user_id = %user_id, recipe_count = recipes.len(), "Calling algorithm")`
  - [ ] Log errors: `tracing::error!(user_id = %user_id, error = %err, "Failed to generate meal plan")`
  - [ ] Add OpenTelemetry span with attributes: user_id, recipe_count, weeks_generated

- [ ] Write integration test (AC: 11)
  - [ ] Create `test_generate_multi_week_with_valid_jwt()` in `crates/api/tests/integration/test_meal_planning_routes.rs`
  - [ ] Setup test database with test user and 10+ favorite recipes
  - [ ] Create valid JWT cookie for test user
  - [ ] Make POST request to `/plan/generate-multi-week` with JWT cookie
  - [ ] Assert response status is 200 OK
  - [ ] Parse JSON response and validate structure matches schema (first_week, navigation, generation_batch_id)
  - [ ] Subscribe to MultiWeekMealPlanGenerated event using `unsafe_oneshot` for synchronous processing
  - [ ] Verify read models updated: meal_plans table, meal_assignments table, shopping_lists table
  - [ ] Assert first week has 21 meal assignments (7 days × 3 meals)

- [ ] Write error scenario integration tests
  - [ ] Test: POST without JWT cookie returns 401 Unauthorized
  - [ ] Test: POST with < 7 favorite recipes returns 400 InsufficientRecipes with category counts
  - [ ] Test: Simulate algorithm timeout, verify 500 response with retry message

- [ ] Register route in Axum router
  - [ ] Add route to `crates/api/src/main.rs` or router module
  - [ ] Ensure authentication middleware is applied to route
  - [ ] Ensure database pool and evento executor are available via Extension

## Dev Notes

### Architecture Patterns
- **Event-Sourced CQRS**: Route handler emits evento events, projections update read models asynchronously
- **TDD Enforced**: Write failing integration test first, implement route handler to pass, refactor
- **Tailwind CSS 4.1+**: Not applicable to backend route (Epic 9 responsibility for frontend rendering)
- **Test Pattern**: Use `unsafe_oneshot` for evento subscriptions in tests to process events synchronously

### Source Tree Components
- **Route Handler**: `crates/api/src/routes/meal_planning.rs` - Create new file if doesn't exist
- **Error Types**: `crates/api/src/errors.rs` - Define `ApiError` enum and `IntoResponse` implementation
- **Domain Algorithm**: `crates/meal_planning/src/algorithm.rs` - Epic 7 implementation (already exists)
- **Integration Tests**: `crates/api/tests/integration/test_meal_planning_routes.rs`
- **Router Configuration**: `crates/api/src/main.rs` or `crates/api/src/router.rs`

### Testing Standards
- **Coverage Target**: >85% line coverage for route handler code (measured via cargo-tarpaulin)
- **Integration Tests**: Full HTTP request/response cycle with database and evento
- **Performance Target**: Route overhead <500ms P95 (excluding algorithm execution time)
- **Test Isolation**: Each test runs in transaction, rolled back after completion

### Key Technical Constraints
- **Authentication**: JWT cookie-based (Epic 1 implementation), middleware extracts `user_id` from claims
- **Database**: SQLite with SQLx for parameterized queries (SQL injection prevention)
- **Evento Events**: Use `unsafe_oneshot` in tests for synchronous projection processing (not `run`)
- **Response Format**: JSON by default, Epic 9 handles HTML rendering via Askama templates

### Algorithm Integration
- **Function Call**: `generate_multi_week_meal_plans(user_id, recipes, preferences).await`
- **Input Validation**: Algorithm expects at least 7 favorite recipes per category (appetizers, main_courses, desserts)
- **Performance**: Algorithm execution <5s for 50 recipes (Epic 7 benchmark)
- **Error Handling**: Algorithm returns `Result<MultiWeekMealPlan, Error>` with typed error variants

### Evento Event Schema
```rust
MultiWeekMealPlanGenerated {
    generation_batch_id: String,  // UUID
    user_id: String,               // UUID from JWT
    weeks: Vec<WeekMealPlanData>,  // 3-5 weeks of meal assignments
    rotation_state: RotationState, // Tracks which recipes were used
    generated_at: DateTime<Utc>,   // Timestamp
}
```

### Project Structure Notes
- Aligns with event-driven monolith architecture
- Follows established pattern: Route → Domain Logic → Evento Event → Projection
- Database connection pooling configured (min 5, max 20 connections)
- Rate limiting: 5 meal plan generations per user per hour (Epic 8 NFR)

### References

**Technical Specification Sections:**
- [Source: docs/tech-spec-epic-8.md#Detailed Design - Services and Modules] - Route handler responsibilities and contracts
- [Source: docs/tech-spec-epic-8.md#APIs and Interfaces] - Route signature and response schema
- [Source: docs/tech-spec-epic-8.md#Workflows and Sequencing - Multi-Week Generation Request Flow] - Complete request flow from frontend to projection
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Performance] - P95 <500ms target for route overhead
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Security] - Authentication, authorization, input validation requirements
- [Source: docs/tech-spec-epic-8.md#Dependencies and Integrations] - SQLx, evento, algorithm dependencies
- [Source: docs/tech-spec-epic-8.md#Acceptance Criteria - Story 8.1] - Full acceptance criteria breakdown
- [Source: docs/tech-spec-epic-8.md#Test Strategy Summary] - TDD approach, test pyramid, coverage targets

**UX Specification:**
- [Source: docs/ux-specification.md#User Flows - Flow 1: New User Onboarding] - End-to-end user journey including meal plan generation

**Architecture Documents:**
- [Source: docs/tech-spec-epic-8.md#System Architecture Alignment] - Axum, evento, SQLite architecture patterns
- [Source: docs/tech-spec-epic-8.md#Design Decisions from Architecture Document] - POST /plan/generate-multi-week returns first week data + navigation links

## Dev Agent Record

### Context Reference

<!-- Story 8.1 context - Epic 8: Enhanced Meal Planning - Backend Routes & Handlers -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

N/A - Story creation phase

### Completion Notes List

- Created from tech spec Epic 8 acceptance criteria 8.1.1-8.1.11
- All tasks derived from Detailed Design section and workflow diagrams
- Integration with Epic 7 algorithm implementation
- TDD approach enforced with integration test requirements

### File List

- `/home/snapiz/projects/github/timayz/imkitchen/docs/stories/story-8.1.md` (this file)
