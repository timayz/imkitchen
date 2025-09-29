use imkitchen_notification::{
    config::{SmtpConfig, SmtpSecurity},
    delivery::{EmailDeliveryError, EmailDeliveryService, EmailStatus},
    smtp::SmtpConnectionManager,
    templates::{
        EmailTemplateRenderer, NotificationEmailData, PasswordResetEmailData, RegistrationEmailData,
    },
};
use std::time::Duration;

#[tokio::test]
async fn test_smtp_integration_full_workflow() {
    // Use development fallback configuration for testing
    let config = SmtpConfig::development_fallback();
    let connection_manager = SmtpConnectionManager::new(config, 2).await.unwrap();
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer).unwrap();

    // Test all email types in sequence
    let registration_data = RegistrationEmailData {
        user_name: "Integration Test User".to_string(),
        verification_url: "https://imkitchen.com/verify?token=integration123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let password_reset_data = PasswordResetEmailData {
        user_name: "Integration Test User".to_string(),
        reset_url: "https://imkitchen.com/reset?token=reset123".to_string(),
        app_name: "IMKitchen".to_string(),
        expiry_hours: 24,
    };

    let notification_data = NotificationEmailData {
        user_name: "Integration Test User".to_string(),
        notification_title: "Recipe Ready".to_string(),
        notification_body: "Your meal plan has been updated with new recipes!".to_string(),
        app_name: "IMKitchen".to_string(),
        action_url: Some("https://imkitchen.com/meal-plan".to_string()),
        action_text: Some("View Meal Plan".to_string()),
    };

    // Attempt to send all email types (will fail gracefully if no SMTP server)
    let registration_result = service
        .send_registration_email("integration@example.com", &registration_data)
        .await;
    let password_reset_result = service
        .send_password_reset_email("integration@example.com", &password_reset_data)
        .await;
    let notification_result = service
        .send_notification_email("integration@example.com", &notification_data)
        .await;

    // Check that the service handled all requests appropriately
    for (result, email_type) in [
        (registration_result, "registration"),
        (password_reset_result, "password_reset"),
        (notification_result, "notification"),
    ] {
        match result {
            Ok(status) => {
                assert!(matches!(status, EmailStatus::Sent | EmailStatus::Queued));
                println!("✓ {} email integration test succeeded", email_type);
            }
            Err(EmailDeliveryError::SmtpConnectionFailed(_)) => {
                println!("⚠ {} email integration test - SMTP connection failed (expected in test environment)", email_type);
            }
            Err(EmailDeliveryError::DeliveryFailed(_)) => {
                println!(
                    "⚠ {} email integration test - delivery failed (expected in test environment)",
                    email_type
                );
            }
            Err(e) => {
                panic!(
                    "Unexpected error in {} email integration test: {}",
                    email_type, e
                );
            }
        }
    }

    // Verify service statistics are being tracked
    let stats = service.get_delivery_stats().await;
    assert!(
        stats.total_attempts >= 3,
        "Should have attempted to send 3 emails"
    );
}

#[tokio::test]
async fn test_smtp_connection_scenarios() {
    // Test various SMTP configurations to ensure proper error handling
    let test_configs = vec![
        (
            "Valid development config",
            SmtpConfig::development_fallback(),
            true, // Should create connection manager successfully
        ),
        (
            "Invalid host config",
            SmtpConfig {
                host: "nonexistent.smtp.invalid".to_string(),
                port: 587,
                username: "test@example.com".to_string(),
                password: "password".to_string(),
                from_email: "noreply@imkitchen.com".to_string(),
                from_name: "IMKitchen".to_string(),
                security: SmtpSecurity::StartTls,
                timeout_seconds: 1,
            },
            true, // Should create connection manager (failure happens on actual connection)
        ),
        (
            "Empty host config",
            SmtpConfig {
                host: "".to_string(), // Empty host might not fail validation at config level
                port: 587,
                username: "test@example.com".to_string(),
                password: "password".to_string(),
                from_email: "noreply@imkitchen.com".to_string(),
                from_name: "IMKitchen".to_string(),
                security: SmtpSecurity::None,
                timeout_seconds: 30,
            },
            true, // Config creation might succeed, failure happens at connection time
        ),
    ];

    for (description, config, should_succeed) in test_configs {
        let result = SmtpConnectionManager::new(config, 1).await;

        if should_succeed {
            assert!(
                result.is_ok(),
                "Config '{}' should succeed: {:?}",
                description,
                result.err()
            );
        } else {
            assert!(result.is_err(), "Config '{}' should fail", description);
        }

        println!("✓ SMTP connection scenario test: {}", description);
    }
}

#[tokio::test]
async fn test_smtp_security_configurations() {
    // Test all security configurations
    let security_configs = vec![
        ("None", SmtpSecurity::None, 25),
        ("StartTLS", SmtpSecurity::StartTls, 587),
        ("SSL", SmtpSecurity::Ssl, 465),
    ];

    for (name, security, default_port) in security_configs {
        let config = SmtpConfig {
            host: "localhost".to_string(),
            port: default_port,
            username: "test@example.com".to_string(),
            password: "password".to_string(),
            from_email: "noreply@imkitchen.com".to_string(),
            from_name: "IMKitchen".to_string(),
            security,
            timeout_seconds: 1, // Short timeout for testing
        };

        let connection_manager = SmtpConnectionManager::new(config, 1).await;
        assert!(
            connection_manager.is_ok(),
            "Security config '{}' should be valid",
            name
        );

        println!("✓ SMTP security configuration test: {}", name);
    }
}

#[tokio::test]
async fn test_smtp_health_check() {
    let config = SmtpConfig::development_fallback();
    let connection_manager = SmtpConnectionManager::new(config, 1).await.unwrap();
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer).unwrap();

    // Health check should complete without panicking
    let health_status = service.health_check().await;

    // In test environment, health check may pass or fail depending on SMTP availability
    // The important thing is that it doesn't panic or hang
    println!("✓ SMTP health check completed: {}", health_status);

    // Health check should be reasonably fast
    let start = std::time::Instant::now();
    let _health_status = service.health_check().await;
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_secs(5),
        "Health check should complete quickly"
    );
}

#[tokio::test]
async fn test_smtp_concurrent_operations() {
    let config = SmtpConfig::development_fallback();
    let connection_manager = SmtpConnectionManager::new(config, 3).await.unwrap(); // Pool of 3
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer).unwrap();

    let data = RegistrationEmailData {
        user_name: "Concurrent Test User".to_string(),
        verification_url: "https://imkitchen.com/verify?token=concurrent123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    // Launch multiple concurrent email sends using Arc for sharing
    let service = std::sync::Arc::new(service);
    let mut handles = Vec::new();
    for i in 0..5 {
        let service_clone = service.clone();
        let data_clone = data.clone();
        let email = format!("concurrent{}@example.com", i);

        let handle = tokio::spawn(async move {
            service_clone
                .send_registration_email(&email, &data_clone)
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    // All should complete (either succeed or fail gracefully)
    for (i, result) in results.into_iter().enumerate() {
        let email_result = result.expect("Task should not panic");
        match email_result {
            Ok(_) => println!("✓ Concurrent email {} succeeded", i),
            Err(EmailDeliveryError::SmtpConnectionFailed(_)) => {
                println!(
                    "⚠ Concurrent email {} failed (expected in test environment)",
                    i
                );
            }
            Err(EmailDeliveryError::DeliveryFailed(_)) => {
                println!(
                    "⚠ Concurrent email {} failed (expected in test environment)",
                    i
                );
            }
            Err(e) => panic!("Unexpected error in concurrent email {}: {}", i, e),
        }
    }

    println!("✓ SMTP concurrent operations test completed");
}

#[tokio::test]
async fn test_smtp_error_recovery() {
    // Test recovery from various error conditions
    let config = SmtpConfig {
        host: "localhost".to_string(),
        port: 9999, // Port that should fail
        username: "test@example.com".to_string(),
        password: "password".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::None,
        timeout_seconds: 1,
    };

    let connection_manager = SmtpConnectionManager::new(config, 1).await.unwrap();
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer).unwrap();

    let data = RegistrationEmailData {
        user_name: "Error Recovery Test".to_string(),
        verification_url: "https://imkitchen.com/verify?token=recovery123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    // Multiple attempts should all fail but not crash
    for i in 0..3 {
        let result = service
            .send_registration_email(&format!("recovery{}@example.com", i), &data)
            .await;
        assert!(result.is_err(), "Should fail to connect to invalid port");

        // Service should remain functional after errors
        let health_status = service.health_check().await;
        // Health check may or may not reflect connection issues depending on implementation
        println!("Health check status after error {}: {}", i, health_status);
    }

    // Statistics should track the failed attempts
    let stats = service.get_delivery_stats().await;
    assert!(
        stats.total_attempts >= 3,
        "Should have tracked failed attempts"
    );
    assert!(stats.failed_deliveries >= 3, "Should have tracked failures");

    println!("✓ SMTP error recovery test completed");
}
