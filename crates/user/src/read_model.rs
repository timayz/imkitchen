use crate::aggregate::UserAggregate;
use crate::error::UserResult;
use crate::events::UserCreated;
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
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, tier, recipe_count, created_at)
        VALUES (?1, ?2, ?3, 'free', 0, ?4)
        "#,
    )
    .bind(&event.data.user_id)
    .bind(&event.data.email)
    .bind(&event.data.password_hash)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    // Acknowledge the event so subscription cursor advances
    context.acknowledge().await?;

    Ok(())
}

/// Public wrapper to access the evento subscription handler
///
/// The handler is registered via:
/// ```ignore
/// evento::subscribe("user-read-model")
///     .aggregator::<UserAggregate>()
///     .data(pool.clone())
///     .handler(on_user_created())
///     .run(&executor)
///     .await?;
/// ```
pub fn on_user_created() -> impl evento::SubscribeHandler<evento::Sqlite> {
    user_created_handler()
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
