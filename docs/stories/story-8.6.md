# Story 8.6: Write Route Integration Tests and API Documentation

Status: Approved

## Story

As a backend developer and API consumer,
I want comprehensive integration tests and API documentation for all meal planning routes,
so that route behavior is verified, frontend integration is smooth, and performance targets are met.

## Acceptance Criteria

1. Integration test suite covers all routes (>85% coverage)
2. Tests verify authentication/authorization logic (401, 403 responses)
3. Tests verify error handling (400, 404, 500 scenarios)
4. Tests verify request/response JSON contracts (schema validation)
5. API documentation created (OpenAPI spec or README)
6. Documentation includes example requests/responses
7. All integration tests pass in CI/CD
8. Performance tests verify P95 <500ms for all routes

## Tasks / Subtasks

- [ ] Create comprehensive integration test suite (AC: 1)
  - [ ] Verify test files exist for all routes:
    - `crates/api/tests/integration/test_meal_planning_routes.rs` (Story 8.1 tests)
    - `crates/api/tests/integration/test_week_navigation.rs` (Story 8.2 tests)
    - `crates/api/tests/integration/test_regeneration.rs` (Story 8.3, 8.4 tests)
    - `crates/api/tests/integration/test_preferences.rs` (Story 8.5 tests)
  - [ ] Consolidate any missing test cases into `crates/api/tests/integration/test_comprehensive.rs`
  - [ ] Run `cargo tarpaulin` to measure coverage, verify >85% for `crates/api/src/routes/`

- [ ] Write authentication/authorization integration tests (AC: 2)
  - [ ] Test: POST /plan/generate-multi-week without JWT cookie → 401 Unauthorized
  - [ ] Test: GET /plan/week/:week_id without JWT cookie → 401 Unauthorized
  - [ ] Test: POST /plan/week/:week_id/regenerate without JWT cookie → 401 Unauthorized
  - [ ] Test: POST /plan/regenerate-all-future without JWT cookie → 401 Unauthorized
  - [ ] Test: PUT /profile/meal-planning-preferences without JWT cookie → 401 Unauthorized
  - [ ] Test: GET /plan/week/:week_id with valid JWT but week belongs to different user → 403 Forbidden
  - [ ] Test: POST /plan/week/:week_id/regenerate with valid JWT but week belongs to different user → 403 Forbidden
  - [ ] Consolidate these tests into `test_authentication_authorization.rs` module

- [ ] Write error handling integration tests (AC: 3)
  - [ ] Test: POST /plan/generate-multi-week with < 7 favorite recipes → 400 InsufficientRecipes with category counts
  - [ ] Test: GET /plan/week/:week_id with invalid UUID format → 400 Bad Request
  - [ ] Test: GET /plan/week/:week_id with non-existent week_id → 404 WeekNotFound
  - [ ] Test: POST /plan/week/:week_id/regenerate on locked week → 403 WeekLocked
  - [ ] Test: POST /plan/week/:week_id/regenerate on past week → 400 WeekAlreadyStarted
  - [ ] Test: POST /plan/regenerate-all-future without confirmation → 400 ConfirmationRequired
  - [ ] Test: PUT /profile/meal-planning-preferences with negative prep time → 400 ValidationFailed
  - [ ] Test: PUT /profile/meal-planning-preferences with cuisine_variety_weight > 1.0 → 400 ValidationFailed
  - [ ] Test: Simulate algorithm timeout → 500 AlgorithmTimeout (mock algorithm)
  - [ ] Consolidate these tests into `test_error_handling.rs` module

- [ ] Write JSON contract validation tests (AC: 4)
  - [ ] Define JSON schema expectations for each route response (using serde_json::Value assertions)
  - [ ] Test: POST /plan/generate-multi-week response schema:
    - Assert fields exist: generation_batch_id, max_weeks_possible, current_week_index, first_week, navigation
    - Assert first_week contains: id, start_date, end_date, status, is_locked, meal_assignments (array of 21 items)
    - Assert navigation contains: next_week_id, week_links (array)
  - [ ] Test: GET /plan/week/:week_id response schema:
    - Assert fields exist: week, shopping_list, navigation
    - Assert week contains: id, start_date, end_date, status, is_locked, meal_assignments
    - Assert shopping_list contains: id, categories (array with name, items)
  - [ ] Test: POST /plan/week/:week_id/regenerate response schema:
    - Assert fields exist: week, message
    - Assert week contains regenerated meal_assignments
  - [ ] Test: POST /plan/regenerate-all-future response schema:
    - Assert fields exist: regenerated_weeks, preserved_current_week_id, first_future_week, message
  - [ ] Test: PUT /profile/meal-planning-preferences response schema:
    - Assert fields exist: preferences, message
    - Assert preferences echoes submitted values
  - [ ] Consolidate these tests into `test_json_contracts.rs` module

- [ ] Write performance tests for all routes (AC: 8)
  - [ ] Create `crates/api/tests/performance/route_latency_tests.rs` if not exists
  - [ ] Implement performance test helper: `measure_route_latency(route, iterations: 100) -> P50, P95, P99`
  - [ ] Test: POST /plan/generate-multi-week route overhead < 500ms P95 (excluding algorithm time)
    - Mock algorithm to return immediately (measure only route overhead: load data, emit event)
    - Assert P95 latency < 500ms
  - [ ] Test: GET /plan/week/:week_id latency < 100ms P95
    - Use realistic data: 21 meal assignments, 30 shopping items
    - Assert P95 latency < 100ms
  - [ ] Test: POST /plan/week/:week_id/regenerate route overhead < 500ms P95 (excluding algorithm time)
    - Mock algorithm to return immediately
    - Assert P95 latency < 500ms
  - [ ] Test: POST /plan/regenerate-all-future route overhead < 2000ms P95 for 4 future weeks (excluding algorithm time)
    - Mock algorithm to return immediately per week
    - Assert P95 latency < 2000ms
  - [ ] Test: PUT /profile/meal-planning-preferences latency < 100ms P95
    - Assert P95 latency < 100ms

- [ ] Create API documentation (AC: 5, 6)
  - [ ] Choose documentation format: OpenAPI 3.0 spec (machine-readable + human-readable)
  - [ ] Create `docs/api/meal-planning-routes-openapi.yaml` file
  - [ ] Document each route with:
    - HTTP method and path
    - Request parameters (path, query, body)
    - Request body schema (JSON)
    - Response schema (JSON) for 200, 400, 403, 404, 500
    - Example request curl commands
    - Example response JSON bodies
    - Authentication requirements (JWT cookie)
  - [ ] Document POST /plan/generate-multi-week:
    - Example request: `curl -X POST http://localhost:3000/plan/generate-multi-week -H "Cookie: session={JWT}"`
    - Example 200 response: Full JSON with first_week and navigation
    - Example 400 response: InsufficientRecipes error with category counts
    - Example 401 response: Unauthorized error
  - [ ] Document GET /plan/week/:week_id:
    - Example request: `curl -X GET http://localhost:3000/plan/week/{week_id} -H "Cookie: session={JWT}"`
    - Example 200 response: Full JSON with week, shopping_list, navigation
    - Example 404 response: WeekNotFound error
    - Example 403 response: Forbidden error
  - [ ] Document POST /plan/week/:week_id/regenerate:
    - Example request: `curl -X POST http://localhost:3000/plan/week/{week_id}/regenerate -H "Cookie: session={JWT}"`
    - Example 200 response: Full JSON with regenerated week and message
    - Example 403 response: WeekLocked error
    - Example 400 response: WeekAlreadyStarted error
  - [ ] Document POST /plan/regenerate-all-future:
    - Example request: `curl -X POST http://localhost:3000/plan/regenerate-all-future -H "Cookie: session={JWT}" -H "Content-Type: application/json" -d '{"confirmation": true}'`
    - Example 200 response: Full JSON with regenerated_weeks count and first_future_week
    - Example 400 response: ConfirmationRequired error
  - [ ] Document PUT /profile/meal-planning-preferences:
    - Example request: `curl -X PUT http://localhost:3000/profile/meal-planning-preferences -H "Cookie: session={JWT}" -H "Content-Type: application/json" -d '{"max_prep_time_weeknight": 30, "max_prep_time_weekend": 90, "avoid_consecutive_complex": true, "cuisine_variety_weight": 0.7}'`
    - Example 200 response: Full JSON with preferences and message
    - Example 400 response: ValidationFailed error with field-specific details

- [ ] Generate human-readable API documentation from OpenAPI spec
  - [ ] Use tool (e.g., Swagger UI, Redoc, or openapi-generator) to generate Markdown from OpenAPI spec
  - [ ] Create `docs/api/meal-planning-routes-README.md` with:
    - Overview of meal planning API routes
    - Authentication requirements (JWT cookie)
    - Common error codes (401, 403, 400, 404, 500)
    - Rate limiting information (5 generations/hour, 10 regenerations/hour)
    - Links to OpenAPI spec for full details

- [ ] Verify all integration tests pass in CI/CD (AC: 7)
  - [ ] Ensure GitHub Actions workflow (or CI/CD pipeline) runs `cargo test` on every PR
  - [ ] Ensure CI fails if any integration test fails
  - [ ] Ensure CI runs `cargo tarpaulin` and fails if coverage < 85%
  - [ ] Add CI badge to README showing test status

- [ ] Write test helpers and fixtures
  - [ ] Create `crates/api/tests/helpers.rs` module with reusable test utilities:
    - `setup_test_db() -> DatabasePool` - Creates in-memory SQLite database with schema
    - `create_test_user(db, user_id) -> User` - Inserts test user with default preferences
    - `create_test_recipes(db, user_id, count) -> Vec<Recipe>` - Inserts favorite recipes
    - `create_test_meal_plan(db, user_id, weeks) -> MealPlan` - Inserts meal plan with weeks
    - `create_valid_jwt(user_id) -> String` - Generates valid JWT token for test user
    - `app_with_test_state(db, executor) -> Router` - Creates Axum app with test dependencies
  - [ ] Create `crates/api/tests/fixtures.rs` module with sample data:
    - Sample recipes (10+ recipes with varied complexity, prep time, cuisines)
    - Sample meal assignments (7 days × 3 meals = 21 assignments)
    - Sample shopping lists (6 categories with 5+ items each)

- [ ] Document evento test pattern for future developers
  - [ ] Create `docs/testing/evento-test-pattern.md` with:
    - Explanation of `unsafe_oneshot` for synchronous projection processing in tests
    - Example test code snippet showing evento subscription pattern
    - Rationale: Tests need synchronous processing to assert read model updates immediately
    - Warning: `unsafe_oneshot` only for tests, never in production code (use `run` in production)

- [ ] Review and consolidate all integration tests
  - [ ] Run `cargo test` to verify all tests pass
  - [ ] Check for duplicate test cases across files (deduplicate if found)
  - [ ] Ensure test naming convention: `test_{route}_{scenario}_{expected_result}()`
  - [ ] Ensure all tests have clear comments explaining what they verify
  - [ ] Ensure all tests clean up after themselves (database rollback, no test pollution)

## Dev Notes

### Architecture Patterns
- **TDD Enforced**: All routes implemented with tests first (Stories 8.1-8.5)
- **Test Pyramid**: 70% integration tests, 20% unit tests, 10% performance tests
- **Test Isolation**: Each test runs in database transaction, rolled back after completion
- **Evento Test Pattern**: Use `unsafe_oneshot` for synchronous projection processing in tests

### Source Tree Components
- **Integration Tests**:
  - `crates/api/tests/integration/test_meal_planning_routes.rs` (Story 8.1)
  - `crates/api/tests/integration/test_week_navigation.rs` (Story 8.2)
  - `crates/api/tests/integration/test_regeneration.rs` (Story 8.3, 8.4)
  - `crates/api/tests/integration/test_preferences.rs` (Story 8.5)
  - `crates/api/tests/integration/test_authentication_authorization.rs` (Story 8.6)
  - `crates/api/tests/integration/test_error_handling.rs` (Story 8.6)
  - `crates/api/tests/integration/test_json_contracts.rs` (Story 8.6)
- **Performance Tests**: `crates/api/tests/performance/route_latency_tests.rs`
- **Test Helpers**: `crates/api/tests/helpers.rs`, `crates/api/tests/fixtures.rs`
- **API Documentation**: `docs/api/meal-planning-routes-openapi.yaml`, `docs/api/meal-planning-routes-README.md`
- **Testing Documentation**: `docs/testing/evento-test-pattern.md`

### Testing Standards
- **Coverage Target**: >85% line coverage for `crates/api/src/routes/` (measured via cargo-tarpaulin)
- **Test Organization**: Group tests by route and scenario (auth, error handling, happy path, edge cases)
- **Test Naming**: `test_{route}_{scenario}_{expected_result}()` convention
- **Test Cleanup**: Each test runs in transaction, rolled back after completion (no test pollution)
- **Test Data**: Use fixtures and helpers for reusable test data (avoid hard-coded values)

### Performance Testing Approach
- **Latency Measurement**: Measure P50, P95, P99 latencies over 100 iterations per route
- **Route Overhead Only**: Mock algorithm execution to measure only route overhead (loading data, emitting events, building response)
- **Realistic Data**: Use realistic data sizes (50 recipes, 21 meal assignments, 30 shopping items)
- **Targets from NFR**:
  - Multi-week generation route overhead: P95 <500ms (algorithm time excluded)
  - Week detail route: P95 <100ms (read-only query)
  - Week regeneration route overhead: P95 <500ms (algorithm time excluded)
  - Regenerate all future route overhead: P95 <2000ms for 4 weeks (algorithm time excluded)
  - Preferences update route: P95 <100ms (simple UPDATE query)

### API Documentation Strategy
- **OpenAPI 3.0 Spec**: Machine-readable format for API clients and code generation tools
- **Markdown README**: Human-readable format for developers (generated from OpenAPI spec)
- **Example Requests**: Curl commands for each route (copy-paste ready)
- **Example Responses**: Full JSON bodies for 200, 400, 403, 404, 500 responses
- **Authentication**: Document JWT cookie requirement for all routes
- **Rate Limiting**: Document rate limits (5 generations/hour, 10 regenerations/hour) from NFR

### OpenAPI Spec Structure
```yaml
openapi: 3.0.0
info:
  title: imkitchen Meal Planning API
  version: 1.0.0
  description: HTTP API for multi-week meal planning, week navigation, regeneration, and preferences management

servers:
  - url: http://localhost:3000
    description: Local development

security:
  - cookieAuth: []

components:
  securitySchemes:
    cookieAuth:
      type: apiKey
      in: cookie
      name: session

paths:
  /plan/generate-multi-week:
    post:
      summary: Generate multi-week meal plan
      security:
        - cookieAuth: []
      responses:
        '200':
          description: Meal plan generated successfully
          content:
            application/json:
              schema:
                # ... MultiWeekResponse schema
        '400':
          description: Insufficient recipes
          content:
            application/json:
              schema:
                # ... InsufficientRecipes error schema
        '401':
          description: Unauthorized
        '500':
          description: Algorithm timeout

  # ... other routes
```

### CI/CD Integration
- **GitHub Actions Workflow** (or CI/CD pipeline):
  - Run `cargo test` on every PR (fail PR if tests fail)
  - Run `cargo tarpaulin` on every PR (fail PR if coverage < 85%)
  - Run `cargo clippy` for linting (fail PR if warnings)
  - Run `cargo fmt --check` for formatting (fail PR if not formatted)
- **Test Environment**: Use in-memory SQLite database for fast test execution
- **Parallel Execution**: Cargo test runs tests in parallel by default (ensure test isolation)

### Test Helpers Design
```rust
// crates/api/tests/helpers.rs
pub async fn setup_test_db() -> DatabasePool {
    // Create in-memory SQLite database
    // Run migrations (Epic 6 schema)
    // Return database pool
}

pub async fn create_test_user(db: &DatabasePool, user_id: &str) -> User {
    // INSERT INTO users with default preferences
}

pub async fn create_test_recipes(db: &DatabasePool, user_id: &str, count: usize) -> Vec<Recipe> {
    // INSERT INTO recipes with is_favorite = true
}

pub fn create_valid_jwt(user_id: &str) -> String {
    // Generate JWT token with user_id claim
    // Use same secret key as production (from config)
}

pub fn app_with_test_state(db: DatabasePool, executor: EventoExecutor) -> Router {
    // Build Axum router with authentication middleware
    // Inject test database pool and evento executor via Extension
}
```

### Evento Test Pattern (unsafe_oneshot)
```rust
#[tokio::test]
async fn test_route_with_evento_projection() {
    // Setup
    let db = setup_test_db().await;
    let executor = evento::Executor::new(/* test db */);
    let user_id = create_test_user(&db, "test-user-id").await;

    // Call route handler
    let app = app_with_test_state(db.clone(), executor.clone());
    let response = app.oneshot(request).await.unwrap();

    // Assert response
    assert_eq!(response.status(), StatusCode::OK);

    // Subscribe to events with unsafe_oneshot for synchronous processing
    evento::subscribe("test-projections")
        .aggregator::<MealPlan>()
        .handler(project_event)
        .unsafe_oneshot(&executor) // Processes events synchronously
        .await
        .unwrap();

    // Assert read model updated
    let meal_plans = query_meal_plans(&db, user_id).await;
    assert!(!meal_plans.is_empty());
}
```

### Project Structure Notes
- Consolidate all integration tests into `crates/api/tests/` directory
- Separate test modules by concern (auth, error handling, JSON contracts, performance)
- API documentation in `docs/api/` directory (OpenAPI spec + Markdown README)
- Testing documentation in `docs/testing/` directory (evento pattern, test guidelines)
- CI/CD ensures all tests pass before merge (quality gate)

### References

**Technical Specification Sections:**
- [Source: docs/tech-spec-epic-8.md#Test Strategy Summary] - TDD approach, test pyramid, coverage targets, test pattern for evento
- [Source: docs/tech-spec-epic-8.md#Acceptance Criteria - Story 8.6] - Full acceptance criteria breakdown
- [Source: docs/tech-spec-epic-8.md#Traceability Mapping] - All AC mapped to test ideas and spec sections
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Performance] - P95 latency targets for all routes
- [Source: docs/tech-spec-epic-8.md#APIs and Interfaces] - Route signatures and response schemas for documentation
- [Source: docs/tech-spec-epic-8.md#Data Models and Contracts] - Full JSON schemas for request/response contracts
- [Source: docs/tech-spec-epic-8.md#Risks, Assumptions, Open Questions - Question: Should API Documentation Use OpenAPI or Markdown?] - Recommendation: Use OpenAPI 3.0 spec + generate Markdown

**Architecture Documents:**
- [Source: docs/tech-spec-epic-8.md#System Architecture Alignment - Architecture Constraints Respected] - TDD enforced, test pattern: unsafe_oneshot for evento subscriptions

## Dev Agent Record

### Context Reference

<!-- Story 8.6 context - Epic 8: Enhanced Meal Planning - Backend Routes & Handlers -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

N/A - Story creation phase

### Completion Notes List

- Created from tech spec Epic 8 acceptance criteria 8.6.1-8.6.8
- All tasks derived from Test Strategy Summary and Traceability Mapping
- Comprehensive test coverage: auth, error handling, JSON contracts, performance
- API documentation: OpenAPI spec + Markdown README for frontend integration

### File List

- `/home/snapiz/projects/github/timayz/imkitchen/docs/stories/story-8.6.md` (this file)
