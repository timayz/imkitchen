use axum::{extract::Extension, http::StatusCode, response::Json};
use imkitchen_core::models::user::User;
use imkitchen_shared::auth_types::{CookingSkillLevel, ErrorResponse, UserPublic};
use tracing::info;

fn convert_user_to_public(user: User) -> UserPublic {
    let dietary_restrictions: Vec<String> =
        serde_json::from_str(&user.dietary_restrictions).unwrap_or_default();

    let cooking_skill_level = match user.cooking_skill_level.as_str() {
        "intermediate" => CookingSkillLevel::Intermediate,
        "advanced" => CookingSkillLevel::Advanced,
        _ => CookingSkillLevel::Beginner,
    };

    UserPublic {
        id: user.id,
        email: user.email,
        name: user.name,
        family_size: user.family_size,
        dietary_restrictions,
        cooking_skill_level,
        email_verified: user.email_verified,
        created_at: user.created_at.to_rfc3339(),
        last_active: user.last_active.to_rfc3339(),
    }
}

/// Get current user profile - requires authentication
pub async fn get_profile(
    Extension(user): Extension<User>,
) -> Result<Json<UserPublic>, (StatusCode, Json<ErrorResponse>)> {
    info!("Profile requested for user: {} ({})", user.name, user.email);

    let user_public = convert_user_to_public(user);
    Ok(Json(user_public))
}

/// Update user profile - requires authentication
pub async fn update_profile(
    Extension(user): Extension<User>,
    Json(_update_data): Json<serde_json::Value>,
) -> Result<Json<UserPublic>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Profile update requested for user: {} ({})",
        user.name, user.email
    );

    // TODO: Implement profile update logic
    // For now, just return current user profile

    let user_public = convert_user_to_public(user);
    Ok(Json(user_public))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use imkitchen_core::models::user::User;

    fn create_test_user() -> User {
        User {
            id: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            name: "Test User".to_string(),
            family_size: 2,
            dietary_restrictions: "[]".to_string(),
            cooking_skill_level: "beginner".to_string(),
            email_verified: true,
            email_verification_token: None,
            email_verification_expires_at: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_profile() {
        let user = create_test_user();
        let result = get_profile(Extension(user.clone())).await;

        assert!(result.is_ok());
        let user_public = result.unwrap().0;
        assert_eq!(user_public.email, user.email);
        assert_eq!(user_public.name, user.name);
    }

    #[tokio::test]
    async fn test_update_profile() {
        let user = create_test_user();
        let update_data = serde_json::json!({"name": "Updated Name"});
        let result = update_profile(Extension(user.clone()), Json(update_data)).await;

        assert!(result.is_ok());
        let user_public = result.unwrap().0;
        assert_eq!(user_public.email, user.email);
    }
}
