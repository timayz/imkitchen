# Story 8.2: Create Week Navigation Route

Status: Approved

## Story

As a backend developer,
I want to create a GET route that displays specific week meal plan details,
so that authenticated users can navigate between weeks and view their scheduled meals with shopping lists.

## Acceptance Criteria

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

## Tasks / Subtasks

- [ ] Define route handler function signature (AC: 1, 2)
  - [ ] Create `get_week_detail` function in `crates/api/src/routes/meal_planning.rs`
  - [ ] Add function signature with Axum extractors: `Extension(user_id)`, `Path(week_id)`, `Extension(db)`
  - [ ] Add `#[get("/plan/week/:week_id")]` attribute
  - [ ] Ensure route is protected by authentication middleware

- [ ] Load week data from read models (AC: 4)
  - [ ] Write SQL query: `SELECT * FROM meal_plans WHERE id = ? AND user_id = ?`
  - [ ] Execute query using SQLx with database pool
  - [ ] Handle result: If no rows found → return 404 WeekNotFound error
  - [ ] Parse result into `MealPlan` domain model with week metadata (id, start_date, end_date, status, is_locked)

- [ ] Verify week belongs to authenticated user (AC: 3, 8)
  - [ ] Check if `week.user_id == user_id` from JWT claims
  - [ ] If mismatch → return 403 Forbidden error with message "This week belongs to a different user"
  - [ ] Log authorization failure with structured tracing

- [ ] Load meal assignments for week (AC: 4)
  - [ ] Write SQL query with JOIN: `SELECT ma.*, r.* FROM meal_assignments ma JOIN recipes r ON ma.recipe_id = r.id WHERE ma.meal_plan_id = ? ORDER BY ma.date, ma.course_type`
  - [ ] Include accompaniment data if applicable (JOIN accompaniments table)
  - [ ] Parse results into `Vec<MealAssignment>` with nested `Recipe` and optional `Accompaniment` data
  - [ ] Include: assignment_id, date, course_type, recipe (id, title, prep_time_min, cook_time_min, complexity), accompaniment, prep_required, algorithm_reasoning

- [ ] Load shopping list for week (AC: 5)
  - [ ] Write SQL query: `SELECT sl.*, si.* FROM shopping_lists sl LEFT JOIN shopping_items si ON sl.id = si.shopping_list_id WHERE sl.meal_plan_id = ?`
  - [ ] Group shopping items by category (Produce, Dairy, Meat & Seafood, Pantry, Frozen, Bakery)
  - [ ] Parse into `ShoppingList` struct with categories and items
  - [ ] Include: shopping_list_id, categories (name, items with ingredient_name, quantity, unit, from_recipe_ids)

- [ ] Build JSON response (AC: 6)
  - [ ] Construct `WeekDetailResponse` struct with:
    - week: { id, start_date, end_date, status, is_locked, meal_assignments }
    - shopping_list: { id, categories with items }
    - navigation: { previous_week_id, next_week_id }
  - [ ] Calculate previous_week_id and next_week_id from database (query sibling weeks by start_date ordering)
  - [ ] Serialize to JSON using serde_json
  - [ ] Return `Ok(Json(response))` with 200 status

- [ ] Implement error handling (AC: 7, 8)
  - [ ] Add `ApiError` variants: WeekNotFound (404), Forbidden (403)
  - [ ] WeekNotFound: Return 404 with JSON body "Week not found or does not belong to you"
  - [ ] Forbidden: Return 403 with JSON body "This week belongs to a different user"
  - [ ] Include structured error logging for debugging

- [ ] Add path parameter validation
  - [ ] Validate `week_id` path parameter is valid UUID format
  - [ ] Return 400 Bad Request if UUID parse fails
  - [ ] Log invalid UUID attempts with structured tracing

- [ ] Add structured logging and tracing
  - [ ] Log request start: `tracing::debug!(week_id = %week_id, user_id = %user_id, "Loading week detail")`
  - [ ] Log authorization check: `tracing::warn!(week_id = %week_id, user_id = %user_id, "Authorization failed: week belongs to different user")`
  - [ ] Add OpenTelemetry span with attributes: user_id, week_id, week_status

- [ ] Write integration tests (AC: 9, 10)
  - [ ] Create `test_get_week_detail_with_valid_week_id()` in `crates/api/tests/integration/test_week_navigation.rs`
  - [ ] Setup test database with test user, meal plan, meal assignments, shopping list
  - [ ] Create valid JWT cookie for test user
  - [ ] Make GET request to `/plan/week/{week_id}` with JWT cookie
  - [ ] Assert response status is 200 OK
  - [ ] Parse JSON response and validate structure (week data, meal_assignments array with 21 items, shopping_list with categories)
  - [ ] Verify navigation links (previous_week_id, next_week_id) are correct

  - [ ] Create `test_get_week_detail_with_invalid_week_id()` test
  - [ ] Make GET request with non-existent week_id UUID
  - [ ] Assert response status is 404 Not Found
  - [ ] Verify error JSON body includes "WeekNotFound" error code

  - [ ] Create `test_get_week_detail_authorization_failure()` test
  - [ ] Create two test users with separate meal plans
  - [ ] Authenticate as user A, attempt to access user B's week_id
  - [ ] Assert response status is 403 Forbidden
  - [ ] Verify error JSON body includes "Forbidden" error code

- [ ] Write performance test
  - [ ] Create `test_week_detail_latency_under_100ms()` in `crates/api/tests/performance/route_latency_tests.rs`
  - [ ] Measure route response time with realistic data (7 days × 3 meals = 21 assignments, 30 shopping items)
  - [ ] Assert P95 latency < 100ms (read-only query target from NFR)

- [ ] Register route in Axum router
  - [ ] Add route to router configuration in `crates/api/src/main.rs` or router module
  - [ ] Ensure authentication middleware is applied
  - [ ] Ensure database pool is available via Extension

## Dev Notes

### Architecture Patterns
- **Read-Only Route**: No evento events emitted, pure read model query for performance
- **Authorization Pattern**: Verify resource ownership (week.user_id == JWT user_id) before returning data
- **Navigation Support**: Calculate previous/next week IDs for frontend week navigation UI
- **Performance Critical**: P95 <100ms target for read-only query (caching not required initially)

### Source Tree Components
- **Route Handler**: `crates/api/src/routes/meal_planning.rs` - Add `get_week_detail` function
- **Error Types**: `crates/api/src/errors.rs` - Add WeekNotFound and Forbidden variants to ApiError enum
- **Integration Tests**: `crates/api/tests/integration/test_week_navigation.rs` (new file)
- **Performance Tests**: `crates/api/tests/performance/route_latency_tests.rs`

### Testing Standards
- **Coverage Target**: 100% coverage for authorization logic (critical security path)
- **Security Testing**: Verify cross-user access prevention (403 Forbidden)
- **Edge Cases**: Invalid UUID format, missing week_id, empty shopping list

### Database Query Optimization
- **Indexes**: Ensure indexes on `meal_plans(user_id, id)` and `meal_assignments(meal_plan_id)` for fast lookups (Epic 6 migrations)
- **JOIN Strategy**: Single query with JOIN for meal assignments + recipes (avoid N+1 queries)
- **Shopping List Query**: Use LEFT JOIN to handle weeks without shopping lists gracefully
- **Navigation Queries**: Use indexed queries on `start_date` to efficiently find previous/next weeks

### Response Format
- **JSON Structure**: Matches schema defined in tech spec section "Data Models and Contracts"
- **Date Formatting**: ISO 8601 format (YYYY-MM-DD) for start_date and end_date
- **Week Status**: Enum values: "past", "current", "future" (calculated based on current date)
- **Shopping List Categories**: Standardized order: Produce, Dairy, Meat & Seafood, Pantry, Frozen, Bakery

### Frontend Integration (Epic 9)
- **TwinSpark Usage**: Frontend uses `ts-req="/plan/week/{week_id}"` to load week view
- **Response Target**: `ts-target="#meal-calendar"` swaps calendar content
- **Navigation**: Previous/Next week buttons use week_links from navigation object
- **Askama Templates**: Epic 9 renders JSON response data via server-side templates

### Project Structure Notes
- Aligns with read model query pattern (no write operations)
- No evento events emitted (pure read path for performance)
- Database connection pooling configured (min 5, max 20 connections)
- Rate limiting not required for read-only route

### References

**Technical Specification Sections:**
- [Source: docs/tech-spec-epic-8.md#APIs and Interfaces - GET /plan/week/:week_id] - Route signature and response schema
- [Source: docs/tech-spec-epic-8.md#Data Models and Contracts] - WeekDetailResponse JSON structure
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Performance] - P95 <100ms target for week detail route
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Security - Authorization] - Verify week.user_id == user_id authorization check
- [Source: docs/tech-spec-epic-8.md#Dependencies and Integrations - Database Integration] - Read model queries: meal_plans, meal_assignments, shopping_lists tables
- [Source: docs/tech-spec-epic-8.md#Acceptance Criteria - Story 8.2] - Full acceptance criteria breakdown
- [Source: docs/tech-spec-epic-8.md#Traceability Mapping] - AC 8.2.1-8.2.8 test ideas

**UX Specification:**
- [Source: docs/ux-specification.md#Key Screens & Layouts - Meal Planning Calendar] - Week view layout requirements (7-day grid desktop, vertical stack mobile)
- [Source: docs/ux-specification.md#Component Library - Calendar Week View Component] - Week navigation UI patterns

**Architecture Documents:**
- [Source: docs/tech-spec-epic-8.md#System Architecture Alignment - Read Models] - meal_plans, meal_assignments, shopping_lists tables queried via SQLx

## Dev Agent Record

### Context Reference

<!-- Story 8.2 context - Epic 8: Enhanced Meal Planning - Backend Routes & Handlers -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

N/A - Story creation phase

### Completion Notes List

- Created from tech spec Epic 8 acceptance criteria 8.2.1-8.2.10
- All tasks derived from Detailed Design and API specifications
- Authorization checks critical for security
- Performance target: P95 <100ms for read-only query

### File List

- `/home/snapiz/projects/github/timayz/imkitchen/docs/stories/story-8.2.md` (this file)
