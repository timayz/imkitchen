# Story 3.1: Generate Initial Meal Plan

Status: ContextReadyDraft

## Story

As a **user with favorite recipes**,
I want **to generate an automated weekly meal plan**,
so that **I don't have to manually plan meals**.

## Acceptance Criteria

1. Home dashboard displays "Generate Meal Plan" button prominently
2. Clicking button triggers meal planning algorithm
3. System analyzes all favorited recipes against user profile constraints
4. Algorithm generates single meal plan with recipes organized by week
5. Week-view calendar displays generated plan with breakfast/lunch/dinner slots filled
6. Generation completes within 5 seconds for up to 50 favorite recipes
7. Progress indicator shown during generation
8. Generated plan automatically becomes active
9. User redirected to calendar view after successful generation
10. If insufficient recipes (<7 favorites), display helpful error: "Add more favorite recipes to generate meal plan (need at least 7)"

## Tasks / Subtasks

### Task 1: Create MealPlan Domain Aggregate (AC: 1-10)
- [ ] Create `crates/meal_planning/` workspace crate in Cargo.toml
- [ ] Define MealPlan aggregate structure with evento integration
- [ ] Implement evento::AggregatorName trait for MealPlan
- [ ] Define domain events: MealPlanGenerated, RecipeUsedInRotation
- [ ] Implement event handlers for aggregate state reconstruction
  - [ ] `meal_plan_generated()` - initializes meal plan state
  - [ ] `recipe_used_in_rotation()` - updates rotation tracking
- [ ] Add bincode Encode/Decode derives for serialization
- [ ] Write unit tests for MealPlan aggregate event sourcing

### Task 2: Implement Multi-Factor Meal Planning Algorithm (AC: 2-3)
- [ ] Create `crates/meal_planning/src/algorithm.rs` module
- [ ] Implement RecipeComplexityCalculator service
  - [ ] Calculate complexity score: (ingredients * 0.3) + (steps * 0.4) + (advance_prep * 0.3)
  - [ ] Map score to Complexity enum (Simple, Moderate, Complex)
- [ ] Implement MealPlanningAlgorithm service
  - [ ] Filter recipes by rotation state (unused in current cycle)
  - [ ] Score recipes by complexity for each meal slot
  - [ ] Apply constraints: availability, dietary restrictions, advance prep timing
  - [ ] Generate 21 meal slot assignments (7 days × 3 meals)
- [ ] Ensure O(n) algorithm complexity where n = favorite recipe count
- [ ] Write unit tests for complexity calculation and constraint satisfaction
  - [ ] Test: Simple recipe (<8 ingredients, <6 steps) assigned to weeknights
  - [ ] Test: Complex recipe (>15 ingredients or advance prep) assigned to weekends
  - [ ] Test: Dietary restrictions filter recipes correctly

### Task 3: Create Meal Plan Read Models (AC: 4-5)
- [ ] Create migration `migrations/008_create_meal_plans_table.sql`
  - [ ] `meal_plans` table: id, user_id, start_date, status (active/archived), rotation_state, created_at
  - [ ] `meal_assignments` table: id, meal_plan_id, date, meal_type (breakfast/lunch/dinner), recipe_id, prep_required
  - [ ] Foreign key constraints and indexes
- [ ] Implement evento subscription handlers for read model projections
  - [ ] `project_meal_plan_generated()` - inserts into meal_plans and meal_assignments
  - [ ] `project_recipe_used_in_rotation()` - updates rotation_state JSON
- [ ] Register subscriptions in main.rs with evento subscriber
- [ ] Write integration tests for read model projection accuracy

### Task 4: Implement Rotation System (AC: 10, references Story 3.3)
- [ ] Create `crates/meal_planning/src/rotation.rs` module
- [ ] Define RotationState struct tracking used recipe IDs and cycle number
- [ ] Implement rotation logic:
  - [ ] Track which recipes used in current cycle
  - [ ] Filter favorites to unused recipes only
  - [ ] Reset cycle when all favorites used once
- [ ] Store rotation state as JSON in meal_plans.rotation_state column
- [ ] Write unit tests for rotation cycle management
  - [ ] Test: Rotation prevents duplicate recipes within same cycle
  - [ ] Test: Rotation resets after all favorites used once

### Task 5: Create HTTP Routes for Meal Plan Generation (AC: 1-2, 7-9)
- [ ] Implement POST /plan/generate route in `src/routes/meal_plan.rs`
  - [ ] Validate user has ≥7 favorite recipes (AC-10)
  - [ ] Query user profile for constraints (availability, skill level, dietary restrictions)
  - [ ] Query favorited recipes with complexity scores
  - [ ] Load rotation state from most recent meal plan
  - [ ] Invoke MealPlanningAlgorithm.generate()
  - [ ] Create MealPlan aggregate with GenerateMealPlan command
  - [ ] Emit MealPlanGenerated event via evento
  - [ ] Return 302 redirect to /plan (calendar view)
- [ ] Implement GET /plan route for calendar view
  - [ ] Query active meal plan for current user
  - [ ] Join meal_assignments with recipes for display data
  - [ ] Render calendar template with meal assignments
- [ ] Add progress indicator to generation flow (TwinSpark or loading spinner)
- [ ] Write integration tests for /plan/generate endpoint
  - [ ] Test: Successful generation with 10 favorites redirects to calendar
  - [ ] Test: Insufficient recipes (<7) returns 422 with error message

### Task 6: Create Askama Templates for Calendar View (AC: 5, 9)
- [ ] Create `templates/pages/meal-calendar.html`
  - [ ] 7-day week view grid layout (responsive: mobile vertical, desktop grid)
  - [ ] Each day shows 3 meal slots (breakfast, lunch, dinner)
  - [ ] Each slot displays: recipe title, image placeholder, prep time
  - [ ] Advance prep indicator icon if recipe.advance_prep_hours > 0
  - [ ] Complexity badge (Simple/Moderate/Complex) per recipe
  - [ ] Empty slot handling ("No meal planned")
  - [ ] "Regenerate Meal Plan" button
- [ ] Create meal-slot component in `templates/components/meal-slot.html`
  - [ ] Reusable meal slot card with recipe details
  - [ ] Clickable to view recipe detail
- [ ] Update `templates/pages/dashboard.html`
  - [ ] Add prominent "Generate Meal Plan" button if no active plan
  - [ ] Show "View Meal Plan" link if plan exists
- [ ] Write E2E tests with Playwright for calendar rendering
  - [ ] Test: Generated meal plan displays 7 days with 21 slots filled

### Task 7: Handle Insufficient Recipes Error (AC: 10)
- [ ] Implement validation in POST /plan/generate route
  - [ ] Query COUNT(recipes WHERE user_id = ? AND is_favorited = true)
  - [ ] If count < 7, return 422 Unprocessable Entity
  - [ ] Error template displays: "You need at least 7 favorite recipes. You currently have {count}."
  - [ ] Include helpful guidance: "Add {7 - count} more recipes to get started!"
  - [ ] Link to /recipes/new and /discover pages
- [ ] Update error.html template to handle InsufficientRecipes error variant
- [ ] Write integration test for insufficient recipes scenario

### Task 8: Performance Testing and Optimization (AC: 6)
- [ ] Load test algorithm with 50 favorite recipes
  - [ ] Verify generation completes in <5 seconds
  - [ ] Profile algorithm execution with `cargo flamegraph` if needed
- [ ] Optimize database queries with proper indexes
  - [ ] Index on recipes(user_id, is_favorited)
  - [ ] Index on meal_plans(user_id, status)
  - [ ] Index on meal_assignments(meal_plan_id, date)
- [ ] Cache user profile and favorite recipes during generation (avoid N+1)
- [ ] Document performance baseline in completion notes

### Task 9: Integration with Shopping List Domain (References Epic 4)
- [ ] Verify MealPlanGenerated event emitted with meal_plan_id
- [ ] Document event payload for shopping list subscription (future Story 4.1)
- [ ] No implementation needed in this story - subscription handled in Epic 4

### Task 10: Write Comprehensive Test Suite (TDD Required)
- [ ] Unit tests in `crates/meal_planning/tests/`
  - [ ] Algorithm constraint satisfaction logic
  - [ ] Rotation cycle management
  - [ ] Complexity calculation accuracy
- [ ] Integration tests in `tests/meal_plan_integration_tests.rs`
  - [ ] Full generation flow with evento event persistence
  - [ ] Read model projection accuracy
  - [ ] HTTP route behavior (success, validation errors)
- [ ] E2E tests in `e2e/tests/meal-planning.spec.ts`
  - [ ] User creates 10 favorite recipes, generates meal plan, views calendar
  - [ ] Validation error shown with <7 favorites
- [ ] Target 80% code coverage for meal_planning crate

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- MealPlan aggregate stores complete history of generation and modification events
- Read models (meal_plans, meal_assignments) built via evento subscriptions
- All meal plan state changes captured as immutable events
- [Source: docs/solution-architecture.md#3.1 - Event Sourcing Pattern]

**Multi-Factor Constraint Satisfaction:**
- Algorithm considers: user availability, recipe complexity, advance prep timing, dietary restrictions, ingredient freshness, equipment conflicts, rotation state
- Weighted scoring function balances constraints: complexity (40%), time fit (40%), freshness (20%)
- Deterministic but varied via randomization seed for generation variety
- [Source: docs/tech-spec-epic-3.md#1 - MealPlanningAlgorithm Overview]

**Recipe Rotation System:**
- Each recipe used exactly once before any recipe repeats
- Rotation state stored as JSON: `{"cycle_number": 1, "used_recipe_ids": ["id1", "id2"]}`
- Cycle resets when all favorites used once, enabling repeated use
- [Source: docs/epics.md - Story 3.3 Recipe Rotation System]

**Performance Requirements:**
- Algorithm must complete in <5 seconds for 50 recipes (NFR)
- O(n) complexity where n = favorite recipe count
- Synchronous execution for MVP (no background jobs)
- [Source: docs/tech-spec-epic-3.md - Performance Requirements]

### Source Tree Components to Touch

**New Files:**
- `crates/meal_planning/Cargo.toml` - New domain crate
- `crates/meal_planning/src/lib.rs` - Module exports
- `crates/meal_planning/src/aggregate.rs` - MealPlan aggregate with evento
- `crates/meal_planning/src/commands.rs` - GenerateMealPlan command
- `crates/meal_planning/src/events.rs` - Domain events (MealPlanGenerated, etc.)
- `crates/meal_planning/src/algorithm.rs` - Meal planning algorithm service
- `crates/meal_planning/src/rotation.rs` - Recipe rotation logic
- `crates/meal_planning/src/read_model.rs` - Query projections
- `crates/meal_planning/src/error.rs` - Domain-specific errors
- `crates/meal_planning/tests/algorithm_tests.rs` - Unit tests
- `crates/meal_planning/tests/rotation_tests.rs` - Rotation unit tests
- `migrations/008_create_meal_plans_table.sql` - Database schema
- `src/routes/meal_plan.rs` - HTTP route handlers
- `templates/pages/meal-calendar.html` - Calendar view template
- `templates/components/meal-slot.html` - Meal slot component
- `tests/meal_plan_integration_tests.rs` - Integration tests
- `e2e/tests/meal-planning.spec.ts` - E2E Playwright tests

**Modified Files:**
- `Cargo.toml` - Add meal_planning workspace member
- `src/main.rs` - Register evento subscriptions for meal plan projections
- `src/routes/mod.rs` - Export meal_plan routes
- `src/server.rs` - Mount /plan routes
- `templates/pages/dashboard.html` - Add "Generate Meal Plan" button
- `src/error.rs` - Add InsufficientRecipes error variant

### Project Structure Notes

**Alignment with unified project structure:**
- Follows DDD bounded context pattern: `crates/meal_planning/` separate from recipe/user domains
- Domain events enable loose coupling: shopping list reacts to MealPlanGenerated via subscription
- CQRS: commands write to event store, queries read from meal_assignments read model
- No conflicts with existing architecture
- [Source: docs/solution-architecture.md#11.1 - Domain Crate Structure]

**Database Schema Alignment:**
- meal_plans table follows naming conventions from solution architecture
- Foreign key relationships: meal_assignments → meal_plans, meal_assignments → recipes
- JSON column for rotation_state enables flexible state tracking without schema changes
- [Source: docs/solution-architecture.md#3.2 - Data Models and Relationships]

**Testing Strategy Alignment:**
- TDD enforced: write tests first, then implementation
- Unit tests: domain logic in crates/meal_planning/tests/
- Integration tests: HTTP routes and evento projections in tests/
- E2E tests: full user journey with Playwright in e2e/
- Target 80% coverage per NFR requirements
- [Source: docs/solution-architecture.md#15 - Testing Strategy]

### References

- **Epic Definition**: [Source: docs/epics.md - Epic 3: Intelligent Meal Planning Engine, Story 3.1 lines 553-577]
- **Technical Specification**: [Source: docs/tech-spec-epic-3.md - MealPlanningAlgorithm, RecipeComplexityCalculator]
- **Architecture**: [Source: docs/solution-architecture.md#3.1 - Event Sourcing, #3.2 - Data Models, #11.1 - Domain Crate Structure]
- **Technology Stack**: [Source: docs/solution-architecture.md#1.1 - evento 1.3+ for event sourcing, SQLx for read models]
- **Rotation Logic**: [Source: docs/epics.md - Story 3.3 Recipe Rotation System lines 605-627]
- **Performance**: [Source: docs/tech-spec-epic-3.md - Algorithm Performance: O(n), <5 second target]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.1.xml`

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
