//! Tests for database helper functions

mod helpers;

#[tokio::test]
async fn test_setup_test_databases() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    // Verify queries database is usable
    let _result = sqlx::query("SELECT 1")
        .fetch_one(&dbs.queries)
        .await
        .expect("Failed to query test database");

    // Verify validation database is usable
    let _result = sqlx::query("SELECT 1")
        .fetch_one(&dbs.validation)
        .await
        .expect("Failed to query validation database");

    // Cleanup
    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

#[test]
fn test_create_test_config() {
    let config = helpers::create_test_config();

    // Verify test config uses in-memory databases
    assert_eq!(config.database.evento_db, ":memory:");
    assert_eq!(config.database.queries_db, ":memory:");
    assert_eq!(config.database.validation_db, ":memory:");

    // Verify test config uses different port
    assert_eq!(config.server.port, 3001);
}
