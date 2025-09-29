# ADR-002: SQLite with Event Sourcing for Data Persistence

## Status
Accepted

## Context
IMKitchen requires a data persistence strategy that provides:

- **Audit Trail**: Complete history of all kitchen operations for food safety compliance
- **Data Integrity**: Reliable data consistency for critical kitchen operations
- **Performance**: Fast queries for real-time kitchen operations
- **Simplicity**: Minimal operational overhead for deployment and maintenance
- **Offline Capability**: Local data storage for resilient kitchen operations
- **Event Reconstruction**: Ability to replay events for debugging and analysis

Traditional RDBMS approaches lose historical data through updates/deletes, making audit trails difficult. NoSQL databases add operational complexity. PostgreSQL with event sourcing requires additional infrastructure.

## Decision
We will use **SQLite with Event Sourcing patterns** implemented through:

1. **SQLite Database**: Embedded database for simplicity and performance
2. **Event Store**: All state changes stored as immutable events
3. **Projections**: Read models built from events for query performance
4. **SQLx**: Type-safe database operations with compile-time query validation
5. **Evento Framework**: Custom event sourcing framework tailored to domain needs

## Alternatives Considered

### PostgreSQL with Traditional CRUD
**Pros:**
- Mature and well-understood
- Strong ACID guarantees
- Rich ecosystem and tooling

**Cons:**
- Requires separate database server
- No built-in audit trail without additional complexity
- Lost historical data on updates/deletes
- Higher operational overhead

### PostgreSQL with Event Sourcing
**Pros:**
- Full event sourcing capabilities
- Strong consistency guarantees
- Advanced features (JSONB, triggers, etc.)

**Cons:**
- Requires database server setup and maintenance
- Network dependency reduces reliability
- Overkill for single-tenant application
- Higher resource requirements

### NoSQL (MongoDB/CouchDB) with Event Sourcing
**Pros:**
- Natural fit for event storage (document-based)
- Flexible schema evolution
- Good horizontal scaling

**Cons:**
- Eventual consistency challenges
- Higher operational complexity
- Less mature tooling for complex queries
- Network dependency for hosted solutions

### In-Memory with Periodic Snapshots
**Pros:**
- Maximum performance
- Simple implementation

**Cons:**
- Data loss risk on crashes
- Limited by available memory
- No persistence guarantees
- Complex backup/restore procedures

### File-based Event Store
**Pros:**
- Simple append-only file operations
- No database dependencies
- Easy backup and restore

**Cons:**
- No query capabilities
- Manual indexing required
- Concurrency challenges
- Limited to single-process access

## Consequences

### Positive
- **Complete Audit Trail**: Every state change preserved as immutable events
- **Data Recovery**: Ability to reconstruct any historical state from events
- **Debugging Capabilities**: Full replay of events for issue investigation
- **Zero-Dependency Deployment**: SQLite embedded, no separate database server required
- **Performance**: Local database access with sub-millisecond query times
- **ACID Guarantees**: Full ACID compliance for critical operations
- **Backup Simplicity**: Single file backup and restore procedures
- **Type Safety**: SQLx provides compile-time query validation
- **Schema Evolution**: Events provide natural versioning and migration paths
- **Offline Resilience**: No network dependencies for data access

### Negative
- **Storage Overhead**: Events require more storage than traditional state-based storage
- **Query Complexity**: Complex queries may require multiple projections
- **Learning Curve**: Event sourcing patterns less familiar than CRUD operations
- **Eventual Consistency**: Projections may lag behind events (mitigated by synchronous updates)
- **Event Schema Evolution**: Requires careful handling of event version changes

### Risks
- **SQLite Limitations**: May hit SQLite limits with extreme scale
  - *Mitigation*: Monitor database size and query performance, plan migration path if needed
- **Event Store Corruption**: Risk of event store corruption affecting all data
  - *Mitigation*: Regular backups, Write-Ahead Logging (WAL), file system integrity checks
- **Complex Event Evolution**: Difficulty evolving event schemas over time
  - *Mitigation*: Event versioning strategy, backward compatibility requirements

## Implementation Notes

### Event Store Schema
```sql
-- Core event store table
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data TEXT NOT NULL,  -- JSON
    event_version INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    sequence_number INTEGER NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_events_aggregate ON events(aggregate_id, sequence_number);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_timestamp ON events(created_at);
```

### Event Sourcing Framework
```rust
// Event trait for all domain events
pub trait DomainEvent: Serialize + DeserializeOwned + Clone + Debug {
    fn event_type(&self) -> &'static str;
    fn event_version(&self) -> u32;
}

// Aggregate trait for domain aggregates
pub trait Aggregate: Default + Clone {
    type Event: DomainEvent;
    type Error: std::error::Error;
    
    fn apply_event(&mut self, event: &Self::Event) -> Result<(), Self::Error>;
    fn aggregate_id(&self) -> &str;
    fn aggregate_type() -> &'static str;
}

// Event store implementation
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn save_events(
        &self,
        aggregate_id: &str,
        expected_version: u64,
        events: Vec<Box<dyn DomainEvent>>,
    ) -> Result<(), EventStoreError>;
    
    async fn load_events(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Result<Vec<Box<dyn DomainEvent>>, EventStoreError>;
}
```

### Projection Management
```rust
// Projection trait for read models
#[async_trait]
pub trait Projection: Send + Sync {
    type Event: DomainEvent;
    
    async fn handle_event(&mut self, event: &Self::Event) -> Result<(), ProjectionError>;
    async fn reset(&mut self) -> Result<(), ProjectionError>;
    fn projection_name(&self) -> &str;
}

// Example: Recipe summary projection
pub struct RecipeSummaryProjection {
    db: Pool<Sqlite>,
}

#[async_trait]
impl Projection for RecipeSummaryProjection {
    type Event = RecipeEvent;
    
    async fn handle_event(&mut self, event: &RecipeEvent) -> Result<(), ProjectionError> {
        match event {
            RecipeEvent::RecipeCreated { id, name, cuisine_type, .. } => {
                sqlx::query!(
                    "INSERT INTO recipe_summaries (id, name, cuisine_type) VALUES (?, ?, ?)",
                    id, name, cuisine_type
                ).execute(&self.db).await?;
            }
            RecipeEvent::RecipeUpdated { id, name, .. } => {
                sqlx::query!(
                    "UPDATE recipe_summaries SET name = ? WHERE id = ?",
                    name, id
                ).execute(&self.db).await?;
            }
            RecipeEvent::RecipeDeleted { id } => {
                sqlx::query!(
                    "DELETE FROM recipe_summaries WHERE id = ?",
                    id
                ).execute(&self.db).await?;
            }
        }
        Ok(())
    }
}
```

### SQLite Configuration for Performance
```rust
// Database connection configuration
pub async fn create_database_pool() -> Result<Pool<Sqlite>, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:imkitchen.db".to_string());
    
    SqlitePoolOptions::new()
        .max_connections(20)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Some(Duration::from_secs(600)))
        .after_connect(|conn, _meta| Box::pin(async move {
            // Enable WAL mode for better concurrency
            sqlx::query("PRAGMA journal_mode = WAL").execute(conn).await?;
            // Set synchronous mode for durability
            sqlx::query("PRAGMA synchronous = NORMAL").execute(conn).await?;
            // Enable foreign key constraints
            sqlx::query("PRAGMA foreign_keys = ON").execute(conn).await?;
            // Set cache size (negative value = KB)
            sqlx::query("PRAGMA cache_size = -64000").execute(conn).await?; // 64MB
            Ok(())
        }))
        .connect(&database_url)
        .await
}
```

### Event Versioning Strategy
```rust
// Event versioning for schema evolution
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "version")]
pub enum RecipeCreatedEvent {
    #[serde(rename = "1")]
    V1 {
        id: String,
        name: String,
        ingredients: Vec<String>,
    },
    #[serde(rename = "2")]
    V2 {
        id: String,
        name: String,
        ingredients: Vec<IngredientV2>,
        cuisine_type: CuisineType,
        created_by: UserId,
    },
}

impl DomainEvent for RecipeCreatedEvent {
    fn event_type(&self) -> &'static str {
        "recipe_created"
    }
    
    fn event_version(&self) -> u32 {
        match self {
            RecipeCreatedEvent::V1 { .. } => 1,
            RecipeCreatedEvent::V2 { .. } => 2,
        }
    }
}
```

### Backup and Recovery
```rust
// Automated backup procedures
pub async fn backup_database(backup_path: &str) -> Result<(), std::io::Error> {
    let source = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:imkitchen.db".to_string());
    
    // Use SQLite's backup API for consistent backups
    tokio::process::Command::new("sqlite3")
        .args([&source, &format!(".backup {}", backup_path)])
        .output()
        .await?;
    
    Ok(())
}

// Point-in-time recovery using events
pub async fn replay_events_to_timestamp(
    event_store: &dyn EventStore,
    target_timestamp: DateTime<Utc>,
) -> Result<(), ReplayError> {
    // Load all events up to timestamp
    // Rebuild projections from events
    // Validate data consistency
}
```

## References
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
- [SQLite Documentation](https://sqlite.org/docs.html)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Command Query Responsibility Segregation (CQRS)](https://martinfowler.com/bliki/CQRS.html)
- [Versioning in an Event Sourced System](https://leanpub.com/esversioning)