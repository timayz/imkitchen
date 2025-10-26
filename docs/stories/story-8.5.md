# Story 8.5: Create User Preferences Update Route

Status: Approved

## Story

As a backend developer,
I want to create a PUT route that updates user meal planning preferences,
so that authenticated users can customize meal plan generation constraints via HTTP API.

## Acceptance Criteria

1. Route `PUT /profile/meal-planning-preferences` created
2. Handler validates input (`max_prep_time_weeknight` > 0, `cuisine_variety_weight` 0.0-1.0)
3. Handler commits `UserMealPlanningPreferencesUpdated` event to evento
4. Handler returns updated preferences in response
5. Returns 400 if validation fails with field-specific error messages
6. Integration test: PUT updates preferences successfully
7. Integration test: PUT with invalid data returns 400 with validation errors

## Tasks / Subtasks

- [ ] Define route handler function signature (AC: 1)
  - [ ] Create `update_meal_planning_preferences` function in `crates/api/src/routes/user_preferences.rs` (new file)
  - [ ] Add function signature with Axum extractors: `Extension(user_id)`, `Json(payload)`, `Extension(db)`, `Extension(executor)`
  - [ ] Define `MealPlanningPreferences` struct with fields: max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight
  - [ ] Add `#[put("/profile/meal-planning-preferences")]` attribute
  - [ ] Ensure route is protected by authentication middleware

- [ ] Define MealPlanningPreferences request struct with validation (AC: 2)
  - [ ] Add `validator` crate dependency to Cargo.toml if not present
  - [ ] Define struct fields:
    ```rust
    pub struct MealPlanningPreferences {
        #[validate(range(min = 1, message = "Must be greater than 0"))]
        pub max_prep_time_weeknight: i32,

        #[validate(range(min = 1, message = "Must be greater than 0"))]
        pub max_prep_time_weekend: i32,

        pub avoid_consecutive_complex: bool,

        #[validate(range(min = 0.0, max = 1.0, message = "Must be between 0.0 and 1.0"))]
        pub cuisine_variety_weight: f64,
    }
    ```
  - [ ] Implement `Validate` trait for struct

- [ ] Implement input validation (AC: 2, 5)
  - [ ] Extract JSON payload from request using `Json<MealPlanningPreferences>` extractor
  - [ ] Call `payload.validate()` method from validator crate
  - [ ] If validation fails → collect field-specific error messages
  - [ ] Build `ValidationFailed` error with HashMap<String, String> mapping field names to error messages
  - [ ] Return 400 Bad Request with validation errors in JSON body
  - [ ] Log validation failures with structured tracing

- [ ] Emit UserMealPlanningPreferencesUpdated evento event (AC: 3)
  - [ ] Build `UserMealPlanningPreferencesUpdated` event struct with:
    - user_id (from JWT claims)
    - max_prep_time_weeknight (from payload)
    - max_prep_time_weekend (from payload)
    - avoid_consecutive_complex (from payload)
    - cuisine_variety_weight (from payload)
    - updated_at: DateTime<Utc> (timestamp)
  - [ ] Call `executor.emit(event).await`
  - [ ] Handle event emission errors → return 500 with internal error message
  - [ ] Log event emission success with structured tracing

- [ ] Build JSON response (AC: 4)
  - [ ] Construct `PreferencesResponse` struct with:
    - preferences: MealPlanningPreferences (echoes submitted values)
    - message: "Meal planning preferences updated. Changes will apply to your next meal plan generation."
  - [ ] Serialize to JSON using serde_json
  - [ ] Return `Ok(Json(response))` with 200 status

- [ ] Implement error handling (AC: 5)
  - [ ] Add `ApiError` variant: ValidationFailed(HashMap<String, String>)
  - [ ] Implement `IntoResponse` for ValidationFailed:
    - Return 400 status
    - JSON body: { "error": "ValidationFailed", "message": "Invalid preferences provided.", "details": { field: error_message } }
  - [ ] Include structured error logging for debugging

- [ ] Add structured logging and tracing
  - [ ] Log request start: `tracing::info!(user_id = %user_id, "Meal planning preferences update requested")`
  - [ ] Log validation: `tracing::debug!(user_id = %user_id, "Validating preferences payload")`
  - [ ] Log validation failure: `tracing::warn!(user_id = %user_id, errors = ?validation_errors, "Preferences validation failed")`
  - [ ] Log successful update: `tracing::info!(user_id = %user_id, "Preferences updated successfully")`
  - [ ] Add OpenTelemetry span with attributes: user_id, updated_fields

- [ ] Write integration tests (AC: 6, 7)
  - [ ] Create `test_update_preferences_successfully()` in `crates/api/tests/integration/test_preferences.rs` (new file)
  - [ ] Setup test database with test user
  - [ ] Create valid JWT cookie for test user
  - [ ] Make PUT request to `/profile/meal-planning-preferences` with valid JSON payload:
    ```json
    {
      "max_prep_time_weeknight": 30,
      "max_prep_time_weekend": 90,
      "avoid_consecutive_complex": true,
      "cuisine_variety_weight": 0.7
    }
    ```
  - [ ] Assert response status is 200 OK
  - [ ] Parse JSON response and validate structure (preferences object, message)
  - [ ] Subscribe to UserMealPlanningPreferencesUpdated event using `unsafe_oneshot` for synchronous processing
  - [ ] Verify read models updated: Query users table, assert preferences fields match submitted values

  - [ ] Create `test_update_preferences_with_invalid_data_returns_400()` test
  - [ ] Make PUT request with invalid JSON payload:
    ```json
    {
      "max_prep_time_weeknight": -5,
      "max_prep_time_weekend": 0,
      "avoid_consecutive_complex": true,
      "cuisine_variety_weight": 1.5
    }
    ```
  - [ ] Assert response status is 400 Bad Request
  - [ ] Parse JSON response and verify:
    - error: "ValidationFailed"
    - details.max_prep_time_weeknight: "Must be greater than 0"
    - details.max_prep_time_weekend: "Must be greater than 0"
    - details.cuisine_variety_weight: "Must be between 0.0 and 1.0"

  - [ ] Create `test_update_preferences_without_jwt_returns_401()` test
  - [ ] Make PUT request without JWT cookie
  - [ ] Assert response status is 401 Unauthorized

  - [ ] Create `test_update_preferences_with_boundary_values()` test
  - [ ] Test boundary values:
    - max_prep_time_weeknight: 1 (minimum valid)
    - max_prep_time_weekend: 1 (minimum valid)
    - cuisine_variety_weight: 0.0 (minimum valid)
    - cuisine_variety_weight: 1.0 (maximum valid)
  - [ ] Assert all boundary values accepted (200 OK)

- [ ] Write performance test
  - [ ] Create `test_preferences_update_latency_under_100ms()` in `crates/api/tests/performance/route_latency_tests.rs`
  - [ ] Measure route response time with valid preferences payload
  - [ ] Assert P95 latency < 100ms (simple UPDATE query target from NFR)

- [ ] Register route in Axum router
  - [ ] Add route to router configuration in `crates/api/src/main.rs` or router module
  - [ ] Ensure authentication middleware is applied
  - [ ] Ensure database pool and evento executor are available via Extension

## Dev Notes

### Architecture Patterns
- **Event-Sourced CQRS**: Route emits UserMealPlanningPreferencesUpdated event, projection updates users table
- **Input Validation**: Use `validator` crate for declarative validation rules (DRY principle)
- **Field-Specific Errors**: Return structured validation errors mapping fields to error messages (UX principle)
- **Immediate Effect**: Preferences apply to NEXT meal plan generation, not retroactive to existing plans

### Source Tree Components
- **Route Handler**: `crates/api/src/routes/user_preferences.rs` (new file) - Create `update_meal_planning_preferences` function
- **Request Types**: Define `MealPlanningPreferences` struct with validator attributes
- **Response Types**: Define `PreferencesResponse` struct with preferences and message
- **Error Types**: `crates/api/src/errors.rs` - Add ValidationFailed variant to ApiError enum
- **Integration Tests**: `crates/api/tests/integration/test_preferences.rs` (new file)
- **Performance Tests**: `crates/api/tests/performance/route_latency_tests.rs`

### Testing Standards
- **Coverage Target**: 100% coverage for validation logic (all validation rules tested)
- **Edge Cases**: Boundary values (min/max), negative values, zero values, out-of-range floats
- **Security Testing**: Verify authentication required (401 without JWT)
- **Validation Testing**: Each validation rule has dedicated test case

### Key Technical Constraints
- **Validation Rules**:
  - max_prep_time_weeknight: Must be > 0 (no zero or negative)
  - max_prep_time_weekend: Must be > 0 (no zero or negative)
  - avoid_consecutive_complex: Boolean (true/false, no validation needed)
  - cuisine_variety_weight: Must be 0.0 <= value <= 1.0 (inclusive range)
- **Rust Validator Crate**: Use `#[validate]` attributes for declarative validation
- **Error Messages**: User-friendly, field-specific, actionable (e.g., "Must be greater than 0")

### Validation Implementation Pattern
```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct MealPlanningPreferences {
    #[validate(range(min = 1))]
    pub max_prep_time_weeknight: i32,

    #[validate(range(min = 1))]
    pub max_prep_time_weekend: i32,

    pub avoid_consecutive_complex: bool,

    #[validate(range(min = 0.0, max = 1.0))]
    pub cuisine_variety_weight: f64,
}

// In handler:
let payload = Json::<MealPlanningPreferences>::from_request(req).await?;
if let Err(errors) = payload.validate() {
    let field_errors: HashMap<String, String> = errors
        .field_errors()
        .iter()
        .map(|(field, errors)| (field.to_string(), errors[0].message.to_string()))
        .collect();
    return Err(ApiError::ValidationFailed(field_errors));
}
```

### Evento Event Schema
```rust
UserMealPlanningPreferencesUpdated {
    user_id: String,                    // UUID from JWT claims
    max_prep_time_weeknight: i32,       // Minutes (e.g., 30)
    max_prep_time_weekend: i32,         // Minutes (e.g., 90)
    avoid_consecutive_complex: bool,    // True/false
    cuisine_variety_weight: f64,        // 0.0-1.0 (e.g., 0.7)
    updated_at: DateTime<Utc>,          // Timestamp
}
```

### Evento Projection (Epic 6 Story 6.6)
- **Projection Handler**: Subscribes to UserMealPlanningPreferencesUpdated event
- **Database Update**: `UPDATE users SET max_prep_time_weeknight = ?, max_prep_time_weekend = ?, avoid_consecutive_complex = ?, cuisine_variety_weight = ? WHERE id = ?`
- **Idempotency**: Last event wins (preferences updates are idempotent)
- **Test Pattern**: Use `unsafe_oneshot` to process projection synchronously in tests

### Workflow Sequence
1. Frontend: User edits preferences in Profile settings form
2. Frontend: User clicks "Save Preferences" button
3. TwinSpark: `ts-req="/profile/meal-planning-preferences" ts-req-method="PUT" ts-data='{preferences}'`
4. Route Handler: Extract JSON payload, validate input
5. Route Handler: Emit UserMealPlanningPreferencesUpdated event
6. Evento Projection: UPDATE users table with new preferences
7. Route Handler: Return 200 OK with updated preferences + message
8. Frontend: TwinSpark displays success toast "Preferences updated. Changes will apply to your next meal plan."

### Preference Application Timeline
- **Immediate**: Preferences saved to users table immediately after event processed
- **Next Meal Plan**: Preferences applied when user generates next meal plan (Story 8.1 loads preferences from users table)
- **Not Retroactive**: Existing meal plans NOT regenerated automatically when preferences change
- **User Control**: User can manually regenerate existing plans (Story 8.3, 8.4) to apply new preferences

### Default Values
- **Backend Defaults** (if user never sets preferences):
  - max_prep_time_weeknight: 45 minutes (typical weeknight cooking time)
  - max_prep_time_weekend: 120 minutes (weekend allows longer prep)
  - avoid_consecutive_complex: true (prevent fatigue from back-to-back complex recipes)
  - cuisine_variety_weight: 0.5 (balanced variety)
- **Database Schema** (Epic 6): Preferences stored as nullable columns, defaults applied at query time if NULL

### Project Structure Notes
- New route file: `crates/api/src/routes/user_preferences.rs` (separate from meal_planning.rs for organization)
- Aligns with event-driven monolith architecture
- Simple pattern: Route → Validate → Evento Event → Response
- Database connection pooling configured (min 5, max 20 connections)
- Rate limiting not critical for preference updates (low-frequency operation)

### References

**Technical Specification Sections:**
- [Source: docs/tech-spec-epic-8.md#APIs and Interfaces - PUT /profile/meal-planning-preferences] - Route signature and response schema
- [Source: docs/tech-spec-epic-8.md#Data Models and Contracts - User Preferences Update Request/Response] - MealPlanningPreferences JSON structure and validation error format
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Performance] - P95 <100ms target for preferences update route
- [Source: docs/tech-spec-epic-8.md#Non-Functional Requirements - Security - Input Validation] - Validation rules for each preference field
- [Source: docs/tech-spec-epic-8.md#Dependencies and Integrations - validator crate] - Use validator 0.18+ for input validation
- [Source: docs/tech-spec-epic-8.md#Acceptance Criteria - Story 8.5] - Full acceptance criteria breakdown
- [Source: docs/tech-spec-epic-8.md#Traceability Mapping] - AC 8.5.1-8.5.5 test ideas

**UX Specification:**
- [Source: docs/ux-specification.md#User Interface Requirements - Profile & Settings] - User profile preferences section

**Architecture Documents:**
- [Source: docs/tech-spec-epic-8.md#System Architecture Alignment - Read Models] - Users table stores meal planning preferences (Epic 6 Story 6.4)

## Dev Agent Record

### Context Reference

<!-- Story 8.5 context - Epic 8: Enhanced Meal Planning - Backend Routes & Handlers -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

N/A - Story creation phase

### Completion Notes List

- Created from tech spec Epic 8 acceptance criteria 8.5.1-8.5.7
- All tasks derived from Detailed Design and API specifications
- Input validation critical for algorithm constraints (prep time > 0, variety weight 0-1)
- Preferences apply to next meal plan generation, not retroactive

### File List

- `/home/snapiz/projects/github/timayz/imkitchen/docs/stories/story-8.5.md` (this file)
