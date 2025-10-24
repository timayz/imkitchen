# SQLx Runtime Queries Guide (No Compile-Time Checking)

**Project:** imkitchen
**Date:** 2025-10-24
**Author:** Winston (Architect Agent)
**Status:** Architectural Standard

---

## Decision: Use Runtime SQLx Queries

**DO NOT use compile-time checked macros** (`sqlx::query!`, `sqlx::query_as!`).

**ALWAYS use runtime queries** (`sqlx::query`, `sqlx::query_as`).

---

## Rationale

### ❌ Compile-Time Checking Issues

```rust
// DON'T USE: Compile-time checked macro
sqlx::query!(
    "INSERT INTO users (id, email) VALUES (?, ?)",
    user_id,
    email
)
```

**Problems:**
- Requires database connection at compile time
- Requires running migrations before compilation
- Breaks CI/CD pipelines (needs database setup)
- Slower compile times (database verification)
- Migration changes require full recompile
- Harder to test with in-memory databases

### ✅ Runtime Queries Benefits

```rust
// DO USE: Runtime query with bind
sqlx::query(
    "INSERT INTO users (id, email) VALUES (?, ?)"
)
.bind(&user_id)
.bind(&email)
.execute(&pool)
.await?
```

**Benefits:**
- ✅ No database needed at compile time
- ✅ Fast compilation
- ✅ CI/CD friendly
- ✅ Easy to test with in-memory DBs
- ✅ Migration changes don't require recompile
- ✅ More flexible for dynamic queries

---

## Pattern Reference

### ✅ INSERT Query (Runtime)

```rust
#[evento::handler(Recipe)]
pub async fn project_recipe_to_list<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO recipe_list (id, user_id, title, complexity, prep_time_min, created_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&event.aggregator_id)
    .bind(&event.metadata.user_id)
    .bind(&event.data.title)
    .bind(&event.data.complexity)
    .bind(event.data.prep_time_min)
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

**Key Points:**
- Use `.bind()` for each `?` placeholder in order
- Pass references (`&`) for owned types when possible
- Pass values directly for `Copy` types (integers, booleans)

---

### ✅ UPDATE Query (Runtime)

```rust
#[evento::handler(Recipe)]
pub async fn update_recipe_favorite<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    sqlx::query("UPDATE recipe_list SET is_favorite = ? WHERE id = ?")
        .bind(event.data.favorited)
        .bind(&event.aggregator_id)
        .execute(context.executor.pool())
        .await?;

    Ok(())
}
```

---

### ✅ SELECT Query (Runtime)

**Option 1: Manual Mapping**

```rust
pub async fn get_recipe_list(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<RecipeCard>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
         FROM recipe_list
         WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let recipes = rows
        .into_iter()
        .map(|row| RecipeCard {
            id: row.get("id"),
            title: row.get("title"),
            recipe_image: row.get("recipe_image"),
            complexity: row.get("complexity"),
            prep_time_min: row.get("prep_time_min"),
            cook_time_min: row.get("cook_time_min"),
            is_favorite: row.get("is_favorite"),
        })
        .collect();

    Ok(recipes)
}
```

**Option 2: Using `query_as` with `FromRow` (Recommended)**

```rust
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct RecipeCard {
    pub id: String,
    pub title: String,
    pub recipe_image: Option<String>,
    pub complexity: String,
    pub prep_time_min: i32,
    pub cook_time_min: i32,
    pub is_favorite: bool,
}

pub async fn get_recipe_list(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<RecipeCard>, sqlx::Error> {
    sqlx::query_as::<_, RecipeCard>(
        "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
         FROM recipe_list
         WHERE user_id = ?"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
```

**Why `query_as` with `FromRow` is better:**
- ✅ Less boilerplate (no manual mapping)
- ✅ Type-safe (column names must match struct fields)
- ✅ Compile-time field name validation (but not database schema)
- ✅ Easier to maintain

---

### ✅ JSON Serialization Pattern

```rust
#[evento::handler(Recipe)]
pub async fn project_recipe_to_detail<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Serialize to JSON strings BEFORE binding
    let ingredients_json = serde_json::to_string(&event.data.ingredients)?;
    let instructions_json = serde_json::to_string(&event.data.instructions)?;

    sqlx::query(
        "INSERT INTO recipe_detail (id, user_id, title, ingredients, instructions, created_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&event.aggregator_id)
    .bind(&event.metadata.user_id)
    .bind(&event.data.title)
    .bind(ingredients_json)   // Bind JSON string
    .bind(instructions_json)  // Bind JSON string
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

---

### ✅ Upsert Pattern (INSERT ... ON CONFLICT)

```rust
#[evento::handler(Recipe)]
pub async fn update_filter_counts<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count)
         VALUES (?, ?, ?, 1)
         ON CONFLICT (user_id, filter_type, filter_value)
         DO UPDATE SET count = count + 1"
    )
    .bind(&event.metadata.user_id)
    .bind("complexity")
    .bind(&event.data.complexity)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

---

### ✅ DELETE Query

```rust
#[evento::handler(Recipe)]
pub async fn remove_recipe_from_list<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM recipe_list WHERE id = ?")
        .bind(&event.aggregator_id)
        .execute(context.executor.pool())
        .await?;

    Ok(())
}
```

---

### ✅ Transaction Pattern

```rust
pub async fn replace_meal_with_transaction(
    pool: &SqlitePool,
    meal_id: &str,
    new_recipe_id: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Delete old assignment
    sqlx::query("DELETE FROM calendar_view WHERE id = ?")
        .bind(meal_id)
        .execute(&mut *tx)
        .await?;

    // Insert new assignment
    sqlx::query(
        "INSERT INTO calendar_view (id, recipe_id, date, meal_type, created_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(meal_id)
    .bind(new_recipe_id)
    .bind(chrono::Utc::now().date_naive().to_string())
    .bind("dinner")
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
```

---

## Testing Pattern

### ✅ In-Memory Database Testing

```rust
#[tokio::test]
async fn test_recipe_projection() {
    // In-memory SQLite (no file, works without compile-time checks)
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Run migrations at test time
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let executor = evento::Executor::new(pool.clone());

    // Create recipe
    let recipe_id = evento::create::<Recipe>()
        .data(&RecipeCreated {
            title: "Test Recipe".to_string(),
            complexity: "simple".to_string(),
            prep_time_min: 20,
            cook_time_min: 30,
        })
        .metadata(&"user-123")
        .commit(&executor)
        .await
        .unwrap();

    // Process projection synchronously
    evento::subscribe("test-projection")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .unsafe_oneshot(&executor)
        .await
        .unwrap();

    // Assert using runtime query
    let result = sqlx::query_as::<_, RecipeCard>(
        "SELECT id, title, complexity FROM recipe_list WHERE id = ?"
    )
    .bind(&recipe_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(result.title, "Test Recipe");
    assert_eq!(result.complexity, "simple");
}
```

---

## Common Patterns Summary

| Operation | Pattern |
|-----------|---------|
| **Insert** | `sqlx::query("INSERT ...").bind(...).execute()` |
| **Update** | `sqlx::query("UPDATE ...").bind(...).execute()` |
| **Delete** | `sqlx::query("DELETE ...").bind(...).execute()` |
| **Select (manual)** | `sqlx::query("SELECT ...").bind(...).fetch_all()` |
| **Select (typed)** | `sqlx::query_as::<_, T>("SELECT ...").bind(...).fetch_all()` |
| **Upsert** | `sqlx::query("INSERT ... ON CONFLICT DO UPDATE ...").bind(...)` |
| **Transaction** | `pool.begin()` → queries → `tx.commit()` |

---

## Bind Types Reference

```rust
// Owned String
.bind(&event.data.title)              // &String

// String literal
.bind("literal_value")                // &str

// Integer (Copy type, no reference needed)
.bind(event.data.prep_time_min)       // i32

// Boolean (Copy type)
.bind(true)                            // bool
.bind(event.data.is_favorite)         // bool

// Option<String>
.bind(&event.data.recipe_image)       // &Option<String>

// JSON serialization
let json_str = serde_json::to_string(&data)?;
.bind(json_str)                        // String

// DateTime
.bind(chrono::Utc::now().to_rfc3339()) // String (ISO 8601)
```

---

## Migration from Compile-Time to Runtime

### Before (Compile-Time ❌)

```rust
sqlx::query!(
    "INSERT INTO users (id, email, created_at) VALUES (?, ?, ?)",
    user_id,
    email,
    created_at
)
.execute(&pool)
.await?;
```

### After (Runtime ✅)

```rust
sqlx::query(
    "INSERT INTO users (id, email, created_at) VALUES (?, ?, ?)"
)
.bind(&user_id)
.bind(&email)
.bind(created_at)
.execute(&pool)
.await?;
```

### Steps:
1. Remove `!` from `query!` → `query`
2. Add `.bind()` for each `?` placeholder
3. Pass `&references` for owned types, values for Copy types
4. Test with `cargo test` (no database connection required at compile time)

---

## Error Handling

**Runtime queries return generic `sqlx::Error`:**

```rust
use sqlx::Error as SqlxError;

match result {
    Err(SqlxError::RowNotFound) => {
        // Handle missing row
    }
    Err(SqlxError::Database(db_err)) => {
        // Handle database constraint violations, etc.
        eprintln!("Database error: {}", db_err.message());
    }
    Err(e) => {
        // Other errors (connection, etc.)
        return Err(e.into());
    }
    Ok(_) => {
        // Success
    }
}
```

---

## Additional Notes

### Type Inference

SQLx runtime queries use SQLite's type affinity, so mismatches may occur at runtime if types don't align:

```rust
// Schema: CREATE TABLE users (age INTEGER)
// This will fail at RUNTIME if 'age' column doesn't exist or has wrong type
sqlx::query("SELECT age FROM users")
    .fetch_one(&pool)
    .await?;
```

**Mitigation:**
- Use `query_as` with `FromRow` derive for type safety
- Write integration tests that exercise all queries
- Use meaningful struct field names matching column names

### Performance

Runtime queries have **no performance penalty** compared to compile-time queries. Both compile to the same prepared statement execution.

---

## Summary

✅ **DO:**
- Use `sqlx::query()` and `sqlx::query_as::<_, T>()`
- Use `.bind()` for parameters
- Use `FromRow` derive for type-safe queries
- Test with in-memory SQLite
- Write integration tests for all queries

❌ **DON'T:**
- Use `sqlx::query!()` or `sqlx::query_as!()`
- Require database connection at compile time
- Assume compile-time schema validation

**Result:** Faster builds, easier CI/CD, more flexible development workflow.

---

_Standard established by Winston (Architect Agent) - 2025-10-24_
