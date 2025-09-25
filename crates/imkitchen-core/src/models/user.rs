use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub family_size: i32,
    pub dietary_restrictions: String, // JSON string
    pub cooking_skill_level: String,
    pub email_verified: bool,
    #[serde(skip_serializing)]
    pub email_verification_token: Option<String>,
    #[serde(skip_serializing)]
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing)]
    pub password_reset_token: Option<String>,
    #[serde(skip_serializing)]
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublic {
    pub id: String,
    pub email: String,
    pub name: String,
    pub family_size: i32,
    pub dietary_restrictions: Vec<String>,
    pub cooking_skill_level: CookingSkillLevel,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CookingSkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

impl From<String> for CookingSkillLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "intermediate" => Self::Intermediate,
            "advanced" => Self::Advanced,
            _ => Self::Beginner,
        }
    }
}

impl From<CookingSkillLevel> for String {
    fn from(level: CookingSkillLevel) -> Self {
        match level {
            CookingSkillLevel::Beginner => "beginner".to_string(),
            CookingSkillLevel::Intermediate => "intermediate".to_string(),
            CookingSkillLevel::Advanced => "advanced".to_string(),
        }
    }
}

impl From<User> for UserPublic {
    fn from(user: User) -> Self {
        let dietary_restrictions: Vec<String> =
            serde_json::from_str(&user.dietary_restrictions).unwrap_or_default();

        let cooking_skill_level = user.cooking_skill_level.into();

        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            family_size: user.family_size,
            dietary_restrictions,
            cooking_skill_level,
            email_verified: user.email_verified,
            created_at: user.created_at,
            last_active: user.last_active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub session_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserSession {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub family_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserPublic,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooking_skill_level_conversion() {
        assert_eq!(
            CookingSkillLevel::from("beginner".to_string()),
            CookingSkillLevel::Beginner
        );
        assert_eq!(
            CookingSkillLevel::from("intermediate".to_string()),
            CookingSkillLevel::Intermediate
        );
        assert_eq!(
            CookingSkillLevel::from("advanced".to_string()),
            CookingSkillLevel::Advanced
        );
        assert_eq!(
            CookingSkillLevel::from("invalid".to_string()),
            CookingSkillLevel::Beginner
        );
    }

    #[test]
    fn test_session_expiration() {
        use chrono::Duration;

        let expired_session = UserSession {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            session_token: "token123".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now() - Duration::hours(2),
        };
        assert!(expired_session.is_expired());

        let valid_session = UserSession {
            id: "test".to_string(),
            user_id: "user123".to_string(),
            session_token: "token123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now() - Duration::hours(1),
        };
        assert!(!valid_session.is_expired());
    }
}
