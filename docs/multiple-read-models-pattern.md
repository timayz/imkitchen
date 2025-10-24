# Multiple Read Models Per Page Pattern

**Project:** imkitchen
**Date:** 2025-10-24
**Author:** Winston (Architect Agent)

---

## Pattern Overview

A single page can (and often should) have **multiple read model tables**, each optimized for a different concern on that page.

---

## Concrete Example: Recipe Library Page

### The Page

**URL:** `/recipes`

**User needs to see:**
1. **Recipe cards** (content) - Title, image, complexity, cook time
2. **Filter sidebar** (metadata) - Filter options with counts ("Simple: 12", "Favorite: 7")

### Traditional Approach (Single Read Model)

**Problem:**
```rust
// Single table with complex aggregation query
sqlx::query!(
    "SELECT
       r.id, r.title, r.image, r.complexity, r.cook_time_min, r.is_favorite,
       (SELECT COUNT(*) FROM recipes WHERE user_id = ? AND complexity = 'simple') as simple_count,
       (SELECT COUNT(*) FROM recipes WHERE user_id = ? AND complexity = 'moderate') as moderate_count,
       (SELECT COUNT(*) FROM recipes WHERE user_id = ? AND is_favorite = true) as favorite_count
     FROM recipes r
     WHERE user_id = ?",
    user_id, user_id, user_id, user_id
)
```

**Issues:**
- ❌ Expensive subqueries repeated for every recipe row
- ❌ Over-fetching (filter counts duplicated across all rows)
- ❌ Slow performance (4 table scans: 1 main + 3 subqueries)
- ❌ Hard to maintain (complex SQL)

---

### Multiple Read Models Approach (Recommended)

**Solution:** Create TWO separate read model tables

#### Read Model 1: `recipe_list` (Content)

```sql
CREATE TABLE recipe_list (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  recipe_image TEXT,
  complexity TEXT NOT NULL,
  prep_time_min INTEGER NOT NULL,
  cook_time_min INTEGER NOT NULL,
  recipe_type TEXT NOT NULL,
  is_favorite BOOLEAN DEFAULT FALSE,
  created_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);
```

**Purpose:** Recipe cards for display

**Query:**
```rust
sqlx::query_as!(
    RecipeCard,
    "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
     FROM recipe_list
     WHERE user_id = ?",
    user_id
)
// Fast: Single table scan, no subqueries
```

#### Read Model 2: `recipe_filter_counts` (Filter Metadata)

```sql
CREATE TABLE recipe_filter_counts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  filter_type TEXT NOT NULL,         -- "complexity"|"recipe_type"|"favorite"
  filter_value TEXT NOT NULL,        -- "simple"|"moderate"|"complex"|"true"
  count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (user_id) REFERENCES users(id),
  UNIQUE (user_id, filter_type, filter_value)
);
```

**Purpose:** Filter facet counts

**Query:**
```rust
sqlx::query_as!(
    FilterCount,
    "SELECT filter_type, filter_value, count
     FROM recipe_filter_counts
     WHERE user_id = ?",
    user_id
)
// Fast: Tiny table (10 rows), simple query
```

**Example Data:**
```
user_id  | filter_type  | filter_value | count
---------|--------------|--------------|------
user-123 | complexity   | simple       | 12
user-123 | complexity   | moderate     | 8
user-123 | complexity   | complex      | 3
user-123 | recipe_type  | appetizer    | 5
user-123 | recipe_type  | main_course  | 15
user-123 | recipe_type  | dessert      | 3
user-123 | favorite     | true         | 7
```

---

## Projection Handlers (Event Sourcing)

### Event: `RecipeCreated`

**Updates BOTH read models:**

```rust
// Projection 1: Recipe List
#[evento::handler(Recipe)]
pub async fn project_recipe_to_list_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO recipe_list (id, user_id, title, recipe_image, complexity,
         prep_time_min, cook_time_min, recipe_type, is_favorite, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        event.aggregator_id,
        event.metadata.user_id,
        event.data.title,
        event.data.image,
        event.data.complexity,
        event.data.prep_time_min,
        event.data.cook_time_min,
        event.data.recipe_type,
        false,
        chrono::Utc::now().to_rfc3339()
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

// Projection 2: Filter Counts
#[evento::handler(Recipe)]
pub async fn update_recipe_filter_counts<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Increment complexity count
    sqlx::query!(
        "INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count)
         VALUES (?, 'complexity', ?, 1)
         ON CONFLICT (user_id, filter_type, filter_value)
         DO UPDATE SET count = count + 1",
        event.metadata.user_id,
        event.data.complexity
    )
    .execute(context.executor.pool())
    .await?;

    // Increment recipe_type count
    sqlx::query!(
        "INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count)
         VALUES (?, 'recipe_type', ?, 1)
         ON CONFLICT (user_id, filter_type, filter_value)
         DO UPDATE SET count = count + 1",
        event.metadata.user_id,
        event.data.recipe_type
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

### Event: `RecipeFavorited`

**Updates BOTH read models:**

```rust
// Projection 1: Update recipe_list favorite flag
#[evento::handler(Recipe)]
pub async fn update_recipe_favorite_status<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE recipe_list SET is_favorite = ? WHERE id = ?",
        event.data.favorited,
        event.aggregator_id
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

// Projection 2: Update filter counts
#[evento::handler(Recipe)]
pub async fn update_favorite_filter_count<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    let delta = if event.data.favorited { 1 } else { -1 };

    sqlx::query!(
        "INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count)
         VALUES (?, 'favorite', 'true', ?)
         ON CONFLICT (user_id, filter_type, filter_value)
         DO UPDATE SET count = count + ?",
        event.metadata.user_id,
        delta,
        delta
    )
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

### Event: `RecipeDeleted`

**Updates BOTH read models:**

```rust
// Projection 1: Remove from recipe_list
#[evento::handler(Recipe)]
pub async fn remove_recipe_from_list<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    // First, fetch recipe to know which counts to decrement
    let recipe = sqlx::query!(
        "SELECT complexity, recipe_type, is_favorite FROM recipe_list WHERE id = ?",
        event.aggregator_id
    )
    .fetch_one(context.executor.pool())
    .await?;

    // Delete from list
    sqlx::query!("DELETE FROM recipe_list WHERE id = ?", event.aggregator_id)
        .execute(context.executor.pool())
        .await?;

    // Decrement counts
    sqlx::query!(
        "UPDATE recipe_filter_counts
         SET count = count - 1
         WHERE user_id = ? AND filter_type = 'complexity' AND filter_value = ?",
        event.metadata.user_id,
        recipe.complexity
    )
    .execute(context.executor.pool())
    .await?;

    sqlx::query!(
        "UPDATE recipe_filter_counts
         SET count = count - 1
         WHERE user_id = ? AND filter_type = 'recipe_type' AND filter_value = ?",
        event.metadata.user_id,
        recipe.recipe_type
    )
    .execute(context.executor.pool())
    .await?;

    if recipe.is_favorite {
        sqlx::query!(
            "UPDATE recipe_filter_counts
             SET count = count - 1
             WHERE user_id = ? AND filter_type = 'favorite' AND filter_value = 'true'",
            event.metadata.user_id
        )
        .execute(context.executor.pool())
        .await?;
    }

    Ok(())
}
```

---

## Subscription Setup

**Register MULTIPLE subscriptions for the same page:**

```rust
// main.rs
async fn setup_projections(executor: &evento::Executor) -> anyhow::Result<()> {
    // Recipe Library Page - Read Model 1: Content
    evento::subscribe("recipe-list-projections")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .handler(update_recipe_favorite_status)
        .handler(remove_recipe_from_list)
        .run(executor)
        .await?;

    // Recipe Library Page - Read Model 2: Filter Counts
    evento::subscribe("recipe-filter-counts-projections")
        .aggregator::<Recipe>()
        .handler(update_recipe_filter_counts)
        .handler(update_favorite_filter_count)
        .run(executor)
        .await?;

    Ok(())
}
```

---

## Route Handler

**Queries BOTH read models:**

```rust
// src/routes/recipes.rs
pub async fn recipe_library_handler(
    auth: Auth,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    // Query 1: Recipe cards (content)
    let recipes = sqlx::query_as!(
        RecipeCard,
        "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
         FROM recipe_list
         WHERE user_id = ?
         ORDER BY created_at DESC",
        auth.user_id
    )
    .fetch_all(&pool)
    .await?;

    // Query 2: Filter counts (metadata)
    let filter_counts = sqlx::query_as!(
        FilterCount,
        "SELECT filter_type, filter_value, count
         FROM recipe_filter_counts
         WHERE user_id = ?",
        auth.user_id
    )
    .fetch_all(&pool)
    .await?;

    // Pass BOTH to template
    Ok(HtmlResponse(RecipeLibraryTemplate {
        recipes,
        filter_counts,
    }))
}
```

---

## Askama Template

**Uses BOTH read models:**

```html
{% extends "base.html" %}

{% block content %}
<div class="recipe-library">
  <!-- Filters sidebar (from recipe_filter_counts) -->
  <aside class="filters">
    <h3>Complexity</h3>
    {% for filter in filter_counts %}
      {% if filter.filter_type == "complexity" %}
        <label>
          <input type="checkbox" name="complexity" value="{{ filter.filter_value }}">
          {{ filter.filter_value | capitalize }} ({{ filter.count }})
        </label>
      {% endif %}
    {% endfor %}

    <h3>Recipe Type</h3>
    {% for filter in filter_counts %}
      {% if filter.filter_type == "recipe_type" %}
        <label>
          <input type="checkbox" name="recipe_type" value="{{ filter.filter_value }}">
          {{ filter.filter_value | title }} ({{ filter.count }})
        </label>
      {% endif %}
    {% endfor %}

    <h3>Favorites</h3>
    {% for filter in filter_counts %}
      {% if filter.filter_type == "favorite" %}
        <label>
          <input type="checkbox" name="favorite" value="true">
          Favorites ({{ filter.count }})
        </label>
      {% endif %}
    {% endfor %}
  </aside>

  <!-- Recipe cards (from recipe_list) -->
  <main class="recipe-grid">
    {% for recipe in recipes %}
      <div class="recipe-card">
        <img src="{{ recipe.recipe_image }}" alt="{{ recipe.title }}">
        <h4>{{ recipe.title }}</h4>
        <span class="badge">{{ recipe.complexity }}</span>
        <span>{{ recipe.prep_time_min + recipe.cook_time_min }} min</span>
        {% if recipe.is_favorite %}
          <span class="favorite">❤️</span>
        {% endif %}
      </div>
    {% endfor %}
  </main>
</div>
{% endblock %}
```

---

## Performance Benefits

### Before (Single Read Model with Subqueries)

**Query Time:** ~150ms
- 20 recipe rows returned
- 3 subqueries executed per row = 60 table scans
- Filter counts duplicated across all rows

### After (Multiple Read Models)

**Query 1 (recipe_list):** ~10ms
- Simple table scan, no joins/subqueries
- 20 rows, ~10KB

**Query 2 (recipe_filter_counts):** ~1ms
- Tiny table (7 rows), simple query
- ~350 bytes

**Total:** ~11ms (13x faster!)

---

## When to Use Multiple Read Models

### ✅ Use multiple read models when:

1. **Different concerns**: Content vs metadata vs statistics
2. **Different update frequencies**: Filter counts change often, content rarely
3. **Performance optimization**: Avoid complex joins/aggregations
4. **Independent scaling**: One table grows large, another stays small
5. **Query patterns differ**: Complex queries vs simple lookups

### ❌ Don't use multiple read models when:

1. **Data tightly coupled**: Always queried together, same update frequency
2. **No performance benefit**: Both tables small, queries simple
3. **Premature optimization**: Adds complexity without measurable gain
4. **Single concern**: All data serves same purpose

---

## Testing Pattern

**Test BOTH projections:**

```rust
#[tokio::test]
async fn test_recipe_created_updates_both_read_models() {
    let pool = setup_test_db().await;
    let executor = evento::Executor::new(pool.clone());

    // Create recipe
    let recipe_id = evento::create::<Recipe>()
        .data(&RecipeCreated {
            title: "Test Recipe".to_string(),
            complexity: "simple".to_string(),
            recipe_type: "main_course".to_string(),
            // ...
        })
        .metadata(&"user-123")
        .commit(&executor)
        .await?;

    // Process BOTH projections (synchronous for testing)
    evento::subscribe("test-list")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .unsafe_oneshot(&executor)
        .await?;

    evento::subscribe("test-counts")
        .aggregator::<Recipe>()
        .handler(update_recipe_filter_counts)
        .unsafe_oneshot(&executor)
        .await?;

    // Assert: recipe_list updated
    let list_entry = sqlx::query!("SELECT title FROM recipe_list WHERE id = ?", recipe_id)
        .fetch_one(&pool)
        .await?;
    assert_eq!(list_entry.title, "Test Recipe");

    // Assert: recipe_filter_counts updated
    let complexity_count = sqlx::query!(
        "SELECT count FROM recipe_filter_counts
         WHERE user_id = ? AND filter_type = 'complexity' AND filter_value = 'simple'",
        "user-123"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(complexity_count.count, 1);

    let type_count = sqlx::query!(
        "SELECT count FROM recipe_filter_counts
         WHERE user_id = ? AND filter_type = 'recipe_type' AND filter_value = 'main_course'",
        "user-123"
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(type_count.count, 1);
}
```

---

## Summary

**Key Takeaways:**

✅ **Pages can have multiple read models** (not just one)
✅ **Each read model serves a specific concern** (content, filters, metrics)
✅ **Events update multiple read models** (one event → many projections)
✅ **Performance benefits** (simple queries, no joins/subqueries)
✅ **Clear separation of concerns** (easier to maintain and test)

**Example Pattern:**
- `/recipes` page → `recipe_list` (content) + `recipe_filter_counts` (filters)
- `/dashboard` page → `dashboard_meals` (meals) + `dashboard_prep_tasks` (tasks) + `dashboard_metrics` (stats)
- `/shopping` page → `shopping_list_view` (items) + `shopping_list_summary` (progress)

---

_Pattern documented by Winston (Architect Agent) - 2025-10-24_
