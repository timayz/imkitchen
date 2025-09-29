use imkitchen_notification::templates::{
    EmailTemplateError, EmailTemplateRenderer, NotificationEmailData, PasswordResetEmailData,
    RegistrationEmailData,
};

#[tokio::test]
async fn test_registration_email_template_rendering() {
    let data = RegistrationEmailData {
        user_name: "John Doe".to_string(),
        verification_url: "https://imkitchen.com/verify?token=abc123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let result = renderer.render_registration_email(&data).await;

    assert!(
        result.is_ok(),
        "Registration email template should render successfully"
    );

    let email = result.unwrap();
    assert!(!email.subject.is_empty());
    assert!(!email.html_body.is_empty());
    assert!(!email.text_body.is_empty());

    // Check that the template contains expected content
    assert!(email.html_body.contains("John Doe"));
    assert!(email.html_body.contains("abc123"));
    assert!(email.text_body.contains("John Doe"));
    assert!(email.text_body.contains("abc123"));
}

#[tokio::test]
async fn test_password_reset_email_template_rendering() {
    let data = PasswordResetEmailData {
        user_name: "Jane Smith".to_string(),
        reset_url: "https://imkitchen.com/reset?token=xyz789".to_string(),
        expiry_hours: 24,
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let result = renderer.render_password_reset_email(&data).await;

    assert!(
        result.is_ok(),
        "Password reset email template should render successfully"
    );

    let email = result.unwrap();
    assert!(!email.subject.is_empty());
    assert!(!email.html_body.is_empty());
    assert!(!email.text_body.is_empty());

    // Check that the template contains expected content
    assert!(email.html_body.contains("Jane Smith"));
    assert!(email.html_body.contains("xyz789"));
    assert!(email.html_body.contains("24"));
    assert!(email.text_body.contains("Jane Smith"));
    assert!(email.text_body.contains("xyz789"));
}

#[tokio::test]
async fn test_notification_email_template_rendering() {
    let data = NotificationEmailData {
        user_name: "Bob Wilson".to_string(),
        notification_title: "Weekly Meal Plan Ready".to_string(),
        notification_body: "Your meal plan for next week is ready to view.".to_string(),
        action_url: Some("https://imkitchen.com/meal-plan/123".to_string()),
        action_text: Some("View Meal Plan".to_string()),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let result = renderer.render_notification_email(&data).await;

    assert!(
        result.is_ok(),
        "Notification email template should render successfully"
    );

    let email = result.unwrap();
    assert!(!email.subject.is_empty());
    assert!(!email.html_body.is_empty());
    assert!(!email.text_body.is_empty());

    // Check that the template contains expected content
    assert!(email.html_body.contains("Bob Wilson"));
    assert!(email.html_body.contains("Weekly Meal Plan Ready"));
    assert!(email.html_body.contains("View Meal Plan"));
    assert!(email.text_body.contains("Bob Wilson"));
    assert!(email.text_body.contains("Weekly Meal Plan Ready"));
}

#[tokio::test]
async fn test_email_template_validation_html_structure() {
    let data = RegistrationEmailData {
        user_name: "Test User".to_string(),
        verification_url: "https://imkitchen.com/verify?token=test".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let email = renderer.render_registration_email(&data).await.unwrap();

    // Validate HTML structure
    assert!(email.html_body.contains("<!DOCTYPE html>"));
    assert!(email.html_body.contains("<html"));
    assert!(email.html_body.contains("<head>"));
    assert!(email.html_body.contains("<body>"));
    assert!(email.html_body.contains("</html>"));

    // Validate text alternative exists
    assert!(!email.text_body.contains("<"));
    assert!(!email.text_body.contains(">"));
}

#[tokio::test]
async fn test_email_template_xss_prevention() {
    let data = RegistrationEmailData {
        user_name: "<script>alert('xss')</script>".to_string(),
        verification_url: "https://imkitchen.com/verify?token=<script>".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let email = renderer.render_registration_email(&data).await.unwrap();

    // Ensure scripts are escaped in HTML
    assert!(!email.html_body.contains("<script>"));

    // Askama uses numeric HTML entities for escaping
    assert!(
        email.html_body.contains("&#60;script&#62;"),
        "Scripts should be HTML escaped"
    );
}

#[tokio::test]
async fn test_email_template_missing_data() {
    let data = RegistrationEmailData {
        user_name: "".to_string(), // Empty name should be handled gracefully
        verification_url: "https://imkitchen.com/verify?token=abc123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();
    let result = renderer.render_registration_email(&data).await;

    // Should either succeed with default handling or fail gracefully
    match result {
        Ok(email) => {
            assert!(!email.html_body.is_empty());
            assert!(!email.text_body.is_empty());
        }
        Err(EmailTemplateError::MissingRequiredData(_)) => {
            // This is acceptable - we caught the empty user name
        }
        Err(EmailTemplateError::ValidationFailed(_)) => {
            // This is also acceptable - validation caught the empty user name
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[tokio::test]
async fn test_email_template_preview_functionality() {
    let renderer = EmailTemplateRenderer::new();

    // Test preview generation for development
    let preview_data = renderer.generate_preview_data().await;
    assert!(
        preview_data.is_ok(),
        "Should be able to generate preview data"
    );

    let previews = preview_data.unwrap();
    assert!(!previews.registration.html_body.is_empty());
    assert!(!previews.password_reset.html_body.is_empty());
    assert!(!previews.notification.html_body.is_empty());
}

#[tokio::test]
async fn test_email_template_localization_support() {
    let data = RegistrationEmailData {
        user_name: "John Doe".to_string(),
        verification_url: "https://imkitchen.com/verify?token=abc123".to_string(),
        app_name: "IMKitchen".to_string(),
    };

    let renderer = EmailTemplateRenderer::new();

    // Test default (English) rendering
    let email_en = renderer.render_registration_email(&data).await.unwrap();
    assert!(email_en.subject.contains("Welcome") || email_en.subject.contains("Verify"));

    // Test that templates support different content (foundation for future localization)
    assert!(!email_en.html_body.is_empty());
    assert!(!email_en.text_body.is_empty());
}
