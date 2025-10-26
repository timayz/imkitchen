# Technical Specification: Enhanced Meal Planning - Algorithm Implementation

Date: 2025-10-26
Author: Jonathan
Epic ID: Epic 7
Status: Draft

---

## Overview

This technical specification covers Epic 7: Enhanced Meal Planning - Algorithm Implementation, which delivers the core intelligent meal planning engine for imkitchen. Building upon the foundational database schema and domain models established in Epic 6, this epic implements the multi-week meal planning algorithm with preference-aware recipe selection, dietary filtering, accompaniment pairing, and rotation management.

The algorithm represents imkitchen's core differentiator—enabling users to automatically generate personalized, constraint-respecting meal plans spanning 1-5 weeks based on their favorite recipes. The implementation prioritizes dietary safety (hard filtering of incompatible recipes), time/skill appropriateness (weeknight vs weekend constraints), cuisine variety (configurable diversity scoring), and recipe rotation fairness (main courses never repeat across weeks, sides can repeat after exhaustion).

## Objectives and Scope

**In Scope:**
- Dietary restriction filtering algorithm (Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher, Custom)
- Preference-aware main course selection (time constraints, skill level, complexity avoidance, cuisine variety scoring)
- Accompaniment pairing logic (preferred categories, compatibility checks)
- Single week meal plan generation (21 assignments: 7 days × 3 courses)
- Multi-week meal plan batch generation (1-5 weeks based on available recipes)
- Recipe rotation state management (main course uniqueness, appetizer/dessert exhaustion/reset logic)
- Shopping list generation per week (ingredient aggregation, categorization)
- Comprehensive unit tests (>80% coverage) and integration tests
- Performance benchmarks (<5 seconds for 5-week generation)

**Out of Scope:**
- HTTP route handlers and API contracts (Epic 8: Backend Routes & Handlers)
- Frontend UI components and user interactions (Epic 9: Frontend UX Implementation)
- Database migration execution (completed in Epic 6)
- User preference management routes (Epic 8)
- Shopping list display and interaction logic (Epic 4 + Epic 9)
- Notification system integration for meal reminders (Epic 4)
- Recipe recommendation engine based on usage patterns (future enhancement)
- Nutritional analysis and macro tracking (future enhancement)

## System Architecture Alignment

This epic aligns with the event-sourced monolith architecture using evento (SQLite event store) for CQRS implementation. The algorithm implementation resides in the `crates/meal_planning/` domain crate with no direct HTTP dependencies—following clean architecture principles where domain logic remains isolated from infrastructure concerns.

**Referenced Architecture Components:**
- **Domain Crate:** `crates/meal_planning/src/algorithm.rs` - Core meal planning logic
- **Domain Models:** `Recipe`, `MealPlan`, `User`, `RotationState` (Epic 6)
- **Rotation Module:** `crates/meal_planning/src/rotation.rs` - State tracking (Epic 6 Story 6.5)
- **Event Store:** evento SQLite - Event sourcing for `MultiWeekMealPlanGenerated`, `SingleWeekRegenerated` events
- **Read Models:** `meal_plans`, `meal_assignments`, `shopping_lists` tables

**Architecture Constraints Respected:**
- TDD enforced: Write tests first, implement to pass (cargo test, >80% coverage via cargo-tarpaulin)
- No external HTTP/IO in algorithm functions (pure business logic, dependency injection for data access)
- Tailwind CSS 4.1+ syntax (N/A for backend Epic 7)
- Test pattern: `unsafe_oneshot` for evento subscriptions in tests (synchronous processing)
- Performance target: <5 seconds for 5-week generation (measured via criterion benchmarks)

**Design Decisions from Architecture Document:**
- 5-week hard cap (section 1.2) to balance planning horizon with computational cost
- Main courses NEVER repeat; appetizers/desserts CAN repeat after exhaustion (section 1.3)
- Dietary restrictions use AND logic—all must be satisfied (section 3.5)
- Cuisine variety weight: 0.0 (repeat OK) to 1.0 (max variety), default 0.7 (section 3.4)
- Weeknight max prep time default: 30 minutes; weekend: 90 minutes (section 3.4)

## Detailed Design

### Services and Modules

| Module/Function | Responsibility | Inputs | Outputs | Owner |
|-----------------|----------------|--------|---------|-------|
| `filter_by_dietary_restrictions` | Filter recipes incompatible with user dietary restrictions (AND logic) | `Vec<Recipe>`, `Vec<DietaryRestriction>` | `Vec<Recipe>` (filtered) | meal_planning crate |
| `select_main_course_with_preferences` | Score and select best main course for specific day respecting time/skill/complexity/cuisine constraints | `Vec<Recipe>`, `UserPreferences`, `RotationState`, `Date`, `DayOfWeek` | `Option<Recipe>` | meal_planning crate |
| `select_accompaniment` | Pair accompaniment with main course based on preferred categories | `&Recipe` (main), `Vec<Recipe>` (accompaniments) | `Option<Recipe>` | meal_planning crate |
| `generate_single_week` | Generate complete week (21 assignments) with rotation tracking | `Vec<Recipe>`, `UserPreferences`, `&mut RotationState`, `Date` (week start) | `WeekMealPlan` | meal_planning crate |
| `generate_multi_week_meal_plans` | Calculate max weeks, generate all weeks in batch, create shopping lists | `UserId`, `Vec<Recipe>` (favorites), `UserPreferences` | `Result<MultiWeekMealPlan, Error>` | meal_planning crate |
| `generate_shopping_list_for_week` | Aggregate ingredients from week's meal assignments, categorize | `Vec<MealAssignment>`, `Vec<Recipe>` | `ShoppingList` | meal_planning crate |
| `RotationState` module | Track used recipes across weeks, manage uniqueness/exhaustion | See rotation.rs (Epic 6 Story 6.5) | State mutations | meal_planning crate |

### Data Models and Contracts

**Core Domain Models** (defined in Epic 6, used by Epic 7 algorithm):

```rust
// Recipe with enhancements
pub struct Recipe {
    id: RecipeId,
    user_id: UserId,
    title: String,
    ingredients: Vec<Ingredient>,
    instructions: Vec<String>,
    prep_time_minutes: u32,
    cook_time_minutes: u32,
    recipe_type: RecipeType,  // Appetizer | MainCourse | Dessert
    complexity: Complexity,    // Simple | Moderate | Complex
    accepts_accompaniment: bool,
    preferred_accompaniments: Vec<AccompanimentCategory>,  // Pasta, Rice, Fries, etc.
    accompaniment_category: Option<AccompanimentCategory>,  // If recipe IS accompaniment
    cuisine: Cuisine,          // Italian, Mexican, Chinese, Custom(String)
    dietary_tags: Vec<DietaryTag>,  // Vegetarian, Vegan, GlutenFree, etc.
}

// User preferences
pub struct UserPreferences {
    dietary_restrictions: Vec<DietaryRestriction>,
    max_prep_time_weeknight: u32,  // minutes, default 30
    max_prep_time_weekend: u32,     // minutes, default 90
    skill_level: SkillLevel,        // Beginner | Intermediate | Advanced
    avoid_consecutive_complex: bool, // default true
    cuisine_variety_weight: f32,    // 0.0-1.0, default 0.7
}

// Rotation state tracking
pub struct RotationState {
    used_main_course_ids: Vec<RecipeId>,         // NEVER repeat
    used_appetizer_ids: Vec<RecipeId>,           // Can repeat after exhaustion
    used_dessert_ids: Vec<RecipeId>,             // Can repeat after exhaustion
    cuisine_usage_count: HashMap<Cuisine, u32>,  // Track diversity
    last_complex_meal_date: Option<Date>,        // Avoid consecutive complex
}

// Week meal plan
pub struct WeekMealPlan {
    id: String,
    user_id: UserId,
    start_date: Date,         // Monday (ISO 8601)
    end_date: Date,           // Sunday
    status: WeekStatus,       // Future | Current | Past | Archived
    is_locked: bool,
    generation_batch_id: String,
    meal_assignments: Vec<MealAssignment>,  // 21 assignments (7 days × 3)
    shopping_list_id: String,
    created_at: DateTime,
}

// Meal assignment
pub struct MealAssignment {
    id: String,
    meal_plan_id: String,
    date: Date,
    course_type: CourseType,  // Appetizer | MainCourse | Dessert
    recipe_id: RecipeId,
    accompaniment_recipe_id: Option<RecipeId>,  // For main courses only
    prep_required: bool,
}

// Multi-week plan
pub struct MultiWeekMealPlan {
    user_id: UserId,
    generation_batch_id: String,
    generated_weeks: Vec<WeekMealPlan>,
    rotation_state: RotationState,
}

// Shopping list
pub struct ShoppingList {
    id: String,
    meal_plan_id: String,
    week_start_date: Date,
    categories: Vec<ShoppingCategory>,
}

pub struct ShoppingCategory {
    name: String,  // Produce, Dairy, Meat, Grains, Pantry, Frozen
    items: Vec<ShoppingItem>,
}

pub struct ShoppingItem {
    ingredient_name: String,
    quantity: f32,
    unit: String,
    from_recipe_ids: Vec<RecipeId>,
}
```

**Enums:**

```rust
pub enum DietaryRestriction {
    Vegetarian,
    Vegan,
    GlutenFree,
    DairyFree,
    NutFree,
    Halal,
    Kosher,
    Custom(String),  // User-defined allergens
}

pub enum SkillLevel {
    Beginner,      // Only Simple recipes
    Intermediate,  // Simple + Moderate
    Advanced,      // All complexity levels
}

pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

pub enum AccompanimentCategory {
    Pasta,
    Rice,
    Fries,
    Salad,
    Bread,
    Vegetable,
    Other,
}

pub enum Cuisine {
    Italian,
    Mexican,
    Chinese,
    Indian,
    Japanese,
    French,
    Mediterranean,
    American,
    Custom(String),
}

pub enum WeekStatus {
    Future,    // Not started
    Current,   // Today in range
    Past,      // Completed
    Archived,  // User archived
}
```

**Error Types:**

```rust
pub enum Error {
    InsufficientRecipes {
        appetizers: usize,
        main_courses: usize,
        desserts: usize,
    },
    NoCompatibleRecipes {
        course_type: CourseType,
        reason: String,
    },
    AlgorithmTimeout,
    InvalidPreferences(String),
}
```

### APIs and Interfaces

Epic 7 focuses on domain logic (algorithm implementation) with no HTTP routes. API contracts are defined in Epic 8. However, the public function signatures for the algorithm module are specified here:

**Public Algorithm Functions:**

```rust
// Story 7.1: Dietary Restriction Filtering
pub fn filter_by_dietary_restrictions(
    recipes: Vec<Recipe>,
    restrictions: &[DietaryRestriction],
) -> Vec<Recipe>;

// Story 7.2: Main Course Selection
pub fn select_main_course_with_preferences(
    available_main_courses: &[Recipe],
    preferences: &UserPreferences,
    rotation_state: &RotationState,
    date: Date,
    day_of_week: DayOfWeek,
) -> Option<Recipe>;

// Story 7.3: Accompaniment Selection
pub fn select_accompaniment(
    main_course: &Recipe,
    available_accompaniments: &[Recipe],
) -> Option<Recipe>;

// Story 7.4: Single Week Generation
pub fn generate_single_week(
    recipes: Vec<Recipe>,
    preferences: &UserPreferences,
    rotation_state: &mut RotationState,
    week_start_date: Date,
) -> Result<WeekMealPlan, Error>;

// Story 7.5: Multi-Week Generation
pub async fn generate_multi_week_meal_plans(
    user_id: UserId,
    favorite_recipes: Vec<Recipe>,
    preferences: UserPreferences,
) -> Result<MultiWeekMealPlan, Error>;

// Story 7.6: Shopping List Generation
pub fn generate_shopping_list_for_week(
    meal_assignments: &[MealAssignment],
    recipes: &[Recipe],
    week_start_date: Date,
) -> ShoppingList;
```

**RotationState Methods** (from Epic 6 Story 6.5):

```rust
impl RotationState {
    pub fn new() -> Self;

    // Main course tracking (NEVER repeat)
    pub fn mark_used_main_course(&mut self, recipe_id: RecipeId);
    pub fn is_main_course_used(&self, recipe_id: &RecipeId) -> bool;

    // Appetizer/dessert tracking (can repeat after exhaustion)
    pub fn mark_used_appetizer(&mut self, recipe_id: RecipeId);
    pub fn mark_used_dessert(&mut self, recipe_id: RecipeId);
    pub fn reset_appetizers_if_all_used(&mut self, total_appetizers: usize);
    pub fn reset_desserts_if_all_used(&mut self, total_desserts: usize);

    // Cuisine variety tracking
    pub fn increment_cuisine_usage(&mut self, cuisine: &Cuisine);
    pub fn get_cuisine_usage(&self, cuisine: &Cuisine) -> u32;

    // Complexity tracking
    pub fn update_last_complex_meal_date(&mut self, date: Date);
    pub fn get_last_complex_meal_date(&self) -> Option<Date>;
}
```

### Workflows and Sequencing

**Multi-Week Meal Plan Generation Flow:**

```
1. User triggers meal plan generation (Epic 8: HTTP POST /plan/generate-multi-week)
   ↓
2. Load user's favorite recipes from database (read model query)
   ↓
3. Load user preferences (dietary restrictions, time constraints, skill level, variety weight)
   ↓
4. Call generate_multi_week_meal_plans(user_id, recipes, preferences)
   ↓
5. Algorithm: filter_by_dietary_restrictions(recipes, restrictions)
   → Returns compatible_recipes
   ↓
6. Algorithm: Calculate max_weeks
   → Count appetizers, main_courses, desserts
   → max_weeks = min(5, min(appetizers, mains, desserts))
   → If max_weeks < 1, return InsufficientRecipes error
   ↓
7. Algorithm: Initialize RotationState::new()
   ↓
8. Algorithm: For each week (0..max_weeks):
   ├─ Calculate week_start_date = next_monday() + (week_index * 7 days)
   ├─ Call generate_single_week(recipes, preferences, &mut rotation_state, week_start_date)
   │  ├─ For each day (Monday..Sunday):
   │  │  ├─ Select appetizer (cycle through, reset if exhausted)
   │  │  ├─ Select main course via select_main_course_with_preferences()
   │  │  │  ├─ Filter by weekday/weekend time constraints
   │  │  │  ├─ Filter by skill level
   │  │  │  ├─ Filter by avoid_consecutive_complex (check rotation_state)
   │  │  │  ├─ Score by cuisine_variety_weight (penalize recent cuisines)
   │  │  │  ├─ Filter out already used main_course_ids (NEVER repeat)
   │  │  │  └─ Return highest-scored recipe
   │  │  ├─ If main accepts_accompaniment, call select_accompaniment()
   │  │  │  ├─ Filter by preferred_accompaniments categories
   │  │  │  └─ Random selection from filtered list
   │  │  ├─ Select dessert (cycle through, reset if exhausted)
   │  │  └─ Create MealAssignment (date, course_type, recipe_id, accompaniment_id)
   │  └─ Return WeekMealPlan (21 assignments)
   └─ Call generate_shopping_list_for_week(week.meal_assignments, recipes)
      ├─ Load all recipes referenced in assignments (main + accompaniments)
      ├─ Aggregate ingredients (combine duplicates by name)
      ├─ Categorize by ingredient type (Produce, Dairy, Meat, Grains, Pantry, Frozen)
      └─ Return ShoppingList
   ↓
9. Return MultiWeekMealPlan {
     user_id,
     generation_batch_id: UUID,
     generated_weeks: Vec<WeekMealPlan>,
     rotation_state,
   }
   ↓
10. Emit MultiWeekMealPlanGenerated event (evento)
    ↓
11. Projection handlers update read models:
    ├─ Insert weeks into meal_plans table
    ├─ Insert assignments into meal_assignments table
    └─ Insert shopping lists into shopping_lists table
```

**Cuisine Variety Scoring Formula** (Story 7.2):

```
For each candidate main course recipe:
  cuisine_score = variety_weight * (1.0 / (cuisine_usage_count[recipe.cuisine] + 1.0))

  // variety_weight = 0.0 → no penalty for repeating cuisines
  // variety_weight = 1.0 → maximum penalty for repeating cuisines
  // Default: 0.7

Example with variety_weight=0.7:
  - Italian used 0 times: score = 0.7 * (1.0 / 1) = 0.70
  - Italian used 1 time:  score = 0.7 * (1.0 / 2) = 0.35
  - Italian used 2 times: score = 0.7 * (1.0 / 3) = 0.23

Select recipe with highest score (most diverse)
```

**Single Week Regeneration Flow** (Story 8.3 API, uses Story 7.4 algorithm):

```
1. User triggers week regeneration (POST /plan/week/:week_id/regenerate)
   ↓
2. Verify week is not locked (status != Current, is_locked == false)
   ↓
3. Load current rotation_state for user's meal plan batch
   ↓
4. Call generate_single_week(recipes, preferences, &mut rotation_state, week_start_date)
   ↓
5. Replace meal_assignments for that week
   ↓
6. Regenerate shopping list for week
   ↓
7. Emit SingleWeekRegenerated event
   ↓
8. Update read models
```

## Non-Functional Requirements

### Performance

**Target Metrics:**
- **Multi-week generation (5 weeks):** <5 seconds (P95), measured with 50 recipes per user
- **Single week generation:** <1 second (P95)
- **Dietary filtering:** <10ms for 100 recipes
- **Main course selection with preferences:** <10ms per selection
- **Shopping list aggregation:** <100ms per week

**Performance Testing:**
- Benchmark suite using `criterion` crate (Story 7.7)
- Realistic test data: 50 recipes (15 appetizers, 20 mains, 15 desserts)
- Measure P50, P95, P99 latencies
- Regression tests in CI/CD pipeline

**Optimization Strategies:**
- Use `Vec` filtering instead of database queries for in-memory algorithm performance
- Pre-filter recipes by dietary restrictions once at start (not per day)
- Cache cuisine usage counts in `HashMap` for O(1) lookups
- Avoid cloning large Recipe structs—use references where possible

### Security

**Data Handling:**
- Algorithm operates on in-memory Recipe data—no direct database access
- User preferences contain sensitive dietary restrictions (allergens)—handle with care
- No personally identifiable information (PII) logged in algorithm traces

**Input Validation:**
- Validate `UserPreferences` fields before passing to algorithm:
  - `max_prep_time_weeknight` > 0
  - `max_prep_time_weekend` > 0
  - `cuisine_variety_weight` between 0.0 and 1.0
  - `dietary_restrictions` non-empty or valid enum variants
- Return `InvalidPreferences(String)` error for violations

**Authorization:**
- Epic 7 has no HTTP layer—authorization enforced in Epic 8 route handlers
- Ensure `user_id` passed to algorithm matches authenticated user

**Error Messages:**
- Avoid leaking implementation details in error messages
- Example: "Insufficient recipes to generate meal plan" (not "Main course Vec exhausted at iteration 14")

**Dependency Security:**
- All dependencies audited via `cargo audit` in CI/CD
- No unsafe code blocks in algorithm implementation unless absolutely necessary and documented

### Reliability/Availability

**Graceful Degradation:**
- If insufficient recipes for 5 weeks, generate as many weeks as possible (minimum 1 week)
- Return `InsufficientRecipes` error with clear counts if unable to generate even 1 week
- If no compatible main course for a specific day, return `NoCompatibleRecipes` error with reason

**Error Recovery:**
- Algorithm is deterministic given same inputs—no side effects on failure
- Rotation state mutations only occur after successful week generation
- Failed generation does not corrupt existing meal plans (read models unchanged)

**Edge Cases Handled:**
- Empty recipe lists → `InsufficientRecipes` error
- All recipes filtered by dietary restrictions → `InsufficientRecipes` error
- Recipe with no compatible accompaniments → `None` returned, proceed without accompaniment
- Appetizer/dessert exhaustion → Automatic reset and cycle restart
- Main course exhaustion mid-week → Return error for that week, halt generation

**Idempotency:**
- Algorithm is NOT idempotent due to random accompaniment selection
- Multiple calls with same inputs may produce different accompaniment pairings
- Main course/appetizer/dessert selection is deterministic (highest-scored, cyclic)

**Testing for Reliability:**
- Edge case unit tests (Story 7.7)
- Property-based testing (optional): Generate random recipe sets, verify no panics
- Fuzzing with invalid inputs (cargo-fuzz, optional)

### Observability

**Logging:**
- Use `tracing` crate for structured logging
- Log levels:
  - `DEBUG`: Algorithm decision details (selected recipe IDs, scores, filtering steps)
  - `INFO`: High-level generation events (multi-week generation started, X weeks generated)
  - `WARN`: Degraded behavior (appetizers exhausted, resetting rotation)
  - `ERROR`: Failures (insufficient recipes, no compatible main course)

**Log Examples:**
```rust
tracing::info!(user_id = %user_id, max_weeks = max_weeks, "Starting multi-week meal plan generation");
tracing::debug!(recipe_id = %recipe.id, score = cuisine_score, "Selected main course");
tracing::warn!(exhausted_count = used_appetizers.len(), total = appetizers.len(), "Appetizer rotation exhausted, resetting");
tracing::error!(error = %err, "Failed to generate meal plan");
```

**Metrics (for Epic 8 integration):**
- `meal_plan_generation_duration_seconds` (histogram) - Total generation time
- `meal_plan_weeks_generated_count` (histogram) - Number of weeks successfully generated
- `meal_plan_generation_errors_total` (counter) - By error type
- `recipe_selection_duration_ms` (histogram) - Per-day main course selection time

**Tracing (OpenTelemetry):**
- Span: `generate_multi_week_meal_plans` with attributes: user_id, recipe_count, max_weeks
- Span: `generate_single_week` with attributes: week_index, week_start_date
- Span: `select_main_course_with_preferences` with attributes: day_of_week, candidate_count

**Error Context:**
- Include rich context in error types:
  - `InsufficientRecipes { appetizers, main_courses, desserts }` - Show exact counts
  - `NoCompatibleRecipes { course_type, reason }` - Explain why no match found

## Dependencies and Integrations

**Rust Crate Dependencies:**

| Dependency | Version | Purpose | Story Reference |
|------------|---------|---------|-----------------|
| `evento` | 1.5+ | Event sourcing framework (SQLite backend), aggregate traits, event handling | All stories (event emission) |
| `chrono` | 0.4+ | Date/time handling (week calculations, Monday start dates, ISO 8601) | Stories 7.4, 7.5 |
| `uuid` | 1.10+ | Generate `generation_batch_id`, `shopping_list_id` (v4 UUIDs) | Story 7.5 |
| `rand` | 0.8+ | Random accompaniment selection (`thread_rng`) | Story 7.3 |
| `tracing` | 0.1+ | Structured logging for observability | All stories |
| `thiserror` | 1.0+ | Custom error type definitions (`Error` enum) | All stories |
| `serde` | 1.0+ | Serialization for evento events (bincode + JSON) | All stories |
| `bincode` | 2.0+ | Binary encoding for evento event persistence | All stories |
| `criterion` | 0.5+ | Benchmarking suite (dev dependency) | Story 7.7 |

**Internal Module Dependencies:**

- `crates/shared_kernel`: Common types (`UserId`, `RecipeId`, `Date`), shared traits
- `crates/meal_planning/src/rotation.rs`: `RotationState` implementation (Epic 6 Story 6.5)
- `crates/recipe`: Recipe domain model (Epic 6 Story 6.2)
- `crates/user`: User domain model with preferences (Epic 6 Story 6.4)

**Database Integration:**

- **Read Models** (Epic 6 Story 6.6 projections):
  - Algorithm receives pre-loaded `Vec<Recipe>` from read model queries
  - No direct SQLx queries in algorithm code (separation of concerns)
  - Route handlers (Epic 8) load data from DB, pass to algorithm

**Event Store Integration:**

- Algorithm returns `MultiWeekMealPlan` struct
- Epic 8 route handlers commit events:
  - `MultiWeekMealPlanGenerated` → evento event store
  - `SingleWeekRegenerated` → evento event store
- Projections (Epic 6 Story 6.6) subscribe to events, update read models

**No External Service Dependencies:**
- Algorithm is pure domain logic with no HTTP/gRPC/message queue integrations
- Self-contained within `crates/meal_planning` module

## Acceptance Criteria (Authoritative)

**Story 7.1: Dietary Restriction Filtering**
1. Function `filter_by_dietary_restrictions(recipes, restrictions)` implemented
2. Filters recipes not matching ALL restrictions (AND logic)
3. Checks Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher tags
4. Custom restrictions check ingredients text (case-insensitive)
5. Handles empty restriction list (returns all recipes)
6. Handles no compatible recipes (returns empty Vec)
7. Unit tests cover all restriction types

**Story 7.2: Main Course Selection with Preferences**
1. Function `select_main_course_with_preferences` implemented
2. Filters by `max_prep_time` (weeknight vs weekend)
3. Filters by `skill_level` (Beginner→Simple, Intermediate→Simple+Moderate, Advanced→All)
4. Filters by `avoid_consecutive_complex` (checks `rotation_state.last_complex_meal_date`)
5. Scores by `cuisine_variety_weight` (penalizes recent cuisines per formula)
6. Returns highest-scored recipe
7. Handles no compatible recipes gracefully (returns `None`)
8. Unit tests cover preference combinations
9. Performance: <10ms for 100 recipes

**Story 7.3: Accompaniment Selection**
1. Function `select_accompaniment(main_course, available)` implemented
2. Returns `None` if `main_course.accepts_accompaniment == false`
3. Filters by `preferred_accompaniments` if specified
4. Selects random from filtered list using `thread_rng`
5. Returns `None` if no compatible accompaniments
6. Allows repetition (not tracked in rotation)
7. Unit tests cover pairing scenarios
8. Random selection uses `rand::thread_rng`

**Story 7.4: Single Week Generation**
1. Function `generate_single_week` implemented
2. Generates 21 assignments (7 days × 3 courses)
3. Assigns: appetizer, main (with optional accompaniment), dessert per day
4. Appetizer/dessert rotation (can repeat after exhausting full list)
5. Main course uses `select_main_course_with_preferences`
6. Accompaniment assigned if `accepts_accompaniment=true`
7. `RotationState` updated after each assignment (marks used recipes)
8. Returns `WeekMealPlan` with `status=Future`, `is_locked=false`
9. Unit tests cover full week generation

**Story 7.5: Multi-Week Meal Plan Generation**
1. Function `generate_multi_week_meal_plans` implemented
2. Calculates `max_weeks = min(5, min(appetizers, mains, desserts))`
3. Returns `InsufficientRecipes` error if `max_weeks < 1`
4. Filters by dietary restrictions before counting recipes
5. Generates weeks sequentially (loop 0..max_weeks)
6. Week dates calculated from next Monday + offset (ISO 8601)
7. Shopping list generated per week via `generate_shopping_list_for_week`
8. Returns `MultiWeekMealPlan` with all weeks and rotation state
9. Performance: <5 seconds for 5 weeks (P95)
10. Unit tests cover various recipe counts (edge cases: 1 week, 5 weeks, insufficient)

**Story 7.6: Shopping List Generation**
1. Function `generate_shopping_list_for_week` implemented
2. Loads recipes from assignments (main + accompaniments)
3. Aggregates ingredients (extracts from all recipes in week)
4. Groups by category (Produce, Dairy, Meat, Grains, Pantry, Frozen)
5. Combines duplicates (2 onions + 1 onion = 3 onions, same ingredient name)
6. Returns `ShoppingList` with categorized items
7. Includes both main AND accompaniment ingredients
8. Unit tests cover aggregation and categorization

**Story 7.7: Algorithm Integration Tests and Benchmarks**
1. Integration test: full multi-week generation with realistic data (50 recipes)
2. Test: dietary restrictions filter correctly (all enums + custom)
3. Test: time/skill constraints respected (weeknight vs weekend, skill levels)
4. Test: main courses never repeat across weeks (uniqueness verified)
5. Test: accompaniments assigned correctly (pairing logic)
6. Benchmark: 5-week generation <5 seconds using `criterion` crate
7. Coverage >80% for algorithm module (measured via `cargo-tarpaulin`)

## Traceability Mapping

| AC # | Spec Section | Component/API | Test Idea |
|------|--------------|---------------|-----------|
| 7.1.1 | APIs and Interfaces | `filter_by_dietary_restrictions` | Unit: Pass recipes with/without tags, verify filtered correctly |
| 7.1.2 | Workflows and Sequencing | Dietary filtering (step 5) | Unit: Multiple restrictions, verify AND logic |
| 7.1.3 | Data Models | `DietaryRestriction` enum | Unit: All enum variants tested |
| 7.1.4 | Workflows and Sequencing | Custom restriction matching | Unit: Case-insensitive ingredient text search |
| 7.1.5 | APIs and Interfaces | Empty restrictions handling | Unit: Empty Vec → returns all recipes |
| 7.1.6 | APIs and Interfaces | No compatible recipes | Unit: All recipes filtered → empty Vec |
| 7.2.1 | APIs and Interfaces | `select_main_course_with_preferences` | Unit: Verify function signature and return type |
| 7.2.2 | Workflows and Sequencing | Weeknight/weekend time filtering | Unit: Weekend recipe (90min) filtered on weeknight (30min max) |
| 7.2.3 | Data Models | `SkillLevel` enum mapping | Unit: Beginner user filters out Moderate/Complex |
| 7.2.4 | Workflows and Sequencing | Consecutive complex avoidance | Unit: Complex meal yesterday → filter Complex today if avoid=true |
| 7.2.5 | Workflows and Sequencing | Cuisine variety scoring | Unit: Italian used 2x → lower score than unused Mexican |
| 7.2.6 | APIs and Interfaces | Highest-scored selection | Unit: Multiple candidates → verify highest score selected |
| 7.2.7 | APIs and Interfaces | No compatible recipes | Unit: All filtered → returns `None` |
| 7.2.9 | Performance | Main course selection speed | Benchmark: <10ms for 100 recipes (criterion) |
| 7.3.1 | APIs and Interfaces | `select_accompaniment` | Unit: Verify function signature |
| 7.3.2 | Data Models | `accepts_accompaniment=false` | Unit: Main with false → returns `None` |
| 7.3.3 | Data Models | `preferred_accompaniments` filtering | Unit: Main prefers Pasta/Rice → filters others |
| 7.3.4 | APIs and Interfaces | Random selection | Unit: Multiple candidates → random (seed for determinism) |
| 7.3.6 | Services and Modules | Repetition allowed | Unit: Same accompaniment used twice in week (not rotation tracked) |
| 7.4.1 | APIs and Interfaces | `generate_single_week` | Unit: Verify function signature |
| 7.4.2 | Workflows and Sequencing | 21 assignments (7×3) | Integration: Count meal_assignments.len() == 21 |
| 7.4.3 | Workflows and Sequencing | 3 courses per day | Integration: Each date has appetizer, main, dessert |
| 7.4.4 | Services and Modules | Appetizer/dessert rotation | Unit: Exhaust 5 appetizers → cycles back to first |
| 7.4.5 | Workflows and Sequencing | Main course selection integration | Integration: Calls `select_main_course_with_preferences` |
| 7.4.6 | Workflows and Sequencing | Accompaniment assignment | Integration: Main with accepts=true gets accompaniment_id |
| 7.4.7 | Services and Modules | `RotationState` updates | Unit: Verify `mark_used_*` called for each recipe |
| 7.4.8 | Data Models | `WeekMealPlan` metadata | Unit: status=Future, is_locked=false |
| 7.5.1 | APIs and Interfaces | `generate_multi_week_meal_plans` | Integration: Full function call with realistic data |
| 7.5.2 | Workflows and Sequencing | Max weeks calculation | Unit: 10 appetizers, 15 mains, 8 desserts → min(5, min(10,15,8)) = 5 |
| 7.5.3 | APIs and Interfaces | `InsufficientRecipes` error | Unit: 0 appetizers → error with counts |
| 7.5.4 | Workflows and Sequencing | Pre-filtering by dietary | Integration: Vegan user → non-vegan recipes excluded from counts |
| 7.5.5 | Workflows and Sequencing | Sequential week generation | Integration: Weeks generated in loop, rotation state mutated |
| 7.5.6 | Workflows and Sequencing | ISO 8601 Monday start | Unit: `next_monday()` helper returns correct date |
| 7.5.7 | Services and Modules | Shopping list per week | Integration: Each week has `shopping_list_id` |
| 7.5.8 | Data Models | `MultiWeekMealPlan` structure | Unit: Contains Vec<WeekMealPlan>, rotation_state |
| 7.5.9 | Performance | 5-week generation speed | Benchmark: <5s P95 with 50 recipes (criterion) |
| 7.6.1 | APIs and Interfaces | `generate_shopping_list_for_week` | Unit: Verify function signature |
| 7.6.2 | Workflows and Sequencing | Load main + accompaniments | Unit: Meal with accompaniment → both recipes loaded |
| 7.6.3 | Services and Modules | Ingredient aggregation | Unit: Extract ingredients from Vec<Recipe> |
| 7.6.4 | Data Models | `ShoppingCategory` grouping | Unit: Onion → Produce, Milk → Dairy |
| 7.6.5 | Services and Modules | Duplicate combination | Unit: 2 onions + 1 onion = 3 onions |
| 7.6.6 | Data Models | `ShoppingList` structure | Unit: Contains Vec<ShoppingCategory> |
| 7.7.1 | Test Strategy | Full integration test | Integration: 50 recipes → 5 weeks generated successfully |
| 7.7.2 | Test Strategy | Dietary filtering end-to-end | Integration: Vegan user → no meat/dairy in plan |
| 7.7.3 | Test Strategy | Time/skill constraints | Integration: Beginner weeknight → only Simple <30min recipes |
| 7.7.4 | Test Strategy | Main course uniqueness | Integration: Assert no duplicate main_course_ids across 5 weeks |
| 7.7.5 | Test Strategy | Accompaniment pairing | Integration: Verify accompaniment_id set when accepts=true |
| 7.7.6 | Test Strategy | Performance benchmarks | Benchmark: Run criterion suite, assert <5s |
| 7.7.7 | Test Strategy | Code coverage | CI: cargo-tarpaulin >80% for crates/meal_planning/src/algorithm.rs |

## Risks, Assumptions, Open Questions

**Risks:**

1. **Risk: Algorithm Performance Degradation with Large Recipe Libraries**
   - Impact: Users with >100 recipes may experience >5s generation times
   - Likelihood: Medium (premium users may accumulate large libraries)
   - Mitigation: Implement early performance benchmarks (Story 7.7), profile with realistic data, optimize filtering algorithms if needed (e.g., pre-indexing by dietary tags)

2. **Risk: Cuisine Variety Scoring May Feel Unpredictable**
   - Impact: Users may not understand why certain recipes selected over others
   - Likelihood: Medium (algorithm is deterministic but complex)
   - Mitigation: Epic 8 route responses include "algorithm reasoning" field explaining selection (e.g., "Assigned to Saturday: more prep time available, Italian cuisine underrepresented this week")

3. **Risk: Edge Case Handling Gaps**
   - Impact: Algorithm may panic or return unexpected results with unusual recipe configurations
   - Likelihood: Low (comprehensive unit tests mitigate)
   - Mitigation: Story 7.7 includes edge case testing (empty lists, all filtered, exhaustion scenarios), optional property-based testing with `quickcheck`

4. **Risk: Insufficient Recipes Error May Block New Users**
   - Impact: Users with <7 favorite recipes cannot generate meal plans, poor onboarding experience
   - Likelihood: High for new users
   - Mitigation: Epic 9 frontend handles error gracefully with "Add X more recipes" CTA and suggestions from community (Discover feature)

**Assumptions:**

1. **Assumption: Epic 6 Domain Models Complete and Tested**
   - Validation: Epic 6 must have >90% test coverage and all migrations passing before Epic 7 begins
   - Impact if False: Algorithm implementation blocked or requires rework

2. **Assumption: Recipe Data Quality**
   - Assumption: Recipes have accurate `prep_time`, `cook_time`, `complexity`, `cuisine`, and `dietary_tags` fields
   - Impact if False: Constraint filtering may produce poor results (e.g., mislabeled Simple recipe takes 2 hours)
   - Mitigation: Epic 2 recipe creation form includes validation, future: user feedback on recipe metadata accuracy

3. **Assumption: Monday Week Start Acceptable to All Users**
   - Assumption: ISO 8601 Monday start is culturally acceptable (some regions use Sunday)
   - Impact if False: User confusion, requires configurable week start
   - Mitigation: MVP uses Monday universally, Epic 9+ could add user preference for week start day

4. **Assumption: Shopping List Ingredient Aggregation is "Good Enough"**
   - Assumption: Simple name-based duplicate detection (case-insensitive string matching) is sufficient
   - Impact if False: Shopping list may show "1 cup onion" and "2 onions" separately (unit conversion missing)
   - Mitigation: Acknowledged technical debt, future enhancement: unit conversion library

5. **Assumption: Random Accompaniment Selection is Acceptable UX**
   - Assumption: Users okay with non-deterministic accompaniment pairings
   - Impact if False: Users regenerate plans repeatedly to get preferred accompaniments
   - Mitigation: Future enhancement: user-configurable "preferred default accompaniments" per main course

**Open Questions:**

1. **Question: Should Algorithm Respect "Favorited" Status Beyond Filtering?**
   - Context: Currently algorithm uses all favorite recipes equally. Should recently favorited recipes be weighted higher?
   - Decision Needed By: Story 7.2 implementation
   - Recommendation: MVP uses equal weighting, add "recency boost" in future iteration if analytics show need

2. **Question: How to Handle Recipe Deletions Mid-Plan?**
   - Context: User deletes recipe currently in active meal plan (future weeks)
   - Decision Needed By: Epic 8 (route handlers)
   - Recommendation: Epic 8 prevents deletion of recipes in locked weeks, allows deletion from future weeks with automatic replacement

3. **Question: Should Cuisine Variety Consider Meal Type?**
   - Context: Should Italian appetizer + Italian dessert on same day count as 2x Italian usage or different meal types tracked separately?
   - Decision Needed By: Story 7.2 implementation
   - Recommendation: MVP tracks cuisine per main course only (appetizers/desserts not cuisine-scored), simpler algorithm

4. **Question: Performance Target Sufficient for Free Tier?**
   - Context: Free tier users limited to 10 recipes (max 1-2 weeks), premium users unlimited
   - Decision Needed By: Story 7.7 benchmarking
   - Recommendation: If 5-week generation <5s with 50 recipes, free tier will be <1s, acceptable

## Test Strategy Summary

**Test-Driven Development (TDD) Enforced:**
- Write failing test → Implement minimal code to pass → Refactor → Repeat
- All stories begin with test creation before implementation

**Test Pyramid:**

1. **Unit Tests (70% of tests)**
   - **Scope:** Individual functions in isolation (filter, select, score)
   - **Framework:** Built-in Rust `#[test]`, `assert_eq!`, `assert!` macros
   - **Coverage:** All functions, all enum variants, all edge cases
   - **Examples:**
     - `test_filter_by_dietary_restrictions_with_vegan()`
     - `test_select_main_course_respects_weeknight_time_limit()`
     - `test_cuisine_variety_scoring_formula()`
     - `test_accompaniment_selection_returns_none_when_not_accepted()`
     - `test_rotation_state_marks_main_course_as_used()`

2. **Integration Tests (25% of tests)**
   - **Scope:** Multi-function workflows (single week generation, multi-week generation)
   - **Framework:** Rust integration tests in `tests/` directory
   - **Coverage:** End-to-end algorithm flows with realistic data
   - **Examples:**
     - `test_generate_single_week_with_realistic_recipes()`
     - `test_multi_week_generation_respects_rotation_state()`
     - `test_shopping_list_includes_main_and_accompaniment_ingredients()`
     - `test_dietary_restrictions_filter_entire_pipeline()`

3. **Performance Benchmarks (5% of tests)**
   - **Scope:** Performance regression detection
   - **Framework:** `criterion` crate
   - **Coverage:** Critical path functions and full multi-week generation
   - **Examples:**
     - `bench_filter_by_dietary_restrictions_100_recipes()`
     - `bench_select_main_course_with_preferences_100_recipes()`
     - `bench_generate_multi_week_meal_plans_5_weeks()`
   - **Thresholds:** <10ms for filtering/selection, <5s P95 for 5-week generation

**Test Data Management:**
- **Fixtures:** Helper functions create realistic `Recipe`, `UserPreferences` structs
- **Deterministic Randomness:** Seed `rand::thread_rng` in tests for reproducibility
- **Edge Cases:** Explicit tests for empty lists, boundary values, exhaustion scenarios

**Test Organization:**
```
crates/meal_planning/
├── src/
│   ├── algorithm.rs          // Implementation
│   └── rotation.rs            // RotationState (Epic 6)
└── tests/
    ├── unit/
    │   ├── test_filtering.rs
    │   ├── test_selection.rs
    │   ├── test_rotation.rs
    │   └── test_shopping.rs
    ├── integration/
    │   ├── test_single_week.rs
    │   └── test_multi_week.rs
    └── benchmarks/
        └── algorithm_benchmarks.rs
```

**Coverage Targets:**
- **Overall:** >80% line coverage for `crates/meal_planning/src/` (measured via `cargo-tarpaulin`)
- **Critical Functions:** 100% coverage for `filter_by_dietary_restrictions`, `select_main_course_with_preferences`
- **CI Enforcement:** Coverage report generated in CI, build fails if <80%

**Test Execution:**
- **Local Development:** `cargo test` (runs all unit + integration tests)
- **Benchmarks:** `cargo bench` (runs criterion benchmarks, requires nightly for full features)
- **Coverage:** `cargo tarpaulin --out Html --output-dir coverage/` (generates HTML report)
- **CI/CD:** GitHub Actions runs `cargo test`, `cargo tarpaulin`, `cargo bench` on every PR

**Test Pattern for evento Integration (Story 7.7):**
```rust
#[tokio::test]
async fn test_multi_week_generation_emits_event() {
    let executor = evento::Executor::new(/* test db */);

    // Generate meal plan
    let result = generate_multi_week_meal_plans(user_id, recipes, prefs).await.unwrap();

    // Emit event
    let event = MultiWeekMealPlanGenerated { /* ... */ };
    executor.emit(event).await.unwrap();

    // Subscribe with unsafe_oneshot for sync processing in tests
    evento::subscribe("test-projections")
        .aggregator::<MealPlan>()
        .handler(project_multi_week_meal_plan_generated)
        .unsafe_oneshot(&executor)  // Sync processing for tests
        .await
        .unwrap();

    // Assert read model updated
    let meal_plans = query_meal_plans_from_db().await;
    assert_eq!(meal_plans.len(), result.generated_weeks.len());
}
```

**Regression Testing:**
- Criterion benchmarks track performance over time, alert on regressions >10%
- Integration tests added for every bug found in production (prevent recurrence)

**Edge Case Test Examples:**
- Empty recipe lists (should return `InsufficientRecipes`)
- All recipes filtered by dietary restrictions (should return `InsufficientRecipes`)
- Single recipe in each category (should generate 1 week only)
- 100 recipes in each category (should cap at 5 weeks, verify performance)
- Appetizer/dessert exhaustion mid-week (should auto-reset, continue generation)
- Main course exhaustion mid-week (should return error, halt generation)
- Accompaniment with empty preferred list (should return `None`, proceed without)
- Weekend vs weeknight prep time edge (89min on weeknight with 90min limit → allowed)
