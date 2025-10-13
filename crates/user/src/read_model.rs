use crate::aggregate::UserAggregate;
use crate::error::UserResult;
use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, PasswordChanged, ProfileCompleted, SkillLevelSet,
    UserCreated, WeeknightAvailabilitySet,
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

/// Handler for SkillLevelSet event (Step 3)
#[evento::handler(UserAggregate)]
async fn skill_level_set_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<SkillLevelSet>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query("UPDATE users SET skill_level = ?1 WHERE id = ?2")
        .bind(&event.data.skill_level)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Handler for WeeknightAvailabilitySet event (Step 4)
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
        .aggregator::<UserAggregate>()
        .data(pool)
        .handler(user_created_handler())
        .handler(password_changed_handler())
        .handler(dietary_restrictions_set_handler())
        .handler(household_size_set_handler())
        .handler(skill_level_set_handler())
        .handler(weeknight_availability_set_handler())
        .handler(profile_completed_handler())
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
