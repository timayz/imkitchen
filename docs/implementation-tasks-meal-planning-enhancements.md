# Implementation Tasks: Enhanced Meal Planning System

**Date:** 2025-10-24
**Version:** 1.0
**Architecture Doc:** `docs/architecture-update-meal-planning-enhancements.md`
**Estimated Timeline:** 9 weeks (6 phases)

---

## Overview

This document breaks down the implementation of three major features:
1. Multi-Week Meal Plan Generation (5 weeks max)
2. Accompaniment Recipe Type System
3. User Preferences Integration

**Delivery Strategy:** Phased rollout with incremental database migrations and feature flags where appropriate.

---

## Phase 1: Database & Domain Foundation (Week 1-2)

**Goal:** Update database schema, domain models, and evento events to support new features.

### Task 1.1: Database Migration

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** Backend Lead

**Description:**
Create and test database migration for all schema changes.

**Acceptance Criteria:**
- [ ] Migration file created: `migrations/XXX_enhanced_meal_planning.sql`
- [ ] All schema changes implemented:
  - [ ] `meal_plans` table: Add `end_date`, `is_locked`, `generation_batch_id`
  - [ ] `recipes` table: Add `accepts_accompaniment`, `preferred_accompaniments`, `accompaniment_category`, `cuisine`, `dietary_tags`
  - [ ] `meal_assignments` table: Add `accompaniment_recipe_id`
  - [ ] `users` table: Add `max_prep_time_weeknight`, `max_prep_time_weekend`, `avoid_consecutive_complex`, `cuisine_variety_weight`
  - [ ] New table: `meal_plan_rotation_state`
- [ ] All indexes created as per design
- [ ] Triggers created: `prevent_locked_week_modification`, `update_meal_plan_status`
- [ ] Migration tested on development database
- [ ] Rollback migration tested and verified
- [ ] Existing data preserved with sensible defaults

**Dependencies:** None

**Files Modified:**
- `migrations/XXX_enhanced_meal_planning.sql` (new)

---

### Task 1.2: Update Recipe Domain Model

**Priority:** 🔴 Critical
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Update Recipe aggregate, commands, events, and read models to support accompaniments and cuisine.

**Acceptance Criteria:**
- [ ] `RecipeType` enum updated with `Accompaniment` variant
- [ ] `AccompanimentCategory` enum created (Pasta, Rice, Fries, Salad, Bread, Vegetable, Other)
- [ ] `Cuisine` enum created with `Custom(String)` variant
- [ ] `DietaryTag` enum created
- [ ] Recipe struct updated with new fields:
  - [ ] `accepts_accompaniment: bool`
  - [ ] `preferred_accompaniments: Vec<AccompanimentCategory>`
  - [ ] `accompaniment_category: Option<AccompanimentCategory>`
  - [ ] `cuisine: Option<Cuisine>`
  - [ ] `dietary_tags: Vec<DietaryTag>`
- [ ] `RecipeCreated` event updated
- [ ] `RecipeUpdated` event updated
- [ ] New event: `RecipeAccompanimentSettingsUpdated`
- [ ] Read model projections updated
- [ ] Unit tests for all new domain logic

**Dependencies:** Task 1.1 (migration)

**Files Modified:**
- `crates/recipe/src/aggregate.rs`
- `crates/recipe/src/commands.rs`
- `crates/recipe/src/events.rs`
- `crates/recipe/src/read_model.rs`
- `crates/recipe/src/lib.rs` (enum exports)

---

### Task 1.3: Update User Domain Model

**Priority:** 🔴 Critical
**Estimate:** 1 day
**Assignee:** Backend Developer

**Description:**
Update User aggregate to support meal planning preferences.

**Acceptance Criteria:**
- [ ] `UserPreferences` struct created/updated with:
  - [ ] `max_prep_time_weeknight: u32`
  - [ ] `max_prep_time_weekend: u32`
  - [ ] `avoid_consecutive_complex: bool`
  - [ ] `cuisine_variety_weight: f32`
- [ ] New command: `UpdateMealPlanningPreferences`
- [ ] New event: `UserMealPlanningPreferencesUpdated`
- [ ] Default values applied: 30 min weeknight, 90 min weekend, true avoid complex, 0.7 variety
- [ ] Read model projections updated
- [ ] Unit tests for preference logic

**Dependencies:** Task 1.1 (migration)

**Files Modified:**
- `crates/user/src/aggregate.rs`
- `crates/user/src/commands.rs`
- `crates/user/src/events.rs`
- `crates/user/src/read_model.rs`

---

### Task 1.4: Update MealPlan Domain Model

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** Backend Developer

**Description:**
Update MealPlan aggregate for multi-week generation and accompaniments.

**Acceptance Criteria:**
- [ ] `WeekMealPlan` struct updated with:
  - [ ] `end_date: Date`
  - [ ] `is_locked: bool`
  - [ ] `generation_batch_id: String`
  - [ ] `status: WeekStatus` enum (Future, Current, Past, Archived)
- [ ] `MealAssignment` struct updated with:
  - [ ] `accompaniment_recipe_id: Option<RecipeId>`
- [ ] `MultiWeekMealPlan` struct created
- [ ] `RotationState` struct created with:
  - [ ] `used_main_course_ids: Vec<RecipeId>`
  - [ ] `used_appetizer_ids: Vec<RecipeId>`
  - [ ] `used_dessert_ids: Vec<RecipeId>`
  - [ ] `cuisine_usage_count: HashMap<Cuisine, u32>`
  - [ ] `last_complex_meal_date: Option<Date>`
- [ ] New events:
  - [ ] `MultiWeekMealPlanGenerated`
  - [ ] `SingleWeekRegenerated`
  - [ ] `AllFutureWeeksRegenerated`
  - [ ] `ShoppingListGenerated` (updated)
- [ ] Read model projections updated
- [ ] Unit tests for multi-week logic

**Dependencies:** Task 1.1 (migration), Task 1.2 (Recipe), Task 1.3 (User)

**Files Modified:**
- `crates/meal_planning/src/aggregate.rs`
- `crates/meal_planning/src/commands.rs`
- `crates/meal_planning/src/events.rs`
- `crates/meal_planning/src/read_model.rs`
- `crates/meal_planning/src/rotation.rs` (new)
- `crates/meal_planning/src/lib.rs`

---

### Task 1.5: Integration Testing - Domain Layer

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Write integration tests for all domain model changes.

**Acceptance Criteria:**
- [ ] Recipe creation with accompaniment settings tested
- [ ] Recipe cuisine and dietary tags tested
- [ ] User preference updates tested
- [ ] Multi-week meal plan generation tested (event flow)
- [ ] Rotation state persistence tested
- [ ] Week locking logic tested
- [ ] Edge cases covered (insufficient recipes, invalid preferences)
- [ ] All tests passing in CI

**Dependencies:** Tasks 1.2, 1.3, 1.4

**Files Modified:**
- `crates/recipe/tests/` (new test files)
- `crates/user/tests/` (new test files)
- `crates/meal_planning/tests/` (new test files)

---

## Phase 2: Algorithm Implementation (Week 3-4)

**Goal:** Implement multi-week generation algorithm with preference awareness.

### Task 2.1: Dietary Restriction Filter

**Priority:** 🔴 Critical
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Implement filtering logic to exclude recipes incompatible with user dietary restrictions.

**Acceptance Criteria:**
- [ ] `filter_by_dietary_restrictions()` function implemented
- [ ] Handles all `DietaryRestriction` variants:
  - [ ] Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher
  - [ ] Custom allergen matching (case-insensitive ingredient search)
- [ ] Returns only compatible recipes
- [ ] Unit tests for all dietary restriction types
- [ ] Edge case: Empty recipe list handled gracefully

**Dependencies:** Task 1.2 (Recipe domain)

**Files Modified:**
- `crates/meal_planning/src/algorithm.rs` (new)
- `crates/meal_planning/src/preferences.rs` (new)

---

### Task 2.2: Time & Skill Level Filtering

**Priority:** 🔴 Critical
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Implement preference-aware recipe filtering for time constraints and skill level.

**Acceptance Criteria:**
- [ ] Weeknight vs weekend time filtering (30 min vs 90 min defaults)
- [ ] Skill level filtering:
  - [ ] Beginner → Simple only
  - [ ] Intermediate → Simple + Moderate
  - [ ] Advanced → All complexity
- [ ] `filter_by_time_constraint()` function
- [ ] `filter_by_skill_level()` function
- [ ] Combined filtering pipeline
- [ ] Unit tests with various user preferences
- [ ] Performance: < 10ms for 100 recipes

**Dependencies:** Task 1.3 (User preferences), Task 2.1

**Files Modified:**
- `crates/meal_planning/src/preferences.rs`

---

### Task 2.3: Cuisine Variety Scoring

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Implement cuisine variety weighting algorithm.

**Acceptance Criteria:**
- [ ] `select_with_cuisine_variety()` function implemented
- [ ] Cuisine usage tracking in `RotationState`
- [ ] Scoring algorithm:
  - [ ] Penalizes recently used cuisines
  - [ ] Applies `cuisine_variety_weight` (0.0-1.0)
  - [ ] 0.0 = allow frequent repetition, 1.0 = maximize variety
- [ ] Handles `Cuisine::Custom(String)` variants
- [ ] Unit tests with various variety weights
- [ ] Performance: < 5ms for scoring 50 recipes

**Dependencies:** Task 1.4 (Rotation state)

**Files Modified:**
- `crates/meal_planning/src/algorithm.rs`
- `crates/meal_planning/src/rotation.rs`

---

### Task 2.4: Accompaniment Selection Logic

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Implement accompaniment selection for main courses.

**Acceptance Criteria:**
- [ ] `select_accompaniment()` function implemented
- [ ] Checks `main_course.accepts_accompaniment` boolean
- [ ] Filters by `preferred_accompaniments` if specified
- [ ] Random selection from filtered accompaniments
- [ ] Accompaniments can repeat (not tracked in rotation)
- [ ] Returns `Option<Recipe>` (None if not accepted or no favorites)
- [ ] Unit tests with various preference combinations
- [ ] Edge case: No accompaniment favorites handled

**Dependencies:** Task 1.2 (Recipe accompaniment fields)

**Files Modified:**
- `crates/meal_planning/src/algorithm.rs`
- `crates/meal_planning/src/accompaniments.rs` (new)

---

### Task 2.5: Single Week Generation

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** Backend Developer

**Description:**
Implement single week meal plan generation with all preference logic.

**Acceptance Criteria:**
- [ ] `generate_single_week()` function implemented
- [ ] Generates 7 days × 3 courses = 21 meal assignments
- [ ] Per-day logic:
  - [ ] Assign appetizer (rotation tracked)
  - [ ] Assign main course with preferences + optional accompaniment
  - [ ] Assign dessert (rotation tracked)
- [ ] Main courses UNIQUE across all weeks (rotation enforced)
- [ ] Appetizers/desserts can repeat after exhaustion
- [ ] Complexity spacing (avoid consecutive complex if preference set)
- [ ] Advance prep scheduling validated
- [ ] Unit tests for full week generation
- [ ] Performance: < 500ms per week

**Dependencies:** Tasks 2.1, 2.2, 2.3, 2.4

**Files Modified:**
- `crates/meal_planning/src/algorithm.rs`

---

### Task 2.6: Multi-Week Generation Orchestrator

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** Backend Developer

**Description:**
Implement top-level multi-week generation orchestrator.

**Acceptance Criteria:**
- [ ] `generate_multi_week_meal_plans()` function implemented
- [ ] Calculates `max_weeks = min(5, min(appetizers, main_courses, desserts))`
- [ ] Generates each week sequentially with updated rotation state
- [ ] Generates shopping list per week
- [ ] Assigns `generation_batch_id` to link weeks
- [ ] Sets week status (Future, Current based on dates)
- [ ] Locks current week automatically
- [ ] Error handling:
  - [ ] Insufficient recipes (< 1 week possible)
  - [ ] Algorithm timeout
- [ ] Unit tests for 1, 3, 5 week scenarios
- [ ] Integration test: Full multi-week flow
- [ ] Performance: < 5 seconds for 5 weeks

**Dependencies:** Task 2.5

**Files Modified:**
- `crates/meal_planning/src/algorithm.rs`
- `crates/meal_planning/src/lib.rs`

---

### Task 2.7: Shopping List Generation

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Update shopping list generation to include accompaniment ingredients.

**Acceptance Criteria:**
- [ ] `generate_shopping_list_for_week()` function updated
- [ ] Includes main recipe ingredients
- [ ] Includes accompaniment recipe ingredients (if present)
- [ ] Aggregates duplicate ingredients
- [ ] Categorizes by grocery section (Produce, Dairy, Meat, Grains & Pasta, etc.)
- [ ] Unit tests with meals containing accompaniments
- [ ] Performance: < 1 second per week

**Dependencies:** Task 2.6

**Files Modified:**
- `crates/meal_planning/src/shopping_list.rs`

---

### Task 2.8: Algorithm Testing & Benchmarking

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Comprehensive testing and performance benchmarking of algorithm.

**Acceptance Criteria:**
- [ ] Algorithm tests with realistic datasets:
  - [ ] 10 recipes per type (generates 5 weeks)
  - [ ] 50 recipes per type (generates 5 weeks, capped)
  - [ ] Edge case: Exactly 5 recipes per type
  - [ ] Edge case: Insufficient recipes (< 3 per type)
- [ ] Dietary restriction filtering validated
- [ ] Cuisine variety weighting validated
- [ ] Complexity spacing validated
- [ ] Performance benchmarks:
  - [ ] < 5 seconds for 5 weeks (50 recipes)
  - [ ] < 10ms dietary filtering
  - [ ] < 5ms cuisine scoring
- [ ] All benchmarks documented
- [ ] 85%+ test coverage on algorithm module

**Dependencies:** Tasks 2.1-2.7

**Files Modified:**
- `crates/meal_planning/benches/algorithm_bench.rs` (new)
- `crates/meal_planning/tests/algorithm_test.rs`

---

## Phase 3: Backend Routes & Handlers (Week 5)

**Goal:** Implement HTTP routes and handlers for all new functionality.

### Task 3.1: Generate Multi-Week Meal Plans Route

**Priority:** 🔴 Critical
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Create route to generate multi-week meal plans.

**Acceptance Criteria:**
- [ ] Route: `POST /plan/generate-multi-week`
- [ ] Handler:
  - [ ] Loads user preferences
  - [ ] Loads favorite recipes
  - [ ] Calls `generate_multi_week_meal_plans()`
  - [ ] Persists all weeks + shopping lists
  - [ ] Returns first week view with week navigation metadata
- [ ] Request validation (user authenticated)
- [ ] Response format:
  ```json
  {
    "generation_batch_id": "uuid",
    "weeks": [...],
    "max_weeks_possible": 5,
    "current_week_index": 0
  }
  ```
- [ ] Error handling:
  - [ ] 400: Insufficient recipes
  - [ ] 500: Algorithm failure
- [ ] Integration test with test database

**Dependencies:** Task 2.6 (Algorithm)

**Files Modified:**
- `crates/api/src/routes/meal_planning.rs`
- `crates/api/src/handlers/meal_planning.rs`

---

### Task 3.2: Regenerate Single Week Route

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Backend Developer

**Description:**
Create route to regenerate a specific week.

**Acceptance Criteria:**
- [ ] Route: `POST /plan/week/:week_id/regenerate`
- [ ] Handler:
  - [ ] Validates week is not locked
  - [ ] Regenerates that week's meal assignments
  - [ ] Updates rotation state
  - [ ] Regenerates shopping list for that week
  - [ ] Returns updated week data
- [ ] Error handling:
  - [ ] 400: Week is locked
  - [ ] 404: Week not found
- [ ] Integration test

**Dependencies:** Task 3.1

**Files Modified:**
- `crates/api/src/routes/meal_planning.rs`
- `crates/api/src/handlers/meal_planning.rs`

---

### Task 3.3: Regenerate All Future Weeks Route

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Create route to regenerate all future (unlocked) weeks.

**Acceptance Criteria:**
- [ ] Route: `POST /plan/regenerate-all-future`
- [ ] Handler:
  - [ ] Identifies all unlocked weeks
  - [ ] Preserves current week (locked)
  - [ ] Regenerates all future weeks
  - [ ] Updates all shopping lists
  - [ ] Returns updated weeks data
- [ ] Confirmation logic (returns confirmation HTML if not confirmed)
- [ ] Request body: `{ "confirmed": true }`
- [ ] Error handling:
  - [ ] 400: No future weeks to regenerate
- [ ] Integration test with multi-week data

**Dependencies:** Task 3.1

**Files Modified:**
- `crates/api/src/routes/meal_planning.rs`
- `crates/api/src/handlers/meal_planning.rs`

---

### Task 3.4: Week Navigation Routes

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Backend Developer

**Description:**
Create routes for navigating between weeks.

**Acceptance Criteria:**
- [ ] Route: `GET /plan/week/:week_id`
- [ ] Handler:
  - [ ] Returns calendar view for specific week
  - [ ] Includes meal assignments with accompaniments
  - [ ] Includes shopping list link
  - [ ] Includes week metadata (status, locked, dates)
- [ ] Route: `GET /plan` (updated)
  - [ ] Returns current week by default
  - [ ] Includes navigation to other weeks
- [ ] Integration tests

**Dependencies:** Task 3.1

**Files Modified:**
- `crates/api/src/routes/meal_planning.rs`
- `crates/api/src/handlers/meal_planning.rs`

---

### Task 3.5: Update User Preferences Route

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Backend Developer

**Description:**
Create route to update meal planning preferences.

**Acceptance Criteria:**
- [ ] Route: `PUT /profile/meal-planning-preferences`
- [ ] Handler:
  - [ ] Validates input (time > 0, variety weight 0.0-1.0)
  - [ ] Updates user preferences
  - [ ] Returns updated preferences
- [ ] Request body:
  ```json
  {
    "max_prep_time_weeknight": 30,
    "max_prep_time_weekend": 90,
    "avoid_consecutive_complex": true,
    "cuisine_variety_weight": 0.7
  }
  ```
- [ ] Integration test

**Dependencies:** Task 1.3 (User preferences)

**Files Modified:**
- `crates/api/src/routes/user.rs`
- `crates/api/src/handlers/user.rs`

---

### Task 3.6: Shopping List Week Selector Route

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Backend Developer

**Description:**
Update shopping list routes for week selection.

**Acceptance Criteria:**
- [ ] Route: `GET /shopping` (updated)
  - [ ] Defaults to current week
  - [ ] Returns week selector metadata
- [ ] Route: `GET /shopping?week=2025-10-28`
  - [ ] Returns shopping list for specific week (Monday date)
- [ ] Response includes week navigation
- [ ] Integration test

**Dependencies:** Task 2.7 (Shopping list generation)

**Files Modified:**
- `crates/api/src/routes/shopping.rs`
- `crates/api/src/handlers/shopping.rs`

---

### Task 3.7: Recipe CRUD Updates

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Update recipe creation/update routes for new fields.

**Acceptance Criteria:**
- [ ] Route: `POST /recipes` (updated)
  - [ ] Accepts `accepts_accompaniment`, `preferred_accompaniments`
  - [ ] Accepts `accompaniment_category` (if type = Accompaniment)
  - [ ] Accepts `cuisine` (enum or Custom string)
  - [ ] Accepts `dietary_tags` array
- [ ] Route: `PUT /recipes/:id` (updated)
  - [ ] Same fields as creation
- [ ] Validation:
  - [ ] `preferred_accompaniments` only valid if `accepts_accompaniment` true
  - [ ] `accompaniment_category` only valid if type = Accompaniment
- [ ] Integration tests for all new fields

**Dependencies:** Task 1.2 (Recipe domain)

**Files Modified:**
- `crates/api/src/routes/recipes.rs`
- `crates/api/src/handlers/recipes.rs`

---

## Phase 4: Frontend UX Implementation (Week 6-7)

**Goal:** Implement all UI changes with Askama templates and TwinSpark.

### Task 4.1: Multi-Week Calendar Component

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** Frontend Developer

**Description:**
Create multi-week calendar view with navigation.

**Acceptance Criteria:**
- [ ] Desktop: Horizontal week tabs (Week 1-5)
- [ ] Mobile: Vertical carousel (swipe left/right)
- [ ] Current week indicator (🔒 locked)
- [ ] Week navigation arrows
- [ ] Meal slots display:
  - [ ] Appetizer, Main Course (+accompaniment), Dessert
  - [ ] Prep indicator
  - [ ] Complexity badge
- [ ] TwinSpark: Week navigation without page reload
- [ ] Responsive: Mobile (<768px), Tablet, Desktop
- [ ] Accessibility: Keyboard navigation, ARIA labels

**Dependencies:** Task 3.4 (Week navigation routes)

**Files Modified:**
- `templates/meal_planning/calendar.html` (new)
- `templates/meal_planning/week_view.html` (new)
- `templates/meal_planning/meal_slot.html` (new)
- `static/css/meal_planning.css`

---

### Task 4.2: Regenerate Confirmation Dialog

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Frontend Developer

**Description:**
Create confirmation modal for "Regenerate All Future Weeks".

**Acceptance Criteria:**
- [ ] Modal component with:
  - [ ] Warning icon
  - [ ] Dynamic text: "Regenerate {N} future weeks?"
  - [ ] Explanation: Current week preserved, shopping lists updated
  - [ ] Cancel button
  - [ ] Confirm button (primary, destructive)
- [ ] TwinSpark: Show modal on button click
- [ ] TwinSpark: POST request on confirm
- [ ] Mobile: Full-screen modal
- [ ] Desktop: Centered modal (max-width 640px)
- [ ] Accessibility: Focus trap, Escape to close

**Dependencies:** Task 3.3 (Regenerate all route)

**Files Modified:**
- `templates/components/confirmation_modal.html` (new)
- `templates/meal_planning/calendar.html` (updated)

---

### Task 4.3: Recipe Form - Accompaniment Settings

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Frontend Developer

**Description:**
Update recipe creation form for accompaniment settings.

**Acceptance Criteria:**
- [ ] Main Course type:
  - [ ] Checkbox: "This dish accepts an accompaniment"
  - [ ] Conditional field: Preferred accompaniments (multi-select)
  - [ ] Options: Pasta, Rice, Fries, Salad, Bread, Vegetables, Other
- [ ] Accompaniment recipe type:
  - [ ] Radio buttons for category selection
- [ ] Form validation (client + server)
- [ ] Responsive layout
- [ ] Accessibility: Labels, error messages

**Dependencies:** Task 3.7 (Recipe CRUD routes)

**Files Modified:**
- `templates/recipes/create.html` (updated)
- `templates/recipes/edit.html` (updated)
- `templates/recipes/form_fields/accompaniment.html` (new)

---

### Task 4.4: Recipe Form - Cuisine Selection

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Frontend Developer

**Description:**
Add cuisine selection with custom input support.

**Acceptance Criteria:**
- [ ] Radio buttons for predefined cuisines
- [ ] "Custom" radio button with text input
- [ ] Input validation (max 50 chars)
- [ ] Helper text: "Used for cuisine variety in meal planning"
- [ ] Responsive grid layout
- [ ] Accessibility: Radio group labeled

**Dependencies:** Task 3.7 (Recipe CRUD routes)

**Files Modified:**
- `templates/recipes/create.html` (updated)
- `templates/recipes/edit.html` (updated)
- `templates/recipes/form_fields/cuisine.html` (new)

---

### Task 4.5: User Preferences Form

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Frontend Developer

**Description:**
Create meal planning preferences form in user profile.

**Acceptance Criteria:**
- [ ] Fields:
  - [ ] Weeknight max prep time (number input, minutes)
  - [ ] Weekend max prep time (number input, minutes)
  - [ ] Avoid consecutive complex (checkbox)
  - [ ] Cuisine variety weight (range slider 0.0-1.0)
- [ ] Slider labels: "Repeat OK" ↔ "Mix it up!"
- [ ] Real-time validation
- [ ] Save button
- [ ] Success toast on save
- [ ] Responsive layout
- [ ] Accessibility: Labels, ARIA for slider

**Dependencies:** Task 3.5 (Preferences route)

**Files Modified:**
- `templates/profile/meal_planning_preferences.html` (new)
- `templates/profile/index.html` (add link)
- `static/css/preferences.css` (new)

---

### Task 4.6: Shopping List Week Selector

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Frontend Developer

**Description:**
Add week selector dropdown to shopping list.

**Acceptance Criteria:**
- [ ] Dropdown: "Week 1 (Oct 28) - Current 🔒", "Week 2 (Nov 4)", etc.
- [ ] TwinSpark: Load shopping list on week selection
- [ ] Mobile: Full-width dropdown
- [ ] Desktop: Inline dropdown (top-right)
- [ ] Default to current week
- [ ] Accessibility: Select labeled

**Dependencies:** Task 3.6 (Shopping list routes)

**Files Modified:**
- `templates/shopping/list.html` (updated)
- `templates/shopping/week_selector.html` (new)

---

### Task 4.7: Meal Slot with Accompaniment Display

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Frontend Developer

**Description:**
Update meal slot component to display accompaniments.

**Acceptance Criteria:**
- [ ] Main course shows:
  - [ ] Recipe title
  - [ ] "+ {Accompaniment Name}" if present
  - [ ] Icons: +R (Rice), +P (Pasta), +S (Salad), +F (Fries), +B (Bread), +V (Vegetables)
- [ ] Mobile: Stacked layout
- [ ] Desktop: Inline with icon
- [ ] Accessibility: Screen reader announces accompaniment

**Dependencies:** Task 4.1 (Calendar component)

**Files Modified:**
- `templates/meal_planning/meal_slot.html` (updated)
- `static/css/meal_planning.css` (updated)

---

### Task 4.8: Tailwind CSS 4.1+ Migration

**Priority:** 🟢 Medium
**Estimate:** 2 days
**Assignee:** Frontend Developer

**Description:**
Migrate all new styles to Tailwind CSS 4.1+ syntax.

**Acceptance Criteria:**
- [ ] Use `@theme` for custom design tokens
- [ ] Use `@utility` for custom utilities
- [ ] Color palette defined in theme
- [ ] Spacing scale (8px grid) defined
- [ ] Typography scale defined
- [ ] No legacy Tailwind 3.x config
- [ ] All new components use Tailwind 4.1+ classes
- [ ] Build process updated

**Dependencies:** Tasks 4.1-4.7

**Files Modified:**
- `static/css/theme.css` (new)
- `tailwind.config.js` (migrate to CSS-based config)
- All template files (updated class names)

---

## Phase 5: Testing & Refinement (Week 8)

**Goal:** Comprehensive testing and bug fixing.

### Task 5.1: End-to-End Testing (Playwright)

**Priority:** 🔴 Critical
**Estimate:** 3 days
**Assignee:** QA Engineer / Frontend Developer

**Description:**
Write E2E tests for critical user flows.

**Acceptance Criteria:**
- [ ] Test: New user onboarding → First meal plan
  - [ ] Add 7 recipes
  - [ ] Generate 5-week meal plan
  - [ ] Navigate between weeks
  - [ ] View shopping list
- [ ] Test: Regenerate single week
  - [ ] Click regenerate button
  - [ ] Verify new meals assigned
  - [ ] Verify shopping list updated
- [ ] Test: Regenerate all future weeks
  - [ ] Click regenerate all
  - [ ] Confirm dialog
  - [ ] Verify 4 future weeks regenerated
  - [ ] Verify current week preserved
- [ ] Test: Recipe with accompaniment
  - [ ] Create main course with accompaniment acceptance
  - [ ] Generate meal plan
  - [ ] Verify accompaniment displayed
  - [ ] Verify shopping list includes accompaniment ingredients
- [ ] Test: User preferences
  - [ ] Update preferences
  - [ ] Generate meal plan
  - [ ] Verify meals respect time constraints
- [ ] All tests passing in CI

**Dependencies:** All Phase 4 tasks

**Files Modified:**
- `tests/e2e/meal_planning/multi_week.spec.ts` (new)
- `tests/e2e/meal_planning/regenerate.spec.ts` (new)
- `tests/e2e/meal_planning/accompaniments.spec.ts` (new)
- `tests/e2e/meal_planning/preferences.spec.ts` (new)

---

### Task 5.2: Performance Testing

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Backend Developer

**Description:**
Load testing and performance optimization.

**Acceptance Criteria:**
- [ ] Load test: 100 concurrent users generating meal plans
- [ ] Target: < 5 seconds response time (p95)
- [ ] Target: < 10 seconds response time (p99)
- [ ] Database query optimization (indexes verified)
- [ ] Algorithm benchmarks verified
- [ ] Memory usage profiling (no leaks)
- [ ] Performance report documented

**Dependencies:** All Phase 3 tasks

**Files Modified:**
- `tests/load/meal_planning_load.rs` (new)
- `docs/performance-report.md` (new)

---

### Task 5.3: Accessibility Audit

**Priority:** 🟡 High
**Estimate:** 2 days
**Assignee:** Frontend Developer

**Description:**
WCAG 2.1 Level AA compliance verification.

**Acceptance Criteria:**
- [ ] Automated testing: axe-core, WAVE (Lighthouse > 90)
- [ ] Manual testing: Keyboard-only navigation
- [ ] Screen reader testing: NVDA (Windows), VoiceOver (Mac)
- [ ] Color contrast verification (4.5:1 minimum)
- [ ] Focus indicators visible (2px outline)
- [ ] ARIA labels verified
- [ ] All issues documented and fixed
- [ ] Accessibility report completed

**Dependencies:** All Phase 4 tasks

**Files Modified:**
- Various templates (accessibility fixes)
- `docs/accessibility-report.md` (new)

---

### Task 5.4: Bug Fixes & Refinement

**Priority:** 🟡 High
**Estimate:** 3 days
**Assignee:** Full Team

**Description:**
Address bugs and edge cases discovered during testing.

**Acceptance Criteria:**
- [ ] All critical bugs fixed
- [ ] All high-priority bugs fixed
- [ ] Edge cases handled gracefully
- [ ] Error messages user-friendly
- [ ] Loading states polished
- [ ] Animations smooth (respect `prefers-reduced-motion`)
- [ ] Mobile experience refined
- [ ] Code review feedback addressed

**Dependencies:** Tasks 5.1, 5.2, 5.3

**Files Modified:**
- Various (bug fixes)

---

## Phase 6: Deployment & Monitoring (Week 9)

**Goal:** Deploy to production with monitoring and rollback plan.

### Task 6.1: Staging Deployment

**Priority:** 🔴 Critical
**Estimate:** 1 day
**Assignee:** DevOps Engineer

**Description:**
Deploy to staging environment for final validation.

**Acceptance Criteria:**
- [ ] Staging environment updated
- [ ] Database migration run successfully
- [ ] All services healthy
- [ ] E2E tests run against staging
- [ ] Performance tests run against staging
- [ ] Staging smoke tests passing
- [ ] Rollback plan tested

**Dependencies:** All previous phases

**Files Modified:**
- Deployment scripts

---

### Task 6.2: Production Database Migration

**Priority:** 🔴 Critical
**Estimate:** 2 hours (planned maintenance window)
**Assignee:** DevOps Engineer + Database Admin

**Description:**
Run database migration in production.

**Acceptance Criteria:**
- [ ] Maintenance window scheduled
- [ ] Database backup created
- [ ] Migration script executed
- [ ] Verification queries run
- [ ] Existing data integrity verified
- [ ] Rollback script ready (tested in staging)
- [ ] Migration duration logged

**Dependencies:** Task 6.1

**Files Modified:**
- None (database only)

---

### Task 6.3: Production Deployment

**Priority:** 🔴 Critical
**Estimate:** 2 hours
**Assignee:** DevOps Engineer

**Description:**
Deploy application to production.

**Acceptance Criteria:**
- [ ] Application deployed (rolling deployment)
- [ ] Health checks passing
- [ ] Smoke tests passing in production
- [ ] Monitoring dashboards active
- [ ] Error tracking enabled (Sentry/equivalent)
- [ ] Performance monitoring enabled
- [ ] Logs aggregation working
- [ ] Rollback plan ready

**Dependencies:** Task 6.2

**Files Modified:**
- Deployment scripts

---

### Task 6.4: Monitoring & Alerting Setup

**Priority:** 🔴 Critical
**Estimate:** 1 day
**Assignee:** DevOps Engineer

**Description:**
Set up monitoring and alerting for new features.

**Acceptance Criteria:**
- [ ] Metrics:
  - [ ] Meal plan generation duration (p50, p95, p99)
  - [ ] Multi-week generation success rate
  - [ ] Algorithm timeout rate
  - [ ] Recipe creation with accompaniments rate
  - [ ] User preference update rate
- [ ] Alerts:
  - [ ] Algorithm failure rate > 5%
  - [ ] Meal plan generation p95 > 10 seconds
  - [ ] Database migration issues
- [ ] Dashboards created (Grafana/equivalent)
- [ ] On-call rotation notified

**Dependencies:** Task 6.3

**Files Modified:**
- Monitoring configuration

---

### Task 6.5: User Communication & Documentation

**Priority:** 🟡 High
**Estimate:** 1 day
**Assignee:** Product Manager + Technical Writer

**Description:**
Communicate new features to users and update documentation.

**Acceptance Criteria:**
- [ ] Release notes published
- [ ] User guide updated:
  - [ ] Multi-week meal planning
  - [ ] Accompaniment recipes
  - [ ] Meal planning preferences
- [ ] In-app onboarding tooltips (optional)
- [ ] Email announcement to users (optional)
- [ ] Blog post (optional)
- [ ] Social media announcement (optional)

**Dependencies:** Task 6.3

**Files Modified:**
- `docs/release-notes.md`
- `docs/user-guide.md`

---

### Task 6.6: Post-Launch Monitoring (Week 10)

**Priority:** 🟡 High
**Estimate:** Ongoing (1 week intensive)
**Assignee:** Full Team

**Description:**
Monitor production performance and user feedback.

**Acceptance Criteria:**
- [ ] Monitor error rates (target < 1%)
- [ ] Monitor performance metrics (targets met)
- [ ] Monitor user adoption:
  - [ ] % users generating multi-week plans
  - [ ] % recipes with accompaniments
  - [ ] % users updating preferences
- [ ] Gather user feedback (support tickets, feedback form)
- [ ] Hot-fix critical issues (< 24 hours)
- [ ] Plan iteration based on feedback

**Dependencies:** Task 6.3

**Files Modified:**
- Various (hot-fixes as needed)

---

## Summary

### Timeline Overview

| Phase | Duration | Tasks | Key Deliverables |
|-------|----------|-------|------------------|
| Phase 1: Database & Domain | 2 weeks | 5 tasks | Schema migration, domain models, events |
| Phase 2: Algorithm | 2 weeks | 8 tasks | Multi-week generation, preferences, accompaniments |
| Phase 3: Backend Routes | 1 week | 7 tasks | All API endpoints, handlers |
| Phase 4: Frontend UX | 2 weeks | 8 tasks | Calendar, forms, preferences, Tailwind 4.1+ |
| Phase 5: Testing | 1 week | 4 tasks | E2E, performance, accessibility, bug fixes |
| Phase 6: Deployment | 1 week | 6 tasks | Staging, production, monitoring, docs |
| **Total** | **9 weeks** | **38 tasks** | Full feature set deployed |

### Resource Allocation

- **Backend Developers:** 2 developers (Phases 1-3, 5)
- **Frontend Developer:** 1 developer (Phase 4, 5)
- **DevOps Engineer:** 1 engineer (Phase 6)
- **QA Engineer:** 1 engineer (Phase 5)
- **Product Manager / Technical Writer:** 1 person (Phase 6)

### Risk Mitigation

**High-Risk Areas:**
1. **Algorithm performance** - Mitigated by early benchmarking (Phase 2)
2. **Database migration** - Mitigated by staging testing + rollback plan
3. **User adoption** - Mitigated by clear onboarding and documentation

**Contingency:**
- Add 1-2 week buffer for unforeseen issues
- Phased rollout option: Deploy features incrementally if needed

---

## Next Steps

1. **Review this implementation plan** with technical leads
2. **Create Jira/Linear tickets** from tasks above
3. **Assign team members** to phases
4. **Kick off Phase 1** (Database & Domain Foundation)
5. **Weekly standups** to track progress

**Status:** ✅ Ready for Team Review
**Document Version:** 1.0
