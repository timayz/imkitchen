use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use meal_planning::{get_dashboard_metrics, get_todays_meals, DashboardMeal};
use notifications::read_model::get_user_prep_tasks_for_today;
use user::validate_jwt;

use crate::error::AppError;
use crate::routes::dashboard::{map_to_todays_meals, TodaysMealsData};
use crate::routes::AppState;

#[derive(Template)]
#[template(path = "pages/landing.html")]
struct LandingTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

#[derive(Template)]
#[template(path = "pages/dashboard.html")]
struct DashboardTemplate {
    pub user: Option<()>,
    pub todays_meals: Option<TodaysMealsData>,
    pub has_meal_plan: bool,
    pub recipe_count: usize,
    pub favorite_count: usize,
    pub prep_tasks: Vec<notifications::read_model::UserNotification>,
    pub current_path: String,
}

/// GET / - Landing page (public) or Dashboard (authenticated)
///
/// If user is authenticated, shows dashboard with today's meals.
/// If user is not authenticated, shows public landing page.
pub async fn get_landing(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user_id = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(claims.sub), // User is authenticated
                _ => None,                       // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    // If authenticated, render dashboard
    if let Some(user_id) = user_id {
        match render_dashboard(&state, &user_id).await {
            Ok(html) => html,
            Err(e) => {
                tracing::error!("Failed to render dashboard: {:?}", e);
                Html(format!("Error rendering dashboard: {}", e))
            }
        }
    } else {
        // If not authenticated, render landing page
        let template = LandingTemplate {
            user: None,
            current_path: "/".to_string(),
        };

        Html(template.render().unwrap_or_else(|e| {
            tracing::error!("Failed to render landing template: {}", e);
            format!("Error rendering template: {}", e)
        }))
    }
}

/// Render dashboard for authenticated user
async fn render_dashboard(state: &AppState, user_id: &str) -> Result<Html<String>, AppError> {
    // Query today's meals from page-specific table (dashboard_meals)
    // No JOINs needed - all recipe metadata is denormalized
    let todays_meal_assignments: Vec<DashboardMeal> = get_todays_meals(user_id, &state.db_pool).await?;

    // Query dashboard metrics from page-specific table (dashboard_metrics)
    let metrics = get_dashboard_metrics(user_id, &state.db_pool).await?;
    let (recipe_count, favorite_count) = metrics
        .map(|m| (m.recipe_count as usize, m.favorite_count as usize))
        .unwrap_or((0, 0));

    // Query today's prep tasks
    let prep_tasks = get_user_prep_tasks_for_today(&state.db_pool, user_id).await?;

    // Map assignments to TodaysMealsData structure
    let todays_meals = if todays_meal_assignments.is_empty() {
        // No meal plan - template will show CTA
        None
    } else {
        Some(map_to_todays_meals(&todays_meal_assignments))
    };

    let has_meal_plan = todays_meals.is_some();

    let template = DashboardTemplate {
        user: Some(()),
        todays_meals,
        has_meal_plan,
        recipe_count,
        favorite_count,
        prep_tasks,
        current_path: "/".to_string(),
    };

    template.render().map(Html).map_err(|e| {
        tracing::error!("Failed to render dashboard template: {:?}", e);
        AppError::InternalError("Failed to render page".to_string())
    })
}
