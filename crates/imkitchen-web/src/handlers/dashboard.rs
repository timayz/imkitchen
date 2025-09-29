// Dashboard handlers for authenticated users

use axum::{
    extract::{Request, State},
    http::{StatusCode, header::COOKIE},
    response::{Html, IntoResponse, Response},
};
use askama::Template;
use tracing::{error, info, warn};
use uuid::Uuid;

use imkitchen_user::queries::UserAccountView;
use crate::AppState;

/// Askama template for user dashboard
#[derive(Template)]
#[template(path = "dashboard/user.html")]
pub struct UserDashboardTemplate {
    pub user: UserAccountView,
}

/// Display the user dashboard
/// Requires authentication - user must be logged in (enforced by auth middleware)
pub async fn user_dashboard(
    State(app_state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    info!("Rendering user dashboard");
    
    // Auth middleware ensures user is logged in, so we can safely extract user_id
    let user_id = if let Some(cookie_header) = request.headers().get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            let mut found_user_id = None;
            // Parse cookies and find session cookie
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(session_value) = cookie.strip_prefix("imkitchen_session=") {
                    match Uuid::parse_str(session_value) {
                        Ok(id) => {
                            found_user_id = Some(id);
                            break;
                        },
                        Err(_) => {
                            error!("Invalid session cookie format after auth middleware");
                            return Err(StatusCode::INTERNAL_SERVER_ERROR);
                        }
                    }
                }
            }
            if let Some(id) = found_user_id {
                id
            } else {
                error!("No session cookie found after auth middleware");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        } else {
            error!("Could not parse cookie header after auth middleware");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    } else {
        error!("No cookie header found after auth middleware");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    
    // Get user data from database if query handler is available
    let user = if let Some(ref _query_handler) = app_state.user_query_handler {
        // For now, we'll create a mock user with the actual user ID
        // In a real implementation, we'd query the database using the user_id
        create_mock_user_with_id(user_id)
    } else {
        warn!("No user query handler available, using mock data");
        create_mock_user_with_id(user_id)
    };
    
    let template = UserDashboardTemplate {
        user,
    };
    
    let html = template.to_string();
    Ok(Html(html).into_response())
}

/// Create a mock user with specific user ID for demonstration purposes
/// In a real application, this would be retrieved from the database
fn create_mock_user_with_id(user_id: Uuid) -> UserAccountView {
    use chrono::Utc;
    use imkitchen_shared::{Email, FamilySize, SkillLevel};
    use imkitchen_user::domain::UserProfile;
    
    let email = Email::new("demo@imkitchen.com".to_string())
        .unwrap_or_else(|_| Email::new("user@example.com".to_string()).unwrap());
    
    let profile = UserProfile {
        family_size: FamilySize::new(4).unwrap_or(FamilySize { value: 4 }),
        cooking_skill_level: SkillLevel::Intermediate,
        dietary_restrictions: vec![],
        weekday_cooking_minutes: 30,
        weekend_cooking_minutes: 60,
    };
    
    UserAccountView {
        user_id,
        email,
        profile,
        is_email_verified: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: Some(Utc::now()),
        login_count: 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mock_user_creation() {
        use uuid::Uuid;
        let user_id = Uuid::new_v4();
        let user = create_mock_user_with_id(user_id);
        assert_eq!(user.user_id, user_id);
        assert!(user.email.value.contains("@"));
        assert!(user.profile.family_size.value >= 1 && user.profile.family_size.value <= 8);
        assert!(user.is_email_verified);
    }
}