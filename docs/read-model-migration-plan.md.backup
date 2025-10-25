# Read Model Migration Plan: Domain-Wide to Page-Specific

**Project:** imkitchen
**Date:** 2025-10-24
**Author:** Winston (Architect Agent)
**Status:** Planning Phase

---

## Executive Summary

This document outlines the migration strategy from domain-wide read models to page-specific read models in the imkitchen architecture. The new approach optimizes query performance, reduces coupling between pages, and aligns with Domain-Driven Design principles.

**Key Changes:**
- Replace shared `recipes`, `meal_plans`, `shopping_lists` tables with page-specific read models
- Each page queries only its dedicated tables
- Forms use `evento::load` for authoritative data (not read models)
- Business logic remains in domain crates (aggregates)
- Tests use `unsafe_oneshot` for deterministic projection testing

---

## Current State Analysis

### Existing Read Model Tables

```
Current Architecture (Domain-Wide Read Models):
├── users (Auth & Profile)
├── recipes (All recipe data - shared by multiple pages)
├── meal_plans (Meal plan state)
├── meal_assignments (Individual meal slots)
├── shopping_lists (Shopping list headers)
├── shopping_list_items (Shopping items)
├── ratings (Community ratings)
└── notifications (Prep reminders)
```

### Problems with Current Approach

1. **Over-fetching**: Recipe list page queries full `recipes` table but only needs title, complexity, times
2. **Tight Coupling**: Multiple pages depend on same `recipes` schema - changes impact all pages
3. **Performance**: Unnecessary data loaded (e.g., full instructions fetched for list cards)
4. **Unclear Boundaries**: Recipe detail and recipe list use same table, obscuring page-specific needs
5. **Difficult Testing**: Hard to test page-specific projections in isolation

---

## Target State Architecture

### Page-Specific Read Model Tables

```
New Architecture (Page-Specific Read Models):
├── users (Auth & Profile - unchanged)
│
├── Dashboard Page:
│   ├── dashboard_meals (today's B/L/D assignments with recipe metadata)
│   └── dashboard_prep_tasks (today's prep tasks)
│
├── Meal Calendar Page:
│   └── calendar_view (week view meal assignments, Mon-Sun)
│
├── Recipe Library Page:
│   ├── recipe_list (recipe cards: title, complexity, times, favorite)
│   └── recipe_filter_counts (facet counts for filters)
│
├── Recipe Detail Page:
│   ├── recipe_detail (full recipe: ingredients, instructions)
│   └── recipe_ratings (aggregated ratings and reviews)
│
├── Shopping Page:
│   └── shopping_list_view (categorized items with checkoff state)
│
└── Notifications:
    └── notifications_queue (prep reminders)
```

### Read Model Mapping

| Page/Feature | Current Table(s) | New Table(s) | Data Included |
|--------------|------------------|--------------|---------------|
| **Dashboard** | `meal_plans`, `meal_assignments`, `recipes` | `dashboard_meals`, `dashboard_prep_tasks` | Today's meals only, prep indicators |
| **Meal Calendar** | `meal_plans`, `meal_assignments`, `recipes` | `calendar_view` | Week view (Mon-Sun), recipe metadata |
| **Recipe Library** | `recipes` | `recipe_list`, `recipe_filter_counts` | Cards (title, complexity, times), Filter facets (counts) |
| **Recipe Detail** | `recipes`, `ratings` | `recipe_detail`, `recipe_ratings` | Full recipe, Aggregated ratings/reviews |
| **Recipe Edit Form** | N/A (uses `evento::load`) | N/A | Loads aggregate directly |
| **Shopping List** | `shopping_lists`, `shopping_list_items` | `shopping_list_view` | Categorized items, week selector |

---

## Migration Strategy

### Phase 1: Parallel Implementation (Weeks 1-2)

**Goal:** Build new page-specific read models alongside existing tables

#### Step 1.1: Create New Migration Files

```sql
-- migrations/101_create_dashboard_meals.sql
CREATE TABLE dashboard_meals (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  date TEXT NOT NULL,               -- ISO 8601 date
  meal_type TEXT NOT NULL,          -- "breakfast"|"lunch"|"dinner"
  recipe_id TEXT NOT NULL,
  recipe_title TEXT NOT NULL,
  recipe_image TEXT,
  complexity TEXT NOT NULL,         -- "simple"|"moderate"|"complex"
  prep_required BOOLEAN NOT NULL,
  prep_time_min INTEGER NOT NULL,
  cook_time_min INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE (user_id, date, meal_type)
);

CREATE INDEX idx_dashboard_meals_user_date ON dashboard_meals(user_id, date);

-- migrations/102_create_dashboard_prep_tasks.sql
CREATE TABLE dashboard_prep_tasks (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  date TEXT NOT NULL,               -- Date prep should be done
  recipe_id TEXT NOT NULL,
  recipe_title TEXT NOT NULL,
  prep_description TEXT NOT NULL,
  hours_before INTEGER NOT NULL,     -- How many hours before meal
  is_completed BOOLEAN DEFAULT FALSE,
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_dashboard_prep_tasks_user_date ON dashboard_prep_tasks(user_id, date, is_completed);

-- migrations/103_create_calendar_view.sql
CREATE TABLE calendar_view (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  week_start_date TEXT NOT NULL,    -- Monday in ISO 8601
  date TEXT NOT NULL,                -- Specific day
  meal_type TEXT NOT NULL,
  recipe_id TEXT NOT NULL,
  recipe_title TEXT NOT NULL,
  recipe_image TEXT,
  complexity TEXT NOT NULL,
  prep_required BOOLEAN NOT NULL,
  algorithm_reasoning TEXT,          -- Why assigned to this day
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE (user_id, date, meal_type)
);

CREATE INDEX idx_calendar_view_week ON calendar_view(user_id, week_start_date);

-- migrations/104_create_recipe_list.sql
CREATE TABLE recipe_list (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  recipe_image TEXT,
  complexity TEXT NOT NULL,
  prep_time_min INTEGER NOT NULL,
  cook_time_min INTEGER NOT NULL,
  recipe_type TEXT NOT NULL,         -- "appetizer"|"main_course"|"dessert"
  is_favorite BOOLEAN DEFAULT FALSE,
  is_shared BOOLEAN DEFAULT FALSE,
  advance_prep_hours INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_recipe_list_user ON recipe_list(user_id);
CREATE INDEX idx_recipe_list_favorite ON recipe_list(user_id, is_favorite);
CREATE INDEX idx_recipe_list_type ON recipe_list(recipe_type);

-- migrations/104b_create_recipe_filter_counts.sql
CREATE TABLE recipe_filter_counts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  filter_type TEXT NOT NULL,         -- "complexity"|"recipe_type"|"favorite"
  filter_value TEXT NOT NULL,        -- "simple"|"appetizer"|"true"
  count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE (user_id, filter_type, filter_value)
);

CREATE INDEX idx_recipe_filter_counts_user ON recipe_filter_counts(user_id);

-- Example data after recipes created:
-- user_id | filter_type  | filter_value | count
-- --------|--------------|--------------|------
-- user-1  | complexity   | simple       | 12
-- user-1  | complexity   | moderate     | 8
-- user-1  | complexity   | complex      | 3
-- user-1  | recipe_type  | appetizer    | 5
-- user-1  | recipe_type  | main_course  | 15
-- user-1  | recipe_type  | dessert      | 3
-- user-1  | favorite     | true         | 7

-- migrations/105_create_recipe_detail.sql
CREATE TABLE recipe_detail (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  recipe_image TEXT,
  ingredients TEXT NOT NULL,         -- JSON array
  instructions TEXT NOT NULL,        -- JSON array
  complexity TEXT NOT NULL,
  prep_time_min INTEGER NOT NULL,
  cook_time_min INTEGER NOT NULL,
  advance_prep_text TEXT,
  serving_size INTEGER NOT NULL,
  recipe_type TEXT NOT NULL,
  is_favorite BOOLEAN DEFAULT FALSE,
  is_shared BOOLEAN DEFAULT FALSE,
  cuisine TEXT,
  tags TEXT,                         -- JSON array
  avg_rating REAL,                   -- Aggregated from ratings table
  rating_count INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_recipe_detail_user ON recipe_detail(user_id);

-- migrations/106_create_shopping_list_view.sql
CREATE TABLE shopping_list_view (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  week_start_date TEXT NOT NULL,    -- Monday
  category TEXT NOT NULL,            -- "produce"|"dairy"|"meat"|"pantry"|"frozen"|"bakery"
  ingredient_name TEXT NOT NULL,
  quantity REAL NOT NULL,
  unit TEXT NOT NULL,
  is_collected BOOLEAN DEFAULT FALSE,
  from_recipe_ids TEXT NOT NULL,     -- JSON array of recipe IDs
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_shopping_list_view_week ON shopping_list_view(user_id, week_start_date);
CREATE INDEX idx_shopping_list_view_category ON shopping_list_view(user_id, week_start_date, category);
```

**Action Items:**
- [ ] Create migration files 101-106
- [ ] Run migrations in development environment
- [ ] Verify table creation with `sqlite3 imkitchen.db .schema`

#### Step 1.2: Implement New Projection Handlers

**File:** `crates/recipe/src/read_models/projections.rs`

```rust
use evento::EventDetails;
use sqlx::SqlitePool;

// Projection for Recipe List Page
#[evento::handler(Recipe)]
pub async fn project_recipe_to_list_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO recipe_list
         (id, user_id, title, recipe_image, complexity, prep_time_min, cook_time_min,
          recipe_type, is_favorite, is_shared, advance_prep_hours, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&event.aggregator_id)
    .bind(&event.metadata.user_id)
    .bind(&event.data.title)
    .bind(&event.data.image)
    .bind(&event.data.complexity)
    .bind(event.data.prep_time_min)
    .bind(event.data.cook_time_min)
    .bind(&event.data.recipe_type)
    .bind(false) // Default not favorite
    .bind(false) // Default not shared
    .bind(event.data.advance_prep_hours)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

// Projection for Recipe Detail Page
#[evento::handler(Recipe)]
pub async fn project_recipe_to_detail_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let ingredients_json = serde_json::to_string(&event.data.ingredients)?;
    let instructions_json = serde_json::to_string(&event.data.instructions)?;
    let tags_json = serde_json::to_string(&event.data.tags)?;

    sqlx::query!(
        "INSERT INTO recipe_detail
         (id, user_id, title, recipe_image, ingredients, instructions, complexity,
          prep_time_min, cook_time_min, advance_prep_text, serving_size, recipe_type,
          is_favorite, is_shared, cuisine, tags, avg_rating, rating_count, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        event.aggregator_id,
        event.metadata.user_id,
        event.data.title,
        event.data.image,
        ingredients_json,
        instructions_json,
        event.data.complexity,
        event.data.prep_time_min,
        event.data.cook_time_min,
        event.data.advance_prep_text,
        event.data.serving_size,
        event.data.recipe_type,
        false,
        false,
        event.data.cuisine,
        tags_json,
        None::<f64>,
        0,
        chrono::Utc::now().to_rfc3339(),
        chrono::Utc::now().to_rfc3339()
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

// Update favorite status (affects BOTH list and detail views)
#[evento::handler(Recipe)]
pub async fn update_recipe_favorite_status<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    // Update list view
    sqlx::query!(
        "UPDATE recipe_list SET is_favorite = ? WHERE id = ?",
        event.data.favorited,
        event.aggregator_id
    )
    .execute(context.executor.pool())
    .await?;

    // Update detail view
    sqlx::query!(
        "UPDATE recipe_detail SET is_favorite = ?, updated_at = ? WHERE id = ?",
        event.data.favorited,
        chrono::Utc::now().to_rfc3339(),
        event.aggregator_id
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

**File:** `crates/meal_planning/src/read_models/projections.rs`

```rust
// Dashboard Meals Projection
#[evento::handler(MealPlan)]
pub async fn project_dashboard_meals<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let today = chrono::Utc::now().date_naive();

    // Insert only TODAY'S meals into dashboard_meals
    for assignment in &event.data.assignments {
        if assignment.date == today {
            // Fetch recipe metadata (could be optimized with join)
            let recipe = sqlx::query!(
                "SELECT title, recipe_image, complexity, prep_time_min, cook_time_min
                 FROM recipe_detail WHERE id = ?",
                assignment.recipe_id
            )
            .fetch_one(context.executor.pool())
            .await?;

            sqlx::query!(
                "INSERT OR REPLACE INTO dashboard_meals
                 (id, user_id, date, meal_type, recipe_id, recipe_title, recipe_image,
                  complexity, prep_required, prep_time_min, cook_time_min, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                format!("{}-{}", assignment.date, assignment.meal_type),
                event.metadata.user_id,
                today.to_string(),
                assignment.meal_type,
                assignment.recipe_id,
                recipe.title,
                recipe.recipe_image,
                recipe.complexity,
                assignment.prep_required,
                recipe.prep_time_min,
                recipe.cook_time_min,
                chrono::Utc::now().to_rfc3339()
            )
            .execute(context.executor.pool())
            .await?;
        }
    }

    Ok(())
}

// Calendar View Projection
#[evento::handler(MealPlan)]
pub async fn project_calendar_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<MealPlanGenerated>,
) -> anyhow::Result<()> {
    let week_start = event.data.week_start_date; // Monday

    // Insert ALL week's meals into calendar_view
    for assignment in &event.data.assignments {
        let recipe = sqlx::query!(
            "SELECT title, recipe_image, complexity FROM recipe_detail WHERE id = ?",
            assignment.recipe_id
        )
        .fetch_one(context.executor.pool())
        .await?;

        sqlx::query!(
            "INSERT OR REPLACE INTO calendar_view
             (id, user_id, week_start_date, date, meal_type, recipe_id, recipe_title,
              recipe_image, complexity, prep_required, algorithm_reasoning, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            format!("{}-{}", assignment.date, assignment.meal_type),
            event.metadata.user_id,
            week_start.to_string(),
            assignment.date.to_string(),
            assignment.meal_type,
            assignment.recipe_id,
            recipe.title,
            recipe.recipe_image,
            recipe.complexity,
            assignment.prep_required,
            assignment.algorithm_reasoning,
            chrono::Utc::now().to_rfc3339()
        )
        .execute(context.executor.pool())
        .await?;
    }

    Ok(())
}
```

**Action Items:**
- [ ] Implement projection handlers in each domain crate
- [ ] Register subscriptions in `src/main.rs`
- [ ] Write unit tests for each projection handler using `unsafe_oneshot`

#### Step 1.3: Register Subscriptions

**File:** `src/main.rs` (subscription setup)

```rust
// Register page-specific projection subscriptions
async fn setup_projections(executor: &evento::Executor) -> anyhow::Result<()> {
    // Recipe List Page projections
    evento::subscribe("recipe-list-projections")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .handler(update_recipe_favorite_status)
        .run(executor)
        .await?;

    // Recipe Detail Page projections
    evento::subscribe("recipe-detail-projections")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_detail_view)
        .handler(update_recipe_favorite_status)
        .run(executor)
        .await?;

    // Dashboard Page projections
    evento::subscribe("dashboard-projections")
        .aggregator::<MealPlan>()
        .handler(project_dashboard_meals)
        .handler(project_dashboard_prep_tasks)
        .run(executor)
        .await?;

    // Calendar Page projections
    evento::subscribe("calendar-projections")
        .aggregator::<MealPlan>()
        .handler(project_calendar_view)
        .run(executor)
        .await?;

    // Shopping List Page projections
    evento::subscribe("shopping-projections")
        .aggregator::<ShoppingList>()
        .handler(project_shopping_list_view)
        .run(executor)
        .await?;

    Ok(())
}
```

**Action Items:**
- [ ] Add subscription setup to main.rs
- [ ] Verify subscriptions start on application boot
- [ ] Monitor logs for subscription errors

---

### Phase 2: Update Route Handlers (Week 3)

**Goal:** Switch handlers to query new page-specific read models

#### Step 2.1: Dashboard Route

**File:** `src/routes/dashboard.rs`

```rust
// BEFORE (queries multiple tables)
pub async fn dashboard_handler(
    auth: Auth,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    // Old: Join meal_plans, meal_assignments, recipes
    let meals = sqlx::query!(
        "SELECT ma.meal_type, r.title, r.image, r.complexity, ma.prep_required
         FROM meal_assignments ma
         JOIN recipes r ON ma.recipe_id = r.id
         JOIN meal_plans mp ON ma.meal_plan_id = mp.id
         WHERE mp.user_id = ? AND ma.date = ?",
        auth.user_id,
        chrono::Utc::now().date_naive().to_string()
    )
    .fetch_all(&pool)
    .await?;

    // ...
}

// AFTER (queries single page-specific table)
pub async fn dashboard_handler(
    auth: Auth,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    // New: Query dashboard_meals table directly
    let meals = sqlx::query_as!(
        DashboardMeal,
        "SELECT meal_type, recipe_id, recipe_title, recipe_image, complexity, prep_required
         FROM dashboard_meals
         WHERE user_id = ? AND date = ?",
        auth.user_id,
        chrono::Utc::now().date_naive().to_string()
    )
    .fetch_all(&pool)
    .await?;

    let prep_tasks = sqlx::query_as!(
        PrepTask,
        "SELECT recipe_title, prep_description, hours_before, is_completed
         FROM dashboard_prep_tasks
         WHERE user_id = ? AND date = ?",
        auth.user_id,
        chrono::Utc::now().date_naive().to_string()
    )
    .fetch_all(&pool)
    .await?;

    Ok(HtmlResponse(DashboardTemplate { meals, prep_tasks }))
}
```

#### Step 2.2: Recipe Library Route

**File:** `src/routes/recipes.rs`

```rust
// BEFORE
pub async fn recipe_list_handler(
    auth: Auth,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let recipes = sqlx::query!(
        "SELECT id, title, image, complexity, prep_time_min, cook_time_min, is_favorite
         FROM recipes
         WHERE user_id = ?",
        auth.user_id
    )
    .fetch_all(&pool)
    .await?;

    // ...
}

// AFTER
pub async fn recipe_list_handler(
    auth: Auth,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let recipes = sqlx::query_as!(
        RecipeListCard,
        "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
         FROM recipe_list
         WHERE user_id = ?
         ORDER BY created_at DESC",
        auth.user_id
    )
    .fetch_all(&pool)
    .await?;

    Ok(HtmlResponse(RecipeListTemplate { recipes }))
}
```

#### Step 2.3: Recipe Detail Route

**File:** `src/routes/recipes.rs`

```rust
// BEFORE
pub async fn recipe_detail_handler(
    auth: Auth,
    Path(recipe_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let recipe = sqlx::query!(
        "SELECT * FROM recipes WHERE id = ? AND user_id = ?",
        recipe_id,
        auth.user_id
    )
    .fetch_one(&pool)
    .await?;

    // ...
}

// AFTER
pub async fn recipe_detail_handler(
    auth: Auth,
    Path(recipe_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let recipe = sqlx::query_as!(
        RecipeDetail,
        "SELECT id, title, recipe_image, ingredients, instructions, complexity,
                prep_time_min, cook_time_min, advance_prep_text, serving_size,
                recipe_type, is_favorite, avg_rating, rating_count
         FROM recipe_detail
         WHERE id = ? AND user_id = ?",
        recipe_id,
        auth.user_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(HtmlResponse(RecipeDetailTemplate { recipe }))
}
```

#### Step 2.4: Edit Recipe Form (Use `evento::load`)

**File:** `src/routes/recipes.rs`

```rust
// CRITICAL: Edit forms use evento::load, NOT read models
pub async fn edit_recipe_form_handler(
    auth: Auth,
    Path(recipe_id): Path<String>,
    State(executor): State<evento::Executor>,
) -> Result<impl IntoResponse, AppError> {
    // Load aggregate for authoritative form data
    let recipe = evento::load::<Recipe>(&recipe_id)
        .run(&executor)
        .await?;

    // Verify ownership
    if recipe.user_id != auth.user_id {
        return Err(AppError::Forbidden);
    }

    Ok(HtmlResponse(EditRecipeTemplate { recipe }))
}
```

**Action Items:**
- [ ] Update all route handlers to use page-specific read models
- [ ] Update form handlers to use `evento::load` for pre-population
- [ ] Update integration tests to assert against new tables
- [ ] Verify no business logic in handlers (move to aggregates if found)

---

### Phase 3: Backfill Existing Data (Week 4)

**Goal:** Populate new read model tables from event stream

#### Step 3.1: Run Backfill Script

**File:** `scripts/backfill_read_models.rs`

```rust
use evento::Executor;
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = SqlitePool::connect("sqlite://imkitchen.db").await?;
    let executor = Executor::new(pool.clone());

    println!("Starting read model backfill...");

    // Replay all events through new projections
    evento::subscribe("backfill-recipe-list")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .handler(update_recipe_favorite_status)
        .unsafe_oneshot(&executor) // Process all historical events
        .await?;

    evento::subscribe("backfill-recipe-detail")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_detail_view)
        .handler(update_recipe_favorite_status)
        .unsafe_oneshot(&executor)
        .await?;

    evento::subscribe("backfill-dashboard")
        .aggregator::<MealPlan>()
        .handler(project_dashboard_meals)
        .handler(project_dashboard_prep_tasks)
        .unsafe_oneshot(&executor)
        .await?;

    evento::subscribe("backfill-calendar")
        .aggregator::<MealPlan>()
        .handler(project_calendar_view)
        .unsafe_oneshot(&executor)
        .await?;

    println!("Backfill complete!");
    Ok(())
}
```

**Command:**
```bash
cargo run --bin backfill_read_models
```

**Action Items:**
- [ ] Create backfill script
- [ ] Run backfill in staging environment
- [ ] Verify data consistency: Compare old vs new tables
- [ ] Run backfill in production (during maintenance window)

---

### Phase 4: Deprecate Old Tables (Week 5)

**Goal:** Remove old domain-wide read model tables

#### Step 4.1: Verify No References

```bash
# Search codebase for old table references
rg "FROM recipes" src/ crates/
rg "recipes r ON" src/ crates/
rg "meal_plans mp" src/ crates/
```

**Action Items:**
- [ ] Confirm no code references old `recipes`, `meal_plans`, `meal_assignments` tables
- [ ] Update all tests to use new page-specific tables
- [ ] Remove old projection handlers

#### Step 4.2: Drop Old Tables (Migration)

**File:** `migrations/107_drop_old_read_models.sql`

```sql
-- Backup old tables first (just in case)
ALTER TABLE recipes RENAME TO recipes_backup_20251024;
ALTER TABLE meal_plans RENAME TO meal_plans_backup_20251024;
ALTER TABLE meal_assignments RENAME TO meal_assignments_backup_20251024;
ALTER TABLE shopping_lists RENAME TO shopping_lists_backup_20251024;
ALTER TABLE shopping_list_items RENAME TO shopping_list_items_backup_20251024;

-- Note: Keep backups for 30 days, then drop manually if no issues
```

**Action Items:**
- [ ] Create drop migration
- [ ] Run in staging environment
- [ ] Monitor for 1 week for issues
- [ ] Run in production
- [ ] Schedule backup table deletion after 30 days

---

## Testing Strategy

### Unit Tests (Projection Handlers)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use evento::Executor;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_recipe_created_projects_to_list_view() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let executor = Executor::new(pool.clone());

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Create recipe
        let recipe_id = evento::create::<Recipe>()
            .data(&RecipeCreated {
                title: "Test Recipe".to_string(),
                complexity: "simple".to_string(),
                prep_time_min: 20,
                cook_time_min: 30,
                // ...
            })
            .metadata(&"user-123")
            .commit(&executor)
            .await
            .unwrap();

        // Process projection (synchronous for testing)
        evento::subscribe("test-list-projection")
            .aggregator::<Recipe>()
            .handler(project_recipe_to_list_view)
            .unsafe_oneshot(&executor)
            .await
            .unwrap();

        // Assert: recipe_list populated
        let result = sqlx::query!(
            "SELECT title, complexity FROM recipe_list WHERE id = ?",
            recipe_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(result.title, "Test Recipe");
        assert_eq!(result.complexity, "simple");
    }

    #[tokio::test]
    async fn test_recipe_favorited_updates_both_views() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let executor = Executor::new(pool.clone());

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // Create recipe
        let recipe_id = evento::create::<Recipe>()
            .data(&RecipeCreated { /* ... */ })
            .commit(&executor)
            .await
            .unwrap();

        // Process initial projections
        evento::subscribe("test-initial")
            .aggregator::<Recipe>()
            .handler(project_recipe_to_list_view)
            .handler(project_recipe_to_detail_view)
            .unsafe_oneshot(&executor)
            .await
            .unwrap();

        // Favorite recipe
        evento::update(&recipe_id)
            .data(&RecipeFavorited { favorited: true })
            .commit(&executor)
            .await
            .unwrap();

        // Process favorite projection
        evento::subscribe("test-favorite")
            .aggregator::<Recipe>()
            .handler(update_recipe_favorite_status)
            .unsafe_oneshot(&executor)
            .await
            .unwrap();

        // Assert: BOTH list and detail views updated
        let list_result = sqlx::query!("SELECT is_favorite FROM recipe_list WHERE id = ?", recipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(list_result.is_favorite, true);

        let detail_result = sqlx::query!("SELECT is_favorite FROM recipe_detail WHERE id = ?", recipe_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(detail_result.is_favorite, true);
    }
}
```

### Integration Tests (Route Handlers)

```rust
#[tokio::test]
async fn test_dashboard_route_uses_page_specific_read_model() {
    let app = test_app().await;
    let client = reqwest::Client::new();

    // Login
    let auth_cookie = login(&client, &app.url).await;

    // Create meal plan (triggers projections)
    create_meal_plan(&client, &app.url, &auth_cookie).await;

    // Wait for projections to process (in tests, use unsafe_oneshot to avoid this)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Request dashboard
    let resp = client.get(&format!("{}/dashboard", app.url))
        .header("Cookie", format!("auth_token={}", auth_cookie))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify response contains today's meals (from dashboard_meals table)
    let body = resp.text().await.unwrap();
    assert!(body.contains("Today's Meals"));
}
```

---

## Rollback Plan

If migration fails, rollback strategy:

### Phase 1-2 Rollback (Parallel Implementation)
- **Action:** Remove new table migrations, revert handler changes
- **Impact:** No data loss, old tables still in use
- **Risk:** Low

### Phase 3 Rollback (After Backfill)
- **Action:** Switch handlers back to old tables, drop new tables
- **Impact:** Minimal, old tables unchanged
- **Risk:** Low

### Phase 4 Rollback (After Old Table Deprecation)
- **Action:** Restore from `*_backup_20251024` tables, revert handlers
- **Impact:** Loss of events from past 30 days (requires event replay)
- **Risk:** Medium

**Mitigation:** Keep backup tables for 30 days before permanent deletion.

---

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| **Query Performance** | 50-100ms (joins) | <20ms (single table) | Dashboard load time |
| **Code Coupling** | High (shared tables) | Low (page-specific) | Impact analysis of schema changes |
| **Test Coverage** | 65% | 80% | `cargo tarpaulin` |
| **Projection Latency** | 200-500ms | <100ms | Event to read model update time |
| **Developer Velocity** | Baseline | +30% | Feature development time |

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| **Phase 1: Parallel Implementation** | Weeks 1-2 | New tables, projection handlers, subscriptions |
| **Phase 2: Update Handlers** | Week 3 | All routes use new read models |
| **Phase 3: Backfill Data** | Week 4 | Historical data in new tables |
| **Phase 4: Deprecate Old Tables** | Week 5 | Drop old tables, cleanup |

**Total Duration:** 5 weeks

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Data inconsistency during migration | Medium | High | Parallel implementation with verification |
| Performance degradation | Low | Medium | Load testing before production deployment |
| Business logic leak into handlers | Medium | High | Code review, enforce "thin handler" pattern |
| Incomplete backfill | Low | High | Dry run in staging, monitoring |
| Breaking changes to existing features | Medium | High | Comprehensive integration tests |

---

## Conclusion

This migration plan provides a structured approach to transitioning from domain-wide to page-specific read models. The phased approach minimizes risk while maximizing architectural benefits:

✅ **Reduced coupling** between pages
✅ **Optimized query performance** per page
✅ **Clear bounded contexts** aligned with DDD
✅ **Improved testability** with `unsafe_oneshot`
✅ **Maintainable codebase** with thin handlers

**Next Steps:**
1. Review and approve migration plan
2. Create tracking tickets for each phase
3. Begin Phase 1: Parallel Implementation

---

_Generated by Winston (Architect Agent) - 2025-10-24_
