use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_config_generate_command() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "config", "generate", "--output", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert!(config_path.exists());
    
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("[database]"));
    assert!(config_content.contains("[server]"));
    assert!(config_content.contains("[logging]"));
    assert!(config_content.contains("[security]"));
    assert!(config_content.contains("[monitoring]"));
}

#[tokio::test] 
async fn test_config_validate_command() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("valid_config.toml");
    
    // Create a valid config file
    let config_content = r#"
[database]
url = "sqlite:test.db"
max_connections = 5
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = "127.0.0.1"
port = 8080
request_timeout = 30
max_body_size = 16777216
enable_cors = false
shutdown_timeout = 30

[logging]
level = "info"
format = "json"
rotation = "daily"
structured = true
console = true
file = true

[security]
session_secret = "this-is-a-very-secure-secret-key-with-32-plus-characters"
session_timeout = 3600
force_https = false
trusted_proxies = []
rate_limit_per_minute = 60

[monitoring]
enable_metrics = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
enable_tracing = true
metrics_interval = 30
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "--config", config_path.to_str().unwrap(), "config", "validate"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("✓ Configuration: Valid"));
}

#[tokio::test]
async fn test_config_show_command() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "config", "show"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Current Configuration:"));
    assert!(stdout.contains("Database URL:"));
    assert!(stdout.contains("Server:"));
    assert!(stdout.contains("Log Level:"));
}

#[tokio::test]
async fn test_environment_variable_precedence() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "config", "show"])
        .env("DATABASE_URL", "sqlite:env_test.db")
        .env("SERVER_PORT", "9000")
        .env("RUST_LOG", "debug")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Database URL: sqlite:env_test.db"));
    assert!(stdout.contains("Server: 0.0.0.0:9000"));
    assert!(stdout.contains("Log Level: debug"));
}

#[tokio::test]
async fn test_cli_argument_precedence() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "--database-url", "sqlite:cli_test.db", "--log-level", "warn", "config", "show"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Database URL: sqlite:cli_test.db"));
    assert!(stdout.contains("Log Level: warn"));
}

#[tokio::test]
async fn test_invalid_config_validation() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("invalid_config.toml");
    
    // Create an invalid config file (port too low)
    let config_content = r#"
[database]
url = ""
max_connections = 0
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = ""
port = 80
request_timeout = 30
max_body_size = 16777216
enable_cors = false
shutdown_timeout = 30

[logging]
level = ""
format = "json"
rotation = "daily"
structured = true
console = true
file = true

[security]
session_secret = "short"
session_timeout = 3600
force_https = false
trusted_proxies = []
rate_limit_per_minute = 60

[monitoring]
enable_metrics = true
metrics_endpoint = ""
health_endpoint = ""
enable_tracing = true
metrics_interval = 30
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "--config", config_path.to_str().unwrap(), "config", "validate"])
        .output()
        .expect("Failed to execute command");
    
    // Should fail validation
    assert!(!output.status.success());
}

#[tokio::test]
async fn test_config_file_priority_order() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("priority_test.toml");
    
    // Create config file with specific values
    let config_content = r#"
[database]
url = "sqlite:file_config.db"
max_connections = 10
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = "192.168.1.1"
port = 4000
request_timeout = 30
max_body_size = 16777216
enable_cors = false
shutdown_timeout = 30

[logging]
level = "trace"
format = "json"
rotation = "daily"
structured = true
console = true
file = true

[security]
session_secret = "file-config-secret-that-is-long-enough-for-validation"
session_timeout = 3600
force_https = false
trusted_proxies = []
rate_limit_per_minute = 60

[monitoring]
enable_metrics = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
enable_tracing = true
metrics_interval = 30
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    // Test that CLI args override both env vars and config file
    let output = std::process::Command::new("cargo")
        .args(&[
            "run", "--", 
            "--config", config_path.to_str().unwrap(),
            "--database-url", "sqlite:cli_override.db",
            "--log-level", "error",
            "config", "show"
        ])
        .env("DATABASE_URL", "sqlite:env_override.db")
        .env("RUST_LOG", "warn")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // CLI args should take precedence
    assert!(stdout.contains("Database URL: sqlite:cli_override.db"));
    assert!(stdout.contains("Log Level: error"));
    
    // Values not overridden should come from config file
    assert!(stdout.contains("Server: 192.168.1.1:4000"));
}

#[tokio::test]
async fn test_web_server_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("web_test.toml");
    
    // Create config with custom web server settings
    let config_content = r#"
[database]
url = "sqlite:web_test.db"
max_connections = 5
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = "127.0.0.1"
port = 8888
request_timeout = 30
max_body_size = 16777216
enable_cors = false
shutdown_timeout = 30

[logging]
level = "info"
format = "pretty"
rotation = "daily"
structured = true
console = true
file = false

[security]
session_secret = "web-test-secret-key-that-is-long-enough-for-validation-purposes"
session_timeout = 3600
force_https = false
trusted_proxies = []
rate_limit_per_minute = 60

[monitoring]
enable_metrics = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
enable_tracing = true
metrics_interval = 30
"#;
    
    fs::write(&config_path, config_content).unwrap();
    
    // Test that health command works with custom config
    let output = std::process::Command::new("cargo")
        .args(&["run", "--", "--config", config_path.to_str().unwrap(), "health"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("✓ Database: Connected"));
    assert!(stdout.contains("✓ Configuration: Valid"));
    assert!(stdout.contains("✓ System: OK"));
}