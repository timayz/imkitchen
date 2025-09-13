use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    #[serde(rename = "beginner")]
    Beginner,
    #[serde(rename = "intermediate")]
    Intermediate,
    #[serde(rename = "advanced")]
    Advanced,
}

impl std::fmt::Display for SkillLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillLevel::Beginner => write!(f, "beginner"),
            SkillLevel::Intermediate => write!(f, "intermediate"),
            SkillLevel::Advanced => write!(f, "advanced"),
        }
    }
}

impl std::str::FromStr for SkillLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "beginner" => Ok(SkillLevel::Beginner),
            "intermediate" => Ok(SkillLevel::Intermediate),
            "advanced" => Ok(SkillLevel::Advanced),
            _ => Err(format!("Invalid skill level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub dietary_preferences: Vec<String>,
    pub skill_level: String, // Stored as string in database for simplicity
    pub household_size: i32,
    pub kitchen_equipment: serde_json::Value, // JSONB in database
    pub email_verified: bool,
    pub verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub dietary_preferences: Option<Vec<String>>,
    pub skill_level: Option<SkillLevel>,
    pub household_size: Option<i32>,
    pub kitchen_equipment: Option<Vec<Equipment>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub dietary_preferences: Vec<String>,
    pub skill_level: String,
    pub household_size: i32,
    pub kitchen_equipment: serde_json::Value,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        UserProfile {
            id: user.id,
            email: user.email,
            dietary_preferences: user.dietary_preferences,
            skill_level: user.skill_level,
            household_size: user.household_size,
            kitchen_equipment: user.kitchen_equipment,
            email_verified: user.email_verified,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: Uuid, // Subject (user ID)
    pub email: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}