use imkitchen_notification::config::{SmtpConfig, SmtpSecurity};
use imkitchen_notification::smtp::{SmtpClient, SmtpConnectionError, SmtpConnectionManager};
use tokio::time::Duration;

#[tokio::test]
async fn test_smtp_client_creation_with_starttls() {
    let config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 587,
        username: "test@example.com".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::StartTls,
        timeout_seconds: 30,
    };

    let result = SmtpClient::new(config);
    assert!(
        result.is_ok(),
        "SMTP client creation should succeed with valid config"
    );

    let client = result.unwrap();
    assert_eq!(client.config().host, "smtp.gmail.com");
    assert_eq!(client.config().port, 587);
}

#[tokio::test]
async fn test_smtp_client_creation_with_ssl() {
    let config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 465,
        username: "test@example.com".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::Ssl,
        timeout_seconds: 30,
    };

    let result = SmtpClient::new(config);
    assert!(
        result.is_ok(),
        "SMTP client creation should succeed with SSL config"
    );
}

#[tokio::test]
async fn test_smtp_client_creation_no_security() {
    let config = SmtpConfig {
        host: "localhost".to_string(),
        port: 1025,
        username: "".to_string(),
        password: "".to_string(),
        from_email: "dev@imkitchen.local".to_string(),
        from_name: "IMKitchen Dev".to_string(),
        security: SmtpSecurity::None,
        timeout_seconds: 30,
    };

    let result = SmtpClient::new(config);
    assert!(
        result.is_ok(),
        "SMTP client creation should succeed with no security"
    );
}

#[tokio::test]
async fn test_smtp_connection_manager_creation() {
    let config = SmtpConfig::development_fallback();

    let manager = SmtpConnectionManager::new(config.clone(), 5).await;
    assert!(
        manager.is_ok(),
        "Connection manager should be created successfully"
    );

    let manager = manager.unwrap();
    assert_eq!(manager.max_connections(), 5);
    assert!(manager.is_healthy().await);
}

#[tokio::test]
async fn test_smtp_connection_health_check() {
    let config = SmtpConfig::development_fallback();
    let manager = SmtpConnectionManager::new(config, 1).await.unwrap();

    // Health check should work even if the server isn't running
    // (it just checks connection pool state, not actual connectivity)
    let health = manager.health_check().await;
    assert!(
        health.is_ok(),
        "Health check should not fail on connection pool state"
    );
}

#[tokio::test]
async fn test_smtp_connection_timeout_configuration() {
    let mut config = SmtpConfig::development_fallback();
    config.timeout_seconds = 5;

    let client = SmtpClient::new(config).unwrap();
    let timeout = client.connection_timeout();
    assert_eq!(timeout, Duration::from_secs(5));
}

#[tokio::test]
async fn test_smtp_connection_retry_logic() {
    let config = SmtpConfig::development_fallback();
    let manager = SmtpConnectionManager::new(config, 1).await.unwrap();

    // Test retry logic configuration
    let retry_config = manager.retry_config();
    assert!(retry_config.max_retries > 0);
    assert!(retry_config.base_delay.as_millis() > 0);
}

#[tokio::test]
async fn test_smtp_connection_graceful_error_handling() {
    let config = SmtpConfig {
        host: "nonexistent.smtp.server".to_string(),
        port: 587,
        username: "test@example.com".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::StartTls,
        timeout_seconds: 1, // Very short timeout
    };

    let client = SmtpClient::new(config).unwrap();

    // This should handle the error gracefully rather than panic
    let result = client.test_connection().await;
    assert!(
        result.is_err(),
        "Connection to nonexistent server should fail gracefully"
    );

    match result.err().unwrap() {
        SmtpConnectionError::ConnectionFailed(_) => (),
        SmtpConnectionError::Timeout => (),
        _ => panic!("Should be connection failed or timeout error"),
    }
}
