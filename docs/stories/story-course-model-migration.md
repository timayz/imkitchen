# Story: Course-Based Meal Planning Model Migration

**Story ID:** COURSE-MODEL-1
**Epic:** Technical Debt / Architecture Improvement
**Priority:** High
**Status:** Approved
**Estimate:** 15-21 hours

## Story Description

As a **system architect**, I need to **migrate the meal planning model from breakfast/lunch/dinner to appetizer/main_course/dessert** so that **the system supports course-based lunch planning instead of three separate daily meals**.

## Background

Current model:
- Each day has 3 meals: breakfast, lunch, dinner
- Meal plan structure: 7 days × 3 meals = 21 meal slots
- Recipes don't have type classification

New model:
- Each day has 1 lunch with 3 courses: appetizer, main course, dessert
- Meal plan structure: 7 days × 3 courses = 21 course slots
- Recipes must specify type: appetizer, main_course, or dessert

## Acceptance Criteria

### AC-1: Database Schema Changes
**Given** the existing database schema with meal_type column
**When** the migration runs
**Then**
- `recipes` table has `recipe_type` column with values: "appetizer", "main_course", or "dessert"
- `meal_assignments` table uses `course_type` instead of `meal_type`
- All existing recipes are classified using intelligent heuristics
- All existing meal assignments are converted (breakfast→appetizer, lunch→main_course, dinner→dessert)
- Appropriate indexes are created for course-based queries

### AC-2: Recipe Domain Updates
**Given** a user creating a new recipe
**When** they submit the create recipe form
**Then**
- Recipe must include `recipe_type` selection (appetizer, main_course, or dessert)
- `RecipeCreated` event includes `recipe_type` field
- `RecipeAggregate` stores `recipe_type` field
- Validation rejects invalid recipe types
- Read model projection includes `recipe_type`

### AC-3: Recipe Update Support
**Given** a user editing an existing recipe
**When** they update recipe details
**Then**
- User can change `recipe_type` value
- `RecipeUpdated` event includes optional `recipe_type` field
- Aggregate state updates when recipe type changes
- Read model reflects the updated type

### AC-4: Meal Planning Algorithm Updates
**Given** a user generating a meal plan
**When** the algorithm runs
**Then**
- Recipes are grouped by course type (appetizer, main_course, dessert)
- Each day gets exactly 1 appetizer, 1 main_course, and 1 dessert
- Algorithm only assigns recipes to matching course slots
- Rotation logic works within each course type independently
- Error is returned if insufficient recipes of any course type

### AC-5: Replace Meal Command Updates
**Given** a user replacing a specific meal slot
**When** they select a replacement recipe
**Then**
- Command uses `course_type` instead of `meal_type`
- Only recipes matching the course type are available as replacements
- `MealReplaced` event uses `course_type` field
- Aggregate state updates correctly

### AC-6: UI Updates - Meal Calendar
**Given** a user viewing the meal calendar
**When** they see the week view
**Then**
- Each day shows 3 course slots: Appetizer, Main Course, Dessert
- Course labels are clearly displayed
- Recipes are grouped by course within each day
- Replace meal functionality works with course-aware recipe selection

### AC-7: UI Updates - Dashboard
**Given** a user viewing the dashboard
**When** they see today's meals
**Then**
- Dashboard displays "Today's Lunch" (singular)
- Shows 3 courses: Appetizer, Main Course, Dessert
- Each course displays the assigned recipe
- Prep indicators work for each course

### AC-8: UI Updates - Recipe Forms
**Given** a user creating or editing a recipe
**When** they view the recipe form
**Then**
- Form includes recipe type selector with options: Appetizer, Main Course, Dessert
- Recipe type is required field (cannot be empty)
- Validation error shown if recipe type not selected
- Edit form pre-selects current recipe type

### AC-9: Backward Compatibility
**Given** existing event data in the event store
**When** aggregates are replayed from events
**Then**
- Old `RecipeCreated` events (without recipe_type) default to "main_course"
- Old `MealAssignment` data (with meal_type) is converted to course_type
- No data loss occurs during migration
- System functions correctly with mixed old/new event data

### AC-10: Test Coverage
**Given** the course-based model implementation
**When** tests are executed
**Then**
- Unit tests cover recipe type validation
- Unit tests cover algorithm course matching logic
- Integration tests cover full meal plan generation with course types
- E2E tests cover UI interactions with course-based model
- All existing tests updated to use course terminology
- Test coverage remains ≥80%

## Technical Implementation

### Migration Plan Reference
See: `docs/migration-course-based-model.md`

### Files to Modify

#### Database
- `migrations/04_v0.5_course_based_model.sql` ✅ COMPLETED

#### Recipe Domain
- `crates/recipe/src/events.rs` - Add `recipe_type` to RecipeCreated, RecipeUpdated
- `crates/recipe/src/aggregate.rs` - Add `recipe_type` field and event handlers
- `crates/recipe/src/commands.rs` - Add `recipe_type` to commands, add validation
- `crates/recipe/src/read_model.rs` - Update projections to include recipe_type

#### Meal Planning Domain
- `crates/meal_planning/src/events.rs` - Rename MealType → CourseType enum
- `crates/meal_planning/src/aggregate.rs` - Update event handlers
- `crates/meal_planning/src/algorithm.rs` - Rewrite to assign by course type
- `crates/meal_planning/src/commands.rs` - Update to use course_type
- `crates/meal_planning/src/constraints.rs` - Update for course-based logic

#### Routes & Handlers
- `src/routes/meal_plan.rs` - Update handlers to use course_type
- `src/routes/dashboard.rs` - Update queries for course-based model
- `src/routes/recipes.rs` - Add recipe_type to form handling

#### Templates
- `templates/pages/meal-calendar.html` - Update to show courses instead of meals
- `templates/pages/dashboard.html` - Update to show "Today's Lunch" with courses
- `templates/pages/recipe-form.html` - Add recipe type selector
- `templates/components/*` - Update meal-related components

#### Tests
- `crates/recipe/tests/recipe_tests.rs` - Add recipe_type validation tests
- `crates/meal_planning/tests/*.rs` - Update to use CourseType
- `tests/meal_plan_integration_tests.rs` - Update assertions
- Add new test cases for course matching

## Dependencies

- ✅ Migration plan document created
- ✅ Database migration file created
- ✅ Architecture document updated

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Database migration runs successfully
- [ ] All code changes implemented
- [ ] All tests passing (unit, integration, E2E)
- [ ] Code coverage ≥80%
- [ ] No regressions in existing functionality
- [ ] UI displays courses correctly
- [ ] Documentation updated

## Dev Agent Record

**Context Reference:**
- Migration Plan: `docs/migration-course-based-model.md`
- Architecture Doc: `docs/solution-architecture.md` (Section 2.0)
- Database Migration: `migrations/04_v0.5_course_based_model.sql`

**Implementation Notes:**
- Follow TDD approach: write tests first, then implementation
- Maintain backward compatibility with existing event data
- Use intelligent heuristics for classifying existing recipes
- Ensure transaction safety during database migration
