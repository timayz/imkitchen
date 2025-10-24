# Technical Specification: Intelligent Meal Planning Engine

Date: 2025-10-11
Author: Jonathan
Epic ID: 3
Status: Draft

---

## Overview

The Intelligent Meal Planning Engine is the core value proposition of imkitchen, delivering automated weekly meal plan generation using multi-factor optimization with recipe rotation. This epic implements a sophisticated constraint satisfaction algorithm that matches recipe complexity to user availability, respects advance preparation timing requirements, ensures ingredient freshness, and maximizes recipe variety through intelligent rotation logic.

The system generates a single active meal plan per user organized by week, filling breakfast/lunch/dinner slots with recipes from the user's favorites. The algorithm considers multiple constraints simultaneously: weeknight availability patterns, cooking skill level, household size, advance preparation lead times, recipe complexity scores, and dietary restrictions. Visual calendar views with advance prep indicators and algorithm transparency features build user trust in the automated system.

**Key Business Rule:** All meal plan generation and regeneration operations create plans for **next week only** (Monday-Sunday starting from the Monday following the current week). This forward-looking approach gives users time to shop and prepare without disrupting the current week's meals.

## Objectives and Scope

### In Scope
1. Automated meal plan generation with multi-factor optimization algorithm
2. **Next-week-only generation:** All plans start from next Monday (Story 3.13)
3. Visual week-view calendar displaying breakfast/lunch/dinner assignments
4. Recipe rotation system ensuring no duplicates until all favorites used once
5. Individual meal slot replacement with constraint-aware suggestions
6. Full meal plan regeneration maintaining rotation state (next week target)
7. Algorithm transparency showing reasoning for meal assignments
8. Home dashboard displaying next week's meals from active plan
9. Recipe complexity calculation service
10. Advance preparation indicator visualization
11. Meal plan persistence and activation management
12. Insufficient recipe validation with user guidance
13. Integration with shopping list generation via domain events

### Out of Scope (Future Enhancements)
- Machine learning-based preference optimization
- Multi-week meal planning (MVP: single week only, always next week)
- Current week meal plan modification (MVP: next week only, current week read-only)
- Meal plan templates or pre-built suggestions
- Collaborative meal planning (family member preferences)
- Recipe scheduling based on weather or seasonal availability
- Integration with calendar systems (Google Calendar, Apple Calendar)
- Meal plan sharing with other users
- Nutritional goal tracking and macronutrient optimization
- Meal plan modification history visualization beyond event sourcing

## System Architecture Alignment

### Architecture Pattern
The Intelligent Meal Planning Engine follows the established event-sourced DDD architecture:

**Domain Crate:** `crates/meal_planning/`
- **Aggregate:** MealPlan (evento aggregate with full event sourcing)
- **Commands:** GenerateMealPlan, ReplaceMealSlot, RegenerateMealPlan
- **Events:** MealPlanGenerated, MealSlotReplaced, MealPlanRegenerated, RecipeUsedInRotation, RotationCycleReset
- **Domain Services:** MealPlanningAlgorithm, RecipeComplexityCalculator, RotationManager
- **Read Models:** meal_plans, meal_assignments, recipe_rotation_state

### Integration Points
1. **Recipe Domain:** Queries favorite recipes with complexity scores and dietary tags
2. **User Domain:** Reads user profile (availability, skill level, dietary restrictions)
3. **Shopping Domain:** Emits MealPlanGenerated/MealSlotReplaced events triggering shopping list generation
4. **Notification Domain:** Advance prep requirements trigger PrepReminderScheduled events

### CQRS Implementation
- **Commands:** Write-side creates/updates MealPlan aggregate via evento
- **Queries:** Read-side queries meal_plans and meal_assignments read models
- **Projections:** Evento subscriptions update read models from event stream

## Detailed Design

### Services and Modules

#### 1. MealPlanningAlgorithm (Core Domain Service)

**Purpose:** Multi-factor constraint satisfaction solver for recipe-to-slot assignment

**Algorithm Overview:**
```rust
// Pseudocode for meal planning algorithm
fn generate_meal_plan(user_profile, favorite_recipes, rotation_state) -> MealPlan {
    // 1. Filter recipes by rotation availability
    let available_recipes = filter_unused_recipes(favorite_recipes, rotation_state);

    // 2. Score recipes by complexity
    let scored_recipes = calculate_complexity_scores(available_recipes);

    // 3. Build constraint matrix
    let constraints = build_constraints(user_profile, scored_recipes);

    // 4. Generate meal slots (7 days × 3 meals = 21 slots)
    // Note: start_date is always Monday (week convention)
    let meal_slots = generate_meal_slots(start_date);

    // 5. Assign recipes to slots via constraint satisfaction
    let assignments = solve_csp(meal_slots, scored_recipes, constraints);

    // 6. Validate solution (all constraints satisfied)
    validate_assignments(assignments, constraints);

    // 7. Return meal plan with assignments and rotation updates
    MealPlan::new(assignments, updated_rotation_state)
}
```

**Constraint Types:**

1. **Availability Constraint:** Match recipe time requirements to user weeknight availability
   - Weeknight slots: recipes with total_time <= user.weeknight_availability_minutes
   - Weekend slots: no time constraints (assume more availability)

2. **Complexity Constraint:** Match recipe difficulty to user skill level and day energy
   - Beginner users: prioritize Simple recipes, limit Complex to weekends
   - Intermediate/Advanced: no restrictions, but balance across week

3. **Advance Prep Constraint:** Schedule recipes with lead time for preparation
   - Recipe requiring 4-hour marinade on Wednesday: assign to Wednesday with prep reminder Tuesday evening
   - Overnight rising (bread): assign to weekend morning, prep reminder night before

4. **Dietary Constraint:** Respect user dietary restrictions and allergens
   - Filter out recipes with restricted ingredients
   - Validate recipe tags match user dietary preferences

5. **Freshness Constraint:** Schedule ingredient-sensitive recipes appropriately
   - Seafood/fish recipes: assign to days 1-3 after shopping (early week)
   - Produce-heavy recipes: prioritize early-to-mid week
   - Shelf-stable recipes: flexible scheduling

6. **Equipment Conflict Constraint:** Avoid back-to-back recipes competing for equipment
   - No two oven-dependent recipes in same day (oven conflict)
   - No two slow-cooker recipes in same day

7. **Rotation Constraint:** Each recipe used once before any repeats
   - Hard constraint: never assign recipe already used in current cycle
   - Reset cycle when all favorites used once

**Performance Requirements:**
- Algorithm complexity: O(n) where n = favorite recipe count
- Target execution time: <5 seconds for 50 recipes
- Deterministic but varied (use seed for randomization to vary assignments)

**Implementation Strategy:**
```rust
pub struct MealPlanningAlgorithm {
    complexity_calculator: RecipeComplexityCalculator,
    rotation_manager: RotationManager,
}

impl MealPlanningAlgorithm {
    pub async fn generate(
        &self,
        user_profile: &UserProfile,
        favorite_recipes: Vec<Recipe>,
        rotation_state: RotationState,
    ) -> Result<MealPlanAssignments, MealPlanningError> {
        // Implementation details in algorithm.rs
    }

    fn score_recipe_for_slot(
        &self,
        recipe: &Recipe,
        slot: &MealSlot,
        user_profile: &UserProfile,
    ) -> f64 {
        // Weighted scoring function
        let complexity_score = self.calculate_complexity_fit(recipe, slot, user_profile);
        let time_score = self.calculate_time_fit(recipe, slot, user_profile);
        let freshness_score = self.calculate_freshness_fit(recipe, slot);

        (complexity_score * 0.4) + (time_score * 0.4) + (freshness_score * 0.2)
    }
}
```

#### 2. RecipeComplexityCalculator (Domain Service)

**Purpose:** Calculate objective complexity scores for recipes to inform meal assignments

**Complexity Factors:**
1. **Ingredient Count:** More ingredients = higher complexity
   - Weight: 30%
   - Scoring: Simple (<8 ingredients), Moderate (8-15), Complex (>15)

2. **Instruction Step Count:** More steps = higher complexity
   - Weight: 40%
   - Scoring: Simple (<6 steps), Moderate (6-10), Complex (>10)

3. **Advance Prep Requirement:** Any advance prep = significant complexity increase
   - Weight: 30%
   - Scoring: None (0), Short prep <4hr (50), Long prep >=4hr (100)

**Formula:**
```rust
complexity_score = (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)

// Map score to complexity enum
if score < 30 => Complexity::Simple
if score 30-60 => Complexity::Moderate
if score > 60 => Complexity::Complex
```

**Implementation:**
```rust
pub struct RecipeComplexityCalculator;

impl RecipeComplexityCalculator {
    pub fn calculate(&self, recipe: &Recipe) -> RecipeComplexity {
        let ingredient_score = recipe.ingredients.len() as f64;
        let step_score = recipe.instructions.len() as f64;
        let prep_score = self.calculate_prep_score(recipe.advance_prep_hours);

        let total = (ingredient_score * 0.3) + (step_score * 0.4) + (prep_score * 0.3);

        match total {
            x if x < 30.0 => RecipeComplexity::Simple,
            x if x <= 60.0 => RecipeComplexity::Moderate,
            _ => RecipeComplexity::Complex,
        }
    }

    fn calculate_prep_score(&self, advance_prep_hours: Option<u32>) -> f64 {
        match advance_prep_hours {
            None => 0.0,
            Some(hours) if hours < 4 => 50.0,
            Some(_) => 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipeComplexity {
    Simple,
    Moderate,
    Complex,
}
```

#### 3. RotationManager (Domain Service)

**Purpose:** Track recipe usage across rotation cycles to ensure maximum variety

**Rotation Cycle Logic:**
1. Initialize cycle when meal plan first generated
2. Track each recipe as "used" when assigned to meal plan
3. Filter out used recipes in subsequent generations/replacements
4. Reset cycle when all favorites used once (or manually triggered)
5. Persist rotation state in read model for cross-session continuity

**Data Structure:**
```rust
pub struct RotationState {
    pub cycle_number: u32,
    pub cycle_started_at: DateTime<Utc>,
    pub used_recipe_ids: HashSet<RecipeId>,
    pub total_favorite_count: usize,
}

impl RotationState {
    pub fn is_recipe_available(&self, recipe_id: &RecipeId) -> bool {
        !self.used_recipe_ids.contains(recipe_id)
    }

    pub fn mark_recipe_used(&mut self, recipe_id: RecipeId) {
        self.used_recipe_ids.insert(recipe_id);
    }

    pub fn should_reset_cycle(&self, current_favorite_count: usize) -> bool {
        // Reset if all favorites used OR favorite count decreased significantly
        self.used_recipe_ids.len() >= self.total_favorite_count
            || current_favorite_count < self.used_recipe_ids.len()
    }

    pub fn reset_cycle(&mut self, new_favorite_count: usize) {
        self.cycle_number += 1;
        self.cycle_started_at = Utc::now();
        self.used_recipe_ids.clear();
        self.total_favorite_count = new_favorite_count;
    }
}
```

**Events:**
- `RecipeUsedInRotation { recipe_id, cycle_number, timestamp }`
- `RotationCycleReset { new_cycle_number, favorite_count, timestamp }`

### Data Models and Contracts

#### Event Store (Evento Managed)

**MealPlan Aggregate Events:**

```rust
// Domain events for MealPlan aggregate
#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct MealPlanGenerated {
    pub user_id: UserId,
    pub start_date: NaiveDate,
    pub assignments: Vec<MealAssignment>,
    pub rotation_state: RotationState,
    pub algorithm_version: String,
    pub generation_duration_ms: u64,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct MealSlotReplaced {
    pub meal_plan_id: MealPlanId,
    pub date: NaiveDate,
    pub meal_type: MealType,
    pub old_recipe_id: RecipeId,
    pub new_recipe_id: RecipeId,
    pub replacement_reason: String,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct MealPlanRegenerated {
    pub meal_plan_id: MealPlanId,
    pub new_assignments: Vec<MealAssignment>,
    pub rotation_state: RotationState,
    pub regeneration_reason: String,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct RecipeUsedInRotation {
    pub recipe_id: RecipeId,
    pub cycle_number: u32,
    pub used_at: DateTime<Utc>,
}

#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct RotationCycleReset {
    pub user_id: UserId,
    pub old_cycle_number: u32,
    pub new_cycle_number: u32,
    pub favorite_count: usize,
    pub reset_at: DateTime<Utc>,
}

// Supporting types
#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct MealAssignment {
    pub date: NaiveDate,
    pub meal_type: MealType,
    pub recipe_id: RecipeId,
    pub prep_required: bool,
    pub assignment_reasoning: String,
}

#[derive(bincode::Encode, bincode::Decode, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}
```

**MealPlan Aggregate:**

```rust
use evento::prelude::*;

#[derive(Default, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct MealPlan {
    pub id: MealPlanId,
    pub user_id: UserId,
    pub start_date: NaiveDate,
    pub is_active: bool,
    pub assignments: Vec<MealAssignment>,
    pub rotation_state: RotationState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[evento::aggregator]
impl MealPlan {
    async fn meal_plan_generated(
        &mut self,
        event: EventDetails<MealPlanGenerated>,
    ) -> anyhow::Result<()> {
        self.id = MealPlanId(event.aggregator_id.clone());
        self.user_id = event.data.user_id;
        self.start_date = event.data.start_date;
        self.is_active = true;
        self.assignments = event.data.assignments;
        self.rotation_state = event.data.rotation_state;
        self.created_at = event.timestamp;
        self.updated_at = event.timestamp;
        Ok(())
    }

    async fn meal_slot_replaced(
        &mut self,
        event: EventDetails<MealSlotReplaced>,
    ) -> anyhow::Result<()> {
        // Find and update the specific assignment
        if let Some(assignment) = self.assignments.iter_mut().find(|a| {
            a.date == event.data.date && a.meal_type == event.data.meal_type
        }) {
            assignment.recipe_id = event.data.new_recipe_id;
        }

        self.updated_at = event.timestamp;
        Ok(())
    }

    async fn meal_plan_regenerated(
        &mut self,
        event: EventDetails<MealPlanRegenerated>,
    ) -> anyhow::Result<()> {
        self.assignments = event.data.new_assignments;
        self.rotation_state = event.data.rotation_state;
        self.updated_at = event.timestamp;
        Ok(())
    }

    async fn recipe_used_in_rotation(
        &mut self,
        event: EventDetails<RecipeUsedInRotation>,
    ) -> anyhow::Result<()> {
        self.rotation_state.mark_recipe_used(event.data.recipe_id);
        Ok(())
    }
}
```

#### Read Models (SQLite Schema)

**meal_plans table:**
```sql
CREATE TABLE meal_plans (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    start_date TEXT NOT NULL,  -- ISO 8601 date (YYYY-MM-DD), always Monday
    end_date TEXT NOT NULL,    -- Computed: start_date + 6 days (Sunday)
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    rotation_cycle_number INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, is_active) -- Only one active plan per user
);

CREATE INDEX idx_meal_plans_user_active ON meal_plans(user_id, is_active);
CREATE INDEX idx_meal_plans_start_date ON meal_plans(start_date);
```

**meal_assignments table:**
```sql
CREATE TABLE meal_assignments (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    date TEXT NOT NULL,            -- ISO 8601 date
    meal_type TEXT NOT NULL,       -- 'breakfast', 'lunch', 'dinner'
    recipe_id TEXT NOT NULL,
    prep_required BOOLEAN NOT NULL DEFAULT FALSE,
    assignment_reasoning TEXT,     -- Human-readable explanation
    complexity TEXT NOT NULL,      -- 'simple', 'moderate', 'complex'
    total_time_minutes INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE RESTRICT,
    UNIQUE(meal_plan_id, date, meal_type)
);

CREATE INDEX idx_meal_assignments_plan ON meal_assignments(meal_plan_id);
CREATE INDEX idx_meal_assignments_date ON meal_assignments(date);
CREATE INDEX idx_meal_assignments_recipe ON meal_assignments(recipe_id);
```

**recipe_rotation_state table:**
```sql
CREATE TABLE recipe_rotation_state (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    cycle_started_at TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    UNIQUE(user_id, cycle_number, recipe_id)
);

CREATE INDEX idx_rotation_user_cycle ON recipe_rotation_state(user_id, cycle_number);
```

#### Read Model Projections

**Evento Subscription Handlers:**

```rust
use evento::prelude::*;
use sqlx::{SqlitePool, query};

// Projection: MealPlanGenerated -> meal_plans + meal_assignments tables
#[evento::handler(MealPlan)]
pub async fn project_meal_plan_generated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let pool = context.executor.pool();

    // 1. Deactivate any existing active meal plan for user
    query(
        "UPDATE meal_plans SET is_active = FALSE WHERE user_id = ? AND is_active = TRUE"
    )
    .bind(&event.data.user_id)
    .execute(pool)
    .await?;

    // 2. Insert new meal plan
    let end_date = event.data.start_date + chrono::Duration::days(6);
    query(
        "INSERT INTO meal_plans (id, user_id, start_date, end_date, is_active, rotation_cycle_number, created_at, updated_at)
         VALUES (?, ?, ?, ?, TRUE, ?, ?, ?)"
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(event.data.start_date.to_string())
    .bind(end_date.to_string())
    .bind(event.data.rotation_state.cycle_number)
    .bind(event.timestamp.to_rfc3339())
    .bind(event.timestamp.to_rfc3339())
    .execute(pool)
    .await?;

    // 3. Insert meal assignments
    for assignment in event.data.assignments {
        query(
            "INSERT INTO meal_assignments (id, meal_plan_id, date, meal_type, recipe_id, prep_required, assignment_reasoning, complexity, total_time_minutes, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&event.aggregator_id)
        .bind(assignment.date.to_string())
        .bind(format!("{:?}", assignment.meal_type).to_lowercase())
        .bind(&assignment.recipe_id)
        .bind(assignment.prep_required)
        .bind(&assignment.assignment_reasoning)
        // Fetch complexity and time from recipes table
        .bind(event.timestamp.to_rfc3339())
        .execute(pool)
        .await?;
    }

    Ok(())
}

// Projection: MealSlotReplaced -> meal_assignments table
#[evento::handler(MealPlan)]
pub async fn project_meal_slot_replaced<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<MealSlotReplaced>,
) -> anyhow::Result<()> {
    let pool = context.executor.pool();

    query(
        "UPDATE meal_assignments
         SET recipe_id = ?, assignment_reasoning = ?
         WHERE meal_plan_id = ? AND date = ? AND meal_type = ?"
    )
    .bind(&event.data.new_recipe_id)
    .bind(&event.data.replacement_reason)
    .bind(&event.data.meal_plan_id)
    .bind(event.data.date.to_string())
    .bind(format!("{:?}", event.data.meal_type).to_lowercase())
    .execute(pool)
    .await?;

    Ok(())
}

// Projection: RecipeUsedInRotation -> recipe_rotation_state table
#[evento::handler(MealPlan)]
pub async fn project_recipe_used_in_rotation<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeUsedInRotation>,
) -> anyhow::Result<()> {
    let pool = context.executor.pool();

    // Get user_id from meal_plan via event metadata or context
    let user_id = event.metadata; // Assuming user_id stored in metadata

    query(
        "INSERT INTO recipe_rotation_state (id, user_id, cycle_number, cycle_started_at, recipe_id, used_at)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, cycle_number, recipe_id) DO NOTHING"
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&user_id)
    .bind(event.data.cycle_number)
    .bind(event.data.used_at.to_rfc3339())
    .bind(&event.data.recipe_id)
    .bind(event.data.used_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}
```

### APIs and Interfaces

#### HTTP Routes (Server-Rendered HTML)

**Route Definitions:**

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use askama::Template;

pub fn meal_plan_routes() -> Router<AppState> {
    Router::new()
        .route("/plan", get(show_meal_calendar))
        .route("/plan/generate", post(generate_meal_plan))
        .route("/plan/regenerate", post(regenerate_meal_plan))
        .route("/plan/meal/:id/replace", post(replace_meal_slot))
        .route("/dashboard", get(show_dashboard))
}

// Route handlers
async fn show_meal_calendar(
    State(state): State<AppState>,
    auth: Auth,
) -> Result<Html<String>, AppError> {
    // Query active meal plan for user
    let meal_plan = query_active_meal_plan(auth.user_id, &state.db).await?;

    // Render calendar template
    let template = MealCalendarTemplate {
        meal_plan,
        today: chrono::Local::now().date_naive(),
        user: auth.user,
    };

    Ok(Html(template.render()?))
}

async fn generate_meal_plan(
    State(state): State<AppState>,
    auth: Auth,
) -> Result<Redirect, AppError> {
    // 1. Query user profile and favorite recipes
    let user_profile = query_user_profile(auth.user_id, &state.db).await?;
    let favorite_recipes = query_favorite_recipes(auth.user_id, &state.db).await?;

    // 2. Validate minimum recipe count
    if favorite_recipes.len() < 7 {
        return Err(AppError::InsufficientRecipes {
            current: favorite_recipes.len(),
            required: 7,
        });
    }

    // 3. Query current rotation state
    let rotation_state = query_rotation_state(auth.user_id, &state.db).await?;

    // 4. Execute meal planning command
    let cmd = GenerateMealPlanCommand {
        user_id: auth.user_id,
        start_date: chrono::Local::now().date_naive(),
        favorite_recipes,
        user_profile,
        rotation_state,
    };

    let meal_plan_id = meal_planning::generate_meal_plan(cmd, &state.executor).await?;

    // 5. Redirect to calendar view
    Ok(Redirect::to("/plan"))
}

async fn replace_meal_slot(
    State(state): State<AppState>,
    auth: Auth,
    Path(assignment_id): Path<String>,
    Form(form): Form<ReplaceMealForm>,
) -> Result<Html<String>, AppError> {
    // 1. Validate assignment belongs to user's active meal plan
    let assignment = query_meal_assignment(assignment_id, &state.db).await?;
    validate_assignment_ownership(&assignment, auth.user_id)?;

    // 2. Execute replacement command
    let cmd = ReplaceMealSlotCommand {
        meal_plan_id: assignment.meal_plan_id,
        date: assignment.date,
        meal_type: assignment.meal_type,
        new_recipe_id: form.new_recipe_id,
        replacement_reason: "User requested replacement".to_string(),
    };

    meal_planning::replace_meal_slot(cmd, &state.executor).await?;

    // 3. Return updated meal slot HTML (TwinSpark AJAX response)
    let updated_assignment = query_meal_assignment(assignment_id, &state.db).await?;
    let template = MealSlotPartial { assignment: updated_assignment };

    Ok(Html(template.render()?))
}

// Form types
#[derive(Deserialize)]
pub struct ReplaceMealForm {
    pub new_recipe_id: RecipeId,
}
```

#### Askama Templates

**meal-calendar.html (Full Page):**

```html
{% extends "base.html" %}

{% block title %}Meal Plan Calendar{% endblock %}

{% block content %}
<div class="container mx-auto px-4 py-8">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-3xl font-bold">Your Meal Plan</h1>
        <div class="space-x-4">
            <button
                ts-req="/plan/regenerate"
                ts-req-method="POST"
                ts-confirm="This will replace your entire meal plan. Continue?"
                class="btn-secondary">
                Regenerate Plan
            </button>
            <a href="/shopping" class="btn-primary">Shopping List</a>
        </div>
    </div>

    <!-- Week navigation -->
    <div class="mb-4 text-center">
        <span class="text-lg">Week of {{ meal_plan.start_date | date_format }}</span>
        <!-- Note: start_date is always Monday (week convention) -->
    </div>

    <!-- Rotation progress -->
    <div class="mb-6 bg-blue-50 p-4 rounded-lg">
        <p class="text-sm text-gray-700">
            Recipe variety: {{ meal_plan.used_recipes_count }} of {{ meal_plan.total_favorites }} favorites used this cycle
        </p>
    </div>

    <!-- Calendar grid -->
    <div class="grid grid-cols-1 md:grid-cols-7 gap-4">
        {% for day in meal_plan.days %}
        <div class="border rounded-lg p-4 {% if day.is_today %}bg-yellow-50 border-yellow-400{% endif %}">
            <h3 class="font-semibold mb-2">{{ day.date | weekday }} {{ day.date | date_format }}</h3>

            <!-- Breakfast -->
            {% if let Some(breakfast) = day.breakfast %}
                {% include "components/meal-slot.html" with assignment=breakfast %}
            {% endif %}

            <!-- Lunch -->
            {% if let Some(lunch) = day.lunch %}
                {% include "components/meal-slot.html" with assignment=lunch %}
            {% endif %}

            <!-- Dinner -->
            {% if let Some(dinner) = day.dinner %}
                {% include "components/meal-slot.html" with assignment=dinner %}
            {% endif %}
        </div>
        {% endfor %}
    </div>
</div>
{% endblock %}
```

**components/meal-slot.html (Reusable Component):**

```html
<div id="meal-slot-{{ assignment.id }}" class="meal-slot bg-white p-3 rounded-lg shadow-sm mb-2">
    <!-- Recipe info -->
    <a href="/recipes/{{ assignment.recipe_id }}" class="block hover:text-blue-600">
        <h4 class="font-medium mb-1">{{ assignment.recipe_title }}</h4>
    </a>

    <!-- Complexity badge -->
    <span class="inline-block px-2 py-1 text-xs rounded
        {% match assignment.complexity %}
        {% when RecipeComplexity::Simple %}bg-green-100 text-green-800
        {% when RecipeComplexity::Moderate %}bg-yellow-100 text-yellow-800
        {% when RecipeComplexity::Complex %}bg-red-100 text-red-800
        {% endmatch %}">
        {{ assignment.complexity }}
    </span>

    <!-- Prep indicator -->
    {% if assignment.prep_required %}
    <span class="inline-block ml-2" title="Advance preparation required">
        <svg class="w-4 h-4 inline text-orange-500"><!-- Clock icon --></svg>
        Prep Required
    </span>
    {% endif %}

    <!-- Time info -->
    <p class="text-sm text-gray-600 mt-1">{{ assignment.total_time_minutes }} min total</p>

    <!-- Algorithm reasoning (tooltip) -->
    <div class="relative group mt-2">
        <button class="text-xs text-gray-500 hover:text-gray-700">
            <svg class="w-4 h-4 inline"><!-- Info icon --></svg>
            Why this day?
        </button>
        <div class="hidden group-hover:block absolute z-10 w-64 p-2 bg-gray-900 text-white text-xs rounded shadow-lg">
            {{ assignment.assignment_reasoning }}
        </div>
    </div>

    <!-- Replace action -->
    <button
        ts-req="/plan/meal/{{ assignment.id }}/replace"
        ts-req-method="POST"
        ts-target="#meal-slot-{{ assignment.id }}"
        ts-swap="outerHTML"
        class="mt-2 text-sm text-blue-600 hover:text-blue-800">
        Replace This Meal
    </button>
</div>
```

**dashboard.html (Today's Meals):**

```html
{% extends "base.html" %}

{% block content %}
<div class="container mx-auto px-4 py-8">
    <h1 class="text-3xl font-bold mb-6">Today's Meals</h1>

    {% if let Some(meal_plan) = active_meal_plan %}
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
            <!-- Breakfast -->
            {% if let Some(breakfast) = meal_plan.today.breakfast %}
            <div class="card">
                <h3 class="text-lg font-semibold mb-2">Breakfast</h3>
                {% include "components/meal-slot.html" with assignment=breakfast %}
            </div>
            {% endif %}

            <!-- Lunch -->
            {% if let Some(lunch) = meal_plan.today.lunch %}
            <div class="card">
                <h3 class="text-lg font-semibold mb-2">Lunch</h3>
                {% include "components/meal-slot.html" with assignment=lunch %}
            </div>
            {% endif %}

            <!-- Dinner -->
            {% if let Some(dinner) = meal_plan.today.dinner %}
            <div class="card">
                <h3 class="text-lg font-semibold mb-2">Dinner</h3>
                {% include "components/meal-slot.html" with assignment=dinner %}
            </div>
            {% endif %}
        </div>

        <!-- Prep reminders for today -->
        {% if !prep_tasks.is_empty() %}
        <div class="bg-orange-50 border border-orange-200 rounded-lg p-4 mb-6">
            <h3 class="font-semibold mb-2">Prep Tasks for Today</h3>
            <ul class="space-y-2">
                {% for task in prep_tasks %}
                <li class="flex items-center">
                    <input type="checkbox" id="task-{{ task.id }}" class="mr-2"
                           {% if task.completed %}checked{% endif %}>
                    <label for="task-{{ task.id }}">{{ task.description }}</label>
                </li>
                {% endfor %}
            </ul>
        </div>
        {% endif %}

        <a href="/plan" class="btn-primary">View Full Calendar</a>
    {% else %}
        <!-- No active meal plan -->
        <div class="text-center py-12">
            <p class="text-gray-600 mb-4">You don't have an active meal plan yet.</p>
            <form method="POST" action="/plan/generate">
                <button type="submit" class="btn-primary">Generate Meal Plan</button>
            </form>
        </div>
    {% endif %}
</div>
{% endblock %}
```

### Workflows and Sequencing

#### Workflow 1: Generate Initial Meal Plan

```
User Action: Click "Generate Meal Plan" on Dashboard
    ↓
1. POST /plan/generate
    ↓
2. Auth middleware validates JWT
    ↓
3. Route handler validates prerequisites:
    - User has ≥7 favorite recipes
    - User profile complete
    ↓
4. Query favorite recipes and rotation state
    ↓
5. Invoke domain command:
    meal_planning::generate_meal_plan(cmd)
    ↓
6. Domain layer (crates/meal_planning/):
    a. MealPlanningAlgorithm.generate()
    b. Filter available recipes by rotation
    c. Calculate complexity scores
    d. Build constraint matrix
    e. Solve CSP for recipe-to-slot assignments
    f. Validate all constraints satisfied
    ↓
7. Create MealPlan aggregate:
    evento::create::<MealPlan>()
        .data(&MealPlanGenerated { ... })
        .commit(&executor)
    ↓
8. MealPlanGenerated event written to event store
    ↓
9. Evento subscriptions (async):
    - project_meal_plan_generated() → updates meal_plans/meal_assignments tables
    - trigger_shopping_list_generation() → emits ShoppingListGenerationRequested event
    ↓
10. Route handler returns redirect:
    Redirect::to("/plan")
    ↓
11. Browser navigates to calendar view
    ↓
12. GET /plan
    ↓
13. Query meal_plans + meal_assignments read models
    ↓
14. Render MealCalendarTemplate with Askama
    ↓
15. HTML returned to browser with full week calendar
```

**Performance Targets:**
- Total flow duration: <5 seconds
- Algorithm execution: <3 seconds (for 50 recipes)
- Event persistence: <100ms
- Read model projection: <500ms (async, doesn't block response)
- Template rendering: <50ms

#### Workflow 2: Replace Individual Meal Slot

```
User Action: Click "Replace This Meal" on meal slot
    ↓
1. POST /plan/meal/:id/replace (AJAX via TwinSpark)
    ↓
2. Auth middleware validates JWT
    ↓
3. Route handler:
    a. Query meal_assignment by ID
    b. Validate ownership (assignment.meal_plan.user_id == auth.user_id)
    c. Query alternative recipes:
        - Same meal type
        - Unused in rotation
        - Match/improve constraints for slot
    ↓
4. User selects replacement recipe (if multiple options shown)
    ↓
5. Invoke domain command:
    meal_planning::replace_meal_slot(cmd)
    ↓
6. Domain layer:
    a. Load MealPlan aggregate from event stream
    b. Validate replacement recipe available
    c. Execute replacement logic
    d. Mark old recipe as available again in rotation
    e. Mark new recipe as used in rotation
    ↓
7. Update MealPlan aggregate:
    evento::load::<MealPlan>(meal_plan_id)
        .data(&MealSlotReplaced { ... })
        .commit(&executor)
    ↓
8. MealSlotReplaced event written
    ↓
9. Evento subscriptions:
    - project_meal_slot_replaced() → updates meal_assignments table
    - trigger_shopping_list_recalculation() → emits ShoppingListUpdateRequested
    ↓
10. Route handler renders partial template:
    MealSlotPartial { updated_assignment }
    ↓
11. HTML fragment returned to browser
    ↓
12. TwinSpark swaps meal-slot div with new content
    ↓
13. Calendar updated in real-time without page reload
```

**Performance Targets:**
- Total flow duration: <2 seconds
- Command execution: <500ms
- Event persistence: <100ms
- Partial template rendering: <50ms

#### Workflow 3: Algorithm Transparency Display

```
User Action: Hover over "Why this day?" info icon on meal slot
    ↓
1. Browser renders tooltip with assignment_reasoning from meal_assignment
    ↓
2. No server request (data already in DOM)
    ↓
3. Tooltip displays human-readable explanation:
    - "Assigned to Saturday: more prep time available (Complex recipe, 75min total)"
    - "Assigned to Tuesday: Quick weeknight meal (Simple, 30min total)"
    - "Prep tonight for tomorrow: Requires 4-hour marinade"
    ↓
4. User understands algorithmic decision
    ↓
5. Trust in intelligent automation increased
```

**Reasoning Generation Logic:**

```rust
fn generate_assignment_reasoning(
    recipe: &Recipe,
    slot: &MealSlot,
    user_profile: &UserProfile,
) -> String {
    let day_name = slot.date.weekday();
    let is_weekend = matches!(slot.date.weekday(), Weekday::Sat | Weekday::Sun);

    let reason = if recipe.complexity == RecipeComplexity::Complex && is_weekend {
        format!("Assigned to {}: more prep time available (Complex recipe, {}min total)",
                day_name, recipe.total_time_minutes)
    } else if recipe.complexity == RecipeComplexity::Simple && !is_weekend {
        format!("Assigned to {}: Quick weeknight meal (Simple recipe, {}min total)",
                day_name, recipe.total_time_minutes)
    } else if recipe.advance_prep_hours.is_some() {
        format!("Prep {} for {}'s meal: Requires {}-hour advance preparation",
                slot.prep_day, day_name, recipe.advance_prep_hours.unwrap())
    } else {
        format!("Assigned to {}: Matches your availability and preferences",
                day_name)
    };

    reason
}
```

## Non-Functional Requirements

### Performance

**Algorithm Performance:**
- **Target:** <5 seconds for 50 favorite recipes
- **Complexity:** O(n) where n = recipe count
- **Strategy:** Greedy constraint satisfaction with randomized tie-breaking
- **Optimization:** Pre-calculate recipe complexity scores on recipe creation/edit
- **Fallback:** If algorithm times out (>10 seconds), return partial plan with user notification

**Database Query Performance:**
- **Active Meal Plan Query:** <100ms (indexed on user_id + is_active)
- **Calendar View Query:** <200ms (join meal_plans + meal_assignments + recipes)
- **Rotation State Query:** <50ms (indexed on user_id + cycle_number)
- **Strategy:** Read models optimized for query patterns, indexed foreign keys

**Page Render Performance:**
- **Full Calendar Page:** <500ms server-side rendering
- **Dashboard Page:** <300ms (simpler query - only today's meals)
- **Partial Meal Slot Update:** <100ms (AJAX fragment)
- **Strategy:** Askama compile-time templates, SQLite prepared statement caching

**Caching Strategy:**
- **Static Assets:** 1 year cache (immutable CSS/JS)
- **User Profile/Recipes:** Read model caching (already materialized view)
- **Active Meal Plan:** No additional caching (query fast enough, ~100ms)
- **Algorithm Results:** Event sourcing provides implicit caching (regenerate only on user action)

### Security

**Input Validation:**
- **Meal Plan Generation:** Validate minimum favorite recipe count (≥7)
- **Meal Replacement:** Validate recipe ownership and assignment ownership
- **Date Boundaries:** Validate meal dates within reasonable range (current week ±4 weeks)

**Authorization:**
- **Meal Plan Access:** User can only view/modify their own meal plans
- **Recipe Access:** User can only assign their own favorite recipes to meal plans
- **Enforcement:** Auth middleware + domain-level user_id validation

**Data Integrity:**
- **Foreign Key Constraints:** meal_assignments.recipe_id → recipes.id (RESTRICT on delete)
- **Unique Constraints:** Only one active meal plan per user
- **Event Sourcing:** Full audit trail of all meal plan changes

**OWASP Compliance:**
- **SQL Injection:** Prevented by SQLx parameterized queries
- **XSS:** Prevented by Askama auto-escaping
- **CSRF:** Protected by SameSite=Lax cookies
- **Access Control:** JWT-based authentication, user_id validation in domain layer

### Reliability/Availability

**Error Handling:**
- **Algorithm Timeout:** Fallback to simplified algorithm or partial plan
- **Insufficient Recipes:** Clear user guidance with count and "Add Recipes" CTA
- **Recipe Deletion:** RESTRICT constraint prevents deletion of assigned recipes, require replacement first
- **Event Store Failure:** Return 500 error, log to OpenTelemetry, retry on transient failures

**Data Consistency:**
- **Event Sourcing:** Eventual consistency acceptable for read models (typical lag: <500ms)
- **Rotation State:** Strongly consistent via aggregate state
- **Active Plan Flag:** Database constraint ensures only one active plan per user

**Failover Strategy:**
- **Algorithm Failure:** Return user-friendly error, suggest regeneration or manual recipe addition
- **Database Connection Loss:** Display cached meal plan (PWA offline mode), queue updates for sync
- **Event Projection Lag:** Display loading indicator if read model not yet updated

### Observability

**Distributed Tracing:**
```rust
#[tracing::instrument(skip(executor))]
pub async fn generate_meal_plan(
    cmd: GenerateMealPlanCommand,
    executor: &SqliteEventExecutor,
) -> Result<MealPlanId, MealPlanningError> {
    tracing::info!(
        user_id = %cmd.user_id,
        favorite_count = cmd.favorite_recipes.len(),
        "Starting meal plan generation"
    );

    let start = std::time::Instant::now();

    // Algorithm execution...

    let duration_ms = start.elapsed().as_millis() as u64;

    tracing::info!(
        user_id = %cmd.user_id,
        duration_ms,
        "Meal plan generation completed"
    );

    Ok(meal_plan_id)
}
```

**Metrics:**
- `meal_plan_generation_duration_ms` (histogram)
- `meal_plan_generation_success_total` (counter)
- `meal_plan_generation_failure_total` (counter, labeled by error type)
- `meal_replacement_total` (counter)
- `rotation_cycle_reset_total` (counter)
- `algorithm_constraint_violations_total` (counter, for debugging)

**Logging:**
- INFO: Meal plan generation started/completed, meal slot replaced
- WARN: Algorithm constraint violations, insufficient recipes
- ERROR: Event store failures, aggregate load failures
- DEBUG: Algorithm decision details (recipe scores, constraint evaluation)

**Dashboards:**
- Average meal plan generation time (p50, p95, p99)
- Meal plan acceptance rate (plans generated vs regenerated within 24h)
- Recipe rotation cycle length (average days until cycle reset)
- User engagement: % users with active meal plans, average meal replacements per week

## Dependencies and Integrations

### Internal Domain Dependencies

**Recipe Domain (crates/recipe):**
- **Query:** Fetch favorite recipes for user
- **Query:** Fetch recipe complexity and dietary tags
- **Event Subscription:** RecipeFavorited/RecipeUnfavorited → update available recipes for meal planning
- **Event Subscription:** RecipeDeleted → mark meal slots for replacement if recipe assigned

**User Domain (crates/user):**
- **Query:** Fetch user profile (availability, skill level, dietary restrictions, household size)
- **Event Subscription:** ProfileUpdated → potentially invalidate cached meal plan assumptions

**Shopping Domain (crates/shopping):**
- **Event Emission:** MealPlanGenerated → trigger ShoppingListGenerationRequested
- **Event Emission:** MealSlotReplaced → trigger ShoppingListUpdateRequested
- **Integration:** Meal assignments provide recipe IDs for ingredient aggregation

**Notification Domain (crates/notifications):**
- **Event Emission:** MealPlanGenerated → trigger PrepReminderScheduled for advance prep recipes
- **Event Emission:** MealSlotReplaced → update/cancel existing prep reminders
- **Integration:** meal_assignments.prep_required flag indicates reminder needed

### External Dependencies

**SQLite Database:**
- **Purpose:** Event store (evento managed) + read models (meal_plans, meal_assignments, rotation_state)
- **Connection:** SQLx connection pool (shared with other domains)
- **Migrations:** Manual migrations for read model schemas

**Evento Library (1.3+):**
- **Purpose:** Event sourcing infrastructure, aggregate management, event store, subscriptions
- **Critical:** Core dependency for domain logic

**Chrono Crate:**
- **Purpose:** Date/time handling (meal dates, rotation timestamps)
- **Version:** 0.4+

**UUID Crate:**
- **Purpose:** Generate unique IDs for meal_plan_id, assignment_id
- **Version:** 1.0+

### Integration Testing

**Test Scenarios:**
1. **Generate meal plan with 10 favorite recipes:**
   - Verify 21 assignments created (7 days × 3 meals)
   - Verify no recipe duplicates in plan
   - Verify rotation state updated correctly

2. **Replace meal slot:**
   - Verify old recipe marked available in rotation
   - Verify new recipe marked used in rotation
   - Verify shopping list update event emitted

3. **Regenerate meal plan:**
   - Verify rotation state preserved across regeneration
   - Verify different recipe assignments (algorithm variation)
   - Verify old meal plan deactivated

4. **Insufficient recipes error:**
   - Attempt generation with 5 favorite recipes
   - Verify clear error message with guidance

5. **Recipe deletion impact:**
   - Delete recipe assigned to meal plan
   - Verify foreign key constraint prevents deletion
   - Verify error message guides user to replace meal first

## Acceptance Criteria (Authoritative)

### Story 3.1: Generate Initial Meal Plan
- [ ] "Generate Meal Plan" button visible on dashboard
- [ ] Generation completes within 5 seconds for 50 recipes
- [ ] Week-view calendar displays with 21 meal slots filled (7 days × 3 meals)
- [ ] User redirected to calendar view after successful generation
- [ ] Progress indicator shown during generation
- [ ] Generated plan automatically becomes active (is_active=TRUE)
- [ ] Error message displayed if <7 favorite recipes with helpful guidance

### Story 3.2: Multi-Factor Meal Planning Algorithm
- [ ] Algorithm analyzes user profile (availability, skill level, household size)
- [ ] Complex recipes assigned to weekends/days with more availability
- [ ] Simple recipes assigned to busy weeknights
- [ ] Advance prep recipes scheduled with proper lead time
- [ ] Dietary restrictions respected (no restricted ingredients)
- [ ] Ingredient freshness considered (seafood early week)
- [ ] Equipment conflicts avoided (no two oven recipes same day)
- [ ] Algorithm produces varied assignments on regeneration

### Story 3.3: Recipe Rotation System
- [ ] Each favorite recipe used exactly once before any repeats
- [ ] Rotation state persists across meal plan generations
- [ ] Meal replacement respects rotation (only offers unused recipes)
- [ ] Adding new favorite includes it in rotation immediately
- [ ] Rotation progress visible: "12 of 20 favorites used this cycle"
- [ ] Rotation cycle resets when all favorites used once

### Story 3.4: Visual Week-View Meal Calendar
- [ ] Calendar displays 7 days (Monday-Sunday, always starting Monday)
- [ ] Each day shows 3 meal slots: breakfast, lunch, dinner
- [ ] Each slot displays: recipe title, prep time, complexity badge
- [ ] Advance prep indicator (clock icon) visible on prep recipes
- [ ] Today's date highlighted with distinct styling
- [ ] Past dates dimmed/grayed out
- [ ] Mobile-responsive: stacks vertically on mobile, grid on desktop

### Story 3.5: View Recipe Details from Calendar
- [ ] Clicking recipe card opens recipe detail modal/page
- [ ] Recipe detail displays: ingredients, instructions, prep/cook times
- [ ] "Replace This Meal" button available on recipe detail
- [ ] Back/close navigation returns to calendar view
- [ ] Recipe detail optimized for kitchen use (large text, high contrast)

### Story 3.6: Replace Individual Meal Slot
- [ ] "Replace This Meal" button visible on each calendar slot
- [ ] System offers 3-5 alternative recipes matching constraints
- [ ] Alternatives respect rotation (only unused recipes)
- [ ] Selected recipe immediately replaces meal in calendar (AJAX update)
- [ ] Replaced recipe returned to rotation pool (available again)
- [ ] Shopping list automatically updates
- [ ] Confirmation message: "Meal replaced successfully"

### Story 3.7: Regenerate Full Meal Plan
- [ ] "Regenerate Meal Plan" button visible on calendar page
- [ ] Confirmation dialog: "This will replace your entire meal plan. Continue?"
- [ ] Regeneration preserves rotation state (doesn't reset cycle)
- [ ] New plan fills all 21 slots with different assignments
- [ ] Calendar updates to show new plan
- [ ] Shopping list regenerated
- [ ] Old meal plan archived (is_active=FALSE)

### Story 3.8: Algorithm Transparency
- [ ] Hovering/tapping info icon shows reasoning tooltip
- [ ] Reasoning displays human-readable explanation:
  - "Assigned to Saturday: more prep time available (Complex, 75min)"
  - "Assigned to Tuesday: Quick weeknight meal (Simple, 30min)"
  - "Prep tonight for tomorrow: Requires 4-hour marinade"
- [ ] Clear, jargon-free language
- [ ] Reasoning adapts to actual assignment factors used

### Story 3.9: Home Dashboard with Today's Meals
- [ ] Dashboard displays "Today's Meals" section at top
- [ ] Shows breakfast, lunch, dinner for today
- [ ] Each meal displays: recipe title, prep time
- [ ] Advance prep indicator if preparation required today
- [ ] "View Full Calendar" link visible
- [ ] If no active plan: "Generate Meal Plan" CTA displayed
- [ ] Today's meals update automatically at midnight

### Story 3.10: Handle Insufficient Recipes
- [ ] Error message: "You need at least 7 favorite recipes to generate a weekly meal plan. You currently have {count}."
- [ ] Helpful guidance: "Add {7 - count} more recipes to get started!"
- [ ] Direct link to "Add Recipe" or "Discover Recipes" page
- [ ] Friendly styling (not alarming red)
- [ ] Count updates in real-time as user adds/removes favorites

### Story 3.11: Meal Plan Persistence and Activation
- [ ] Generated meal plan stored in database
- [ ] Exactly one meal plan active per user at a time (database constraint)
- [ ] Active meal plan automatically loaded on dashboard/calendar
- [ ] Meal plan persists across browser sessions and devices
- [ ] Regeneration archives old plan, creates new active plan
- [ ] is_active flag managed correctly

### Story 3.12: Recipe Complexity Calculation
- [ ] Complexity calculated on recipe creation/update
- [ ] Scoring: Simple (<30), Moderate (30-60), Complex (>60)
- [ ] Factors: ingredient count (30%), step count (40%), advance prep (30%)
- [ ] Complexity badge stored in read model
- [ ] Recalculated automatically when recipe edited
- [ ] Complexity visible on recipe cards throughout app

## Traceability Mapping

### Epic 3 → PRD Requirements

| Story ID | PRD Functional Requirement | PRD User Journey |
|----------|----------------------------|------------------|
| 3.1 | FR-4: Automated Meal Plan Generation | Journey 1: Steps 4-5 (First Meal Plan Generation) |
| 3.2 | FR-4: Multi-factor optimization | Journey 1: Step 4 (Algorithm analyzes constraints) |
| 3.3 | FR-5: Recipe Rotation System | Journey 2: Steps 3-5 (Expanding Recipe Variety) |
| 3.4 | FR-6: Visual Meal Calendar | Journey 1: Step 5 (Reviewing Generated Plan) |
| 3.5 | - | Journey 1: Step 5 (Reviewing Recipe Details) |
| 3.6 | FR-7: Meal Plan Regeneration (individual) | Journey 1: Step 5, Journey 3: Steps 2-3 (Meal Replacement) |
| 3.7 | FR-7: Meal Plan Regeneration (full) | Journey 2: Step 4 (Regeneration with New Recipe) |
| 3.8 | - | UX Principle 3: Trust Through Transparency |
| 3.9 | FR-16: Home Dashboard | Journey 1: Step 7 (Dashboard Displaying Today's Meals) |
| 3.10 | - | Error handling for FR-4 prerequisites |
| 3.11 | FR-4: Meal plan persistence | Journey 1: Step 5 (Meal Plan Saved) |
| 3.12 | FR-4: Recipe complexity factor | Journey 1: Step 4 (Algorithm analyzes complexity) |

### Epic 3 → Architecture Components

| Component | Architecture Section | Implementation Location |
|-----------|----------------------|-------------------------|
| MealPlan Aggregate | Section 3.2: Data Models | `crates/meal_planning/src/aggregate.rs` |
| MealPlanningAlgorithm | Section 11.1: Domain Crate Structure | `crates/meal_planning/src/algorithm.rs` |
| RecipeComplexityCalculator | Section 11.1: Domain Services | `crates/meal_planning/src/complexity.rs` |
| RotationManager | Section 11.1: Domain Services | `crates/meal_planning/src/rotation.rs` |
| meal_plans table | Section 3.2: Read Models | `migrations/003_create_meal_plans_table.sql` |
| meal_assignments table | Section 3.2: Read Models | `migrations/003_create_meal_plans_table.sql` |
| recipe_rotation_state table | Section 3.2: Read Models | `migrations/003_create_meal_plans_table.sql` |
| /plan routes | Section 2.3: Page Routing | `src/routes/meal_plan.rs` |
| MealCalendarTemplate | Section 7.1: Component Structure | `templates/pages/meal-calendar.html` |
| MealSlotPartial | Section 7.1: Component Structure | `templates/partials/meal-slot-content.html` |

### Epic 3 → Non-Functional Requirements

| NFR | Epic 3 Implementation | Verification Method |
|-----|------------------------|---------------------|
| NFR-1: Performance (<3s page load) | Calendar view <500ms render, Dashboard <300ms | Lighthouse audit, E2E timing tests |
| NFR-1: Algorithm performance (<5s) | MealPlanningAlgorithm.generate() <3s for 50 recipes | Unit test with timing assertions |
| NFR-2: Zero data loss | Event sourcing + read model projections | Integration tests verify event replay |
| NFR-3: Horizontal scaling | Stateless algorithm, read models cacheable | Load testing with multiple pods |
| NFR-8: Observability | OpenTelemetry tracing on all commands/queries | Dashboard monitoring in Grafana |

## Risks, Assumptions, Open Questions

### Risks

**High Risk:**
1. **Algorithm Performance Degradation:**
   - **Risk:** Algorithm exceeds 5-second target with complex user constraints
   - **Mitigation:** Implement timeout fallback (simplified algorithm), profile algorithm performance early
   - **Fallback:** If timeout occurs, generate partial plan and notify user, suggest reducing favorites

2. **Rotation State Inconsistency:**
   - **Risk:** Concurrent meal plan operations corrupt rotation state
   - **Mitigation:** Aggregate-level locking via evento, eventual consistency acceptable for read models
   - **Testing:** Property-based tests for rotation invariants, concurrent operation testing

**Medium Risk:**
3. **User Rejection of Automated Assignments:**
   - **Risk:** Users find algorithm suggestions unrealistic, excessive meal replacements
   - **Mitigation:** Algorithm transparency (show reasoning), easy replacement mechanism, user feedback loop
   - **Monitoring:** Track regeneration rate and replacement frequency as quality metrics

4. **Recipe Deletion Cascade Complexity:**
   - **Risk:** Deleting assigned recipe breaks active meal plans
   - **Mitigation:** RESTRICT foreign key constraint, require user to replace meal before deletion
   - **UX:** Clear error message guides user to replacement flow

**Low Risk:**
5. **Timezone Handling for Rotation Dates:**
   - **Risk:** Rotation state timestamps misaligned with user local time
   - **Mitigation:** Store timestamps in UTC, convert to user timezone for display only
   - **Testing:** Test with users in different timezones

### Assumptions

**User Behavior:**
- Users willing to accept automated meal assignments 85% of time (15% replacement rate acceptable)
- Users mark at least 7 recipes as favorites before attempting meal plan generation
- Users tolerate regeneration taking up to 5 seconds (acceptable wait time for value delivered)
- Algorithm transparency builds user trust (hypothesis to validate)

**Technical:**
- Event sourcing overhead acceptable for meal planning domain (low write volume, ~1-2 operations/day per user)
- SQLite query performance sufficient for meal plan calendar view (<200ms for 21-row join)
- Eventual consistency acceptable for read model updates (typical lag <500ms)
- Constraint satisfaction can be solved with greedy algorithm (no NP-hard optimization needed for MVP)

**Data:**
- Recipe complexity scoring formula (30/40/30 weights) approximates real-world difficulty
- Freshness constraints (seafood days 1-3) represent typical shopping patterns
- User availability patterns stable week-to-week (no dynamic adjustment needed in MVP)

### Open Questions

1. **Algorithm Variation Strategy:**
   - **Question:** How much variation should regeneration produce? Same recipes in different slots vs entirely different recipes?
   - **Current Approach:** Use randomized seed for tie-breaking in scoring, produces varied assignments
   - **Decision Needed:** Validate with user testing, may need tunable "variety" parameter

2. **Rotation Cycle Reset Trigger:**
   - **Question:** Should rotation reset automatically when all favorites used, or require user action?
   - **Current Approach:** Automatic reset when all favorites used once
   - **Consideration:** Manual reset option for users who want to "replay" favorite cycle

3. **Meal Type Flexibility:**
   - **Question:** Should algorithm allow recipes to move between meal types (breakfast recipe at lunch)?
   - **Current Approach:** MVP restricts to single meal type per recipe (simplifies constraints)
   - **Future:** Allow meal type flexibility with recipe metadata (suitable_for: [breakfast, lunch])

4. **Multi-Week Planning:**
   - **Question:** When to implement multi-week planning vs single-week MVP?
   - **Current Approach:** MVP single week only (21 slots)
   - **Future:** Extend to 2-4 week planning when user demand validated

5. **Constraint Priority:**
   - **Question:** What happens when constraints conflict (e.g., no Simple recipes left but weeknight slot needs filling)?
   - **Current Approach:** Soft constraints with scoring, algorithm always produces valid solution (may violate preference)
   - **Improvement:** Explicit constraint priority ranking, user notification of constraint violations

## Test Strategy Summary

### Unit Tests (TDD)

**MealPlanningAlgorithm Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_respects_availability_constraints() {
        let user_profile = UserProfile {
            weeknight_availability_minutes: 45,
            ..Default::default()
        };

        let recipes = vec![
            Recipe::simple_30min(), // Should assign to weeknight
            Recipe::complex_90min(), // Should assign to weekend
        ];

        let assignments = algorithm.generate(user_profile, recipes, RotationState::new()).unwrap();

        // Verify complex recipe assigned to weekend
        let weekend_assignments = assignments.filter(|a| a.date.weekday() == Weekday::Sat);
        assert!(weekend_assignments.any(|a| a.recipe.complexity == Complexity::Complex));

        // Verify simple recipe assigned to weeknight
        let weeknight_assignments = assignments.filter(|a| a.date.weekday() == Weekday::Tue);
        assert!(weeknight_assignments.any(|a| a.recipe.complexity == Complexity::Simple));
    }

    #[test]
    fn test_rotation_prevents_duplicates() {
        let mut rotation = RotationState::new();
        let recipe_id = RecipeId::new();

        // Mark recipe as used
        rotation.mark_recipe_used(recipe_id);

        // Verify recipe not available
        assert!(!rotation.is_recipe_available(&recipe_id));

        // Reset cycle
        rotation.reset_cycle(10);

        // Verify recipe available again
        assert!(rotation.is_recipe_available(&recipe_id));
    }

    #[test]
    fn test_complexity_calculator() {
        let calculator = RecipeComplexityCalculator::new();

        // Simple recipe: 5 ingredients, 4 steps, no advance prep
        let simple_recipe = Recipe {
            ingredients: vec![/* 5 ingredients */],
            instructions: vec![/* 4 steps */],
            advance_prep_hours: None,
        };
        assert_eq!(calculator.calculate(&simple_recipe), RecipeComplexity::Simple);

        // Complex recipe: 18 ingredients, 12 steps, 4-hour marinade
        let complex_recipe = Recipe {
            ingredients: vec![/* 18 ingredients */],
            instructions: vec![/* 12 steps */],
            advance_prep_hours: Some(4),
        };
        assert_eq!(calculator.calculate(&complex_recipe), RecipeComplexity::Complex);
    }
}
```

**RotationManager Tests:**
```rust
#[test]
fn test_rotation_cycle_lifecycle() {
    let mut rotation = RotationState::new();
    let recipes = vec![RecipeId::new(), RecipeId::new(), RecipeId::new()];

    // Use all recipes
    for recipe_id in &recipes {
        rotation.mark_recipe_used(*recipe_id);
    }

    // Verify cycle should reset
    assert!(rotation.should_reset_cycle(recipes.len()));

    // Reset cycle
    rotation.reset_cycle(recipes.len());

    // Verify all recipes available again
    for recipe_id in &recipes {
        assert!(rotation.is_recipe_available(recipe_id));
    }
}
```

### Integration Tests

**Meal Plan Generation Flow:**
```rust
#[tokio::test]
async fn test_generate_meal_plan_integration() {
    let app = test_app().await;
    let client = reqwest::Client::new();

    // Setup: Create user with 10 favorite recipes
    let user = create_test_user(&app).await;
    let recipes = create_favorite_recipes(&app, user.id, 10).await;

    // Execute: Generate meal plan
    let resp = client.post(&format!("{}/plan/generate", app.url))
        .header("Cookie", format!("auth_token={}", user.auth_token))
        .send()
        .await
        .unwrap();

    // Verify: Redirects to calendar
    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    assert!(resp.headers().get("Location").unwrap().to_str().unwrap().ends_with("/plan"));

    // Verify: Meal plan created in database
    let meal_plan = query_active_meal_plan(user.id, &app.db).await.unwrap();
    assert_eq!(meal_plan.assignments.len(), 21); // 7 days × 3 meals

    // Verify: No duplicate recipes
    let recipe_ids: HashSet<_> = meal_plan.assignments.iter().map(|a| a.recipe_id).collect();
    assert_eq!(recipe_ids.len(), meal_plan.assignments.len()); // All unique

    // Verify: Events persisted
    let events = query_events(&app.executor, meal_plan.id).await.unwrap();
    assert!(events.iter().any(|e| matches!(e, MealPlanGenerated { .. })));
}

#[tokio::test]
async fn test_replace_meal_slot_integration() {
    let app = test_app().await;
    let client = reqwest::Client::new();

    // Setup: User with active meal plan
    let user = create_test_user_with_meal_plan(&app, 15).await;
    let assignment = user.meal_plan.assignments[0];

    // Find replacement recipe (unused in rotation)
    let replacement_recipe = user.recipes.iter()
        .find(|r| !user.meal_plan.used_recipe_ids.contains(&r.id))
        .unwrap();

    // Execute: Replace meal slot
    let resp = client.post(&format!("{}/plan/meal/{}/replace", app.url, assignment.id))
        .header("Cookie", format!("auth_token={}", user.auth_token))
        .form(&[("new_recipe_id", replacement_recipe.id.to_string())])
        .send()
        .await
        .unwrap();

    // Verify: Returns HTML fragment
    assert_eq!(resp.status(), StatusCode::OK);
    let html = resp.text().await.unwrap();
    assert!(html.contains(&replacement_recipe.title));

    // Verify: Database updated
    let updated_assignment = query_meal_assignment(assignment.id, &app.db).await.unwrap();
    assert_eq!(updated_assignment.recipe_id, replacement_recipe.id);

    // Verify: Rotation state updated
    let rotation = query_rotation_state(user.id, &app.db).await.unwrap();
    assert!(!rotation.used_recipe_ids.contains(&assignment.recipe_id)); // Old recipe available
    assert!(rotation.used_recipe_ids.contains(&replacement_recipe.id)); // New recipe used
}
```

### E2E Tests (Playwright)

**Meal Planning User Flow:**
```typescript
test('generate and view meal plan', async ({ page }) => {
  // 1. Login as user with 10 favorite recipes
  await page.goto('/login');
  await page.fill('input[name="email"]', 'test@example.com');
  await page.fill('input[name="password"]', 'password123');
  await page.click('button[type="submit"]');

  // 2. Navigate to dashboard
  await page.waitForURL('/dashboard');

  // 3. Generate meal plan
  await page.click('button:has-text("Generate Meal Plan")');
  await page.waitForURL('/plan');

  // 4. Verify calendar displays 7 days
  const days = await page.$$('.calendar-day');
  expect(days.length).toBe(7);

  // 5. Verify 21 meal slots filled
  const meals = await page.$$('.meal-slot');
  expect(meals.length).toBe(21);

  // 6. Verify no duplicate recipes visible
  const recipeTitles = await page.$$eval('.meal-slot h4', els => els.map(el => el.textContent));
  const uniqueTitles = new Set(recipeTitles);
  expect(uniqueTitles.size).toBe(recipeTitles.length);

  // 7. Verify complexity badges present
  const complexityBadges = await page.$$('.complexity-badge');
  expect(complexityBadges.length).toBe(21);

  // 8. Verify prep indicators on some meals
  const prepIndicators = await page.$$('span:has-text("Prep Required")');
  expect(prepIndicators.length).toBeGreaterThan(0);
});

test('replace meal slot', async ({ page }) => {
  // 1. Login and navigate to calendar with active meal plan
  await loginAndNavigateToCalendar(page);

  // 2. Click "Replace This Meal" on first dinner slot
  await page.click('.meal-slot:has-text("Dinner") button:has-text("Replace This Meal")');

  // 3. Wait for AJAX update (TwinSpark)
  await page.waitForResponse(resp => resp.url().includes('/plan/meal/') && resp.status() === 200);

  // 4. Verify meal slot updated with new recipe
  const updatedSlot = await page.$('.meal-slot:has-text("Dinner")');
  const newTitle = await updatedSlot.$eval('h4', el => el.textContent);
  expect(newTitle).not.toBeNull();

  // 5. Verify confirmation message displayed
  await expect(page.locator('.toast:has-text("Meal replaced successfully")')).toBeVisible();
});

test('algorithm transparency tooltip', async ({ page }) => {
  await loginAndNavigateToCalendar(page);

  // Hover over "Why this day?" info icon
  await page.hover('.meal-slot .info-icon');

  // Verify reasoning tooltip displays
  const tooltip = page.locator('.tooltip');
  await expect(tooltip).toBeVisible();

  // Verify tooltip contains reasoning text
  const tooltipText = await tooltip.textContent();
  expect(tooltipText).toMatch(/(more prep time available|Quick weeknight meal|advance preparation)/);
});
```

### Property-Based Tests

**Rotation Invariant Tests:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_rotation_never_duplicates_before_reset(
        recipe_ids in prop::collection::vec(any::<u64>(), 5..20)
    ) {
        let mut rotation = RotationState::new();
        let recipe_ids: Vec<RecipeId> = recipe_ids.into_iter().map(RecipeId::from).collect();

        // Use each recipe once
        for recipe_id in &recipe_ids {
            rotation.mark_recipe_used(*recipe_id);
        }

        // Verify no recipe available before reset
        for recipe_id in &recipe_ids {
            assert!(!rotation.is_recipe_available(recipe_id));
        }

        // Reset cycle
        rotation.reset_cycle(recipe_ids.len());

        // Verify all recipes available after reset
        for recipe_id in &recipe_ids {
            assert!(rotation.is_recipe_available(recipe_id));
        }
    }

    #[test]
    fn prop_algorithm_always_produces_valid_plan(
        favorite_count in 7usize..50,
        weeknight_availability in 30u32..120,
    ) {
        let user_profile = UserProfile {
            weeknight_availability_minutes: weeknight_availability,
            ..Default::default()
        };

        let recipes = generate_random_recipes(favorite_count);

        let result = algorithm.generate(user_profile, recipes, RotationState::new());

        // Algorithm must always succeed (no panic, no infinite loop)
        assert!(result.is_ok());

        // Plan must have 21 assignments
        let plan = result.unwrap();
        assert_eq!(plan.assignments.len(), 21);

        // All assignments must have valid recipe IDs
        for assignment in plan.assignments {
            assert!(!assignment.recipe_id.is_nil());
        }
    }
}
```

### Performance Tests

**Algorithm Benchmark:**
```rust
#[bench]
fn bench_meal_planning_algorithm_50_recipes(b: &mut Bencher) {
    let user_profile = UserProfile::default();
    let recipes = generate_recipes(50);
    let rotation_state = RotationState::new();
    let algorithm = MealPlanningAlgorithm::new();

    b.iter(|| {
        let result = algorithm.generate(&user_profile, &recipes, &rotation_state);
        assert!(result.is_ok());
    });

    // Assert average time <3 seconds
}

#[bench]
fn bench_calendar_view_query(b: &mut Bencher) {
    let pool = setup_test_db_with_meal_plan();
    let user_id = UserId::new();

    b.iter(|| {
        let meal_plan = block_on(query_active_meal_plan(user_id, &pool));
        assert!(meal_plan.is_some());
    });

    // Assert average time <100ms
}
```

---

**Document Status:** Draft
**Next Steps:** Review with architect, validate algorithm design, approve for implementation
**Estimated Implementation Time:** 3-4 weeks (12 stories, including algorithm development and extensive testing)

---

_This technical specification follows BMad Method conventions and provides authoritative implementation guidance for Epic 3: Intelligent Meal Planning Engine._
