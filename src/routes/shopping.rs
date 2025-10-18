use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Serialize;
use shopping::{
    generate_shopping_list, read_model::get_shopping_list_by_week, GenerateShoppingListCommand,
};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// GET /shopping - Display shopping list for current week
pub async fn show_shopping_list(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    // Get current week's Monday (week_start_date)
    let today = Utc::now().date_naive();
    let week_start = get_week_start(today);
    let week_start_str = week_start.format("%Y-%m-%d").to_string();

    // Query shopping list for this week
    let shopping_list = get_shopping_list_by_week(user_id, &week_start_str, &state.db_pool).await?;

    if let Some(list) = shopping_list {
        // Group items by category for display
        let grouped = list.group_by_category();

        // Prepare category data for template
        let mut categories: Vec<CategoryGroup> = grouped
            .into_iter()
            .map(|(category_name, items)| {
                let items_data: Vec<ShoppingItem> = items
                    .iter()
                    .map(|item| ShoppingItem {
                        id: item.id.clone(),
                        ingredient_name: item.ingredient_name.clone(),
                        quantity: item.quantity,
                        unit: item.unit.clone(),
                        is_collected: item.is_collected,
                    })
                    .collect();

                CategoryGroup {
                    name: category_name,
                    item_count: items_data.len(),
                    items: items_data,
                }
            })
            .collect();

        // Sort categories in a logical grocery store order
        categories.sort_by_key(|cat| match cat.name.as_str() {
            "Produce" => 0,
            "Dairy" => 1,
            "Meat" => 2,
            "Bakery" => 3,
            "Pantry" => 4,
            "Frozen" => 5,
            "Other" => 6,
            _ => 7,
        });

        let template = ShoppingListTemplate {
            user: Some(()),
            week_start_date: week_start_str,
            categories,
            has_items: true,
        };

        Ok(Html(template.render().map_err(|e| {
            AppError::InternalError(format!("Template render error: {}", e))
        })?))
    } else {
        // No shopping list for this week
        let template = ShoppingListTemplate {
            user: Some(()),
            week_start_date: week_start_str,
            categories: vec![],
            has_items: false,
        };

        Ok(Html(template.render().map_err(|e| {
            AppError::InternalError(format!("Template render error: {}", e))
        })?))
    }
}

/// POST /shopping/generate - Generate shopping list for current week
pub async fn generate_shopping_list_handler(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    // Get current week's Monday
    let today = Utc::now().date_naive();
    let week_start = get_week_start(today);
    let week_start_str = week_start.format("%Y-%m-%d").to_string();

    // Query active meal plan for user
    let meal_plan =
        meal_planning::read_model::MealPlanQueries::get_active_meal_plan(user_id, &state.db_pool)
            .await?
            .ok_or_else(|| AppError::ValidationError("No active meal plan found".to_string()))?;

    // Query all meal assignments for the meal plan
    let assignments = meal_planning::read_model::MealPlanQueries::get_meal_assignments(
        &meal_plan.id,
        &state.db_pool,
    )
    .await?;

    if assignments.is_empty() {
        return Err(AppError::ValidationError(
            "Meal plan has no recipes assigned".to_string(),
        ));
    }

    // Query all recipes from meal assignments and extract ingredients
    let mut all_ingredients: Vec<(String, f32, String)> = Vec::new();

    for assignment in &assignments {
        // Query recipe to get ingredients
        let recipe: Option<recipe::read_model::RecipeReadModel> =
            sqlx::query_as::<_, recipe::read_model::RecipeReadModel>(
                r#"
                SELECT id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min,
                       advance_prep_hours, serving_size, is_favorite, is_shared, complexity, cuisine,
                       dietary_tags, created_at, updated_at
                FROM recipes
                WHERE id = ?
                "#,
            )
            .bind(&assignment.recipe_id)
            .fetch_optional(&state.db_pool)
            .await?;

        if let Some(recipe) = recipe {
            // Parse ingredients JSON
            let ingredients: Vec<recipe::events::Ingredient> =
                serde_json::from_str(&recipe.ingredients).map_err(|e| {
                    AppError::InternalError(format!(
                        "Failed to parse ingredients for recipe {}: {}",
                        recipe.id, e
                    ))
                })?;

            // Add ingredients to list
            for ing in ingredients {
                all_ingredients.push((ing.name, ing.quantity, ing.unit));
            }
        }
    }

    // Generate shopping list command
    let cmd = GenerateShoppingListCommand {
        user_id: user_id.to_string(),
        meal_plan_id: meal_plan.id.clone(),
        week_start_date: week_start_str.clone(),
        ingredients: all_ingredients,
    };

    // Execute command
    let _shopping_list_id = generate_shopping_list(cmd, &state.evento_executor)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to generate shopping list: {}", e)))?;

    // Wait for projection (simple delay since we're using unsafe_oneshot in tests, but run() in production)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Redirect back to shopping list page
    Ok(Redirect::to("/shopping"))
}

/// Helper function to get the Monday of the current week
fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}

/// Template data for shopping list page
#[derive(Template)]
#[template(path = "pages/shopping-list.html")]
pub struct ShoppingListTemplate {
    pub user: Option<()>,
    pub week_start_date: String,
    pub categories: Vec<CategoryGroup>,
    pub has_items: bool,
}

/// Category group for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct CategoryGroup {
    pub name: String,
    pub item_count: usize,
    pub items: Vec<ShoppingItem>,
}

/// Shopping item for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct ShoppingItem {
    pub id: String,
    pub ingredient_name: String,
    pub quantity: f32,
    pub unit: String,
    pub is_collected: bool,
}
