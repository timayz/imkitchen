use crate::aggregate::UserAggregate;
use crate::error::UserResult;
use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, NotificationPermissionChanged, PasswordChanged,
    ProfileCompleted, ProfileUpdated, RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeShared,
    SubscriptionUpgraded, UserCreated, UserMealPlanningPreferencesUpdated,
    WeeknightAvailabilitySet,
};
use evento::{AggregatorName, Context, EventDetails, Executor};
use sqlx::{Row, SqlitePool};

/// Async evento subscription handler for UserCreated events
///
/// This handler projects UserCreated events from the evento event store
/// into the users read model table for efficient querying.
#[evento::handler(UserAggregate)]
async fn user_created_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserCreated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL insert to project event into read model
    // Use event.aggregator_id as the primary key (user id)
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at)
        VALUES (?1, ?2, ?3, 'free', 0, ?4)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.email)
    .bind(&event.data.password_hash)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for PasswordChanged events
///
/// This handler projects PasswordChanged events from the evento event store
/// into the users read model table, updating the password_hash field.
#[evento::handler(UserAggregate)]
async fn password_changed_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<PasswordChanged>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL update to project event into read model
    // Use event.aggregator_id to identify which user to update
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = ?1
        WHERE id = ?2
        "#,
    )
    .bind(&event.data.password_hash)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Handler for DietaryRestrictionsSet event (Step 1)
#[evento::handler(UserAggregate)]
async fn dietary_restrictions_set_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<DietaryRestrictionsSet>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let dietary_restrictions_json = serde_json::to_string(&event.data.dietary_restrictions)?;

    sqlx::query("UPDATE users SET dietary_restrictions = ?1 WHERE id = ?2")
        .bind(&dietary_restrictions_json)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for HouseholdSizeSet event (Step 2)
#[evento::handler(UserAggregate)]
async fn household_size_set_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<HouseholdSizeSet>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query("UPDATE users SET household_size = ?1 WHERE id = ?2")
        .bind(event.data.household_size)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for WeeknightAvailabilitySet event (Step 3)
#[evento::handler(UserAggregate)]
async fn weeknight_availability_set_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<WeeknightAvailabilitySet>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query("UPDATE users SET weeknight_availability = ?1 WHERE id = ?2")
        .bind(&event.data.weeknight_availability)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for ProfileCompleted event - marks onboarding as complete
#[evento::handler(UserAggregate)]
async fn profile_completed_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ProfileCompleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query("UPDATE users SET onboarding_completed = 1 WHERE id = ?1")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for ProfileUpdated event - updates profile with COALESCE logic
///
/// This handler supports partial updates by only updating non-None fields.
/// Uses SQL COALESCE pattern to preserve existing values when field is None.
/// Updates updated_at timestamp for audit trail (AC-7).
#[evento::handler(UserAggregate)]
async fn profile_updated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ProfileUpdated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Build dynamic SQL query based on which fields are present
    let mut updates: Vec<String> = Vec::new();
    let mut bindings: Vec<String> = Vec::new();

    // Process dietary_restrictions (convert Vec<String> to JSON)
    if let Some(ref dietary_restrictions) = event.data.dietary_restrictions {
        let dietary_json = serde_json::to_string(dietary_restrictions)?;
        bindings.push(dietary_json);
        updates.push(format!("dietary_restrictions = ?{}", bindings.len()));
    }

    // Process household_size
    if let Some(household_size) = event.data.household_size {
        bindings.push(household_size.to_string());
        updates.push(format!("household_size = ?{}", bindings.len()));
    }

    // Process weeknight_availability
    if let Some(ref weeknight_availability) = event.data.weeknight_availability {
        bindings.push(weeknight_availability.clone());
        updates.push(format!("weeknight_availability = ?{}", bindings.len()));
    }

    // Always update updated_at timestamp
    bindings.push(event.data.updated_at.clone());
    updates.push(format!("updated_at = ?{}", bindings.len()));

    // Only execute if there are fields to update
    if !updates.is_empty() {
        let sql = format!(
            "UPDATE users SET {} WHERE id = ?{}",
            updates.join(", "),
            bindings.len() + 1
        );

        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }
        query = query.bind(&event.aggregator_id);

        query.execute(&pool).await?;
    }

    Ok(())
}

/// Handler for RecipeCreated event - increment recipe_count in users table
///
/// This handler listens to RecipeCreated events from the recipe domain and
/// increments the recipe_count in the users read model table for freemium enforcement.
#[evento::handler(UserAggregate)]
async fn recipe_created_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query("UPDATE users SET recipe_count = recipe_count + 1 WHERE id = ?1")
        .bind(&event.data.user_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for RecipeDeleted event - decrement recipe_count in users table
///
/// This handler listens to RecipeDeleted events from the recipe domain and
/// decrements the recipe_count in the users read model table to free up a slot.
#[evento::handler(UserAggregate)]
async fn recipe_deleted_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Use MAX to prevent negative counts
    sqlx::query("UPDATE users SET recipe_count = MAX(0, recipe_count - 1) WHERE id = ?1")
        .bind(&event.data.user_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for RecipeShared event - adjust recipe_count in users table
///
/// This handler listens to RecipeShared events from the recipe domain and
/// adjusts the recipe_count based on whether the recipe was shared or unshared.
/// Shared recipes do NOT count toward the freemium limit.
#[evento::handler(UserAggregate)]
async fn recipe_shared_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    if event.data.shared {
        // Recipe was shared - decrement count (shared recipes don't count toward limit)
        sqlx::query("UPDATE users SET recipe_count = MAX(0, recipe_count - 1) WHERE id = ?1")
            .bind(&event.data.user_id)
            .execute(&pool)
            .await?;
    } else {
        // Recipe was unshared - increment count (now counts toward limit)
        sqlx::query("UPDATE users SET recipe_count = recipe_count + 1 WHERE id = ?1")
            .bind(&event.data.user_id)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

/// Handler for RecipeFavorited event - update favorite_count in users table
///
/// This handler listens to RecipeFavorited events from the recipe domain and
/// increments or decrements the favorite_count in the users read model table.
/// This provides O(1) favorite count queries instead of O(n) COUNT(*) queries.
#[evento::handler(UserAggregate)]
async fn recipe_favorited_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    if event.data.favorited {
        // Increment favorite_count when favorited
        sqlx::query("UPDATE users SET favorite_count = favorite_count + 1 WHERE id = ?1")
            .bind(&event.data.user_id)
            .execute(&pool)
            .await?;
    } else {
        // Decrement favorite_count when unfavorited (use MAX to prevent negative)
        sqlx::query("UPDATE users SET favorite_count = MAX(0, favorite_count - 1) WHERE id = ?1")
            .bind(&event.data.user_id)
            .execute(&pool)
            .await?;
    }

    Ok(())
}

/// Handler for SubscriptionUpgraded event - update tier and Stripe metadata in users table
///
/// This handler projects SubscriptionUpgraded events from the evento event store
/// into the users read model table, updating the tier, stripe_customer_id, and
/// stripe_subscription_id fields.
#[evento::handler(UserAggregate)]
async fn subscription_upgraded_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<SubscriptionUpgraded>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE users
        SET tier = ?1, stripe_customer_id = ?2, stripe_subscription_id = ?3
        WHERE id = ?4
        "#,
    )
    .bind(&event.data.new_tier)
    .bind(&event.data.stripe_customer_id)
    .bind(&event.data.stripe_subscription_id)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Handler for NotificationPermissionChanged event - update notification permission in users table
///
/// AC #3, #5, #8: Tracks notification permission status and denial timestamp for grace period
///
/// This handler projects NotificationPermissionChanged events from the evento event store
/// into the users read model table, updating the notification_permission_status and
/// last_permission_denial_at fields.
#[evento::handler(UserAggregate)]
async fn notification_permission_changed_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<NotificationPermissionChanged>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE users
        SET notification_permission_status = ?1, last_permission_denial_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(&event.data.permission_status)
    .bind(&event.data.last_permission_denial_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Handler for UserMealPlanningPreferencesUpdated event (Story 6.6 AC-5, AC-6, AC-7)
///
/// This handler updates user meal planning preferences in the users table when
/// the user configures algorithm preferences for personalized meal planning.
///
/// Epic 6: Enhanced Meal Planning System
///
/// **Updated Fields:**
/// - max_prep_time_weeknight (minutes)
/// - max_prep_time_weekend (minutes)
/// - avoid_consecutive_complex (boolean)
/// - cuisine_variety_weight (float 0.0-1.0)
/// - dietary_restrictions (JSON TEXT - Vec<DietaryRestriction>)
/// - household_size (u32)
/// - skill_level (string: "Beginner", "Intermediate", "Advanced")
/// - weeknight_availability (JSON string)
#[evento::handler(UserAggregate)]
async fn user_meal_planning_preferences_updated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserMealPlanningPreferencesUpdated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // All fields from the event are directly stored
    // dietary_restrictions is already JSON string in the event
    sqlx::query(
        r#"
        UPDATE users
        SET max_prep_time_weeknight = ?1,
            max_prep_time_weekend = ?2,
            avoid_consecutive_complex = ?3,
            cuisine_variety_weight = ?4,
            dietary_restrictions = ?5,
            household_size = ?6,
            skill_level = ?7,
            weeknight_availability = ?8,
            updated_at = ?9
        WHERE id = ?10
        "#,
    )
    .bind(event.data.max_prep_time_weeknight as i32)
    .bind(event.data.max_prep_time_weekend as i32)
    .bind(event.data.avoid_consecutive_complex)
    .bind(event.data.cuisine_variety_weight)
    .bind(&event.data.dietary_restrictions) // Already JSON string
    .bind(event.data.household_size as i32)
    .bind(&event.data.skill_level)
    .bind(&event.data.weeknight_availability)
    .bind(&event.data.updated_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Create user event subscription for read model projection
///
/// Returns a subscription builder that can be run with `.run(&executor).await`
///
/// Usage in main.rs:
/// ```no_run
/// # use sqlx::SqlitePool;
/// # use evento::Sqlite;
/// # async fn example(pool: SqlitePool, executor: Sqlite) -> anyhow::Result<()> {
/// user::user_projection(pool.clone()).run(&executor).await?;
/// # Ok(())
/// # }
/// ```
pub fn user_projection(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("user-read-model")
        .data(pool)
        .handler(user_created_handler())
        .handler(password_changed_handler())
        .handler(dietary_restrictions_set_handler())
        .handler(household_size_set_handler())
        .handler(weeknight_availability_set_handler())
        .handler(profile_completed_handler())
        .handler(profile_updated_handler())
        .handler(recipe_created_handler())
        .handler(recipe_deleted_handler())
        .handler(recipe_shared_handler())
        .handler(recipe_favorited_handler())
        .handler(subscription_upgraded_handler())
        .handler(notification_permission_changed_handler())
        .handler(user_meal_planning_preferences_updated_handler())
}

/// Query user by email for uniqueness check in read model
///
/// This function queries the materialized read model (users table)
/// NOT the event store. Used for validation before creating UserCreated events.
///
/// Returns Some(user_id) if user exists, None otherwise
pub async fn query_user_by_email(email: &str, pool: &SqlitePool) -> UserResult<Option<String>> {
    let result = sqlx::query("SELECT id FROM users WHERE email = ?1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    match result {
        Some(row) => {
            let id: String = row.get("id");
            Ok(Some(id))
        }
        None => Ok(None),
    }
}

/// User data for login authentication
pub struct UserLoginData {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub tier: String,
}

/// Query user by email for login authentication
///
/// Returns complete user data needed for login: id, email, password_hash, tier
/// Used by POST /login route handler to verify credentials and generate JWT
///
/// Returns Some(UserLoginData) if user exists, None otherwise
pub async fn query_user_for_login(
    email: &str,
    pool: &SqlitePool,
) -> UserResult<Option<UserLoginData>> {
    let result = sqlx::query("SELECT id, email, password_hash, tier FROM users WHERE email = ?1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    match result {
        Some(row) => {
            let user_data = UserLoginData {
                id: row.get("id"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                tier: row.get("tier"),
            };
            Ok(Some(user_data))
        }
        None => Ok(None),
    }
}

/// User notification permission data for grace period check (AC #8)
pub struct UserNotificationPermission {
    pub permission_status: String, // "not_asked", "granted", "denied", "skipped"
    pub last_permission_denial_at: Option<String>, // RFC3339 timestamp
}

/// Query user notification permission status for grace period check
///
/// AC #8: Check if user can be prompted for permission (30-day grace period)
///
/// Returns UserNotificationPermission if user exists, None otherwise
pub async fn query_user_notification_permission(
    user_id: &str,
    pool: &SqlitePool,
) -> UserResult<Option<UserNotificationPermission>> {
    let result = sqlx::query(
        "SELECT notification_permission_status, last_permission_denial_at FROM users WHERE id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => {
            let permission_data = UserNotificationPermission {
                permission_status: row.get("notification_permission_status"),
                last_permission_denial_at: row.get("last_permission_denial_at"),
            };
            Ok(Some(permission_data))
        }
        None => Ok(None),
    }
}

/// Check if grace period allows prompting user for notification permission
///
/// AC #8: Don't re-prompt if user denied within last 30 days
///
/// Returns true if user can be prompted (grace period expired or never denied)
pub async fn can_prompt_for_notification_permission(
    user_id: &str,
    pool: &SqlitePool,
) -> UserResult<bool> {
    let permission = query_user_notification_permission(user_id, pool).await?;

    match permission {
        Some(perm) => {
            // If status is "denied" and denial timestamp exists, check grace period
            if perm.permission_status == "denied" {
                if let Some(denial_timestamp_str) = perm.last_permission_denial_at {
                    // Parse denial timestamp
                    if let Ok(denial_timestamp) =
                        chrono::DateTime::parse_from_rfc3339(&denial_timestamp_str)
                    {
                        let now = chrono::Utc::now();
                        let grace_period_days = chrono::Duration::days(30);

                        // Check if 30 days have elapsed since denial
                        if now.signed_duration_since(denial_timestamp) < grace_period_days {
                            return Ok(false); // Grace period active, cannot prompt
                        }
                    }
                }
            }

            // Can prompt if: status != "denied" OR grace period expired OR no denial timestamp
            Ok(true)
        }
        None => Ok(false), // User not found
    }
}
