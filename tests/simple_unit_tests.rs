#[tokio::test]
async fn test_jwt_secret_validation() {
    use imkitchen::config::{Settings, ConfigError};
    use std::env;
    
    // Store original values
    let original_jwt = env::var("JWT_SECRET").ok();
    let original_db = env::var("DATABASE_URL").ok();
    
    // Test too short JWT secret
    env::set_var("JWT_SECRET", "short");
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
    let result = Settings::new();
    assert!(matches!(result.unwrap_err(), ConfigError::JwtSecretTooShort));
    
    // Test valid JWT secret (must be exactly 32+ chars) 
    env::set_var("JWT_SECRET", "this_is_a_valid_jwt_secret_with_32_characters");
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
    let result = Settings::new();
    // Just check we don't get the JwtSecretTooShort error specifically
    if let Err(ConfigError::JwtSecretTooShort) = result {
        panic!("JWT secret should be valid with 32+ chars");
    }
    // If we get other errors (like database issues), that's fine for this test
    
    // Restore original values
    match original_jwt {
        Some(val) => env::set_var("JWT_SECRET", val),
        None => env::remove_var("JWT_SECRET"),
    }
    match original_db {
        Some(val) => env::set_var("DATABASE_URL", val),
        None => env::remove_var("DATABASE_URL"),
    }
}

#[tokio::test]
async fn test_database_connection_creation() {
    use imkitchen::config::database;
    
    // Test with invalid URL (should fail gracefully)
    let invalid_url = "postgresql://invalid:invalid@localhost:9999/nonexistent";
    let result = database::create_pool(invalid_url).await;
    assert!(result.is_err(), "Invalid database URL should fail");
}

#[tokio::test]
async fn test_redis_client_creation() {
    use imkitchen::config::redis;
    
    let redis_url = "redis://localhost:6379";
    let result = redis::create_client(&redis_url).await;
    assert!(result.is_ok(), "Redis client creation should succeed with valid URL");
}

#[tokio::test]
async fn test_config_defaults() {
    use imkitchen::config::{Settings, Environment};
    use std::env;
    
    // Set minimum required env vars
    env::set_var("JWT_SECRET", "this_is_a_valid_jwt_secret_with_32_characters");
    env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
    
    let settings = Settings::new().expect("Settings should load with valid required vars");
    
    // Test defaults
    assert_eq!(settings.server.host, "0.0.0.0");
    assert_eq!(settings.server.port, 3000);
    assert_eq!(settings.redis.url, "redis://localhost:6379");
    assert_eq!(settings.app.environment, Environment::Development);
    
    // Cleanup
    env::remove_var("JWT_SECRET");
    env::remove_var("DATABASE_URL");
}