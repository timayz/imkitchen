use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_extra::extract::CookieJar;
use user::validate_jwt;

use crate::routes::AppState;

#[derive(Template)]
#[template(path = "pages/landing.html")]
struct LandingTemplate {
    pub user: Option<()>, // Some(()) if authenticated, None if not
    pub current_path: String,
}

/// GET / - Landing page (public, but shows different content if authenticated)
pub async fn get_landing(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // Try to extract authentication from cookie (optional - no redirect on failure)
    let user = if let Some(cookie) = jar.get("auth_token") {
        // Validate JWT
        if let Ok(claims) = validate_jwt(cookie.value(), &state.jwt_secret) {
            // Verify user exists in read model
            let user_exists = sqlx::query("SELECT id FROM users WHERE id = ?1")
                .bind(&claims.sub)
                .fetch_optional(&state.db_pool)
                .await;

            match user_exists {
                Ok(Some(_)) => Some(()), // User is authenticated
                _ => None,               // User not found or error
            }
        } else {
            None // Invalid JWT
        }
    } else {
        None // No auth cookie
    };

    let template = LandingTemplate {
        user,
        current_path: "/".to_string(),
    };

    Html(template.render().unwrap_or_else(|e| {
        tracing::error!("Failed to render landing template: {}", e);
        format!("Error rendering template: {}", e)
    }))
}
