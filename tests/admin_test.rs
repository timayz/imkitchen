//! Tests for admin user management functionality

mod helpers;

// Removed unused import
use imkitchen::queries::user::{get_user, get_user_profile, subscribe_user_query};
use imkitchen_user::aggregate::User;
use imkitchen_user::command::{
    subscribe_user_command, ActivateUserInput, Command, RegisterUserInput, SuspendUserInput,
    TogglePremiumBypassInput,
};
use imkitchen_user::event::EventMetadata;
use ulid::Ulid;

/// Test AC1: is_admin flag correctly set in user aggregate and projection
#[tokio::test]
async fn test_is_admin_flag_in_aggregate_and_projection() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    // Register admin user
    let command = Command::new(dbs.evento.clone());
    let metadata = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };

    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            metadata.clone(),
        )
        .await
        .expect("Failed to register admin user");

    // Process admin registration events
    helpers::process_user_events(&dbs)
        .await
        .expect("Failed to process admin events");

    // Verify aggregate has is_admin = true
    let admin_result = evento::load::<User, _>(&dbs.evento, &admin_id)
        .await
        .expect("Failed to load admin aggregate");
    assert!(admin_result.item.is_admin);
    assert_eq!(admin_result.item.status, Some("active".to_string()));

    // Verify projection has is_admin = true
    let admin_row = get_user(&dbs.queries, &admin_id)
        .await
        .expect("Failed to get admin from projection")
        .expect("Admin not found in projection");
    assert!(admin_row.is_admin);

    // Register regular user
    let metadata2 = EventMetadata {
        user_id: None,
        request_id: Ulid::new().to_string(),
    };
    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            metadata2,
        )
        .await
        .expect("Failed to register regular user");

    // Process regular user registration events
    helpers::process_user_events(&dbs)
        .await
        .expect("Failed to process user events");

    // Verify aggregate has is_admin = false
    let user_result = evento::load::<User, _>(&dbs.evento, &user_id)
        .await
        .expect("Failed to load user aggregate");
    assert!(!user_result.item.is_admin);

    // Verify projection has is_admin = false
    let user_row = get_user(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user from projection")
        .expect("User not found in projection");
    assert!(!user_row.is_admin);

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test AC4: Admin can suspend user account
#[tokio::test]
async fn test_admin_can_suspend_user() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create admin user
    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register admin");

    // Create regular user
    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user");

    // Process registration events
    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Admin suspends user
    command
        .suspend_user(
            SuspendUserInput {
                user_id: user_id.clone(),
                reason: Some("Test suspension".to_string()),
            },
            EventMetadata {
                user_id: Some(admin_id.clone()),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to suspend user");

    // Process suspension events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Verify aggregate shows suspended
    let user_result = evento::load::<User, _>(&dbs.evento, &user_id)
        .await
        .expect("Failed to load user");
    assert!(user_result.item.is_suspended);

    // Verify projection shows suspended
    let user_row = get_user(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user from projection")
        .expect("User not found");
    assert!(user_row.is_suspended);

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test AC4: Admin can activate suspended user
#[tokio::test]
async fn test_admin_can_activate_user() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create admin and user
    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register admin");

    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user");

    // Process registration events
    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Suspend user first
    command
        .suspend_user(
            SuspendUserInput {
                user_id: user_id.clone(),
                reason: Some("Test".to_string()),
            },
            EventMetadata {
                user_id: Some(admin_id.clone()),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to suspend user");

    // Process suspension events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Activate user
    command
        .activate_user(
            ActivateUserInput {
                user_id: user_id.clone(),
            },
            EventMetadata {
                user_id: Some(admin_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to activate user");

    // Process activation events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Verify aggregate shows not suspended
    let user_result = evento::load::<User, _>(&dbs.evento, &user_id)
        .await
        .expect("Failed to load user");
    assert!(!user_result.item.is_suspended);

    // Verify projection shows not suspended
    let user_row = get_user(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user from projection")
        .expect("User not found");
    assert!(!user_row.is_suspended);

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test AC5: Suspended user cannot log in
#[tokio::test]
async fn test_suspended_user_cannot_login() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create admin and user
    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register admin");

    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user");

    // Process registration events
    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Get user's hashed password
    let user_row = get_user(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    // Suspend user
    command
        .suspend_user(
            SuspendUserInput {
                user_id: user_id.clone(),
                reason: Some("Test".to_string()),
            },
            EventMetadata {
                user_id: Some(admin_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to suspend user");

    // Attempt login
    let login_result = command
        .login_user(
            imkitchen_user::command::LoginUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
            },
            user_id.clone(),
            user_row.hashed_password,
            EventMetadata {
                user_id: Some(user_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await;

    // Verify login fails
    assert!(login_result.is_err());
    let err_msg = login_result.unwrap_err().to_string();
    assert!(err_msg.contains("suspended") || err_msg.contains("Suspended"));

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test AC7: Admin can toggle premium bypass flag
#[tokio::test]
async fn test_admin_can_toggle_premium_bypass() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create admin and user
    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register admin");

    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user");

    // Process registration events
    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Enable premium bypass
    command
        .toggle_premium_bypass(
            TogglePremiumBypassInput {
                user_id: user_id.clone(),
                premium_bypass: true,
            },
            EventMetadata {
                user_id: Some(admin_id.clone()),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to toggle premium bypass");

    // Process toggle events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Verify aggregate has premium_bypass = true
    let user_result = evento::load::<User, _>(&dbs.evento, &user_id)
        .await
        .expect("Failed to load user");
    assert!(user_result.item.premium_bypass);

    // Verify projection has premium_bypass = true
    let profile = get_user_profile(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user profile");
    assert!(profile.premium_bypass);

    // Disable premium bypass
    command
        .toggle_premium_bypass(
            TogglePremiumBypassInput {
                user_id: user_id.clone(),
                premium_bypass: false,
            },
            EventMetadata {
                user_id: Some(admin_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to toggle premium bypass");

    // Process toggle events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // Verify aggregate has premium_bypass = false
    let user_result = evento::load::<User, _>(&dbs.evento, &user_id)
        .await
        .expect("Failed to load user");
    assert!(!user_result.item.premium_bypass);

    // Verify projection has premium_bypass = false
    let profile = get_user_profile(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user profile");
    assert!(!profile.premium_bypass);

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test non-admin user cannot suspend accounts
#[tokio::test]
async fn test_non_admin_cannot_suspend() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create two regular users
    let user1_id = command
        .register_user(
            RegisterUserInput {
                email: "user1@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user1");

    let user2_id = command
        .register_user(
            RegisterUserInput {
                email: "user2@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user2");

    // User1 tries to suspend User2
    let suspend_result = command
        .suspend_user(
            SuspendUserInput {
                user_id: user2_id.clone(),
                reason: Some("Test".to_string()),
            },
            EventMetadata {
                user_id: Some(user1_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await;

    // Verify operation fails
    assert!(suspend_result.is_err());
    let err_msg = suspend_result.unwrap_err().to_string();
    assert!(err_msg.contains("Unauthorized") || err_msg.contains("admin"));

    // Verify user2 is not suspended
    let user2_result = evento::load::<User, _>(&dbs.evento, &user2_id)
        .await
        .expect("Failed to load user2");
    assert!(!user2_result.item.is_suspended);

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}

/// Test AC8: Integration test for suspend → login rejection → activate → login success
#[tokio::test]
async fn test_suspend_login_activate_flow() {
    let dbs = helpers::setup_test_databases()
        .await
        .expect("Failed to setup test databases");

    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");

    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let command = Command::new(dbs.evento.clone());

    // Create admin and user
    let admin_id = command
        .register_user(
            RegisterUserInput {
                email: "admin@example.com".to_string(),
                password: "Admin123!".to_string(),
                is_admin: Some(true),
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register admin");

    let user_id = command
        .register_user(
            RegisterUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
                is_admin: None,
            },
            EventMetadata {
                user_id: None,
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to register user");

    // Process registration events
    subscribe_user_command::<evento::Sqlite>(dbs.validation.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user command");
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    let user_row = get_user(&dbs.queries, &user_id)
        .await
        .expect("Failed to get user")
        .expect("User not found");

    // 1. Suspend user
    command
        .suspend_user(
            SuspendUserInput {
                user_id: user_id.clone(),
                reason: Some("Test".to_string()),
            },
            EventMetadata {
                user_id: Some(admin_id.clone()),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to suspend user");

    // Process suspension events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // 2. Verify login fails while suspended
    let login_result = command
        .login_user(
            imkitchen_user::command::LoginUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
            },
            user_id.clone(),
            user_row.hashed_password.clone(),
            EventMetadata {
                user_id: Some(user_id.clone()),
                request_id: Ulid::new().to_string(),
            },
        )
        .await;
    assert!(login_result.is_err());

    // 3. Activate user
    command
        .activate_user(
            ActivateUserInput {
                user_id: user_id.clone(),
            },
            EventMetadata {
                user_id: Some(admin_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await
        .expect("Failed to activate user");

    // Process activation events
    subscribe_user_query::<evento::Sqlite>(dbs.queries.clone())
        .unsafe_oneshot(&dbs.evento)
        .await
        .expect("Failed to subscribe user query");

    // 4. Verify login succeeds after activation
    let login_result = command
        .login_user(
            imkitchen_user::command::LoginUserInput {
                email: "user@example.com".to_string(),
                password: "User123!".to_string(),
            },
            user_id.clone(),
            user_row.hashed_password,
            EventMetadata {
                user_id: Some(user_id),
                request_id: Ulid::new().to_string(),
            },
        )
        .await;
    assert!(login_result.is_ok());

    helpers::cleanup_test_databases(dbs)
        .await
        .expect("Failed to cleanup");
}
