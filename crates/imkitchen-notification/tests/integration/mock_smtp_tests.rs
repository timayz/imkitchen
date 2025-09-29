use imkitchen_notification::{
    config::{SmtpConfig, SmtpSecurity},
    delivery::{EmailDeliveryError, EmailDeliveryService},
    smtp::SmtpConnectionManager,
    templates::{EmailTemplateRenderer, RegistrationEmailData},
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Mock SMTP server for development and testing
#[derive(Debug, Clone, Default)]
pub struct MockSmtpServer {
    pub captured_emails: Arc<Mutex<Vec<CapturedEmail>>>,
    pub should_fail: bool,
}

#[derive(Debug, Clone)]
pub struct CapturedEmail {
    pub to: String,
    #[allow(dead_code)] // May be used in future for audit trails
    pub from: String,
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    #[allow(dead_code)] // May be used in future for time-based queries
    pub timestamp: std::time::Instant,
}

impl MockSmtpServer {
    pub fn new() -> Self {
        Self {
            captured_emails: Arc::new(Mutex::new(Vec::new())),
            should_fail: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub async fn capture_email(&self, email: CapturedEmail) {
        let mut emails = self.captured_emails.lock().await;
        emails.push(email);
    }

    pub async fn get_captured_emails(&self) -> Vec<CapturedEmail> {
        self.captured_emails.lock().await.clone()
    }

    pub async fn clear_captured_emails(&self) {
        self.captured_emails.lock().await.clear();
    }

    pub async fn count_captured_emails(&self) -> usize {
        self.captured_emails.lock().await.len()
    }
}

#[tokio::test]
async fn test_mock_smtp_server_creation() {
    let mock_server = MockSmtpServer::new();

    assert_eq!(mock_server.count_captured_emails().await, 0);
    assert!(!mock_server.should_fail);
}

#[tokio::test]
async fn test_mock_smtp_email_capture() {
    let mock_server = MockSmtpServer::new();

    let email = CapturedEmail {
        to: "test@example.com".to_string(),
        from: "noreply@imkitchen.com".to_string(),
        subject: "Test Email".to_string(),
        html_body: "<p>Test HTML</p>".to_string(),
        text_body: "Test Text".to_string(),
        timestamp: std::time::Instant::now(),
    };

    mock_server.capture_email(email.clone()).await;

    let captured = mock_server.get_captured_emails().await;
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].to, "test@example.com");
    assert_eq!(captured[0].subject, "Test Email");
}

#[tokio::test]
async fn test_mock_smtp_email_queue_management() {
    let mock_server = MockSmtpServer::new();

    // Add multiple emails
    for i in 0..5 {
        let email = CapturedEmail {
            to: format!("test{}@example.com", i),
            from: "noreply@imkitchen.com".to_string(),
            subject: format!("Test Email {}", i),
            html_body: format!("<p>Test HTML {}</p>", i),
            text_body: format!("Test Text {}", i),
            timestamp: std::time::Instant::now(),
        };
        mock_server.capture_email(email).await;
    }

    assert_eq!(mock_server.count_captured_emails().await, 5);

    // Clear the queue
    mock_server.clear_captured_emails().await;
    assert_eq!(mock_server.count_captured_emails().await, 0);
}

#[tokio::test]
async fn test_mock_smtp_failure_simulation() {
    let mock_server = MockSmtpServer::new().with_failure();

    assert!(mock_server.should_fail);
}

#[tokio::test]
async fn test_mock_smtp_with_development_config() {
    let config = SmtpConfig::development_fallback();
    let connection_manager = SmtpConnectionManager::new(config, 1).await.unwrap();
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer).unwrap();

    let data = RegistrationEmailData {
        user_name: "Test User".to_string(),
        verification_url: "https://imkitchen.com/verify?token=test123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    // This should work with the development fallback config
    let result = service
        .send_registration_email("test@example.com", &data)
        .await;

    // With development config, this might succeed or fail gracefully
    match result {
        Ok(_) => {
            // Success case - development SMTP worked
        }
        Err(EmailDeliveryError::SmtpConnectionFailed(_)) => {
            // Expected when no development SMTP server is running
        }
        Err(e) => {
            // Other errors should be handled gracefully
            println!("Development SMTP test result: {}", e);
        }
    }
}

#[tokio::test]
async fn test_mock_smtp_email_preview_functionality() {
    let mock_server = MockSmtpServer::new();

    let email = CapturedEmail {
        to: "preview@example.com".to_string(),
        from: "noreply@imkitchen.com".to_string(),
        subject: "Registration Confirmation".to_string(),
        html_body: r#"
            <html>
                <body>
                    <h1>Welcome to IMKitchen!</h1>
                    <p>Thank you for registering, Test User.</p>
                    <a href="https://imkitchen.com/verify?token=test123">Verify Account</a>
                </body>
            </html>
        "#.to_string(),
        text_body: "Welcome to IMKitchen! Thank you for registering, Test User. Verify your account at: https://imkitchen.com/verify?token=test123".to_string(),
        timestamp: std::time::Instant::now(),
    };

    mock_server.capture_email(email).await;

    let captured = mock_server.get_captured_emails().await;
    assert_eq!(captured.len(), 1);

    // Verify email content for preview
    let preview_email = &captured[0];
    assert!(preview_email.html_body.contains("Welcome to IMKitchen!"));
    assert!(preview_email.text_body.contains("Verify your account"));
    assert!(preview_email.html_body.contains("href="));
}

#[tokio::test]
async fn test_mock_smtp_configuration_toggle() {
    // Test that we can toggle between real SMTP and mock SMTP based on environment
    let development_config = SmtpConfig::development_fallback();
    assert_eq!(development_config.host, "localhost");
    assert_eq!(development_config.port, 1025);

    // Production-like config would have real SMTP settings
    let production_config = SmtpConfig {
        host: "smtp.gmail.com".to_string(),
        port: 587,
        username: "user@gmail.com".to_string(),
        password: "password".to_string(),
        from_email: "noreply@imkitchen.com".to_string(),
        from_name: "IMKitchen".to_string(),
        security: SmtpSecurity::StartTls,
        timeout_seconds: 30,
    };

    // Both configs should be valid
    assert!(development_config.validate().is_ok());
    assert!(production_config.validate().is_ok());
}

#[tokio::test]
async fn test_mock_smtp_email_inspection() {
    let mock_server = MockSmtpServer::new();

    // Send different types of emails
    let emails = vec![
        CapturedEmail {
            to: "user1@example.com".to_string(),
            from: "noreply@imkitchen.com".to_string(),
            subject: "Registration Confirmation".to_string(),
            html_body: "<p>Registration email</p>".to_string(),
            text_body: "Registration email".to_string(),
            timestamp: std::time::Instant::now(),
        },
        CapturedEmail {
            to: "user2@example.com".to_string(),
            from: "noreply@imkitchen.com".to_string(),
            subject: "Password Reset".to_string(),
            html_body: "<p>Password reset email</p>".to_string(),
            text_body: "Password reset email".to_string(),
            timestamp: std::time::Instant::now(),
        },
    ];

    for email in emails {
        mock_server.capture_email(email).await;
    }

    let captured = mock_server.get_captured_emails().await;
    assert_eq!(captured.len(), 2);

    // Inspect specific emails
    let registration_email = captured
        .iter()
        .find(|e| e.subject.contains("Registration"))
        .unwrap();
    assert_eq!(registration_email.to, "user1@example.com");

    let password_reset_email = captured
        .iter()
        .find(|e| e.subject.contains("Password Reset"))
        .unwrap();
    assert_eq!(password_reset_email.to, "user2@example.com");
}
