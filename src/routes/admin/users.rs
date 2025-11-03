//! Admin user management route handlers

use super::AppState;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
// Timestamp formatting removed - using simple i64 timestamps
use imkitchen_user::command::{
    ActivateUserInput, Command, SuspendUserInput, TogglePremiumBypassInput,
};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

use crate::auth::jwt::AuthUser;
use crate::queries::user::{get_total_user_count, get_user, list_all_users};

/// Admin users page template
#[derive(Template)]
#[template(path = "pages/admin/users.html")]
struct AdminUsersTemplate {
    users: Vec<UserRowDisplay>,
    total_users: i64,
    premium_users: i64,
    suspended_users: i64,
    page: i32,
    total_pages: i64,
    has_more: bool,
}

/// User row partial template (for Twinspark updates)
#[derive(Template)]
#[template(path = "partials/admin/user-row.html")]
struct UserRowTemplate {
    user: UserRowDisplay,
}

/// User display model
#[derive(Debug, Clone)]
struct UserRowDisplay {
    id: String,
    email: String,
    is_admin: bool,
    is_suspended: bool,
    premium_bypass: bool,
    #[allow(dead_code)]
    created_at: i64,
    created_at_formatted: i64,
}

/// Query parameters for pagination
#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    page: i32,
}

fn default_page() -> i32 {
    1
}

/// GET /admin/users - List all users with pagination
pub async fn list_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<PaginationQuery>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        page = query.page,
        "Admin listing users"
    );

    let per_page = 20;

    // Get total user count
    let total_count = match get_total_user_count(&state.query_pool).await {
        Ok(count) => count,
        Err(e) => {
            error!(error = %e, "Failed to get total user count");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load user count",
            )
                .into_response();
        }
    };

    let total_pages = (total_count as f64 / per_page as f64).ceil() as i64;

    // Get users for current page
    let users_result = match list_all_users(&state.query_pool, query.page, per_page).await {
        Ok(users) => users,
        Err(e) => {
            error!(error = %e, "Failed to list users");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load users",
            )
                .into_response();
        }
    };

    // Convert to display models (premium_bypass now included in UserRow from JOIN)
    let user_displays: Vec<UserRowDisplay> = users_result
        .into_iter()
        .map(|user| UserRowDisplay {
            id: user.id,
            email: user.email,
            is_admin: user.is_admin,
            is_suspended: user.is_suspended,
            premium_bypass: user.premium_bypass,
            created_at: user.created_at,
            created_at_formatted: user.created_at,
        })
        .collect();

    // Calculate stats
    let suspended_users = user_displays.iter().filter(|u| u.is_suspended).count() as i64;
    let premium_users = user_displays.iter().filter(|u| u.premium_bypass).count() as i64;

    let template = AdminUsersTemplate {
        users: user_displays,
        total_users: total_count,
        premium_users,
        suspended_users,
        page: query.page,
        total_pages,
        has_more: query.page < total_pages as i32,
    };

    match template.render() {
        Ok(html) => axum::response::Html(html).into_response(),
        Err(e) => {
            error!(error = %e, "Failed to render template");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render page",
            )
                .into_response()
        }
    }
}

/// POST /admin/users/{id}/suspend - Suspend user account
pub async fn suspend_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<String>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        target_user_id = %user_id,
        "Suspending user account"
    );

    let command = Command::new(state.evento.clone());

    let metadata = EventMetadata {
        user_id: Some(auth_user.user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    let input = SuspendUserInput {
        user_id: user_id.clone(),
        reason: Some("Suspended by admin".to_string()),
    };

    // Get current user data before suspension
    let user = match get_user(&state.query_pool, &user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (axum::http::StatusCode::NOT_FOUND, "User not found").into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to get user");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get user",
            )
                .into_response();
        }
    };

    match command.suspend_user(input, metadata).await {
        Ok(_) => {
            // Return optimistic response - we know is_suspended is now true
            render_user_row_optimistic(
                user.id,
                user.email,
                user.is_admin,
                true, // is_suspended = true (optimistic)
                user.premium_bypass,
                user.created_at,
            )
        }
        Err(e) => {
            error!(error = %e, user_id = %user_id, "Failed to suspend user");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to suspend user",
            )
                .into_response()
        }
    }
}

/// POST /admin/users/{id}/activate - Activate user account
pub async fn activate_user(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<String>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        target_user_id = %user_id,
        "Activating user account"
    );

    let command = Command::new(state.evento.clone());

    let metadata = EventMetadata {
        user_id: Some(auth_user.user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    let input = ActivateUserInput {
        user_id: user_id.clone(),
    };

    // Get current user data before activation
    let user = match get_user(&state.query_pool, &user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (axum::http::StatusCode::NOT_FOUND, "User not found").into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to get user");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get user",
            )
                .into_response();
        }
    };

    match command.activate_user(input, metadata).await {
        Ok(_) => {
            // Return optimistic response - we know is_suspended is now false
            render_user_row_optimistic(
                user.id,
                user.email,
                user.is_admin,
                false, // is_suspended = false (optimistic)
                user.premium_bypass,
                user.created_at,
            )
        }
        Err(e) => {
            error!(error = %e, user_id = %user_id, "Failed to activate user");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to activate user",
            )
                .into_response()
        }
    }
}

/// POST /admin/users/{id}/premium-bypass - Toggle premium bypass flag
pub async fn toggle_premium_bypass(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(user_id): Path<String>,
) -> Response {
    info!(
        admin_user_id = %auth_user.user_id,
        target_user_id = %user_id,
        "Toggling premium bypass flag"
    );

    // Get current user data and bypass state
    let user = match get_user(&state.query_pool, &user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (axum::http::StatusCode::NOT_FOUND, "User not found").into_response();
        }
        Err(e) => {
            error!(error = %e, "Failed to get user");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get user",
            )
                .into_response();
        }
    };

    let command = Command::new(state.evento.clone());

    let metadata = EventMetadata {
        user_id: Some(auth_user.user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    let new_bypass = !user.premium_bypass; // Toggle the current value
    let input = TogglePremiumBypassInput {
        user_id: user_id.clone(),
        premium_bypass: new_bypass,
    };

    match command.toggle_premium_bypass(input, metadata).await {
        Ok(_) => {
            // Return optimistic response - we know premium_bypass is now toggled
            render_user_row_optimistic(
                user.id,
                user.email,
                user.is_admin,
                user.is_suspended,
                new_bypass, // premium_bypass = toggled value (optimistic)
                user.created_at,
            )
        }
        Err(e) => {
            error!(error = %e, user_id = %user_id, "Failed to toggle premium bypass");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to toggle premium bypass",
            )
                .into_response()
        }
    }
}

/// Helper function to render user row with optimistic values
fn render_user_row_optimistic(
    id: String,
    email: String,
    is_admin: bool,
    is_suspended: bool,
    premium_bypass: bool,
    created_at: i64,
) -> Response {
    let user_display = UserRowDisplay {
        id,
        email,
        is_admin,
        is_suspended,
        premium_bypass,
        created_at,
        created_at_formatted: created_at,
    };

    let template = UserRowTemplate { user: user_display };

    match template.render() {
        Ok(html) => {
            // Return with proper content type for Twinspark partial updates
            (
                [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to render user row template");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to render user row",
            )
                .into_response()
        }
    }
}
