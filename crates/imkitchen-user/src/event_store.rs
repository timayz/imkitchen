// Event store implementation for user domain events with SQLx persistence

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::events::{
    DietaryRestrictionsChanged, FamilySizeChanged, UserLoggedIn, UserPasswordChanged,
    UserProfileUpdated, UserRegistered,
};

/// Generic trait for domain events
pub trait DomainEvent: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Clone {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
}

/// Event store for persisting and retrieving domain events
#[derive(Debug, Clone)]
pub struct EventStore {
    pool: SqlitePool,
}

/// Persisted event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub id: i64,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub event_data: String, // JSON serialized event
    pub version: i64,
    pub created_at: DateTime<Utc>,
}

impl EventStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Store a domain event in the event store
    pub async fn store_event<T: DomainEvent>(
        &self,
        event: &T,
    ) -> Result<PersistedEvent, EventStoreError> {
        let aggregate_id = event.aggregate_id();
        let event_type = event.event_type();
        let event_data = serde_json::to_string(event)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        // Get the next version for this aggregate
        let next_version = self.get_next_version(aggregate_id).await?;

        let result = sqlx::query(
            r#"
            INSERT INTO user_events (aggregate_id, event_type, event_data, version, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(aggregate_id.to_string())
        .bind(event_type)
        .bind(&event_data)
        .bind(next_version)
        .bind(Utc::now().format("%Y-%m-%d %H:%M:%S%.3fZ").to_string())
        .execute(&self.pool)
        .await
        .map_err(EventStoreError::DatabaseError)?;

        let created_at = Utc::now();

        Ok(PersistedEvent {
            id: result.last_insert_rowid(),
            aggregate_id,
            event_type: event_type.to_string(),
            event_data,
            version: next_version,
            created_at,
        })
    }

    /// Get all events for a specific aggregate
    pub async fn get_events_for_aggregate(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<PersistedEvent>, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id, aggregate_id, event_type, event_data, version, created_at
            FROM user_events
            WHERE aggregate_id = ?
            ORDER BY version ASC
            "#,
        )
        .bind(aggregate_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(EventStoreError::DatabaseError)?;

        let events = rows
            .into_iter()
            .map(|row| {
                let aggregate_id = Uuid::parse_str(row.get("aggregate_id"))
                    .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;

                let created_at = DateTime::parse_from_rfc3339(row.get("created_at"))
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);

                Ok(PersistedEvent {
                    id: row.get("id"),
                    aggregate_id,
                    event_type: row.get("event_type"),
                    event_data: row.get("event_data"),
                    version: row.get("version"),
                    created_at,
                })
            })
            .collect::<Result<Vec<_>, EventStoreError>>()?;

        Ok(events)
    }

    /// Get events by type
    pub async fn get_events_by_type(
        &self,
        event_type: &str,
    ) -> Result<Vec<PersistedEvent>, EventStoreError> {
        let rows = sqlx::query(
            r#"
            SELECT id, aggregate_id, event_type, event_data, version, created_at
            FROM user_events
            WHERE event_type = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(event_type)
        .fetch_all(&self.pool)
        .await
        .map_err(EventStoreError::DatabaseError)?;

        let events = rows
            .into_iter()
            .map(|row| {
                let aggregate_id = Uuid::parse_str(row.get("aggregate_id"))
                    .map_err(|e| EventStoreError::DeserializationError(e.to_string()))?;

                let created_at = DateTime::parse_from_rfc3339(row.get("created_at"))
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);

                Ok(PersistedEvent {
                    id: row.get("id"),
                    aggregate_id,
                    event_type: row.get("event_type"),
                    event_data: row.get("event_data"),
                    version: row.get("version"),
                    created_at,
                })
            })
            .collect::<Result<Vec<_>, EventStoreError>>()?;

        Ok(events)
    }

    /// Get the next version number for an aggregate
    async fn get_next_version(&self, aggregate_id: Uuid) -> Result<i64, EventStoreError> {
        let result = sqlx::query(
            r#"
            SELECT COALESCE(MAX(version), 0) + 1 as next_version
            FROM user_events
            WHERE aggregate_id = ?
            "#,
        )
        .bind(aggregate_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(EventStoreError::DatabaseError)?;

        Ok(result.get("next_version"))
    }

    /// Get event count for an aggregate (useful for tracking)
    pub async fn get_event_count(&self, aggregate_id: Uuid) -> Result<i64, EventStoreError> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM user_events
            WHERE aggregate_id = ?
            "#,
        )
        .bind(aggregate_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(EventStoreError::DatabaseError)?;

        Ok(result.get("count"))
    }
}

/// Event store error types
#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: i64, actual: i64 },
}

// Implement DomainEvent for all user events
impl DomainEvent for UserRegistered {
    fn event_type(&self) -> &'static str {
        "UserRegistered"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

impl DomainEvent for UserLoggedIn {
    fn event_type(&self) -> &'static str {
        "UserLoggedIn"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.logged_in_at
    }
}

impl DomainEvent for UserPasswordChanged {
    fn event_type(&self) -> &'static str {
        "UserPasswordChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.changed_at
    }
}

impl DomainEvent for UserProfileUpdated {
    fn event_type(&self) -> &'static str {
        "UserProfileUpdated"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl DomainEvent for DietaryRestrictionsChanged {
    fn event_type(&self) -> &'static str {
        "DietaryRestrictionsChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.changed_at
    }
}

impl DomainEvent for FamilySizeChanged {
    fn event_type(&self) -> &'static str {
        "FamilySizeChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.changed_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{DietaryRestriction, FamilySize};

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:")
            .await
            .expect("Failed to create in-memory database");

        // Create the user_events table
        sqlx::query(
            r#"
            CREATE TABLE user_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                aggregate_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                version INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(aggregate_id, version)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create user_events table");

        pool
    }

    #[tokio::test]
    async fn test_store_dietary_restrictions_changed_event() {
        let pool = setup_test_db().await;
        let event_store = EventStore::new(pool);

        let user_id = Uuid::new_v4();
        let event = DietaryRestrictionsChanged::new(
            user_id,
            vec![DietaryRestriction::Vegetarian],
            vec![
                DietaryRestriction::Vegetarian,
                DietaryRestriction::GlutenFree,
            ],
        );

        let persisted = event_store.store_event(&event).await.unwrap();

        assert_eq!(persisted.aggregate_id, user_id);
        assert_eq!(persisted.event_type, "DietaryRestrictionsChanged");
        assert_eq!(persisted.version, 1);
    }

    #[tokio::test]
    async fn test_store_family_size_changed_event() {
        let pool = setup_test_db().await;
        let event_store = EventStore::new(pool);

        let user_id = Uuid::new_v4();
        let event = FamilySizeChanged::new(
            user_id,
            FamilySize::new(2).unwrap(),
            FamilySize::new(4).unwrap(),
        );

        let persisted = event_store.store_event(&event).await.unwrap();

        assert_eq!(persisted.aggregate_id, user_id);
        assert_eq!(persisted.event_type, "FamilySizeChanged");
        assert_eq!(persisted.version, 1);
    }

    #[tokio::test]
    async fn test_get_events_for_aggregate() {
        let pool = setup_test_db().await;
        let event_store = EventStore::new(pool);

        let user_id = Uuid::new_v4();

        // Store multiple events
        let event1 =
            DietaryRestrictionsChanged::new(user_id, vec![], vec![DietaryRestriction::Vegetarian]);
        let event2 = FamilySizeChanged::new(
            user_id,
            FamilySize::new(2).unwrap(),
            FamilySize::new(4).unwrap(),
        );

        event_store.store_event(&event1).await.unwrap();
        event_store.store_event(&event2).await.unwrap();

        let events = event_store.get_events_for_aggregate(user_id).await.unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].version, 1);
        assert_eq!(events[1].version, 2);
    }

    #[tokio::test]
    async fn test_get_events_by_type() {
        let pool = setup_test_db().await;
        let event_store = EventStore::new(pool);

        let user_id1 = Uuid::new_v4();
        let user_id2 = Uuid::new_v4();

        // Store events of same type for different users
        let event1 =
            DietaryRestrictionsChanged::new(user_id1, vec![], vec![DietaryRestriction::Vegetarian]);
        let event2 =
            DietaryRestrictionsChanged::new(user_id2, vec![], vec![DietaryRestriction::Vegan]);

        event_store.store_event(&event1).await.unwrap();
        event_store.store_event(&event2).await.unwrap();

        let events = event_store
            .get_events_by_type("DietaryRestrictionsChanged")
            .await
            .unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "DietaryRestrictionsChanged");
        assert_eq!(events[1].event_type, "DietaryRestrictionsChanged");
    }

    #[tokio::test]
    async fn test_event_count() {
        let pool = setup_test_db().await;
        let event_store = EventStore::new(pool);

        let user_id = Uuid::new_v4();

        // Initially no events
        let count = event_store.get_event_count(user_id).await.unwrap();
        assert_eq!(count, 0);

        // Store an event
        let event =
            DietaryRestrictionsChanged::new(user_id, vec![], vec![DietaryRestriction::Vegetarian]);
        event_store.store_event(&event).await.unwrap();

        // Count should be 1
        let count = event_store.get_event_count(user_id).await.unwrap();
        assert_eq!(count, 1);
    }
}
