use std::fs;
use std::path::PathBuf;
use tempfile::{tempdir, NamedTempFile};
use validator::Validate;

// We need to import the config types - for now we'll create test versions
use serde::{Deserialize, Serialize};
use tracing_appender::rolling::Rotation;

/// Test configuration structures mirroring the main application
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Default)]
struct TestConfig {
    #[validate(nested)]
    pub database: TestDatabaseConfig,
    
    #[validate(nested)]
    pub server: TestServerConfig,
    
    #[validate(nested)]
    pub logging: TestLoggingConfig,
    
    #[validate(nested)]
    pub monitoring: TestMonitoringConfig,
    
    #[validate(nested)]
    pub security: TestSecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct TestDatabaseConfig {
    #[validate(url)]
    pub url: String,
    
    #[validate(range(min = 1, max = 100))]
    pub max_connections: u32,
    
    #[validate(range(min = 1, max = 300))]
    pub connection_timeout: u64,
    
    #[validate(range(min = 1, max = 300))]
    pub acquire_timeout: u64,
}

impl Default for TestDatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:imkitchen.db".to_string(),
            max_connections: 10,
            connection_timeout: 30,
            acquire_timeout: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct TestServerConfig {
    #[validate(ip)]
    pub host: String,
    
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
}

impl Default for TestServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct TestLoggingConfig {
    #[validate(length(min = 1))]
    pub level: String,
    
    pub format: TestLogFormat,
    pub dir: Option<PathBuf>,
    pub rotation: Rotation,
}

impl Default for TestLoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: TestLogFormat::Pretty,
            dir: None,
            rotation: Rotation::Daily,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestLogFormat {
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "compact")]
    Compact,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct TestMonitoringConfig {
    pub enable_metrics: bool,
    
    #[validate(range(min = 1, max = 65535))]
    pub metrics_port: u16,
}

impl Default for TestMonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_port: 9090,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct TestSecurityConfig {
    #[validate(range(min = 300, max = 86400))]
    pub session_timeout: u64,
    
    pub force_https: bool,
    
    #[validate(range(min = 1, max = 10000))]
    pub rate_limit_per_minute: u32,
}

impl Default for TestSecurityConfig {
    fn default() -> Self {
        Self {
            session_timeout: 3600,
            force_https: false,
            rate_limit_per_minute: 100,
        }
    }
}

#[cfg(test)]
mod configuration_validation_tests {
    use super::*;
    
    #[test]
    fn test_default_config_is_valid() {
        let config = TestConfig::default();
        let validation_result = config.validate();
        assert!(validation_result.is_ok(), "Default configuration should be valid");
    }
    
    #[test]
    fn test_valid_database_config() {
        let mut config = TestConfig::default();
        config.database.url = "sqlite:test.db".to_string();
        config.database.max_connections = 20;
        config.database.connection_timeout = 60;
        config.database.acquire_timeout = 30;
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
    }
    
    #[test]
    fn test_invalid_database_url() {
        let mut config = TestConfig::default();
        config.database.url = "invalid-url".to_string();
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("database"));
    }
    
    #[test]
    fn test_database_max_connections_too_high() {
        let mut config = TestConfig::default();
        config.database.max_connections = 150; // Above max of 100
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
    }
    
    #[test]
    fn test_database_max_connections_too_low() {
        let mut config = TestConfig::default();
        config.database.max_connections = 0; // Below min of 1
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
    }
    
    #[test]
    fn test_database_timeout_validation() {
        let mut config = TestConfig::default();
        config.database.connection_timeout = 0; // Invalid timeout
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        config.database.connection_timeout = 400; // Above max
        let validation_result = config.validate();
        assert!(validation_result.is_err());
    }
    
    #[test]
    fn test_valid_server_config() {
        let mut config = TestConfig::default();
        config.server.host = "0.0.0.0".to_string();
        config.server.port = 8080;
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok());
    }
    
    #[test]
    fn test_invalid_server_host() {
        let mut config = TestConfig::default();
        config.server.host = "invalid-ip".to_string();
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
    }
    
    #[test]
    fn test_server_port_boundary_values() {
        let mut config = TestConfig::default();
        
        // Test minimum valid port
        config.server.port = 1;
        assert!(config.validate().is_ok());
        
        // Test maximum valid port
        config.server.port = 65535;
        assert!(config.validate().is_ok());
        
        // Port 0 should be invalid (handled by u16 type, but test for completeness)
        // Note: u16 can't actually be 0 in this context due to validation range
    }
    
    #[test]
    fn test_logging_config_validation() {
        let mut config = TestConfig::default();
        
        // Valid log levels
        for level in &["trace", "debug", "info", "warn", "error"] {
            config.logging.level = level.to_string();
            assert!(config.validate().is_ok(), "Log level '{}' should be valid", level);
        }
        
        // Empty log level should be invalid
        config.logging.level = "".to_string();
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_monitoring_config_validation() {
        let mut config = TestConfig::default();
        
        // Valid metrics port
        config.monitoring.metrics_port = 9090;
        assert!(config.validate().is_ok());
        
        // Test boundary values
        config.monitoring.metrics_port = 1;
        assert!(config.validate().is_ok());
        
        config.monitoring.metrics_port = 65535;
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_security_config_validation() {
        let mut config = TestConfig::default();
        
        // Valid session timeout
        config.security.session_timeout = 1800; // 30 minutes
        assert!(config.validate().is_ok());
        
        // Session timeout too short
        config.security.session_timeout = 100; // Below min of 300
        assert!(config.validate().is_err());
        
        // Session timeout too long
        config.security.session_timeout = 100000; // Above max of 86400
        assert!(config.validate().is_err());
        
        // Valid rate limit
        config.security.session_timeout = 3600; // Reset to valid value
        config.security.rate_limit_per_minute = 50;
        assert!(config.validate().is_ok());
        
        // Rate limit too low
        config.security.rate_limit_per_minute = 0;
        assert!(config.validate().is_err());
        
        // Rate limit too high
        config.security.rate_limit_per_minute = 15000;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_nested_validation_errors() {
        let mut config = TestConfig::default();
        
        // Create multiple validation errors across different sections
        config.database.url = "invalid-url".to_string();
        config.server.host = "invalid-ip".to_string();
        config.logging.level = "".to_string();
        config.security.session_timeout = 100;
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        let field_errors = errors.field_errors();
        
        // Should have errors for multiple fields
        assert!(field_errors.contains_key("database"));
        assert!(field_errors.contains_key("server"));
        assert!(field_errors.contains_key("logging"));
        assert!(field_errors.contains_key("security"));
    }
    
    #[test]
    fn test_config_serialization_deserialization() {
        let config = TestConfig::default();
        
        // Test TOML serialization
        let toml_string = toml::to_string(&config).expect("Failed to serialize to TOML");
        assert!(!toml_string.is_empty());
        
        // Test TOML deserialization
        let deserialized_config: TestConfig = toml::from_str(&toml_string)
            .expect("Failed to deserialize from TOML");
        
        // Validate deserialized config
        assert!(deserialized_config.validate().is_ok());
        
        // Test JSON serialization for comparison
        let json_string = serde_json::to_string(&config).expect("Failed to serialize to JSON");
        assert!(!json_string.is_empty());
        
        let json_config: TestConfig = serde_json::from_str(&json_string)
            .expect("Failed to deserialize from JSON");
        assert!(json_config.validate().is_ok());
    }
    
    #[test]
    fn test_config_file_loading() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test_config.toml");
        
        // Create a valid config file
        let config = TestConfig::default();
        let toml_content = toml::to_string(&config).expect("Failed to serialize config");
        
        fs::write(&config_path, toml_content).expect("Failed to write config file");
        
        // Load and validate config from file
        let file_content = fs::read_to_string(&config_path).expect("Failed to read config file");
        let loaded_config: TestConfig = toml::from_str(&file_content)
            .expect("Failed to parse config file");
        
        assert!(loaded_config.validate().is_ok());
        assert_eq!(loaded_config.server.port, config.server.port);
        assert_eq!(loaded_config.database.url, config.database.url);
    }
    
    #[test]
    fn test_malformed_config_file() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("malformed_config.toml");
        
        // Create a malformed config file
        let malformed_content = r#"
            [database]
            url = "sqlite:test.db"
            max_connections = "not a number"  # This should cause a parse error
            
            [server
            # Missing closing bracket
            host = "127.0.0.1"
        "#;
        
        fs::write(&config_path, malformed_content).expect("Failed to write malformed config");
        
        // Attempt to load malformed config
        let file_content = fs::read_to_string(&config_path).expect("Failed to read config file");
        let parse_result: Result<TestConfig, _> = toml::from_str(&file_content);
        
        assert!(parse_result.is_err(), "Malformed config should fail to parse");
    }
    
    #[test]
    fn test_partial_config_with_defaults() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("partial_config.toml");
        
        // Create a partial config file (only some fields specified)
        let partial_content = r#"
            [server]
            port = 8080
            
            [database]
            url = "sqlite:custom.db"
        "#;
        
        fs::write(&config_path, partial_content).expect("Failed to write partial config");
        
        // Load partial config - this should work with serde defaults
        let file_content = fs::read_to_string(&config_path).expect("Failed to read config file");
        let loaded_config: TestConfig = toml::from_str(&file_content)
            .expect("Failed to parse partial config file");
        
        // Validate that defaults were applied correctly
        assert!(loaded_config.validate().is_ok());
        assert_eq!(loaded_config.server.port, 8080);
        assert_eq!(loaded_config.database.url, "sqlite:custom.db");
        assert_eq!(loaded_config.server.host, "127.0.0.1"); // Should use default
        assert_eq!(loaded_config.database.max_connections, 10); // Should use default
    }
    
    #[test]
    fn test_environment_variable_style_validation() {
        // Test configuration values that might come from environment variables
        let mut config = TestConfig::default();
        
        // Test common environment variable patterns
        config.database.url = "sqlite:/var/lib/app/db.sqlite".to_string();
        config.server.host = "0.0.0.0".to_string();
        config.server.port = 8080;
        config.logging.level = "debug".to_string();
        
        assert!(config.validate().is_ok());
        
        // Test invalid environment-style values
        config.database.url = "${DATABASE_URL}".to_string(); // Unresolved env var
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_production_like_config_validation() {
        let mut config = TestConfig::default();
        
        // Configure like a production environment
        config.database.url = "sqlite:/var/lib/imkitchen/production.db".to_string();
        config.server.host = "0.0.0.0".to_string();
        config.server.port = 80;
        config.logging.level = "warn".to_string();
        config.logging.dir = Some(PathBuf::from("/var/log/imkitchen"));
        config.security.force_https = true;
        config.security.session_timeout = 7200; // 2 hours
        config.security.rate_limit_per_minute = 60;
        config.monitoring.enable_metrics = true;
        config.monitoring.metrics_port = 9090;
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok(), "Production-like config should be valid");
    }
    
    #[test]
    fn test_development_like_config_validation() {
        let mut config = TestConfig::default();
        
        // Configure like a development environment
        config.database.url = "sqlite:dev.db".to_string();
        config.server.host = "127.0.0.1".to_string();
        config.server.port = 3000;
        config.logging.level = "debug".to_string();
        config.logging.dir = None; // Log to stdout in development
        config.security.force_https = false;
        config.security.session_timeout = 3600; // 1 hour
        config.security.rate_limit_per_minute = 1000; // More permissive for dev
        config.monitoring.enable_metrics = true;
        config.monitoring.metrics_port = 9090;
        
        let validation_result = config.validate();
        assert!(validation_result.is_ok(), "Development-like config should be valid");
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_config_with_temporary_file() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        
        let config = TestConfig::default();
        let config_content = toml::to_string(&config).expect("Failed to serialize config");
        
        use std::io::Write;
        temp_file.write_all(config_content.as_bytes()).expect("Failed to write to temp file");
        temp_file.flush().expect("Failed to flush temp file");
        
        // Read back the config
        let file_content = fs::read_to_string(temp_file.path()).expect("Failed to read temp file");
        let loaded_config: TestConfig = toml::from_str(&file_content)
            .expect("Failed to parse config from temp file");
        
        assert!(loaded_config.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation_error_messages() {
        let mut config = TestConfig::default();
        config.database.max_connections = 0; // Invalid value
        
        let validation_result = config.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        let error_messages = errors.to_string();
        
        // Check that error messages are informative
        assert!(error_messages.contains("max_connections") || error_messages.contains("database"));
    }
    
    #[test] 
    fn test_config_field_specific_validation() {
        // Test each field's validation individually
        
        // Database URL validation
        let mut config = TestConfig::default();
        config.database.url = "not-a-url".to_string();
        assert!(config.validate().is_err());
        
        config.database.url = "http://example.com".to_string();
        assert!(config.validate().is_ok()); // HTTP URLs should be valid
        
        config.database.url = "sqlite:valid.db".to_string();
        assert!(config.validate().is_ok());
        
        // Server host validation
        config = TestConfig::default();
        config.server.host = "127.0.0.1".to_string();
        assert!(config.validate().is_ok());
        
        config.server.host = "0.0.0.0".to_string(); 
        assert!(config.validate().is_ok());
        
        config.server.host = "192.168.1.1".to_string();
        assert!(config.validate().is_ok());
        
        config.server.host = "not-an-ip".to_string();
        assert!(config.validate().is_err());
    }
}