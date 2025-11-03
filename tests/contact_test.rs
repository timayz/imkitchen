//! Contact form tests for submission, projection, and admin access

mod helpers;

use helpers::{create_test_config, setup_test_databases};
use imkitchen::email::EmailService;
use imkitchen::queries::contact::{
    get_contact_message, list_contact_messages, subscribe_contact_query,
};
use imkitchen_user::aggregate::ContactMessage;
use imkitchen_user::command::{
    Command, MarkContactMessageReadInput, ResolveContactMessageInput, SubmitContactFormInput,
};
use imkitchen_user::event::EventMetadata;
use ulid::Ulid;
use validator::Validate;

/// Test: Visitor can submit contact form without authentication
#[tokio::test]
async fn test_visitor_can_submit_contact_form() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config();
    let email_service = EmailService::new_mock(&config.email)?;

    // Set up subscriptions with unsafe_oneshot for synchronous event processing
    subscribe_contact_query(dbs.queries.clone(), email_service)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command
    let command = Command::new(dbs.evento.clone());

    // Test valid contact form submission (no user_id - public access)
    let input = SubmitContactFormInput {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        subject: "General Inquiry".to_string(),
        message: "This is a test message from the contact form.".to_string(),
    };

    let metadata = EventMetadata {
        user_id: None, // Public form - no authentication
        request_id: Ulid::new().to_string(),
    };

    // Submit contact form
    let message_id = command.submit_contact_form(input, metadata.clone()).await?;

    // Process events synchronously
    let email_service2 = EmailService::new_mock(&config.email)?;
    subscribe_contact_query(dbs.queries.clone(), email_service2)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify message was created in projection
    let message = get_contact_message(&dbs.queries, &message_id).await?;
    assert!(message.is_some(), "Message should exist in projection");

    let message = message.unwrap();
    assert_eq!(message.name, "John Doe");
    assert_eq!(message.email, "john@example.com");
    assert_eq!(message.subject, "General Inquiry");
    assert_eq!(message.status, "new");

    // Verify aggregate state
    let aggregate_result = evento::load::<ContactMessage, _>(&dbs.evento, &message_id).await?;
    assert_eq!(aggregate_result.item.status, Some("new".to_string()));

    Ok(())
}

/// Test: Form validation - invalid email format
#[tokio::test]
async fn test_contact_form_validation_invalid_email() -> anyhow::Result<()> {
    let input = SubmitContactFormInput {
        name: "John Doe".to_string(),
        email: "not-an-email".to_string(),
        subject: "Test".to_string(),
        message: "Test message".to_string(),
    };

    // Validate should fail
    let result = input.validate();
    assert!(result.is_err(), "Validation should fail for invalid email");

    let errors = result.unwrap_err();
    assert!(
        errors.field_errors().contains_key("email"),
        "Email field should have validation error"
    );

    Ok(())
}

/// Test: Form validation - required fields
#[tokio::test]
async fn test_contact_form_validation_required_fields() -> anyhow::Result<()> {
    // Test empty name
    let input1 = SubmitContactFormInput {
        name: "".to_string(),
        email: "test@example.com".to_string(),
        subject: "Test".to_string(),
        message: "Test message".to_string(),
    };
    assert!(input1.validate().is_err(), "Name should be required");

    // Test empty subject
    let input2 = SubmitContactFormInput {
        name: "John Doe".to_string(),
        email: "test@example.com".to_string(),
        subject: "".to_string(),
        message: "Test message".to_string(),
    };
    assert!(input2.validate().is_err(), "Subject should be required");

    // Test empty message
    let input3 = SubmitContactFormInput {
        name: "John Doe".to_string(),
        email: "test@example.com".to_string(),
        subject: "Test".to_string(),
        message: "".to_string(),
    };
    assert!(input3.validate().is_err(), "Message should be required");

    Ok(())
}

/// Test: Query handler projects submission to contact_messages table
#[tokio::test]
async fn test_query_handler_projects_submission() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;
    let config = create_test_config();
    let email_service = EmailService::new_mock(&config.email)?;

    subscribe_contact_query(dbs.queries.clone(), email_service)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    let command = Command::new(dbs.evento.clone());

    // Submit multiple messages
    for i in 1..=3 {
        let input = SubmitContactFormInput {
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            subject: format!("Subject {}", i),
            message: format!("Message {}", i),
        };

        let metadata = EventMetadata {
            user_id: None,
            request_id: Ulid::new().to_string(),
        };

        command.submit_contact_form(input, metadata).await?;
    }

    // Process events
    let email_service2 = EmailService::new(&config.email)?;
    subscribe_contact_query(dbs.queries.clone(), email_service2)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify all messages in projection
    let messages = list_contact_messages(&dbs.queries, None).await?;
    assert_eq!(messages.len(), 3, "Should have 3 messages");

    // Verify they're sorted by created_at (newest first)
    assert!(
        messages[0].created_at >= messages[1].created_at,
        "Messages should be sorted by created_at DESC"
    );

    Ok(())
}

/// Test: Admin can mark message as read
#[tokio::test]
async fn test_admin_can_mark_message_read() -> anyhow::Result<()> {
    use imkitchen::queries::user::subscribe_user_query;
    use imkitchen_user::command::{subscribe_user_command, RegisterUserInput};

    let dbs = setup_test_databases().await?;
    let config = create_test_config();
    let email_service = EmailService::new_mock(&config.email)?;

    // Set up user subscriptions (needed for admin user creation)
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    let command = Command::new(dbs.evento.clone());

    // Create an admin user first
    let admin_input = RegisterUserInput {
        email: "admin@example.com".to_string(),
        password: "AdminPass123".to_string(),
        is_admin: Some(true),
    };
    let admin_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };
    let admin_user_id = command.register_user(admin_input, admin_metadata).await?;

    // Process events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Submit a message
    let input = SubmitContactFormInput {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        subject: "Test Subject".to_string(),
        message: "Test message".to_string(),
    };

    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let message_id = command.submit_contact_form(input, metadata).await?;

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify initial status
    let message = get_contact_message(&dbs.queries, &message_id)
        .await?
        .unwrap();
    assert_eq!(message.status, "new");

    // Admin marks as read
    let mark_read_input = MarkContactMessageReadInput {
        message_id: message_id.clone(),
    };
    let mark_read_metadata = EventMetadata {
        user_id: Some(admin_user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    command
        .mark_contact_message_read(mark_read_input, mark_read_metadata)
        .await?;

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify updated status
    let updated_message = get_contact_message(&dbs.queries, &message_id)
        .await?
        .unwrap();
    assert_eq!(updated_message.status, "read");

    Ok(())
}

/// Test: Admin can resolve message
#[tokio::test]
async fn test_admin_can_resolve_message() -> anyhow::Result<()> {
    use imkitchen::queries::user::subscribe_user_query;
    use imkitchen_user::command::{subscribe_user_command, RegisterUserInput};

    let dbs = setup_test_databases().await?;
    let config = create_test_config();
    let email_service = EmailService::new_mock(&config.email)?;

    // Set up user subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    let command = Command::new(dbs.evento.clone());

    // Create an admin user first
    let admin_input = RegisterUserInput {
        email: "admin@example.com".to_string(),
        password: "AdminPass123".to_string(),
        is_admin: Some(true),
    };
    let admin_user_id = command
        .register_user(
            admin_input,
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await?;

    // Process events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Submit a message
    let input = SubmitContactFormInput {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        subject: "Test Subject".to_string(),
        message: "Test message".to_string(),
    };

    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let message_id = command.submit_contact_form(input, metadata).await?;

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Admin resolves message
    let resolve_input = ResolveContactMessageInput {
        message_id: message_id.clone(),
    };
    let resolve_metadata = EventMetadata {
        user_id: Some(admin_user_id),
        request_id: Ulid::new().to_string(),
    };

    command
        .resolve_contact_message(resolve_input, resolve_metadata)
        .await?;

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service)
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify updated status
    let updated_message = get_contact_message(&dbs.queries, &message_id)
        .await?
        .unwrap();
    assert_eq!(updated_message.status, "resolved");

    Ok(())
}

/// Test: Filter messages by status
#[tokio::test]
async fn test_filter_messages_by_status() -> anyhow::Result<()> {
    use imkitchen::queries::user::subscribe_user_query;
    use imkitchen_user::command::{subscribe_user_command, RegisterUserInput};

    let dbs = setup_test_databases().await?;
    let config = create_test_config();
    let email_service = EmailService::new_mock(&config.email)?;

    // Set up user subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    let command = Command::new(dbs.evento.clone());

    // Create an admin user first
    let admin_input = RegisterUserInput {
        email: "admin@example.com".to_string(),
        password: "AdminPass123".to_string(),
        is_admin: Some(true),
    };
    let admin_user_id = command
        .register_user(
            admin_input,
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await?;

    // Process events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Submit multiple messages
    let mut message_ids = Vec::new();
    for i in 1..=3 {
        let input = SubmitContactFormInput {
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            subject: format!("Subject {}", i),
            message: format!("Message {}", i),
        };

        let metadata = EventMetadata {
            user_id: None,
            request_id: Ulid::new().to_string(),
        };

        let msg_id = command.submit_contact_form(input, metadata).await?;
        message_ids.push(msg_id);
    }

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Mark first message as read
    let mark_read_input = MarkContactMessageReadInput {
        message_id: message_ids[0].clone(),
    };
    command
        .mark_contact_message_read(
            mark_read_input,
            EventMetadata {
                user_id: Some(admin_user_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await?;

    // Process events
    subscribe_contact_query(dbs.queries.clone(), email_service.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Filter by status "new"
    let new_messages = list_contact_messages(&dbs.queries, Some("new")).await?;
    assert_eq!(new_messages.len(), 2, "Should have 2 new messages");

    // Filter by status "read"
    let read_messages = list_contact_messages(&dbs.queries, Some("read")).await?;
    assert_eq!(read_messages.len(), 1, "Should have 1 read message");

    Ok(())
}
