use sqlx::SqlitePool;
use tokio::time::{interval, Duration};
use tracing::{error, info};

pub struct SessionCleanupService {
    db_pool: SqlitePool,
    cleanup_interval: Duration,
}

impl SessionCleanupService {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool,
            cleanup_interval: Duration::from_secs(3600), // Run every hour
        }
    }

    pub fn new_with_interval(db_pool: SqlitePool, interval: Duration) -> Self {
        Self {
            db_pool,
            cleanup_interval: interval,
        }
    }

    /// Start the background session cleanup task
    /// This should be called when the server starts
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let pool = self.db_pool.clone();
        let interval_duration = self.cleanup_interval;

        tokio::spawn(async move {
            let mut cleanup_timer = interval(interval_duration);

            info!(
                "Session cleanup service started with interval: {:?}",
                interval_duration
            );

            loop {
                cleanup_timer.tick().await;

                info!("Running session cleanup task...");

                match Self::cleanup_expired_sessions(&pool).await {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            info!("Cleaned up {} expired sessions", cleaned_count);
                        }
                    }
                    Err(e) => {
                        error!("Session cleanup failed: {}", e);
                    }
                }
            }
        })
    }

    /// Clean up expired sessions from the database
    pub async fn cleanup_expired_sessions(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        // Delete all expired sessions
        let result = sqlx::query("DELETE FROM user_sessions WHERE expires_at <= datetime('now')")
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Run cleanup immediately (for testing or manual cleanup)
    pub async fn run_cleanup_now(&self) -> Result<u64, sqlx::Error> {
        Self::cleanup_expired_sessions(&self.db_pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as ChronoDuration, Utc};
    use sqlx::{Row, SqlitePool};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Create the necessary tables
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

        pool
    }

    async fn create_test_user(pool: &SqlitePool, email: &str) -> String {
        let user_id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, created_at, updated_at, last_active) 
             VALUES (?, ?, ?, ?, datetime('now'), datetime('now'), datetime('now'))"
        )
        .bind(&user_id)
        .bind(email)
        .bind("dummy_hash")
        .bind("Test User")
        .execute(pool)
        .await
        .unwrap();

        user_id
    }

    async fn create_test_session(
        pool: &SqlitePool,
        user_id: &str,
        expires_in_minutes: i64,
    ) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let expires_at = Utc::now() + ChronoDuration::minutes(expires_in_minutes);

        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, session_token, expires_at, created_at) 
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&session_id)
        .bind(user_id)
        .bind(format!("session_{}", session_id))
        .bind(expires_at.format("%Y-%m-%d %H:%M:%S").to_string())
        .bind(Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
        .execute(pool)
        .await
        .unwrap();

        session_id
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let pool = setup_test_db().await;

        // Create test user first
        let user_id = create_test_user(&pool, "test@example.com").await;

        // Create expired session (1 hour ago)
        let _expired_session = create_test_session(&pool, &user_id, -60).await;

        // Create valid session (1 hour from now)
        let valid_session = create_test_session(&pool, &user_id, 60).await;

        // Run cleanup
        let cleaned_count = SessionCleanupService::cleanup_expired_sessions(&pool)
            .await
            .unwrap();

        // Should have cleaned 1 expired session
        assert_eq!(cleaned_count, 1);

        // Check that only valid session remains
        let remaining_sessions = sqlx::query("SELECT COUNT(*) as count FROM user_sessions")
            .fetch_one(&pool)
            .await
            .unwrap();

        let count: i64 = remaining_sessions.try_get("count").unwrap();
        assert_eq!(count, 1);

        // Verify it's the correct session
        let session = sqlx::query("SELECT id FROM user_sessions LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        let session_id: String = session.try_get("id").unwrap();
        assert_eq!(session_id, valid_session);
    }

    #[tokio::test]
    async fn test_cleanup_no_expired_sessions() {
        let pool = setup_test_db().await;

        // Create test user first
        let user_id = create_test_user(&pool, "test2@example.com").await;

        // Create only valid sessions
        let _valid_session1 = create_test_session(&pool, &user_id, 60).await;
        let _valid_session2 = create_test_session(&pool, &user_id, 120).await;

        // Run cleanup
        let cleaned_count = SessionCleanupService::cleanup_expired_sessions(&pool)
            .await
            .unwrap();

        // Should have cleaned 0 sessions
        assert_eq!(cleaned_count, 0);

        // Check that both sessions remain
        let remaining_sessions = sqlx::query("SELECT COUNT(*) as count FROM user_sessions")
            .fetch_one(&pool)
            .await
            .unwrap();

        let count: i64 = remaining_sessions.try_get("count").unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_service_creation() {
        let pool = setup_test_db().await;
        let service = SessionCleanupService::new(pool.clone());

        // Default interval should be 1 hour
        assert_eq!(service.cleanup_interval, Duration::from_secs(3600));

        // Test custom interval
        let custom_interval = Duration::from_secs(1800); // 30 minutes
        let custom_service = SessionCleanupService::new_with_interval(pool, custom_interval);
        assert_eq!(custom_service.cleanup_interval, custom_interval);
    }
}
