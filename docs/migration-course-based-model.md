# Migration Plan: Course-Based Meal Planning Model

**Date:** 2025-10-20
**Author:** System Architecture
**Status:** Planning

## Overview

This document outlines the migration strategy from the current **breakfast/lunch/dinner** meal planning model to a new **appetizer/main_course/dessert** course-based model.

## Changes Summary

### Conceptual Change
- **Old Model:** 7 days × 3 meals (breakfast, lunch, dinner) = 21 meal slots
- **New Model:** 7 days × 3 courses (appetizer, main_course, dessert) = 21 course slots
- **Key Difference:** Each day now has a single lunch with 3 courses instead of 3 separate meals

### Technical Changes

#### 1. Recipe Domain
- Add `recipe_type` field: `"appetizer"`, `"main_course"`, or `"dessert"`
- Required for all recipes (no default value)
- Used by meal planning algorithm to match recipes to course slots

#### 2. Meal Planning Domain
- Rename `meal_type` → `course_type`
- Update enum values: `Breakfast/Lunch/Dinner` → `Appetizer/MainCourse/Dessert`
- Update algorithm to assign by course type matching

## Migration Strategy

### Phase 1: Database Schema Changes

#### Migration File: `migrations/XX_course_based_model.sql`

```sql
-- Step 1: Add recipe_type column to recipes table (with temporary default)
ALTER TABLE recipes ADD COLUMN recipe_type TEXT NOT NULL DEFAULT 'main_course';

-- Step 2: Update existing recipes based on heuristics
-- Heuristic: Recipes with advance_prep or cook_time > 30 minutes → main_course
--            Recipes with "dessert" in title → dessert
--            Remaining recipes → main_course (safe default)

UPDATE recipes
SET recipe_type = 'dessert'
WHERE LOWER(title) LIKE '%dessert%'
   OR LOWER(title) LIKE '%cake%'
   OR LOWER(title) LIKE '%cookie%'
   OR LOWER(title) LIKE '%pie%'
   OR LOWER(title) LIKE '%pudding%'
   OR LOWER(title) LIKE '%ice cream%';

UPDATE recipes
SET recipe_type = 'appetizer'
WHERE LOWER(title) LIKE '%appetizer%'
   OR LOWER(title) LIKE '%starter%'
   OR LOWER(title) LIKE '%salad%'
   OR LOWER(title) LIKE '%soup%'
   OR LOWER(complexity) = 'simple'
AND recipe_type = 'main_course'; -- Only update if not already dessert

-- All remaining recipes stay as 'main_course' (default)

-- Step 3: Add index for course-based queries
CREATE INDEX idx_recipes_type ON recipes(recipe_type);

-- Step 4: Rename meal_type to course_type in meal_assignments
ALTER TABLE meal_assignments RENAME COLUMN meal_type TO course_type;

-- Step 5: Update existing meal_assignments values
UPDATE meal_assignments SET course_type = 'appetizer' WHERE course_type = 'breakfast';
UPDATE meal_assignments SET course_type = 'main_course' WHERE course_type = 'lunch';
UPDATE meal_assignments SET course_type = 'dessert' WHERE course_type = 'dinner';

-- Step 6: Remove DEFAULT from recipe_type (enforce explicit values going forward)
-- SQLite doesn't support ALTER COLUMN, so we need to recreate the table
CREATE TABLE recipes_new (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  ingredients TEXT NOT NULL,
  instructions TEXT NOT NULL,
  prep_time_min INTEGER,
  cook_time_min INTEGER,
  advance_prep_hours INTEGER,
  serving_size INTEGER,
  recipe_type TEXT NOT NULL, -- No DEFAULT - must be explicit
  is_favorite BOOLEAN DEFAULT FALSE,
  is_shared BOOLEAN DEFAULT FALSE,
  complexity TEXT,
  cuisine TEXT,
  dietary_tags TEXT,
  manual_override BOOLEAN DEFAULT FALSE,
  original_recipe_id TEXT,
  original_author TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT,
  deleted_at TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Copy data
INSERT INTO recipes_new SELECT * FROM recipes;

-- Drop old table and rename
DROP TABLE recipes;
ALTER TABLE recipes_new RENAME TO recipes;

-- Recreate indexes
CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_favorite ON recipes(user_id, is_favorite);
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = TRUE;
CREATE INDEX idx_recipes_type ON recipes(recipe_type);
```

### Phase 2: Event Schema Changes

**No migration needed** - Events are immutable in event sourcing.

**Strategy:** Add `recipe_type` to new `RecipeCreated` events going forward. Old events without `recipe_type` will use a default value during replay.

**Backward Compatibility:**
- Old `RecipeCreated` events (before migration): Set `recipe_type = "main_course"` during aggregate reconstruction
- New `RecipeCreated` events (after migration): Include explicit `recipe_type` field

### Phase 3: Code Changes

#### 3.1 Recipe Domain

**Files to modify:**

1. `crates/recipe/src/events.rs`
```rust
// Add recipe_type to RecipeCreated event
pub struct RecipeCreated {
    pub user_id: String,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,
    pub recipe_type: String, // NEW: "appetizer", "main_course", "dessert"
    pub prep_time_min: Option<u32>,
    // ... rest of fields
}

// Add recipe_type to RecipeUpdated event
pub struct RecipeUpdated {
    pub title: Option<String>,
    pub recipe_type: Option<String>, // NEW: Allow updating recipe type
    pub ingredients: Option<Vec<Ingredient>>,
    // ... rest of fields
}
```

2. `crates/recipe/src/aggregate.rs`
```rust
pub struct RecipeAggregate {
    pub recipe_id: String,
    pub user_id: String,
    pub title: String,
    pub recipe_type: String, // NEW: "appetizer", "main_course", "dessert"
    pub ingredients: Vec<Ingredient>,
    // ... rest of fields
}

// Update event handlers
async fn recipe_created(&mut self, event: EventDetails<RecipeCreated>) -> anyhow::Result<()> {
    self.recipe_id = event.aggregator_id.clone();
    self.user_id = event.data.user_id;
    self.title = event.data.title;
    self.recipe_type = event.data.recipe_type; // NEW
    // ... rest of assignments
}

async fn recipe_updated(&mut self, event: EventDetails<RecipeUpdated>) -> anyhow::Result<()> {
    if let Some(title) = event.data.title {
        self.title = title;
    }
    if let Some(recipe_type) = event.data.recipe_type { // NEW
        self.recipe_type = recipe_type;
    }
    // ... rest of updates
}
```

3. `crates/recipe/src/commands.rs`
```rust
// Add validation for recipe_type
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(min = 3, max = 200))]
    pub title: String,

    #[validate(custom = "validate_recipe_type")]
    pub recipe_type: String, // NEW: Must be "appetizer", "main_course", or "dessert"

    #[validate(length(min = 1))]
    pub ingredients: Vec<Ingredient>,
    // ... rest of fields
}

// Validation function
fn validate_recipe_type(recipe_type: &str) -> Result<(), validator::ValidationError> {
    match recipe_type {
        "appetizer" | "main_course" | "dessert" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_recipe_type")),
    }
}

// Update create_recipe command
pub async fn create_recipe(
    command: CreateRecipeCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<String> {
    command.validate()?;

    // ... existing validation

    let aggregator_id = evento::create::<RecipeAggregate>()
        .data(&RecipeCreated {
            user_id: user_id.to_string(),
            title: command.title,
            recipe_type: command.recipe_type, // NEW
            ingredients: command.ingredients,
            // ... rest of fields
        })
        // ... rest of evento calls
}
```

4. `crates/recipe/src/read_model.rs`
```rust
// Update projection handler to include recipe_type
#[evento::handler(RecipeAggregate)]
async fn project_recipe_created<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO recipes (
            id, user_id, title, recipe_type, ingredients, instructions,
            prep_time_min, cook_time_min, advance_prep_hours, serving_size,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        event.aggregator_id,
        event.data.user_id,
        event.data.title,
        event.data.recipe_type, // NEW
        serde_json::to_string(&event.data.ingredients)?,
        // ... rest of fields
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

#### 3.2 Meal Planning Domain

**Files to modify:**

1. `crates/meal_planning/src/events.rs`
```rust
// Replace MealType enum with CourseType
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CourseType {
    Appetizer,
    MainCourse,
    Dessert,
}

impl CourseType {
    pub fn as_str(&self) -> &str {
        match self {
            CourseType::Appetizer => "appetizer",
            CourseType::MainCourse => "main_course",
            CourseType::Dessert => "dessert",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "appetizer" => Ok(CourseType::Appetizer),
            "main_course" => Ok(CourseType::MainCourse),
            "dessert" => Ok(CourseType::Dessert),
            // Backward compatibility for old data
            "breakfast" => Ok(CourseType::Appetizer),
            "lunch" => Ok(CourseType::MainCourse),
            "dinner" => Ok(CourseType::Dessert),
            _ => Err(format!("Invalid course type: {}", s)),
        }
    }
}

// Update MealAssignment
pub struct MealAssignment {
    pub date: String,
    pub course_type: String, // CHANGED from meal_type
    pub recipe_id: String,
    pub prep_required: bool,
    pub assignment_reasoning: Option<String>,
}
```

2. `crates/meal_planning/src/algorithm.rs`
```rust
// Update algorithm to assign by course type
pub async fn generate_meal_plan(
    user_id: &str,
    start_date: NaiveDate,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> Result<Vec<MealAssignment>, MealPlanError> {
    // Load user's favorite recipes
    let favorites = load_user_favorites(user_id, pool).await?;

    // Group recipes by course type
    let appetizers: Vec<_> = favorites.iter()
        .filter(|r| r.recipe_type == "appetizer")
        .collect();
    let main_courses: Vec<_> = favorites.iter()
        .filter(|r| r.recipe_type == "main_course")
        .collect();
    let desserts: Vec<_> = favorites.iter()
        .filter(|r| r.recipe_type == "dessert")
        .collect();

    let mut assignments = Vec::new();

    // Generate 7 days of meals (each with 3 courses)
    for day_offset in 0..7 {
        let date = start_date + Duration::days(day_offset);
        let date_str = date.format("%Y-%m-%d").to_string();

        // Assign appetizer
        let appetizer = select_recipe_for_course(&appetizers, &rotation_state, CourseType::Appetizer)?;
        assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: CourseType::Appetizer.as_str().to_string(),
            recipe_id: appetizer.id.clone(),
            prep_required: appetizer.advance_prep_hours.is_some(),
            assignment_reasoning: Some(format!("Selected {} for appetizer", appetizer.title)),
        });

        // Assign main course
        let main = select_recipe_for_course(&main_courses, &rotation_state, CourseType::MainCourse)?;
        assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: CourseType::MainCourse.as_str().to_string(),
            recipe_id: main.id.clone(),
            prep_required: main.advance_prep_hours.is_some(),
            assignment_reasoning: Some(format!("Selected {} for main course", main.title)),
        });

        // Assign dessert
        let dessert = select_recipe_for_course(&desserts, &rotation_state, CourseType::Dessert)?;
        assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: CourseType::Dessert.as_str().to_string(),
            recipe_id: dessert.id.clone(),
            prep_required: dessert.advance_prep_hours.is_some(),
            assignment_reasoning: Some(format!("Selected {} for dessert", dessert.title)),
        });
    }

    Ok(assignments)
}
```

3. `crates/meal_planning/src/commands.rs`
```rust
// Update ReplaceMealCommand
pub struct ReplaceMealCommand {
    pub meal_plan_id: String,
    pub date: String,
    pub course_type: String, // CHANGED from meal_type
}

// Update validation
impl ReplaceMealCommand {
    pub fn validate(&self) -> Result<(), MealPlanError> {
        // Validate course_type
        CourseType::parse(&self.course_type)
            .map_err(|e| MealPlanError::InvalidCourseType(e))?;

        Ok(())
    }
}
```

#### 3.3 Routes & Handlers

**Files to modify:**

1. `src/routes/meal_plan.rs`
```rust
// Update replace_meal handler
async fn replace_meal_handler(
    State(state): State<AppState>,
    auth: Auth,
    Path(meal_plan_id): Path<String>,
    Form(form): Form<ReplaceMealForm>,
) -> Result<impl IntoResponse, AppError> {
    let command = ReplaceMealCommand {
        meal_plan_id: meal_plan_id.clone(),
        date: form.date,
        course_type: form.course_type, // CHANGED from meal_type
    };

    // ... rest of handler
}

// Update form struct
#[derive(Deserialize)]
struct ReplaceMealForm {
    date: String,
    course_type: String, // CHANGED from meal_type
    new_recipe_id: String,
}
```

2. `src/routes/dashboard.rs`
```rust
// Update dashboard query to use course_type
let meals = sqlx::query!(
    "SELECT date, course_type, recipe_id, prep_required
     FROM meal_assignments
     WHERE meal_plan_id = ?
     ORDER BY date,
       CASE course_type
         WHEN 'appetizer' THEN 1
         WHEN 'main_course' THEN 2
         WHEN 'dessert' THEN 3
       END",
    meal_plan_id
)
.fetch_all(&state.pool)
.await?;
```

#### 3.4 Templates

**Files to modify:**

1. `templates/pages/meal-calendar.html`
```html
<!-- Update meal slot display -->
<div class="day-column">
  <h3>{{ date }}</h3>

  <!-- Appetizer slot -->
  <div class="course-slot appetizer">
    <span class="course-label">Appetizer</span>
    <div class="recipe-card">{{ appetizer_recipe.title }}</div>
  </div>

  <!-- Main Course slot -->
  <div class="course-slot main-course">
    <span class="course-label">Main Course</span>
    <div class="recipe-card">{{ main_course_recipe.title }}</div>
  </div>

  <!-- Dessert slot -->
  <div class="course-slot dessert">
    <span class="course-label">Dessert</span>
    <div class="recipe-card">{{ dessert_recipe.title }}</div>
  </div>
</div>
```

2. `templates/pages/dashboard.html`
```html
<!-- Update today's meals display -->
<div class="todays-lunch">
  <h2>Today's Lunch</h2>

  <div class="course appetizer">
    <h3>Appetizer</h3>
    <div class="recipe-info">{{ appetizer.title }}</div>
  </div>

  <div class="course main-course">
    <h3>Main Course</h3>
    <div class="recipe-info">{{ main_course.title }}</div>
  </div>

  <div class="course dessert">
    <h3>Dessert</h3>
    <div class="recipe-info">{{ dessert.title }}</div>
  </div>
</div>
```

### Phase 4: Testing

#### 4.1 Update Existing Tests

**Files to modify:**
- `crates/recipe/tests/recipe_tests.rs` - Add recipe_type to test fixtures
- `crates/meal_planning/tests/*.rs` - Update to use CourseType enum
- `tests/meal_plan_integration_tests.rs` - Update assertions for course-based model

#### 4.2 New Test Cases

1. **Recipe Type Validation Tests**
```rust
#[tokio::test]
async fn test_create_recipe_requires_valid_type() {
    // Test that invalid recipe_type is rejected
    let cmd = CreateRecipeCommand {
        title: "Test Recipe".to_string(),
        recipe_type: "invalid_type".to_string(), // Should fail
        // ...
    };

    let result = create_recipe(cmd, "user123", &executor, &pool).await;
    assert!(matches!(result, Err(RecipeError::ValidationError(_))));
}

#[tokio::test]
async fn test_create_recipe_accepts_all_valid_types() {
    for recipe_type in ["appetizer", "main_course", "dessert"] {
        let cmd = CreateRecipeCommand {
            title: format!("Test {}", recipe_type),
            recipe_type: recipe_type.to_string(),
            // ...
        };

        let result = create_recipe(cmd, "user123", &executor, &pool).await;
        assert!(result.is_ok());
    }
}
```

2. **Meal Plan Algorithm Tests**
```rust
#[tokio::test]
async fn test_algorithm_assigns_by_course_type() {
    // Create test recipes of each type
    let appetizer_id = create_test_recipe("user123", "appetizer", &executor, &pool).await;
    let main_id = create_test_recipe("user123", "main_course", &executor, &pool).await;
    let dessert_id = create_test_recipe("user123", "dessert", &executor, &pool).await;

    // Generate meal plan
    let assignments = generate_meal_plan("user123", today, &executor, &pool).await.unwrap();

    // Verify assignments match course types
    for assignment in assignments {
        let recipe = load_recipe(&assignment.recipe_id, &pool).await;
        assert_eq!(recipe.recipe_type, assignment.course_type);
    }
}
```

### Phase 5: Deployment

#### Deployment Steps

1. **Pre-deployment validation**
   ```bash
   # Run full test suite
   cargo test --all-features

   # Run E2E tests
   cd e2e && npx playwright test
   ```

2. **Database backup**
   ```bash
   # Backup SQLite database before migration
   cp /data/imkitchen.db /data/imkitchen.db.backup-$(date +%Y%m%d)
   ```

3. **Deploy migration**
   ```bash
   # Run migration
   imkitchen migrate
   ```

4. **Verify migration**
   ```bash
   # Check recipe_type column exists and has values
   sqlite3 /data/imkitchen.db "SELECT recipe_type, COUNT(*) FROM recipes GROUP BY recipe_type;"

   # Check course_type column exists
   sqlite3 /data/imkitchen.db "SELECT course_type, COUNT(*) FROM meal_assignments GROUP BY course_type;"
   ```

5. **Deploy application**
   ```bash
   # Build and deploy new version
   docker build -t imkitchen:course-model .
   kubectl set image deployment/imkitchen imkitchen=imkitchen:course-model
   ```

6. **Post-deployment validation**
   - Create new recipe with explicit recipe_type
   - Generate meal plan
   - Verify course assignments are correct

### Rollback Plan

If issues arise:

1. **Revert application**
   ```bash
   kubectl rollout undo deployment/imkitchen
   ```

2. **Restore database** (if needed)
   ```bash
   cp /data/imkitchen.db.backup-YYYYMMDD /data/imkitchen.db
   ```

3. **Manual data fix** (if partial migration)
   ```sql
   -- Rename course_type back to meal_type
   ALTER TABLE meal_assignments RENAME COLUMN course_type TO meal_type;

   -- Revert values
   UPDATE meal_assignments SET meal_type = 'breakfast' WHERE meal_type = 'appetizer';
   UPDATE meal_assignments SET meal_type = 'lunch' WHERE meal_type = 'main_course';
   UPDATE meal_assignments SET meal_type = 'dinner' WHERE meal_type = 'dessert';
   ```

## Risk Assessment

### High Risk
- **Data loss during table recreation** - Mitigated by database backup
- **Event replay issues with old events** - Mitigated by default value handling

### Medium Risk
- **Algorithm fails to find recipes** - Mitigated by validation before deployment
- **User confusion with new model** - Mitigated by documentation/UI updates

### Low Risk
- **Performance degradation** - New index on recipe_type should improve performance

## Timeline Estimate

- **Phase 1 (DB Migration):** 1 hour
- **Phase 2 (Events):** N/A (handled in code)
- **Phase 3 (Code Changes):** 8-12 hours
- **Phase 4 (Testing):** 4-6 hours
- **Phase 5 (Deployment):** 2 hours

**Total:** 15-21 hours (~2-3 days)

## Success Criteria

- ✅ All existing recipes have valid `recipe_type` values
- ✅ All existing meal assignments use `course_type` instead of `meal_type`
- ✅ New recipes require explicit `recipe_type` selection
- ✅ Meal plan algorithm assigns recipes based on course type matching
- ✅ All tests pass (unit, integration, E2E)
- ✅ UI displays courses correctly (appetizer, main course, dessert)
- ✅ No data loss or corruption

## Post-Migration Tasks

1. Update user documentation
2. Create announcement for existing users explaining new model
3. Monitor error logs for course type validation failures
4. Gather user feedback on new course-based experience
