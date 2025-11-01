//! Tests for configuration system

use imkitchen::Config;

#[test]
fn test_config_loads_from_default_toml() {
    // Test that default config can be loaded
    let config = Config::load(None).expect("Failed to load config");

    // Verify default values
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.database.evento_db, "evento.db");
    assert_eq!(config.database.queries_db, "queries.db");
    assert_eq!(config.database.validation_db, "validation.db");
    assert_eq!(config.logging.level, "info");
}

#[test]
fn test_config_has_all_required_fields() {
    let config = Config::load(None).expect("Failed to load config");

    // Verify all sections exist and have required fields
    assert!(!config.server.host.is_empty());
    assert!(config.server.port > 0);
    assert!(!config.database.evento_db.is_empty());
    assert!(!config.database.queries_db.is_empty());
    assert!(!config.database.validation_db.is_empty());
    assert!(!config.logging.level.is_empty());
    assert!(!config.logging.format.is_empty());
}
