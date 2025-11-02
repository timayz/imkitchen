//! User query handlers and projections

use evento::{AggregatorName, Context, EventDetails, Executor};
use imkitchen_user::aggregate::User;
use imkitchen_user::event::{
    EventMetadata, UserLoggedIn, UserProfileUpdated, UserRegistered, UserRegistrationFailed,
    UserRegistrationSucceeded,
};
use sqlx::{Row, SqlitePool};
use tracing::{error, info};

/// User row from projection table
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub hashed_password: String,
    pub is_admin: bool,
    pub is_suspended: bool,
    pub created_at: i64,
    pub last_login_at: Option<i64>,
}

/// User registration status for polling
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserStatus {
    pub id: String,
    pub status: String,
    pub error: Option<String>,
}

/// User profile row from projection table
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub dietary_restrictions: Vec<String>,
    pub cuisine_variety_weight: f32,
    pub household_size: Option<i32>,
    pub is_premium_active: bool,
    pub premium_bypass: bool,
}

/// Handler for UserRegistrationSucceeded event
#[evento::handler(User)]
async fn on_user_registration_succeeded<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserRegistrationSucceeded, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        user_id = %event.aggregator_id,
        email = %event.data.email,
        "Processing UserRegistrationSucceeded event"
    );

    // Insert into users projection table
    sqlx::query(
        "INSERT INTO users (id, email, hashed_password, is_admin, is_suspended, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.email)
    .bind(&event.data.hashed_password)
    .bind(false) // is_admin defaults to false
    .bind(false) // is_suspended defaults to false
    .bind(event.timestamp)
    .execute(&pool)
    .await?;

    info!(
        user_id = %event.aggregator_id,
        "User projection created successfully"
    );

    Ok(())
}

/// Handler for UserRegistrationFailed event
#[evento::handler(User)]
async fn on_user_registration_failed<E: Executor>(
    _context: &Context<'_, E>,
    event: EventDetails<UserRegistrationFailed, EventMetadata>,
) -> anyhow::Result<()> {
    error!(
        user_id = %event.aggregator_id,
        error = %event.data.error,
        "User registration failed"
    );

    // No projection update needed - just log the failure
    // The polling endpoint will check aggregate status

    Ok(())
}

/// Handler for UserLoggedIn event
#[evento::handler(User)]
async fn on_user_logged_in<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserLoggedIn, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        user_id = %event.aggregator_id,
        "Processing UserLoggedIn event"
    );

    // Update last_login_at timestamp
    sqlx::query("UPDATE users SET last_login_at = ? WHERE id = ?")
        .bind(event.timestamp)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for UserProfileUpdated event
#[evento::handler(User)]
async fn on_user_profile_updated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserProfileUpdated, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        user_id = %event.aggregator_id,
        restrictions_count = event.data.dietary_restrictions.len(),
        "Processing UserProfileUpdated event"
    );

    // Serialize dietary_restrictions as JSON
    let dietary_restrictions_json = serde_json::to_string(&event.data.dietary_restrictions)?;

    // Insert or update user_profiles table (upsert)
    sqlx::query(
        "INSERT INTO user_profiles (user_id, dietary_restrictions, cuisine_variety_weight, household_size)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
            dietary_restrictions = excluded.dietary_restrictions,
            cuisine_variety_weight = excluded.cuisine_variety_weight,
            household_size = excluded.household_size",
    )
    .bind(&event.aggregator_id)
    .bind(dietary_restrictions_json)
    .bind(event.data.cuisine_variety_weight)
    .bind(event.data.household_size)
    .execute(&pool)
    .await?;

    info!(
        user_id = %event.aggregator_id,
        "User profile projection updated successfully"
    );

    Ok(())
}

/// Create subscription builder for user query handlers
pub fn subscribe_user_query<E: Executor + Clone>(pool: SqlitePool) -> evento::SubscribeBuilder<E> {
    evento::subscribe::<E>("user-query")
        .data(pool)
        .handler(on_user_registration_succeeded())
        .handler(on_user_registration_failed())
        .handler(on_user_logged_in())
        .handler(on_user_profile_updated())
        .skip::<User, UserRegistered>()
}

/// Get user by ID
pub async fn get_user(pool: &SqlitePool, user_id: &str) -> anyhow::Result<Option<UserRow>> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, hashed_password, is_admin, is_suspended, created_at, last_login_at
         FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Get user by email
pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> anyhow::Result<Option<UserRow>> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, hashed_password, is_admin, is_suspended, created_at, last_login_at
         FROM users WHERE email = ?",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Get user registration status (for polling during async registration)
pub async fn get_user_status(
    pool: &SqlitePool,
    user_id: &str,
) -> anyhow::Result<Option<UserStatus>> {
    // First check if user exists in projection
    let user_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    if user_exists {
        Ok(Some(UserStatus {
            id: user_id.to_string(),
            status: "success".to_string(),
            error: None,
        }))
    } else {
        // User not in projection yet - check aggregate status
        // This would require evento access, which we don't have in query functions
        // For now, return pending
        Ok(Some(UserStatus {
            id: user_id.to_string(),
            status: "pending".to_string(),
            error: None,
        }))
    }
}

/// Get user profile by user ID, returns defaults if profile doesn't exist
pub async fn get_user_profile(pool: &SqlitePool, user_id: &str) -> anyhow::Result<UserProfile> {
    // Query the user_profiles table
    let result = sqlx::query(
        "SELECT user_id, dietary_restrictions, cuisine_variety_weight, household_size,
                is_premium_active, premium_bypass
         FROM user_profiles
         WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => {
            // Parse dietary_restrictions from JSON
            let dietary_restrictions_json: String = row.try_get("dietary_restrictions")?;
            let dietary_restrictions: Vec<String> =
                serde_json::from_str(&dietary_restrictions_json).unwrap_or_default();

            Ok(UserProfile {
                user_id: row.try_get("user_id")?,
                dietary_restrictions,
                cuisine_variety_weight: row.try_get("cuisine_variety_weight")?,
                household_size: row.try_get("household_size")?,
                is_premium_active: row.try_get("is_premium_active")?,
                premium_bypass: row.try_get("premium_bypass")?,
            })
        }
        None => {
            // Return defaults when profile doesn't exist
            Ok(UserProfile {
                user_id: user_id.to_string(),
                dietary_restrictions: Vec::new(),
                cuisine_variety_weight: 0.7,
                household_size: None,
                is_premium_active: false,
                premium_bypass: false,
            })
        }
    }
}
