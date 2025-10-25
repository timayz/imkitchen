/// Query functions for page-specific read models (Recipe Library and Recipe Detail pages)
///
/// These functions query the new page-specific tables (recipe_list, recipe_detail, recipe_filter_counts, recipe_ratings)
/// instead of the old domain-centric table (recipes).
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Recipe list card data (denormalized, no JOINs needed)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeListCard {
    pub id: String,
    pub title: String,
    pub recipe_type: String,
    pub image_url: Option<String>,
    pub complexity: Option<String>,
    pub cuisine: Option<String>,
    pub dietary_tags: Option<String>, // JSON array
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub is_favorite: i32,             // SQLite boolean
    pub is_shared: i32,               // SQLite boolean
    pub avg_rating: Option<f64>,
    pub rating_count: Option<i32>,
    pub created_at: String,
}

/// Recipe detail data (full recipe with all fields)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeDetailData {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub recipe_type: String,
    pub ingredients: String,       // JSON array
    pub instructions: String,      // JSON array
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub advance_prep_hours: Option<i32>,
    pub serving_size: Option<i32>,
    pub complexity: Option<String>,
    pub cuisine: Option<String>,
    pub dietary_tags: Option<String>, // JSON array
    pub is_favorite: i32,              // SQLite boolean
    pub is_shared: i32,                // SQLite boolean
    pub original_recipe_id: Option<String>,
    pub original_author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Recipe ratings data (aggregated)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecipeRatingsData {
    pub recipe_id: String,
    pub avg_stars: f64,
    pub rating_count: i32,
    pub five_star_count: i32,
    pub four_star_count: i32,
    pub three_star_count: i32,
    pub two_star_count: i32,
    pub one_star_count: i32,
    pub recent_reviews: String, // JSON array
}

/// Filter facet count data
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FilterCount {
    pub filter_type: String,
    pub filter_value: String,
    pub count: i32,
}

/// Query all recipes for user (Recipe Library page)
///
/// Returns recipe cards with denormalized rating data.
pub async fn get_recipe_list(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<RecipeListCard>, sqlx::Error> {
    sqlx::query_as::<_, RecipeListCard>(
        r#"
        SELECT id, title, recipe_type, image_url, complexity, cuisine, dietary_tags,
               prep_time_min, cook_time_min, is_favorite, is_shared,
               avg_rating, rating_count, created_at
        FROM recipe_list
        WHERE user_id = ?1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Query recipes with filters (Recipe Library page)
///
/// Supports filtering by complexity, cuisine, recipe_type, shared_status, and favorite status.
#[allow(clippy::too_many_arguments)]
/// Supports pagination with limit and offset.
pub async fn get_recipe_list_filtered(
    user_id: &str,
    complexity: Option<&str>,
    cuisine: Option<&str>,
    recipe_type: Option<&str>,
    shared_status: Option<&str>,
    favorites_only: bool,
    limit: Option<i32>,
    offset: Option<i32>,
    pool: &SqlitePool,
) -> Result<Vec<RecipeListCard>, sqlx::Error> {
    let mut query = String::from(
        r#"
        SELECT id, title, recipe_type, image_url, complexity, cuisine, dietary_tags,
               prep_time_min, cook_time_min, is_favorite, is_shared,
               avg_rating, rating_count, created_at
        FROM recipe_list
        WHERE user_id = ?1 AND deleted_at IS NULL
        "#,
    );

    let mut bind_idx = 2;

    if complexity.is_some() {
        query.push_str(&format!(" AND complexity = ?{}", bind_idx));
        bind_idx += 1;
    }

    if cuisine.is_some() {
        query.push_str(&format!(" AND cuisine = ?{}", bind_idx));
        bind_idx += 1;
    }

    if recipe_type.is_some() {
        query.push_str(&format!(" AND recipe_type = ?{}", bind_idx));
        bind_idx += 1;
    }

    if let Some(status) = shared_status {
        match status {
            "private" => query.push_str(" AND is_shared = 0"),
            "shared" => query.push_str(" AND is_shared = 1"),
            _ => {} // Ignore invalid values
        }
    }

    if favorites_only {
        query.push_str(" AND is_favorite = 1");
    }

    query.push_str(" ORDER BY created_at DESC");

    if limit.is_some() {
        query.push_str(&format!(" LIMIT ?{}", bind_idx));
        bind_idx += 1;
    }

    if offset.is_some() {
        query.push_str(&format!(" OFFSET ?{}", bind_idx));
    }

    let mut q = sqlx::query_as::<_, RecipeListCard>(&query).bind(user_id);

    if let Some(c) = complexity {
        q = q.bind(c);
    }

    if let Some(cu) = cuisine {
        q = q.bind(cu);
    }

    if let Some(rt) = recipe_type {
        q = q.bind(rt);
    }

    if let Some(lim) = limit {
        q = q.bind(lim);
    }

    if let Some(off) = offset {
        q = q.bind(off);
    }

    q.fetch_all(pool).await
}

/// Query filter counts for user (Recipe Library page)
///
/// Returns facet counts for filters (e.g., "Simple: 12", "Moderate: 8").
pub async fn get_filter_counts(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<FilterCount>, sqlx::Error> {
    sqlx::query_as::<_, FilterCount>(
        r#"
        SELECT filter_type, filter_value, count
        FROM recipe_filter_counts
        WHERE user_id = ?1 AND count > 0
        ORDER BY filter_type, filter_value
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Query recipe detail by ID (Recipe Detail page)
///
/// Returns full recipe data with all fields.
pub async fn get_recipe_detail(
    recipe_id: &str,
    user_id: &str,
    pool: &SqlitePool,
) -> Result<Option<RecipeDetailData>, sqlx::Error> {
    sqlx::query_as::<_, RecipeDetailData>(
        r#"
        SELECT id, user_id, title, recipe_type, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               complexity, cuisine, dietary_tags, is_favorite, is_shared,
               original_recipe_id, original_author, created_at, updated_at
        FROM recipe_detail
        WHERE id = ?1 AND user_id = ?2 AND deleted_at IS NULL
        "#,
    )
    .bind(recipe_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Query recipe ratings by recipe ID (Recipe Detail page)
///
/// Returns aggregated ratings and star distribution.
pub async fn get_recipe_ratings(
    recipe_id: &str,
    pool: &SqlitePool,
) -> Result<Option<RecipeRatingsData>, sqlx::Error> {
    sqlx::query_as::<_, RecipeRatingsData>(
        r#"
        SELECT recipe_id, avg_stars, rating_count,
               five_star_count, four_star_count, three_star_count,
               two_star_count, one_star_count, recent_reviews
        FROM recipe_ratings
        WHERE recipe_id = ?1
        "#,
    )
    .bind(recipe_id)
    .fetch_optional(pool)
    .await
}

/// Query shared recipes (Community Discovery page)
///
/// Returns recipes shared to community, ordered by rating.
pub async fn get_shared_recipes(
    pool: &SqlitePool,
    limit: i32,
) -> Result<Vec<RecipeListCard>, sqlx::Error> {
    sqlx::query_as::<_, RecipeListCard>(
        r#"
        SELECT id, title, recipe_type, image_url, complexity, cuisine, dietary_tags,
               prep_time_min, cook_time_min, is_favorite, is_shared,
               avg_rating, rating_count, created_at
        FROM recipe_list
        WHERE is_shared = 1 AND deleted_at IS NULL
        ORDER BY avg_rating DESC, rating_count DESC, created_at DESC
        LIMIT ?1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Query recipe count for user
///
/// Returns total and favorite recipe counts.
pub async fn get_recipe_counts(
    user_id: &str,
    pool: &SqlitePool,
) -> Result<(i32, i32), sqlx::Error> {
    let result: (i32, i32) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            SUM(CASE WHEN is_favorite = 1 THEN 1 ELSE 0 END) as favorites
        FROM recipe_list
        WHERE user_id = ?1 AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Query shared recipes with filters for discovery page
///
/// Returns full recipe detail data with all filters applied.
#[allow(clippy::too_many_arguments)]
/// Uses recipe_detail table (shared community recipes).
pub async fn get_shared_recipes_filtered(
    pool: &SqlitePool,
    cuisine: Option<&str>,
    min_rating: Option<f64>,
    max_prep_time: Option<i32>,
    dietary: Option<&str>,
    search: Option<&str>,
    sort: Option<&str>,
    page: Option<u32>,
) -> Result<Vec<RecipeDetailData>, sqlx::Error> {
    let mut query_str = String::from(
        r#"
        SELECT id, user_id, title, recipe_type, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               complexity, cuisine, dietary_tags, is_favorite, is_shared,
               original_recipe_id, original_author, created_at, updated_at
        FROM recipe_detail
        WHERE is_shared = 1 AND deleted_at IS NULL
        "#,
    );

    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    // Cuisine filter
    if let Some(c) = cuisine {
        conditions.push(format!("cuisine = ?{}", bind_values.len() + 1));
        bind_values.push(c.to_string());
    }

    // Prep time filter (total time)
    if let Some(max_prep) = max_prep_time {
        conditions.push(format!(
            "(COALESCE(prep_time_min, 0) + COALESCE(cook_time_min, 0)) <= ?{}",
            bind_values.len() + 1
        ));
        bind_values.push(max_prep.to_string());
    }

    // Dietary tags filter (JSON array contains check)
    if let Some(dietary_str) = dietary {
        for tag in dietary_str.split(',') {
            let tag = tag.trim();
            if !tag.is_empty() {
                conditions.push(format!(
                    "dietary_tags LIKE ?{}",
                    bind_values.len() + 1
                ));
                bind_values.push(format!("%\"{}%", tag));
            }
        }
    }

    // Search filter (title, cuisine)
    if let Some(search_str) = search {
        if !search_str.trim().is_empty() {
            conditions.push(format!(
                "(title LIKE ?{} OR cuisine LIKE ?{})",
                bind_values.len() + 1,
                bind_values.len() + 2
            ));
            let search_pattern = format!("%{}%", search_str.trim());
            bind_values.push(search_pattern.clone());
            bind_values.push(search_pattern);
        }
    }

    // Add conditions to query
    if !conditions.is_empty() {
        query_str.push_str(" AND ");
        query_str.push_str(&conditions.join(" AND "));
    }

    // Sorting - join with recipe_ratings for rating-based sorts
    let sort_clause = match sort.unwrap_or("rating") {
        "rating" => {
            // Need to LEFT JOIN recipe_ratings for sorting by rating
            query_str = format!(
                r#"
                SELECT rd.id, rd.user_id, rd.title, rd.recipe_type, rd.ingredients, rd.instructions,
                       rd.prep_time_min, rd.cook_time_min, rd.advance_prep_hours, rd.serving_size,
                       rd.complexity, rd.cuisine, rd.dietary_tags, rd.is_favorite, rd.is_shared,
                       rd.original_recipe_id, rd.original_author, rd.created_at, rd.updated_at
                FROM recipe_detail rd
                LEFT JOIN recipe_ratings rr ON rd.id = rr.recipe_id
                WHERE rd.is_shared = 1 AND rd.deleted_at IS NULL
                {}
                ORDER BY COALESCE(rr.avg_stars, 0) DESC, COALESCE(rr.rating_count, 0) DESC, rd.created_at DESC
                "#,
                if !conditions.is_empty() {
                    format!(" AND {}", conditions.join(" AND "))
                } else {
                    String::new()
                }
            );
            ""
        }
        "newest" => " ORDER BY created_at DESC",
        "oldest" => " ORDER BY created_at ASC",
        "prep_time" => " ORDER BY COALESCE(prep_time_min, 0) + COALESCE(cook_time_min, 0) ASC",
        _ => " ORDER BY created_at DESC",
    };

    if !sort_clause.is_empty() {
        query_str.push_str(sort_clause);
    }

    // Pagination
    let page_num = page.unwrap_or(1);
    let limit = 20;
    let offset = (page_num - 1) * limit;
    query_str.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    // Apply min_rating filter if specified (requires rating JOIN)
    if let Some(min_rating_val) = min_rating {
        if !query_str.contains("LEFT JOIN recipe_ratings") {
            // Need to rewrite query to include JOIN
            query_str = format!(
                r#"
                SELECT rd.id, rd.user_id, rd.title, rd.recipe_type, rd.ingredients, rd.instructions,
                       rd.prep_time_min, rd.cook_time_min, rd.advance_prep_hours, rd.serving_size,
                       rd.complexity, rd.cuisine, rd.dietary_tags, rd.is_favorite, rd.is_shared,
                       rd.original_recipe_id, rd.original_author, rd.created_at, rd.updated_at
                FROM recipe_detail rd
                LEFT JOIN recipe_ratings rr ON rd.id = rr.recipe_id
                WHERE rd.is_shared = 1 AND rd.deleted_at IS NULL AND COALESCE(rr.avg_stars, 0) >= ?{}
                {}
                ORDER BY created_at DESC
                LIMIT {} OFFSET {}
                "#,
                bind_values.len() + 1,
                if !conditions.is_empty() {
                    format!(" AND {}", conditions.join(" AND "))
                } else {
                    String::new()
                },
                limit,
                offset
            );
        }
        bind_values.push(min_rating_val.to_string());
    }

    // Build and execute query
    let mut query = sqlx::query_as::<_, RecipeDetailData>(&query_str);
    for value in bind_values {
        query = query.bind(value);
    }

    query.fetch_all(pool).await
}
