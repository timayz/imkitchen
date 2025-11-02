//! User commands

use crate::aggregate::User;
use crate::event::{
    EventMetadata, UserLoggedIn, UserRegistered, UserRegistrationFailed, UserRegistrationSucceeded,
};
use argon2::{password_hash::PasswordHasher, Argon2};
use evento::{AggregatorName, Context, EventDetails, Executor};
use sqlx::SqlitePool;
use tracing::{error, info};
use validator::Validate;

/// Validate password complexity (uppercase, lowercase, number)
fn validate_password_complexity(password: &str) -> Result<(), validator::ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_number = password.chars().any(|c| c.is_numeric());

    if !has_uppercase || !has_lowercase || !has_number {
        return Err(validator::ValidationError::new(
            "password_complexity"
        ).with_message(
            "Password must contain at least one uppercase letter, one lowercase letter, and one number".into()
        ));
    }

    Ok(())
}

/// Input for user registration
#[derive(Validate)]
pub struct RegisterUserInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(
        length(min = 8, message = "Password must be at least 8 characters"),
        custom(function = "validate_password_complexity")
    )]
    pub password: String,
}

/// Input for user login
#[derive(Validate)]
pub struct LoginUserInput {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    pub password: String,
}

/// User command handlers
pub struct Command<E: Executor> {
    evento: E,
}

impl<E: Executor> Command<E> {
    pub fn new(evento: E) -> Self {
        Self { evento }
    }

    /// Register a new user
    pub async fn register_user(
        &self,
        input: RegisterUserInput,
        metadata: EventMetadata,
    ) -> anyhow::Result<String> {
        info!(
            email = %input.email,
            request_id = %metadata.request_id,
            "Starting user registration"
        );

        // Validate input
        input.validate()?;

        // Hash password with Argon2id
        use password_hash::{rand_core::OsRng, SaltString};

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2
            .hash_password(input.password.as_bytes(), &salt)
            .map_err(|e| {
                error!(error = %e, "Failed to hash password");
                anyhow::anyhow!("Failed to hash password")
            })?
            .to_string();

        // Create user aggregate with UserRegistered event
        let user_id = evento::create::<User>()
            .data(&UserRegistered {
                email: input.email.clone(),
                hashed_password,
            })?
            .metadata(&metadata)?
            .commit(&self.evento)
            .await?;

        info!(
            user_id = %user_id,
            email = %input.email,
            "User registration initiated"
        );

        Ok(user_id)
    }

    /// Login an existing user
    pub async fn login_user(
        &self,
        input: LoginUserInput,
        user_id: String,
        hashed_password: String,
        metadata: EventMetadata,
    ) -> anyhow::Result<()> {
        info!(
            user_id = %user_id,
            request_id = %metadata.request_id,
            "User login attempt"
        );

        // Validate input
        input.validate()?;

        // Load user aggregate to check if suspended
        let user_result = evento::load::<User, _>(&self.evento, &user_id).await?;
        let user_aggregate = &user_result.item;

        if user_aggregate.is_suspended {
            error!(user_id = %user_id, "Login attempt for suspended user");
            return Err(anyhow::anyhow!("Account is suspended"));
        }

        // Check if registration is complete
        if user_aggregate.status != Some("active".to_string()) {
            error!(
                user_id = %user_id,
                status = ?user_aggregate.status,
                "Login attempt for non-active user"
            );
            return Err(anyhow::anyhow!("Account is not active"));
        }

        // Verify password
        use argon2::{
            password_hash::{PasswordHash, PasswordVerifier},
            Argon2,
        };

        let parsed_hash = PasswordHash::new(&hashed_password)
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;

        Argon2::default()
            .verify_password(input.password.as_bytes(), &parsed_hash)
            .map_err(|_| {
                error!(user_id = %user_id, "Invalid password attempt");
                anyhow::anyhow!("Invalid email or password")
            })?;

        // Emit UserLoggedIn event
        evento::save::<User>(&user_id)
            .data(&UserLoggedIn {})?
            .metadata(&metadata)?
            .commit(&self.evento)
            .await?;

        info!(user_id = %user_id, "User logged in successfully");

        Ok(())
    }
}

/// Command handler for UserRegistered event - validates email uniqueness
#[evento::handler(User)]
async fn on_user_registered<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<UserRegistered, EventMetadata>,
) -> anyhow::Result<()> {
    let pool = context.extract::<SqlitePool>();

    info!(
        user_id = %event.aggregator_id,
        email = %event.data.email,
        "Validating user registration"
    );

    // Check if email already exists in validation table
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_emails WHERE email = ?)")
            .bind(&event.data.email)
            .fetch_one(&pool)
            .await?;

    if exists {
        // Email already exists - emit failure event
        error!(
            user_id = %event.aggregator_id,
            email = %event.data.email,
            "Email already registered"
        );

        evento::save::<User>(&event.aggregator_id)
            .data(&UserRegistrationFailed {
                error: "Email already registered".to_string(),
            })?
            .metadata(&event.metadata)?
            .commit(context.executor)
            .await?;

        return Ok(());
    }

    // Email is unique - insert into validation table
    sqlx::query("INSERT INTO user_emails (email, user_id) VALUES (?, ?)")
        .bind(&event.data.email)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    // Emit success event
    evento::save::<User>(&event.aggregator_id)
        .data(&UserRegistrationSucceeded {
            email: event.data.email.clone(),
            hashed_password: event.data.hashed_password.clone(),
        })?
        .metadata(&event.metadata)?
        .commit(context.executor)
        .await?;

    info!(
        user_id = %event.aggregator_id,
        "User registration validated successfully"
    );

    Ok(())
}

/// Create subscription builder for user command handlers
pub fn subscribe_user_command<E: Executor + Clone>(
    pool: SqlitePool,
) -> evento::SubscribeBuilder<E> {
    use crate::event::{UserRegistrationFailed, UserRegistrationSucceeded};

    evento::subscribe::<E>("user-command")
        .data(pool)
        .handler(on_user_registered())
        .skip::<User, UserRegistrationSucceeded>()
        .skip::<User, UserRegistrationFailed>()
        .skip::<User, UserLoggedIn>()
}
