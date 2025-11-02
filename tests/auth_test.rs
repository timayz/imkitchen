//! Authentication tests for user registration and login

mod helpers;

use helpers::setup_test_databases;
use imkitchen::queries::user::{get_user, get_user_by_email, subscribe_user_query};
use imkitchen_user::aggregate::User;
use imkitchen_user::command::{subscribe_user_command, Command, LoginUserInput, RegisterUserInput};
use imkitchen_user::event::EventMetadata;
use ulid::Ulid;
use validator::Validate;

/// Test: User can register with valid email and password
#[tokio::test]
async fn test_user_can_register_with_valid_credentials() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions with unsafe_oneshot for synchronous event processing
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command
    let command = Command::new(dbs.evento.clone());

    // Test valid registration
    let input = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "Password123".to_string(),
    };

    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    // Register user
    let user_id = command.register_user(input, metadata.clone()).await?;

    // Process events synchronously
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify user was created in projection
    let user = get_user(&dbs.queries, &user_id).await?;
    assert!(user.is_some(), "User should exist in projection");
    let user = user.unwrap();
    assert_eq!(user.email, "test@example.com");
    assert!(!user.is_admin);
    assert!(!user.is_suspended);

    // Verify aggregate state
    let aggregate_result = evento::load::<User, _>(&dbs.evento, &user_id).await?;
    assert_eq!(aggregate_result.item.status, Some("active".to_string()));

    Ok(())
}

/// Test: Registration fails with invalid email format
#[tokio::test]
async fn test_registration_fails_with_invalid_email() -> anyhow::Result<()> {
    let input = RegisterUserInput {
        email: "not-an-email".to_string(),
        password: "Password123".to_string(),
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

/// Test: Registration fails with weak password (less than 8 characters)
#[tokio::test]
async fn test_registration_fails_with_weak_password() -> anyhow::Result<()> {
    let input = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "Short1".to_string(), // Only 6 characters
    };

    // Validate should fail
    let result = input.validate();
    assert!(result.is_err(), "Validation should fail for weak password");

    let errors = result.unwrap_err();
    assert!(
        errors.field_errors().contains_key("password"),
        "Password field should have validation error"
    );

    Ok(())
}

/// Test: Registration fails without password complexity (uppercase, lowercase, number)
#[tokio::test]
async fn test_registration_fails_without_password_complexity() -> anyhow::Result<()> {
    // Test: no uppercase
    let input1 = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "password123".to_string(), // No uppercase
    };
    assert!(
        input1.validate().is_err(),
        "Should fail without uppercase letter"
    );

    // Test: no lowercase
    let input2 = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "PASSWORD123".to_string(), // No lowercase
    };
    assert!(
        input2.validate().is_err(),
        "Should fail without lowercase letter"
    );

    // Test: no number
    let input3 = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "PasswordOnly".to_string(), // No number
    };
    assert!(input3.validate().is_err(), "Should fail without number");

    // Test: valid password with all requirements
    let input4 = RegisterUserInput {
        email: "test@example.com".to_string(),
        password: "Password123".to_string(), // Has all requirements
    };
    assert!(
        input4.validate().is_ok(),
        "Should pass with all requirements"
    );

    Ok(())
}

/// Test: Email uniqueness validation prevents duplicate registration
#[tokio::test]
async fn test_email_uniqueness_prevents_duplicate_registration() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command
    let command = Command::new(dbs.evento.clone());

    // Register first user
    let input1 = RegisterUserInput {
        email: "duplicate@example.com".to_string(),
        password: "Password123".to_string(),
    };

    let metadata1 = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let user_id1 = command.register_user(input1, metadata1).await?;

    // Process events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify first user succeeded
    let user1 = get_user(&dbs.queries, &user_id1).await?;
    assert!(user1.is_some(), "First user should be created");

    // Try to register second user with same email
    let input2 = RegisterUserInput {
        email: "duplicate@example.com".to_string(),
        password: "DifferentPass123".to_string(),
    };

    let metadata2 = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let user_id2 = command.register_user(input2, metadata2).await?;

    // Process events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify second user failed
    let aggregate2_result = evento::load::<User, _>(&dbs.evento, &user_id2).await?;
    assert_eq!(
        aggregate2_result.item.status,
        Some("failed".to_string()),
        "Second user registration should fail due to duplicate email"
    );

    // Verify only one user exists in query database
    let user2 = get_user(&dbs.queries, &user_id2).await?;
    assert!(
        user2.is_none(),
        "Failed registration should not create user in projection"
    );

    Ok(())
}

/// Test: User can login with correct credentials
#[tokio::test]
async fn test_user_can_login_with_correct_credentials() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command
    let command = Command::new(dbs.evento.clone());

    // Register user
    let register_input = RegisterUserInput {
        email: "login@example.com".to_string(),
        password: "Password123".to_string(),
    };

    let register_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let _user_id = command
        .register_user(register_input, register_metadata)
        .await?;

    // Process registration events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Get user from query database (like login route does)
    let user = get_user_by_email(&dbs.queries, "login@example.com").await?;
    assert!(user.is_some(), "User should exist in query database");
    let user = user.unwrap();

    // Now attempt login
    let login_input = LoginUserInput {
        email: "login@example.com".to_string(),
        password: "Password123".to_string(),
    };

    let login_metadata = EventMetadata {
        user_id: Some(user.id.clone()),
        request_id: Ulid::new().to_string(),
    };

    // Login should succeed
    let login_result = command
        .login_user(
            login_input,
            user.id.clone(),
            user.hashed_password,
            login_metadata,
        )
        .await;
    assert!(
        login_result.is_ok(),
        "Login should succeed with correct credentials"
    );

    Ok(())
}

/// Test: Login fails with incorrect password
#[tokio::test]
async fn test_login_fails_with_incorrect_password() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command
    let command = Command::new(dbs.evento.clone());

    // Register user
    let register_input = RegisterUserInput {
        email: "wrongpass@example.com".to_string(),
        password: "CorrectPass123".to_string(),
    };

    let register_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    command
        .register_user(register_input, register_metadata)
        .await?;

    // Process registration events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Get user from query database
    let user = get_user_by_email(&dbs.queries, "wrongpass@example.com").await?;
    assert!(user.is_some(), "User should exist in query database");
    let user = user.unwrap();

    // Try login with wrong password
    let login_input = LoginUserInput {
        email: "wrongpass@example.com".to_string(),
        password: "WrongPassword123".to_string(),
    };

    let login_metadata = EventMetadata {
        user_id: Some(user.id.clone()),
        request_id: Ulid::new().to_string(),
    };

    // Login should fail
    let login_result = command
        .login_user(login_input, user.id, user.hashed_password, login_metadata)
        .await;
    assert!(
        login_result.is_err(),
        "Login should fail with incorrect password"
    );

    Ok(())
}

/// Test: Protected route accessible with valid JWT (tested via middleware)
#[tokio::test]
async fn test_jwt_token_generation_and_validation() -> anyhow::Result<()> {
    use imkitchen::auth::{generate_token, validate_token};

    let secret = "test-secret-key-for-jwt-testing";
    let user_id = "test-user-123";
    let lifetime_seconds = 3600; // 1 hour

    // Generate token
    let token = generate_token(user_id.to_string(), false, secret, lifetime_seconds)?;
    assert!(!token.is_empty(), "Token should be generated");

    // Verify token
    let auth_user = validate_token(&token, secret)?;
    assert_eq!(auth_user.user_id, user_id, "User ID should match");
    assert!(!auth_user.is_admin, "User should not be admin");

    // Test admin token
    let admin_token = generate_token(user_id.to_string(), true, secret, lifetime_seconds)?;
    let admin_user = validate_token(&admin_token, secret)?;
    assert!(admin_user.is_admin, "User should be admin");

    Ok(())
}

/// Test: JWT verification fails with invalid token
#[tokio::test]
async fn test_jwt_verification_fails_with_invalid_token() -> anyhow::Result<()> {
    use imkitchen::auth::validate_token;

    let secret = "test-secret-key";
    let invalid_token = "invalid.jwt.token";

    let result = validate_token(invalid_token, secret);
    assert!(
        result.is_err(),
        "Verification should fail with invalid token"
    );

    Ok(())
}
