use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    #[serde(rename = "familySize")]
    pub family_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetConfirmRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserPublic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(rename = "familySize")]
    pub family_size: i32,
    #[serde(rename = "dietaryRestrictions")]
    pub dietary_restrictions: Vec<String>,
    #[serde(rename = "cookingSkillLevel")]
    pub cooking_skill_level: CookingSkillLevel,
    #[serde(rename = "cookingTimePreferences")]
    pub cooking_time_preferences: CookingTimePreferences,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String, // ISO 8601 string for JSON compatibility
    #[serde(rename = "lastActive")]
    pub last_active: String, // ISO 8601 string for JSON compatibility
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CookingSkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CookingTimePreferences {
    pub weekday_max_minutes: i32,
    pub weekend_max_minutes: i32,
}

impl Default for CookingTimePreferences {
    fn default() -> Self {
        Self {
            weekday_max_minutes: 30,
            weekend_max_minutes: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdateRequest {
    pub name: Option<String>,
    #[serde(rename = "familySize")]
    pub family_size: Option<i32>,
    #[serde(rename = "dietaryRestrictions")]
    pub dietary_restrictions: Option<Vec<String>>,
    #[serde(rename = "cookingSkillLevel")]
    pub cooking_skill_level: Option<CookingSkillLevel>,
    #[serde(rename = "cookingTimePreferences")]
    pub cooking_time_preferences: Option<CookingTimePreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<UserPublic>,
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            error,
            code: None,
        }
    }

    pub fn with_code(error: String, code: String) -> Self {
        Self {
            success: false,
            error,
            code: Some(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            family_size: Some(4),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"familySize\":4"));

        let deserialized: RegisterRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.family_size, Some(4));
    }

    #[test]
    fn test_user_public_field_renaming() {
        let user = UserPublic {
            id: "123".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            family_size: 4,
            dietary_restrictions: vec!["vegetarian".to_string()],
            cooking_skill_level: CookingSkillLevel::Intermediate,
            cooking_time_preferences: CookingTimePreferences::default(),
            email_verified: true,
            created_at: "2023-01-01T00:00:00Z".to_string(),
            last_active: "2023-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"familySize\""));
        assert!(json.contains("\"dietaryRestrictions\""));
        assert!(json.contains("\"cookingSkillLevel\""));
        assert!(json.contains("\"emailVerified\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"lastActive\""));
    }

    #[test]
    fn test_cooking_skill_level_serialization() {
        assert_eq!(
            serde_json::to_string(&CookingSkillLevel::Beginner).unwrap(),
            "\"beginner\""
        );
        assert_eq!(
            serde_json::to_string(&CookingSkillLevel::Intermediate).unwrap(),
            "\"intermediate\""
        );
        assert_eq!(
            serde_json::to_string(&CookingSkillLevel::Advanced).unwrap(),
            "\"advanced\""
        );
    }

    #[test]
    fn test_error_response() {
        let error = ErrorResponse::new("Test error".to_string());
        assert!(!error.success);
        assert_eq!(error.error, "Test error");
        assert!(error.code.is_none());

        let error_with_code = ErrorResponse::with_code(
            "Validation error".to_string(),
            "VALIDATION_FAILED".to_string(),
        );
        assert!(!error_with_code.success);
        assert_eq!(error_with_code.error, "Validation error");
        assert_eq!(error_with_code.code, Some("VALIDATION_FAILED".to_string()));
    }
}
