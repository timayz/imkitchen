//! Authentication tests for user registration and login

mod helpers;

use helpers::setup_test_databases;
use imkitchen::queries::user::{
    get_user, get_user_by_email, get_user_profile, subscribe_user_query,
};
use imkitchen_user::aggregate::User;
use imkitchen_user::command::{
    subscribe_user_command, Command, LoginUserInput, RegisterUserInput, UpdateProfileInput,
};
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
        is_admin: None,
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
        is_admin: None,
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
        is_admin: None,
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
        is_admin: None,
        password: "password123".to_string(), // No uppercase
    };
    assert!(
        input1.validate().is_err(),
        "Should fail without uppercase letter"
    );

    // Test: no lowercase
    let input2 = RegisterUserInput {
        email: "test@example.com".to_string(),
        is_admin: None,
        password: "PASSWORD123".to_string(), // No lowercase
    };
    assert!(
        input2.validate().is_err(),
        "Should fail without lowercase letter"
    );

    // Test: no number
    let input3 = RegisterUserInput {
        email: "test@example.com".to_string(),
        is_admin: None,
        password: "PasswordOnly".to_string(), // No number
    };
    assert!(input3.validate().is_err(), "Should fail without number");

    // Test: valid password with all requirements
    let input4 = RegisterUserInput {
        email: "test@example.com".to_string(),
        is_admin: None,
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
        is_admin: None,
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
        is_admin: None,
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
        is_admin: None,
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
        is_admin: None,
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

/// Test: User can update dietary restrictions
#[tokio::test]
async fn test_user_can_update_dietary_restrictions() -> anyhow::Result<()> {
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

    // Register user first
    let register_input = RegisterUserInput {
        email: "profile@example.com".to_string(),
        is_admin: None,
        password: "Password123".to_string(),
    };

    let register_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let user_id = command
        .register_user(register_input, register_metadata)
        .await?;

    // Process registration events
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Update profile
    let profile_input = UpdateProfileInput {
        dietary_restrictions: vec!["Vegetarian".to_string(), "Gluten-free".to_string()],
        cuisine_variety_weight: 0.8,
        household_size: Some(4),
    };

    let profile_metadata = EventMetadata {
        user_id: Some(user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    command
        .update_profile(user_id.clone(), profile_input, profile_metadata)
        .await?;

    // Process profile update events
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify profile was updated in projection
    let profile = get_user_profile(&dbs.queries, &user_id).await?;
    assert_eq!(profile.dietary_restrictions.len(), 2);
    assert!(profile
        .dietary_restrictions
        .contains(&"Vegetarian".to_string()));
    assert!(profile
        .dietary_restrictions
        .contains(&"Gluten-free".to_string()));
    assert_eq!(profile.cuisine_variety_weight, 0.8);
    assert_eq!(profile.household_size, Some(4));

    Ok(())
}

/// Test: Cuisine variety weight validation (must be between 0.0 and 1.0)
#[tokio::test]
async fn test_cuisine_variety_weight_validation() -> anyhow::Result<()> {
    let _dbs = setup_test_databases().await?;

    // Test: weight below 0.0
    let input1 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: -0.1,
        household_size: None,
    };
    assert!(input1.validate().is_err(), "Should fail with weight < 0.0");

    // Test: weight above 1.0
    let input2 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 1.1,
        household_size: None,
    };
    assert!(input2.validate().is_err(), "Should fail with weight > 1.0");

    // Test: valid weight at boundaries
    let input3 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 0.0,
        household_size: None,
    };
    assert!(input3.validate().is_ok(), "Should pass with weight = 0.0");

    let input4 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 1.0,
        household_size: None,
    };
    assert!(input4.validate().is_ok(), "Should pass with weight = 1.0");

    // Test: valid weight in middle
    let input5 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 0.7,
        household_size: None,
    };
    assert!(input5.validate().is_ok(), "Should pass with weight = 0.7");

    Ok(())
}

/// Test: Household size validation (must be > 0 if provided)
#[tokio::test]
async fn test_household_size_validation() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    let command = Command::new(dbs.evento.clone());

    // Test: household_size = 0 should fail
    let input1 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 0.7,
        household_size: Some(0),
    };

    let metadata1 = EventMetadata {
        user_id: Some("test-user-1".to_string()),
        request_id: Ulid::new().to_string(),
    };

    let result1 = command
        .update_profile("test-user-1".to_string(), input1, metadata1)
        .await;
    assert!(result1.is_err(), "Should fail with household_size = 0");

    // Test: household_size < 0 should fail
    let input2 = UpdateProfileInput {
        dietary_restrictions: vec![],
        cuisine_variety_weight: 0.7,
        household_size: Some(-1),
    };

    let metadata2 = EventMetadata {
        user_id: Some("test-user-2".to_string()),
        request_id: Ulid::new().to_string(),
    };

    let result2 = command
        .update_profile("test-user-2".to_string(), input2, metadata2)
        .await;
    assert!(result2.is_err(), "Should fail with household_size < 0");

    Ok(())
}

/// Test: Profile query returns defaults when profile doesn't exist
#[tokio::test]
async fn test_profile_query_returns_defaults_when_not_exists() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    let non_existent_user_id = "non-existent-user";

    // Query for profile that doesn't exist
    let profile = get_user_profile(&dbs.queries, non_existent_user_id).await?;

    // Should return defaults
    assert_eq!(profile.user_id, non_existent_user_id);
    assert_eq!(profile.dietary_restrictions.len(), 0);
    assert_eq!(profile.cuisine_variety_weight, 0.7);
    assert_eq!(profile.household_size, None);
    assert!(!profile.is_premium_active);
    assert!(!profile.premium_bypass);

    Ok(())
}

/// Test: Multiple profile updates preserve latest state (idempotency)
#[tokio::test]
async fn test_multiple_profile_updates_preserve_latest_state() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Create command and register user
    let command = Command::new(dbs.evento.clone());

    let register_input = RegisterUserInput {
        email: "multiupdate@example.com".to_string(),
        is_admin: None,
        password: "Password123".to_string(),
    };

    let register_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let user_id = command
        .register_user(register_input, register_metadata)
        .await?;

    // Process registration
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // First update
    let profile_input1 = UpdateProfileInput {
        dietary_restrictions: vec!["Vegan".to_string()],
        cuisine_variety_weight: 0.5,
        household_size: Some(2),
    };

    let metadata1 = EventMetadata {
        user_id: Some(user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    command
        .update_profile(user_id.clone(), profile_input1, metadata1)
        .await?;

    // Process first update
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Second update (overwrites first)
    let profile_input2 = UpdateProfileInput {
        dietary_restrictions: vec!["Vegetarian".to_string(), "Gluten-free".to_string()],
        cuisine_variety_weight: 0.9,
        household_size: Some(5),
    };

    let metadata2 = EventMetadata {
        user_id: Some(user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    command
        .update_profile(user_id.clone(), profile_input2, metadata2)
        .await?;

    // Process second update
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify latest state is preserved
    let profile = get_user_profile(&dbs.queries, &user_id).await?;
    assert_eq!(profile.dietary_restrictions.len(), 2);
    assert!(profile
        .dietary_restrictions
        .contains(&"Vegetarian".to_string()));
    assert!(profile
        .dietary_restrictions
        .contains(&"Gluten-free".to_string()));
    assert_eq!(profile.cuisine_variety_weight, 0.9);
    assert_eq!(profile.household_size, Some(5));

    Ok(())
}

/// Test: Profile update without household_size preserves it as None
#[tokio::test]
async fn test_profile_update_without_household_size() -> anyhow::Result<()> {
    let dbs = setup_test_databases().await?;

    // Set up subscriptions
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    let command = Command::new(dbs.evento.clone());

    let register_input = RegisterUserInput {
        email: "optional@example.com".to_string(),
        is_admin: None,
        password: "Password123".to_string(),
    };

    let register_metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let user_id = command
        .register_user(register_input, register_metadata)
        .await?;

    // Process registration
    subscribe_user_command(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Update without household_size
    let profile_input = UpdateProfileInput {
        dietary_restrictions: vec!["Keto".to_string()],
        cuisine_variety_weight: 0.6,
        household_size: None,
    };

    let metadata = EventMetadata {
        user_id: Some(user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    command
        .update_profile(user_id.clone(), profile_input, metadata)
        .await?;

    // Process update
    subscribe_user_query(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await?;

    // Verify household_size is None
    let profile = get_user_profile(&dbs.queries, &user_id).await?;
    assert_eq!(profile.household_size, None);
    assert_eq!(profile.dietary_restrictions.len(), 1);
    assert!(profile.dietary_restrictions.contains(&"Keto".to_string()));

    Ok(())
}
