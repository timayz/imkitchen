use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
    Extension, Form,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use shopping::{
    generate_shopping_list, read_model::get_shopping_list_by_week, GenerateShoppingListCommand,
};

use crate::error::AppError;
use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// Query parameters for shopping list page
#[derive(Deserialize)]
pub struct ShoppingListQuery {
    /// Week start date in ISO 8601 format (YYYY-MM-DD, Monday)
    /// If not provided, defaults to current week
    week: Option<String>,
}

/// GET /shopping - Display shopping list for selected week (or current week by default)
///
/// Query parameters:
/// - ?week=YYYY-MM-DD (optional) - Week start date (must be Monday)
///
/// AC #1, #3, #8: Week selection via URL query param
pub async fn show_shopping_list(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Query(query): Query<ShoppingListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    // Get current week's Monday
    let today = Utc::now().date_naive();
    let current_week_monday = get_week_start(today);

    // Parse selected week from query param or default to current week (AC #3)
    let selected_week = if let Some(week_param) = query.week {
        // Validate and parse the week parameter (validates Monday, date range, etc.)
        shopping::validate_week_date(&week_param)?;
        week_param
    } else {
        // Default to current week
        current_week_monday.format("%Y-%m-%d").to_string()
    };

    // Generate week options for dropdown (current + 4 future weeks) - AC #2, #5
    let week_options = generate_week_options(current_week_monday);

    let week_start_str = selected_week.clone();

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
            week_start_date: week_start_str.clone(),
            selected_week: selected_week.clone(),
            week_options: week_options.clone(),
            categories,
            has_items: true,
        };

        Ok(Html(template.render().map_err(|e| {
            AppError::InternalError(format!("Template render error: {}", e))
        })?))
    } else {
        // No shopping list for this week (AC #4)
        let template = ShoppingListTemplate {
            user: Some(()),
            week_start_date: week_start_str.clone(),
            selected_week: selected_week.clone(),
            week_options: week_options.clone(),
            categories: vec![],
            has_items: false,
        };

        Ok(Html(template.render().map_err(|e| {
            AppError::InternalError(format!("Template render error: {}", e))
        })?))
    }
}

/// Generate week options for dropdown (current week + 4 future weeks)
///
/// AC #2: Options format: "This Week", "Next Week", "Week of {date}"
/// AC #4: Current week highlighted
/// AC #5: Up to 4 weeks ahead
fn generate_week_options(current_week_monday: NaiveDate) -> Vec<WeekOption> {
    let mut options = Vec::new();

    for weeks_ahead in 0..=4 {
        let week_monday = current_week_monday + Duration::weeks(weeks_ahead);
        let iso_date = week_monday.format("%Y-%m-%d").to_string();

        let label = if weeks_ahead == 0 {
            "This Week".to_string()
        } else if weeks_ahead == 1 {
            "Next Week".to_string()
        } else {
            // Format: "Week of Oct 21"
            format!("Week of {}", week_monday.format("%b %d"))
        };

        options.push(WeekOption {
            label,
            iso_date,
            is_current: weeks_ahead == 0,
        });
    }

    options
}

/// Form data for shopping list generation
#[derive(Deserialize)]
pub struct GenerateShoppingListForm {
    /// Week to generate shopping list for (optional, defaults to current week)
    week: Option<String>,
}

/// POST /shopping/generate - Generate shopping list for selected week
///
/// Accepts optional week parameter via form data. If not provided, generates for current week.
pub async fn generate_shopping_list_handler(
    Extension(auth): Extension<Auth>,
    State(state): State<AppState>,
    Form(form): Form<GenerateShoppingListForm>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = &auth.user_id;

    // Get selected week or default to current week
    let today = Utc::now().date_naive();
    let current_week_monday = get_week_start(today);

    let week_start_str = if let Some(week_param) = form.week {
        // Validate the week parameter
        shopping::validate_week_date(&week_param)?;
        week_param
    } else {
        // Default to current week
        current_week_monday.format("%Y-%m-%d").to_string()
    };

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

    // Redirect back to shopping list page for the selected week
    let redirect_url = format!("/shopping?week={}", week_start_str);
    Ok(Redirect::to(&redirect_url))
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
    pub selected_week: String,         // ISO 8601 date (YYYY-MM-DD)
    pub week_options: Vec<WeekOption>, // Dropdown options (current + 4 future weeks)
    pub categories: Vec<CategoryGroup>,
    pub has_items: bool,
}

/// Week option for dropdown selector
#[derive(Debug, Clone, Serialize)]
pub struct WeekOption {
    pub label: String,    // "This Week", "Next Week", "Week of Oct 21"
    pub iso_date: String, // "2025-10-21" (ISO 8601)
    pub is_current: bool, // True if this is the current week (for highlighting)
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
