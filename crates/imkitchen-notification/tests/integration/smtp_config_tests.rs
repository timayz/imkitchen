use imkitchen_notification::config::smtp::{SmtpConfig, SmtpSecurity};
use std::env;

#[tokio::test]
async fn test_smtp_config_validation_valid_config() {
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

    let result = config.validate();
    assert!(result.is_ok(), "Valid SMTP config should pass validation");
}

#[tokio::test]
async fn test_smtp_config_validation_invalid_email() {
    let config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 587,
        username: "invalid-email".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::StartTls,
        timeout_seconds: 30,
    };

    let result = config.validate();
    assert!(result.is_err(), "Invalid email should fail validation");
}

#[tokio::test]
async fn test_smtp_config_validation_invalid_port() {
    let config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 0, // Invalid port (0 is not allowed)
        username: "test@example.com".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::StartTls,
        timeout_seconds: 30,
    };

    let result = config.validate();
    assert!(result.is_err(), "Invalid port should fail validation");
}

#[tokio::test]
async fn test_smtp_config_from_env_valid() {
    // Set up environment variables
    env::set_var("SMTP_HOST", "smtp.gmail.com");
    env::set_var("SMTP_PORT", "587");
    env::set_var("SMTP_USERNAME", "test@example.com");
    env::set_var("SMTP_PASSWORD", "password123");
    env::set_var("SMTP_FROM_EMAIL", "noreply@imkitchen.com");
    env::set_var("SMTP_FROM_NAME", "IMKitchen");

    let result = SmtpConfig::from_env();
    assert!(
        result.is_ok(),
        "Valid environment variables should create config"
    );

    let config = result.unwrap();
    assert_eq!(config.host, "smtp.gmail.com");
    assert_eq!(config.port, 587);
    assert_eq!(config.username, "test@example.com");

    // Clean up
    env::remove_var("SMTP_HOST");
    env::remove_var("SMTP_PORT");
    env::remove_var("SMTP_USERNAME");
    env::remove_var("SMTP_PASSWORD");
    env::remove_var("SMTP_FROM_EMAIL");
    env::remove_var("SMTP_FROM_NAME");
}

#[tokio::test]
async fn test_smtp_config_from_env_missing_required() {
    // Ensure no SMTP env vars are set and save original values
    let original_vars: Vec<_> = [
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USERNAME",
        "SMTP_PASSWORD",
        "SMTP_FROM_EMAIL",
        "SMTP_FROM_NAME",
    ]
    .iter()
    .map(|var| (*var, env::var(var).ok()))
    .collect();

    // Remove all SMTP env vars
    for (var, _) in &original_vars {
        env::remove_var(var);
    }

    let result = SmtpConfig::from_env();
    assert!(result.is_err(), "Missing environment variables should fail");

    // Restore original environment variables
    for (var, value) in original_vars {
        if let Some(val) = value {
            env::set_var(var, val);
        }
    }
}

#[tokio::test]
async fn test_smtp_config_fallback_development() {
    let config = SmtpConfig::development_fallback();

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 1025); // Mailhog default port
    assert!(config.username.is_empty());
    assert!(config.password.is_empty());
    assert_eq!(config.from_email, "dev@imkitchen.local");
    assert_eq!(config.security, SmtpSecurity::None);
}

#[tokio::test]
async fn test_smtp_security_variants() {
    let ssl_config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 465,
        username: "test@example.com".to_string(),
        password: "password123".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::Ssl,
        timeout_seconds: 30,
    };

    assert!(ssl_config.validate().is_ok());
    assert!(ssl_config.requires_encryption());

    let none_config = SmtpConfig {
        host: "localhost".to_string(),
        port: 1025,
        username: "".to_string(),
        password: "".to_string(),
        from_email: "dev@imkitchen.local".to_string(),
        from_name: "IMKitchen Dev".to_string(),
        security: SmtpSecurity::None,
        timeout_seconds: 30,
    };

    assert!(none_config.validate().is_ok());
    assert!(!none_config.requires_encryption());
}
