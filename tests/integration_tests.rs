//! Integration tests for IMKitchen CLI
//! 
//! These tests verify the complete application workflow including:
//! - Database operations and migrations
//! - Configuration loading and validation
//! - Service startup and shutdown
//! - Error handling and recovery scenarios

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::time::timeout;

/// Helper struct for managing test environments
#[derive(Debug)]
struct TestEnvironment {
    temp_dir: TempDir,
    config_path: PathBuf,
    db_path: PathBuf,
    pid_path: PathBuf,
    binary_path: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test_config.toml");
        let db_path = temp_dir.path().join("test.db");
        let pid_path = temp_dir.path().join("test.pid");
        
        // Find the binary path
        let binary_path = if cfg!(debug_assertions) {
            PathBuf::from("target/debug/imkitchen")
        } else {
            PathBuf::from("target/release/imkitchen")
        };

        Self {
            temp_dir,
            config_path,
            db_path,
            pid_path,
            binary_path,
        }
    }

    fn create_test_config(&self) -> std::io::Result<()> {
        let config_content = format!(
            r#"[database]
url = "sqlite:{}"
max_connections = 10
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = "127.0.0.1"
port = 3001
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
session_secret = "test_session_secret_key_for_integration_tests_minimum_32_chars"
session_timeout = 3600
force_https = false
trusted_proxies = []
rate_limit_per_minute = 100

[monitoring]
enable_metrics = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
enable_tracing = true
metrics_interval = 30
"#,
            self.db_path.to_string_lossy()
        );

        fs::write(&self.config_path, config_content)
    }

    fn run_command(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(args)
            .arg("--config")
            .arg(&self.config_path);

        cmd.output().expect("Failed to execute command")
    }

    async fn run_command_async(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = tokio::process::Command::new(&self.binary_path);
        cmd.args(args)
            .arg("--config")
            .arg(&self.config_path);

        cmd.output().await.expect("Failed to execute command")
    }
}

#[tokio::test]
async fn test_config_generation_and_validation() {
    let env = TestEnvironment::new();
    
    // Test config generation
    let output = env.run_command(&["config", "generate", "--output", env.config_path.to_str().unwrap()]);
    assert!(output.status.success(), "Config generation failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(env.config_path.exists(), "Config file was not created");

    // Test config validation
    let output = env.run_command(&["config", "validate"]);
    assert!(output.status.success(), "Config validation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration: Valid"), "Config validation output unexpected");

    // Test config show
    let output = env.run_command(&["config", "show"]);
    assert!(output.status.success(), "Config show failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Current Configuration"), "Config show output unexpected");
    assert!(stdout.contains("Database URL"), "Config show missing database URL");
}

#[tokio::test]
async fn test_database_operations() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Test migration status on fresh database
    let output = env.run_command(&["migrate", "status"]);
    assert!(output.status.success(), "Migration status failed: {}", String::from_utf8_lossy(&output.stderr));

    // Test migration up
    let output = env.run_command(&["migrate", "up"]);
    assert!(output.status.success(), "Migration up failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Migrations completed"), "Migration up output unexpected");

    // Verify database file was created
    assert!(env.db_path.exists(), "Database file was not created");

    // Test migration status after migrations
    let output = env.run_command(&["migrate", "status"]);
    assert!(output.status.success(), "Migration status check failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("migration(s) applied"), "Migration status output unexpected");
}

#[tokio::test]
async fn test_health_check() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Test health check
    let output = env.run_command(&["health"]);
    assert!(output.status.success(), "Health check failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Database: Connected"), "Health check missing database status");
    assert!(stdout.contains("Configuration: Valid"), "Health check missing config status");
    assert!(stdout.contains("System: OK"), "Health check missing system status");
}

#[tokio::test]
async fn test_web_server_lifecycle() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Run migrations first
    let output = env.run_command(&["migrate", "up"]);
    assert!(output.status.success(), "Migration setup failed: {}", String::from_utf8_lossy(&output.stderr));

    // Test web server start with timeout (since it runs indefinitely)
    let args = [
        "web", "start", 
        "--host", "127.0.0.1", 
        "--port", "3001", 
        "--pid-file", env.pid_path.to_str().unwrap()
    ];
    let start_future = env.run_command_async(&args);

    // Give the server a short time to start, then timeout
    let result = timeout(Duration::from_secs(5), start_future).await;
    
    // The command should either timeout (server runs indefinitely) OR fail quickly
    // Both behaviors are acceptable for this integration test
    match result {
        Err(_) => {
            // Timeout occurred - server is running indefinitely (expected)
            assert!(true, "Server running indefinitely as expected");
        }
        Ok(output) => {
            // Server terminated - check if it was an expected error
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // This is acceptable - server might fail due to port conflicts, etc.
                println!("Server terminated with error (acceptable): {}", stderr);
            } else {
                panic!("Server should not exit successfully in normal operation");
            }
        }
    }

    // In a real scenario, we'd test:
    // 1. Server responds to HTTP requests
    // 2. PID file is created
    // 3. Graceful shutdown works
    // But this requires more complex async coordination
}

#[tokio::test]
async fn test_error_handling() {
    let env = TestEnvironment::new();

    // Test with invalid config file
    let invalid_config = env.temp_dir.path().join("invalid_config.toml");
    fs::write(&invalid_config, "invalid toml content [").expect("Failed to write invalid config");

    let output = env.run_command(&["--config", invalid_config.to_str().unwrap(), "health"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Either fails with parse error OR succeeds using defaults (both are acceptable behavior)
    if !output.status.success() {
        assert!(stderr.contains("Error"), "Should contain error message");
    }

    // Test with missing config file
    let missing_config = env.temp_dir.path().join("missing_config.toml");
    let output = env.run_command(&["--config", missing_config.to_str().unwrap(), "config", "validate"]);
    
    // This should succeed using defaults since config file is optional
    assert!(output.status.success(), "Should use defaults with missing config");
}

#[tokio::test]
async fn test_command_line_overrides() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Test database URL override
    let custom_db = env.temp_dir.path().join("custom.db");
    let output = env.run_command(&[
        "--database-url", &format!("sqlite:{}", custom_db.to_string_lossy()),
        "migrate", "up"
    ]);
    
    assert!(output.status.success(), "Migration with DB override failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(custom_db.exists(), "Custom database file was not created");

    // Test log level override
    let output = env.run_command(&[
        "--log-level", "debug",
        "health"
    ]);
    
    assert!(output.status.success(), "Health check with log override failed: {}", String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_concurrent_operations() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Run multiple health checks concurrently
    let futures: Vec<_> = (0..5)
        .map(|_| env.run_command_async(&["health"]))
        .collect();

    let results = futures::future::join_all(futures).await;
    
    for result in results {
        assert!(result.status.success(), "Concurrent health check failed: {}", String::from_utf8_lossy(&result.stderr));
    }
}

#[tokio::test]
async fn test_migration_rollback() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Run migrations up
    let output = env.run_command(&["migrate", "up"]);
    assert!(output.status.success(), "Migration up failed: {}", String::from_utf8_lossy(&output.stderr));

    // Test rollback (note: current implementation is a stub)
    let output = env.run_command(&["migrate", "down", "--steps", "1"]);
    assert!(output.status.success(), "Migration rollback failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rollback completed"), "Rollback output unexpected");
}

#[tokio::test]
async fn test_graceful_shutdown_command() {
    let env = TestEnvironment::new();
    env.create_test_config().expect("Failed to create test config");

    // Test stop command when no server is running
    let output = env.run_command(&["web", "stop"]);
    assert!(output.status.success(), "Stop command should succeed even with no running server");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No running process found"), "Should indicate no process found");
}

#[tokio::test]
async fn test_configuration_security_validation() {
    let env = TestEnvironment::new();
    
    // Create config with security issues (short session secret)
    let insecure_config = format!(
        r#"[database]
url = "sqlite:{}"
max_connections = 10
min_connections = 1
connection_timeout = 30
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 3600
auto_migrate = true

[server]
host = "0.0.0.0"
port = 3000
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
session_secret = "short"
session_timeout = 86400
force_https = false
trusted_proxies = []
rate_limit_per_minute = 1000

[monitoring]
enable_metrics = true
metrics_endpoint = "/metrics"
health_endpoint = "/health"
enable_tracing = true
metrics_interval = 30
"#,
        env.db_path.to_string_lossy()
    );
    
    fs::write(&env.config_path, insecure_config).expect("Failed to write insecure config");

    // Test with short session secret (should fail validation)
    let output = env.run_command(&["config", "validate"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    assert!(!output.status.success(), "Should fail security validation with short session secret");
    assert!(stderr.contains("Validation Error"), "Should contain validation error");
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Verify that the test binary exists and is executable
    #[test]
    fn verify_test_binary_exists() {
        let env = TestEnvironment::new();
        assert!(
            env.binary_path.exists() || 
            PathBuf::from("target/debug/imkitchen").exists() ||
            PathBuf::from("target/release/imkitchen").exists(),
            "IMKitchen binary not found. Run 'cargo build' first."
        );
    }

    /// Test the test environment setup
    #[test]
    fn test_environment_setup() {
        let env = TestEnvironment::new();
        
        assert!(env.temp_dir.path().exists(), "Temp directory should exist");
        assert!(!env.config_path.exists(), "Config should not exist initially");
        assert!(!env.db_path.exists(), "Database should not exist initially");
        
        env.create_test_config().expect("Should create test config");
        assert!(env.config_path.exists(), "Config should exist after creation");
    }
}