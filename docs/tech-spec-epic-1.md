# Technical Specification: User Authentication & Profile Management

Date: 2025-10-11
Author: Jonathan
Epic ID: 1
Status: Draft

---

## Overview

Epic 1 establishes the foundational user authentication and profile management system for imkitchen, an intelligent meal planning platform. This epic enables secure user registration, JWT cookie-based authentication, profile management with dietary preferences, and freemium tier enforcement with a 10-recipe limit for free users. The implementation leverages event sourcing via evento to maintain a complete audit trail of user actions and state changes, supporting future compliance requirements and analytics capabilities.

The user domain serves as the entry point for all authenticated interactions in the system, providing identity management, subscription tier enforcement, and profile data that feeds the intelligent meal planning algorithm. Premium subscription upgrades via Stripe integration enable users to unlock unlimited recipe creation and advanced features, supporting the platform's freemium business model with a target 15% conversion rate within 60 days.

## Objectives and Scope

### Objectives

- Enable secure user registration and authentication with industry-standard security practices (Argon2 password hashing, JWT tokens)
- Provide profile management for dietary restrictions, household size, skill level, and cooking availability that inform meal planning algorithms
- Enforce freemium tier limits (10 recipes for free users) at the domain level to drive premium conversions
- Integrate Stripe payment processing for seamless premium subscription upgrades
- Support password reset flows via email with secure token-based verification
- Maintain complete audit trail of user lifecycle events via evento event sourcing

### In Scope

- User registration with email/password authentication (minimum 8 characters)
- JWT cookie-based session management with HTTP-only, secure, SameSite cookies
- Login/logout flows with secure token generation and validation
- Password reset request flow with email token delivery
- Password reset completion with token verification
- User profile creation and editing (dietary restrictions, household size, skill level, weeknight availability)
- Freemium tier enforcement (10 recipe limit for free users)
- Premium subscription upgrade flow via Stripe Checkout
- Stripe webhook handling for subscription lifecycle events
- User aggregate event sourcing (UserCreated, ProfileUpdated, SubscriptionUpgraded events)
- User read model projections for query optimization
- Auth middleware for protecting authenticated routes

### Out of Scope

- Social authentication (Google, Facebook OAuth) - deferred to post-MVP
- Two-factor authentication (2FA) - deferred to premium feature roadmap
- Email verification on registration - deferred to reduce onboarding friction
- Account deletion and GDPR anonymization - deferred to separate compliance epic
- Multi-user family profiles - deferred to future enhancement
- Session management across devices - stateless JWT approach sufficient for MVP
- Refresh token pattern - 7-day JWT expiration sufficient for MVP

## System Architecture Alignment

### Event-Sourced Architecture

Epic 1 implements the `user` domain crate within the event-sourced monolithic architecture, following the established patterns:

- **User Aggregate**: evento aggregate managing user lifecycle state (authentication, profile, subscription)
- **Domain Events**: `UserCreated`, `ProfileUpdated`, `SubscriptionUpgraded`, `PasswordResetRequested`, `PasswordChanged`
- **Read Model Projections**: `users` table updated via evento subscriptions for optimized query access
- **CQRS Pattern**: Commands write events to evento stream, queries read from materialized `users` table

### Integration Points

- **Root Binary Routes**: `/register`, `/login`, `/logout`, `/password-reset`, `/profile`, `/subscription` routes in `src/routes/auth.rs` and `src/routes/profile.rs`
- **Auth Middleware**: `src/middleware/auth.rs` validates JWT cookies and extracts user claims for protected routes
- **Askama Templates**: `templates/pages/register.html`, `templates/pages/login.html`, `templates/pages/profile.html` for server-side rendering
- **External Services**:
  - SMTP (lettre) for password reset emails
  - Stripe (async-stripe) for payment processing
  - SQLite (evento + SQLx) for event store and read models (runtime queries only)

### Cross-Domain Dependencies

- **Recipe Domain**: User ID references in recipe creation commands, freemium limit validation before recipe creation
- **Meal Planning Domain**: User profile data (dietary restrictions, availability) consumed by meal planning algorithm
- **Notification Domain**: User notification preferences for preparation reminders

## Detailed Design

### Services and Modules

#### User Domain Crate (`crates/user/`)

**Module Structure**:

```
crates/user/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── aggregate.rs           # UserAggregate (evento)
│   ├── commands.rs            # Command structs and handlers
│   ├── events.rs              # Event structs
│   ├── read_model.rs          # User query projections
│   ├── password.rs            # Argon2 password hashing utilities
│   ├── subscription.rs        # Subscription tier logic
│   └── error.rs               # Domain-specific errors
└── tests/
    ├── aggregate_tests.rs     # Aggregate behavior tests
    └── read_model_tests.rs    # Projection tests
```

#### Key Components

**1. UserAggregate (aggregate.rs)**

```rust
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct UserAggregate {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String,

    // Profile
    pub dietary_restrictions: Vec<String>,
    pub household_size: Option<u32>,
    pub skill_level: Option<SkillLevel>,
    pub weeknight_availability: Option<String>, // JSON time range

    // Subscription
    pub tier: SubscriptionTier,
    pub recipe_count: u32,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
}

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Expert,
}

#[derive(Serialize, Deserialize, bincode::Encode, bincode::Decode, Clone, Debug, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Premium,
}

#[evento::aggregator]
impl UserAggregate {
    // Event handlers rebuild aggregate state
    async fn user_created(&mut self, event: EventDetails<UserCreated>) -> anyhow::Result<()> {
        self.user_id = event.aggregator_id.clone();
        self.email = event.data.email.clone();
        self.password_hash = event.data.password_hash.clone();
        self.created_at = event.data.created_at.clone();
        self.tier = SubscriptionTier::Free;
        self.recipe_count = 0;
        Ok(())
    }

    async fn profile_updated(&mut self, event: EventDetails<ProfileUpdated>) -> anyhow::Result<()> {
        if let Some(dietary) = event.data.dietary_restrictions {
            self.dietary_restrictions = dietary;
        }
        if let Some(size) = event.data.household_size {
            self.household_size = Some(size);
        }
        if let Some(skill) = event.data.skill_level {
            self.skill_level = Some(skill);
        }
        if let Some(avail) = event.data.weeknight_availability {
            self.weeknight_availability = Some(avail);
        }
        Ok(())
    }

    async fn subscription_upgraded(&mut self, event: EventDetails<SubscriptionUpgraded>) -> anyhow::Result<()> {
        self.tier = event.data.new_tier.clone();
        self.stripe_customer_id = event.data.stripe_customer_id.clone();
        self.stripe_subscription_id = event.data.stripe_subscription_id.clone();
        Ok(())
    }

    async fn password_changed(&mut self, event: EventDetails<PasswordChanged>) -> anyhow::Result<()> {
        self.password_hash = event.data.new_password_hash.clone();
        Ok(())
    }
}
```

**2. Commands (commands.rs)**

```rust
use validator::Validate;

#[derive(Validate)]
pub struct RegisterUserCommand {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Validate)]
pub struct UpdateProfileCommand {
    pub user_id: String,
    pub dietary_restrictions: Option<Vec<String>>,

    #[validate(range(min = 1, max = 20))]
    pub household_size: Option<u32>,

    pub skill_level: Option<SkillLevel>,
    pub weeknight_availability: Option<String>,
}

pub struct UpgradeSubscriptionCommand {
    pub user_id: String,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: String,
}

#[derive(Validate)]
pub struct RequestPasswordResetCommand {
    #[validate(email)]
    pub email: String,
}

#[derive(Validate)]
pub struct ResetPasswordCommand {
    pub reset_token: String,

    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

// Command handlers
pub async fn register_user<E: evento::Executor>(
    cmd: RegisterUserCommand,
    executor: &E,
) -> Result<String, UserError> {
    cmd.validate()?;

    // Check if email already exists
    let existing = query_user_by_email(&cmd.email, executor).await?;
    if existing.is_some() {
        return Err(UserError::EmailAlreadyExists);
    }

    // Hash password
    let password_hash = hash_password(&cmd.password)?;

    // Create aggregate with UserCreated event
    let user_id = uuid::Uuid::new_v4().to_string();
    evento::create::<UserAggregate>()
        .data(&UserCreated {
            email: cmd.email,
            password_hash,
            created_at: chrono::Utc::now().to_rfc3339(),
        })?
        .aggregator_id(&user_id)
        .commit(executor)
        .await?;

    Ok(user_id)
}

pub async fn update_profile<E: evento::Executor>(
    cmd: UpdateProfileCommand,
    executor: &E,
) -> Result<(), UserError> {
    cmd.validate()?;

    // Load aggregate
    let aggregate = evento::load::<UserAggregate>(&cmd.user_id, executor).await?;

    // Append ProfileUpdated event
    evento::append::<UserAggregate>(&cmd.user_id)
        .data(&ProfileUpdated {
            dietary_restrictions: cmd.dietary_restrictions,
            household_size: cmd.household_size,
            skill_level: cmd.skill_level,
            weeknight_availability: cmd.weeknight_availability,
        })?
        .commit(executor)
        .await?;

    Ok(())
}

pub async fn validate_recipe_creation<E: evento::Executor>(
    user_id: &str,
    executor: &E,
) -> Result<(), UserError> {
    let user = query_user_by_id(user_id, executor).await?
        .ok_or(UserError::UserNotFound)?;

    // Enforce freemium limit
    if user.tier == SubscriptionTier::Free && user.recipe_count >= 10 {
        return Err(UserError::RecipeLimitReached);
    }

    Ok(())
}
```

**3. Events (events.rs)**

```rust
use evento::AggregatorName;

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct UserCreated {
    pub email: String,
    pub password_hash: String,
    pub created_at: String,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct ProfileUpdated {
    pub dietary_restrictions: Option<Vec<String>>,
    pub household_size: Option<u32>,
    pub skill_level: Option<SkillLevel>,
    pub weeknight_availability: Option<String>,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct SubscriptionUpgraded {
    pub new_tier: SubscriptionTier,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct PasswordResetRequested {
    pub email: String,
    pub reset_token: String,
    pub expires_at: String,
}

#[derive(AggregatorName, bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct PasswordChanged {
    pub new_password_hash: String,
}
```

**4. Password Hashing (password.rs)**

```rust
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

pub fn hash_password(password: &str) -> Result<String, UserError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| UserError::PasswordHashingError(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, UserError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| UserError::PasswordHashingError(e.to_string()))?;

    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

pub fn generate_reset_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const TOKEN_LEN: usize = 32;
    let mut rng = rand::thread_rng();

    (0..TOKEN_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

**5. JWT Token Generation (src/routes/auth.rs)**

```rust
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // User ID
    pub email: String,
    pub tier: String,     // "free" | "premium"
    pub exp: u64,         // Expiration timestamp
    pub iat: u64,         // Issued at
}

pub fn generate_jwt(user_id: &str, email: &str, tier: &SubscriptionTier, secret: &str) -> Result<String, AuthError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .ok_or(AuthError::TokenGenerationError)?
        .timestamp() as u64;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        tier: match tier {
            SubscriptionTier::Free => "free".to_string(),
            SubscriptionTier::Premium => "premium".to_string(),
        },
        exp: expiration,
        iat: chrono::Utc::now().timestamp() as u64,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .map_err(|e| AuthError::TokenGenerationError)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AuthError::InvalidToken)
}
```

### Data Models and Contracts

#### Users Table (Read Model)

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,                          -- UUID
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,                  -- Argon2 hash
    created_at TEXT NOT NULL,                     -- ISO 8601 timestamp

    -- Profile fields
    dietary_restrictions TEXT,                    -- JSON array: ["vegetarian", "gluten-free"]
    household_size INTEGER,                       -- NULL if not set
    skill_level TEXT,                             -- "beginner" | "intermediate" | "expert" | NULL
    weeknight_availability TEXT,                  -- JSON: {"start": "18:00", "duration_minutes": 45}

    -- Subscription
    tier TEXT NOT NULL DEFAULT 'free',            -- "free" | "premium"
    recipe_count INTEGER NOT NULL DEFAULT 0,      -- Incremented on recipe creation
    stripe_customer_id TEXT,                      -- Stripe customer ID (NULL for free)
    stripe_subscription_id TEXT,                  -- Stripe subscription ID (NULL for free)

    -- Password reset
    reset_token TEXT,                             -- Reset token (NULL when not active)
    reset_token_expires_at TEXT,                  -- ISO 8601 timestamp (NULL when not active)

    updated_at TEXT NOT NULL                      -- Last profile update timestamp
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_reset_token ON users(reset_token) WHERE reset_token IS NOT NULL;
```

#### Read Model Projections

```rust
// crates/user/src/read_model.rs

#[evento::handler(UserAggregate)]
pub async fn project_user_created<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<UserCreated>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, tier, recipe_count, updated_at)
        VALUES (?, ?, ?, ?, 'free', 0, ?)
        "#
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.email)
    .bind(&event.data.password_hash)
    .bind(&event.data.created_at)
    .bind(&event.data.created_at)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

#[evento::handler(UserAggregate)]
pub async fn project_profile_updated<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<ProfileUpdated>,
) -> anyhow::Result<()> {
    let dietary = event.data.dietary_restrictions
        .as_ref()
        .map(|v| serde_json::to_string(v))
        .transpose()?;

    let skill = event.data.skill_level
        .as_ref()
        .map(|s| match s {
            SkillLevel::Beginner => "beginner",
            SkillLevel::Intermediate => "intermediate",
            SkillLevel::Expert => "expert",
        });

    sqlx::query(
        r#"
        UPDATE users
        SET dietary_restrictions = COALESCE(?, dietary_restrictions),
            household_size = COALESCE(?, household_size),
            skill_level = COALESCE(?, skill_level),
            weeknight_availability = COALESCE(?, weeknight_availability),
            updated_at = ?
        WHERE id = ?
        "#
    )
    .bind(dietary)
    .bind(event.data.household_size)
    .bind(skill)
    .bind(event.data.weeknight_availability)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(&event.aggregator_id)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}

#[evento::handler(UserAggregate)]
pub async fn project_subscription_upgraded<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<SubscriptionUpgraded>,
) -> anyhow::Result<()> {
    let tier = match event.data.new_tier {
        SubscriptionTier::Free => "free",
        SubscriptionTier::Premium => "premium",
    };

    sqlx::query(
        r#"
        UPDATE users
        SET tier = ?,
            stripe_customer_id = ?,
            stripe_subscription_id = ?,
            updated_at = ?
        WHERE id = ?
        "#
    )
    .bind(tier)
    .bind(&event.data.stripe_customer_id)
    .bind(&event.data.stripe_subscription_id)
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(&event.aggregator_id)
    .execute(context.executor.pool())
    .await?;

    Ok(())
}
```

### APIs and Interfaces

#### Authentication Routes

**POST /register** - User Registration
```rust
#[derive(Deserialize, Validate)]
struct RegisterForm {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8))]
    password: String,

    #[validate(must_match = "password")]
    password_confirm: String,
}

async fn register_handler(
    State(app_state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate
    form.validate()?;

    // 2. Register user (domain command)
    let cmd = RegisterUserCommand {
        email: form.email.clone(),
        password: form.password,
    };
    let user_id = user::register_user(cmd, &app_state.executor).await?;

    // 3. Generate JWT
    let token = generate_jwt(&user_id, &form.email, &SubscriptionTier::Free, &app_state.jwt_secret)?;

    // 4. Set cookie and redirect
    let cookie = format!(
        "auth_token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800",
        token
    );

    Ok((
        StatusCode::SEE_OTHER,
        [("Set-Cookie", cookie), ("Location", "/dashboard".to_string())],
        ()
    ))
}
```

**POST /login** - User Login
```rust
#[derive(Deserialize, Validate)]
struct LoginForm {
    #[validate(email)]
    email: String,
    password: String,
}

async fn login_handler(
    State(app_state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Validate
    form.validate()?;

    // 2. Query user by email
    let user = user::query_user_by_email(&form.email, &app_state.executor)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    // 3. Verify password
    let valid = user::verify_password(&form.password, &user.password_hash)?;
    if !valid {
        return Err(AuthError::InvalidCredentials.into());
    }

    // 4. Generate JWT
    let token = generate_jwt(&user.id, &user.email, &user.tier, &app_state.jwt_secret)?;

    // 5. Set cookie and redirect
    let cookie = format!(
        "auth_token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=604800",
        token
    );

    Ok((
        StatusCode::SEE_OTHER,
        [("Set-Cookie", cookie), ("Location", "/dashboard".to_string())],
        ()
    ))
}
```

**POST /logout** - User Logout
```rust
async fn logout_handler() -> impl IntoResponse {
    let cookie = "auth_token=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0";

    (
        StatusCode::SEE_OTHER,
        [("Set-Cookie", cookie), ("Location", "/".to_string())],
        ()
    )
}
```

#### Password Reset Routes

**POST /password-reset** - Request Password Reset
```rust
#[derive(Deserialize, Validate)]
struct PasswordResetRequestForm {
    #[validate(email)]
    email: String,
}

async fn password_reset_request_handler(
    State(app_state): State<AppState>,
    Form(form): Form<PasswordResetRequestForm>,
) -> Result<impl IntoResponse, AppError> {
    form.validate()?;

    // 1. Find user by email
    let user = user::query_user_by_email(&form.email, &app_state.executor).await?;

    if let Some(user) = user {
        // 2. Generate reset token
        let reset_token = user::generate_reset_token();
        let expires_at = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(1))
            .unwrap()
            .to_rfc3339();

        // 3. Append PasswordResetRequested event
        evento::append::<UserAggregate>(&user.id)
            .data(&PasswordResetRequested {
                email: user.email.clone(),
                reset_token: reset_token.clone(),
                expires_at,
            })?
            .commit(&app_state.executor)
            .await?;

        // 4. Send email
        let reset_link = format!("{}/password-reset/{}", app_state.base_url, reset_token);
        send_password_reset_email(&user.email, &reset_link, &app_state.smtp_config).await?;
    }

    // Always return success to prevent email enumeration
    Ok(Redirect::to("/password-reset/sent"))
}
```

**POST /password-reset/:token** - Complete Password Reset
```rust
#[derive(Deserialize, Validate)]
struct PasswordResetForm {
    #[validate(length(min = 8))]
    password: String,

    #[validate(must_match = "password")]
    password_confirm: String,
}

async fn password_reset_complete_handler(
    State(app_state): State<AppState>,
    Path(token): Path<String>,
    Form(form): Form<PasswordResetForm>,
) -> Result<impl IntoResponse, AppError> {
    form.validate()?;

    // 1. Find user by reset token
    let user = user::query_user_by_reset_token(&token, &app_state.executor)
        .await?
        .ok_or(AuthError::InvalidResetToken)?;

    // 2. Verify token not expired
    let expires_at = chrono::DateTime::parse_from_rfc3339(&user.reset_token_expires_at.unwrap())
        .map_err(|_| AuthError::InvalidResetToken)?;
    if chrono::Utc::now() > expires_at {
        return Err(AuthError::ExpiredResetToken.into());
    }

    // 3. Hash new password
    let new_password_hash = user::hash_password(&form.password)?;

    // 4. Append PasswordChanged event
    evento::append::<UserAggregate>(&user.id)
        .data(&PasswordChanged { new_password_hash })?
        .commit(&app_state.executor)
        .await?;

    // 5. Clear reset token
    user::clear_reset_token(&user.id, &app_state.executor).await?;

    Ok(Redirect::to("/login?reset=success"))
}
```

#### Profile Routes

**GET /profile** - View Profile
```rust
async fn profile_page_handler(
    auth: Auth,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user = user::query_user_by_id(&auth.user_id, &app_state.executor)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let template = ProfilePageTemplate { user };
    Ok(Html(template.render()?))
}
```

**PUT /profile** - Update Profile
```rust
#[derive(Deserialize, Validate)]
struct UpdateProfileForm {
    dietary_restrictions: Option<String>,  // Comma-separated

    #[validate(range(min = 1, max = 20))]
    household_size: Option<u32>,

    skill_level: Option<String>,           // "beginner" | "intermediate" | "expert"
    weeknight_availability: Option<String>, // JSON
}

async fn update_profile_handler(
    auth: Auth,
    State(app_state): State<AppState>,
    Form(form): Form<UpdateProfileForm>,
) -> Result<impl IntoResponse, AppError> {
    form.validate()?;

    let dietary = form.dietary_restrictions
        .map(|s| s.split(',').map(|v| v.trim().to_string()).collect());

    let skill = form.skill_level
        .and_then(|s| match s.as_str() {
            "beginner" => Some(SkillLevel::Beginner),
            "intermediate" => Some(SkillLevel::Intermediate),
            "expert" => Some(SkillLevel::Expert),
            _ => None,
        });

    let cmd = UpdateProfileCommand {
        user_id: auth.user_id,
        dietary_restrictions: dietary,
        household_size: form.household_size,
        skill_level: skill,
        weeknight_availability: form.weeknight_availability,
    };

    user::update_profile(cmd, &app_state.executor).await?;

    Ok(Redirect::to("/profile?updated=true"))
}
```

#### Subscription Routes

**POST /subscription/upgrade** - Upgrade to Premium
```rust
async fn upgrade_subscription_handler(
    auth: Auth,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user = user::query_user_by_id(&auth.user_id, &app_state.executor)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // 1. Create Stripe checkout session
    let stripe_client = stripe::Client::new(&app_state.stripe_secret_key);

    let checkout_session = stripe::CheckoutSession::create(
        &stripe_client,
        stripe::CreateCheckoutSession {
            mode: Some(stripe::CheckoutSessionMode::Subscription),
            success_url: Some(&format!("{}/subscription/success", app_state.base_url)),
            cancel_url: Some(&format!("{}/subscription", app_state.base_url)),
            customer_email: Some(&user.email),
            line_items: Some(vec![
                stripe::CreateCheckoutSessionLineItems {
                    price: app_state.stripe_price_id.clone(),
                    quantity: Some(1),
                    ..Default::default()
                }
            ]),
            metadata: Some([("user_id".to_string(), auth.user_id.clone())].into()),
            ..Default::default()
        },
    )
    .await?;

    // 2. Redirect to Stripe Checkout
    Ok(Redirect::to(&checkout_session.url.unwrap()))
}
```

**POST /webhooks/stripe** - Stripe Webhook Handler
```rust
async fn stripe_webhook_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    // 1. Verify webhook signature
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::InvalidWebhookSignature)?;

    let event = stripe::Webhook::construct_event(
        &body,
        signature,
        &app_state.stripe_webhook_secret,
    )
    .map_err(|_| AuthError::InvalidWebhookSignature)?;

    // 2. Handle event
    match event.type_ {
        stripe::EventType::CheckoutSessionCompleted => {
            let session: stripe::CheckoutSession = serde_json::from_value(event.data.object)?;

            let user_id = session.metadata
                .as_ref()
                .and_then(|m| m.get("user_id"))
                .ok_or(AuthError::MissingWebhookMetadata)?;

            // Append SubscriptionUpgraded event
            evento::append::<UserAggregate>(user_id)
                .data(&SubscriptionUpgraded {
                    new_tier: SubscriptionTier::Premium,
                    stripe_customer_id: session.customer.map(|c| c.id().to_string()),
                    stripe_subscription_id: session.subscription.map(|s| s.id().to_string()),
                })?
                .commit(&app_state.executor)
                .await?;
        }
        stripe::EventType::CustomerSubscriptionDeleted => {
            // Handle subscription cancellation (downgrade to free)
            let subscription: stripe::Subscription = serde_json::from_value(event.data.object)?;

            // Query user by stripe_subscription_id
            let user = user::query_user_by_stripe_subscription(&subscription.id, &app_state.executor)
                .await?
                .ok_or(AuthError::UserNotFound)?;

            // Downgrade to free tier
            evento::append::<UserAggregate>(&user.id)
                .data(&SubscriptionUpgraded {
                    new_tier: SubscriptionTier::Free,
                    stripe_customer_id: None,
                    stripe_subscription_id: None,
                })?
                .commit(&app_state.executor)
                .await?;
        }
        _ => {
            // Ignore other event types
        }
    }

    Ok(StatusCode::OK)
}
```

### Workflows and Sequencing

#### Registration Flow

```
1. User navigates to /register
   ↓
2. GET /register → Render registration form (Askama template)
   ↓
3. User submits form (email, password, password_confirm)
   ↓
4. POST /register → Validate form
   ↓
5. Check email uniqueness (query read model)
   ↓ (email available)
6. Hash password with Argon2
   ↓
7. Create UserAggregate with UserCreated event → evento stream
   ↓
8. evento subscription → Insert into users table (read model)
   ↓
9. Generate JWT with user_id, email, tier=Free
   ↓
10. Set HTTP-only cookie (auth_token, 7-day expiration)
   ↓
11. Redirect to /dashboard (302)
   ↓
12. Auth middleware validates JWT → Extract user_id
   ↓
13. Dashboard page rendered with user data
```

#### Login Flow

```
1. User navigates to /login
   ↓
2. GET /login → Render login form
   ↓
3. User submits form (email, password)
   ↓
4. POST /login → Validate form
   ↓
5. Query user by email (read model)
   ↓ (user found)
6. Verify password with Argon2 (compare hash)
   ↓ (password valid)
7. Generate JWT with user_id, email, tier
   ↓
8. Set HTTP-only cookie (auth_token)
   ↓
9. Redirect to /dashboard (302)
   ↓
10. Auth middleware validates JWT → Extract user_id
   ↓
11. Dashboard page rendered
```

#### Password Reset Flow

```
Request Phase:
1. User navigates to /password-reset
   ↓
2. GET /password-reset → Render request form
   ↓
3. User submits email
   ↓
4. POST /password-reset → Validate email
   ↓
5. Query user by email (read model)
   ↓ (if user exists)
6. Generate random reset token (32 chars)
   ↓
7. Append PasswordResetRequested event (token, expires_at: +1 hour)
   ↓
8. evento subscription → Update users table (reset_token, reset_token_expires_at)
   ↓
9. Send email with reset link (lettre SMTP)
   ↓
10. Always redirect to /password-reset/sent (prevent enumeration)

Completion Phase:
1. User clicks reset link in email
   ↓
2. GET /password-reset/:token → Query user by token
   ↓ (token valid, not expired)
3. Render reset form (new password, confirm)
   ↓
4. User submits new password
   ↓
5. POST /password-reset/:token → Validate password
   ↓
6. Hash new password with Argon2
   ↓
7. Append PasswordChanged event
   ↓
8. evento subscription → Update users table (password_hash)
   ↓
9. Clear reset token (update read model)
   ↓
10. Redirect to /login?reset=success
```

#### Upgrade to Premium Flow

```
1. User navigates to /subscription (authenticated)
   ↓
2. GET /subscription → Render subscription page (tier status, upgrade button)
   ↓
3. User clicks "Upgrade to Premium" button
   ↓
4. POST /subscription/upgrade → Query user data
   ↓
5. Create Stripe Checkout Session
   - mode: subscription
   - price: $9.99/month
   - customer_email: user email
   - metadata: user_id
   ↓
6. Redirect to Stripe Checkout URL (hosted page)
   ↓
7. User completes payment on Stripe
   ↓
8. Stripe sends checkout.session.completed webhook
   ↓
9. POST /webhooks/stripe → Verify signature
   ↓
10. Extract user_id from metadata
   ↓
11. Append SubscriptionUpgraded event (tier=Premium, stripe_customer_id, stripe_subscription_id)
   ↓
12. evento subscription → Update users table (tier, stripe IDs)
   ↓
13. User redirected back to /subscription/success
   ↓
14. Success page shows "You're now a premium member!"
   ↓
15. User can now create unlimited recipes (validation passes)
```

## Non-Functional Requirements

### Performance

- **Auth Response Time**: Login/register endpoints < 500ms at 95th percentile
- **JWT Validation**: < 10ms per request (middleware overhead)
- **Password Hashing**: Argon2 default params target ~100ms (acceptable for registration/login)
- **Read Model Queries**: < 50ms with indexed lookups on email, user_id
- **Event Projection**: < 100ms from event commit to read model update

### Security

- **Password Hashing**: Argon2id with OWASP-recommended parameters (memory=65536 KB, iterations=3, parallelism=4)
- **JWT Tokens**: HS256 algorithm, 32-byte secret, 7-day expiration
- **Cookie Security**: HTTP-only (prevents XSS), Secure flag (HTTPS only), SameSite=Lax (CSRF protection)
- **Password Requirements**: Minimum 8 characters (enforced via validator)
- **Password Reset Tokens**: 32-character random string, 1-hour expiration
- **Stripe Webhooks**: Signature verification with webhook secret
- **Input Validation**: validator crate for all form inputs, SQL injection prevention via SQLx parameterized queries
- **Email Enumeration Protection**: Always return success on password reset request regardless of email existence

### Reliability/Availability

- **Event Sourcing Audit Trail**: Complete history of user state changes for compliance and debugging
- **Idempotent Projections**: evento subscriptions handle duplicate events gracefully
- **Graceful Degradation**: JWT validation failures redirect to login (no 500 errors)
- **Email Delivery**: Retry logic for SMTP failures (background job queue - future enhancement)
- **Stripe Webhook Retries**: Stripe automatically retries failed webhooks

### Observability

- **Tracing**: OpenTelemetry instrumentation on all command handlers and route handlers
  - Trace: User registration flow (register_user command, event commit, projection)
  - Trace: Login flow (password verification, JWT generation)
  - Trace: Password reset flow (token generation, email send)
- **Metrics**:
  - Counter: user_registrations_total
  - Counter: user_logins_total (labels: success/failure)
  - Counter: password_resets_requested_total
  - Counter: subscription_upgrades_total
  - Histogram: auth_jwt_generation_duration_seconds
- **Logging**:
  - Info: User registration (user_id, email)
  - Info: Successful login (user_id)
  - Warn: Failed login attempts (email, reason)
  - Info: Password reset requested (email)
  - Info: Subscription upgraded (user_id, tier)
  - Error: Stripe webhook signature verification failures

## Dependencies and Integrations

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
# Core
evento = { version = "1.3", features = ["sqlite"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1.40", features = ["full"] }
axum = "0.8"
serde = { version = "1.0", features = ["derive"] }

# Authentication & Security
jsonwebtoken = "9.3"
argon2 = "0.5"
uuid = { version = "1.10", features = ["v4", "serde"] }
rand = "0.8"

# Validation
validator = { version = "0.20", features = ["derive"] }

# Templates
askama = "0.14"

# External Services
async-stripe = { version = "0.39", features = ["runtime-tokio", "webhook-events"] }
lettre = { version = "0.11", features = ["tokio1-native-tls"] }
reqwest = { version = "0.12", features = ["json"] }

# Observability
tracing = "0.1"
tracing-opentelemetry = "0.31"

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
thiserror = "1.0"
anyhow = "1.0"
```

### External Service Integrations

**1. SMTP Service (lettre)**
- Purpose: Send password reset emails
- Configuration:
  - `SMTP_HOST`: SMTP server hostname (e.g., smtp.gmail.com)
  - `SMTP_PORT`: SMTP port (587 for TLS)
  - `SMTP_USERNAME`: SMTP auth username
  - `SMTP_PASSWORD`: SMTP auth password
- Email Templates:
  - Password reset: Subject "Reset your imkitchen password", body with reset link
- Error Handling: Log failures, return success to user (prevent enumeration)

**2. Stripe (async-stripe)**
- Purpose: Premium subscription payment processing
- Configuration:
  - `STRIPE_SECRET_KEY`: Stripe secret API key (sk_live_xxx)
  - `STRIPE_WEBHOOK_SECRET`: Webhook signing secret (whsec_xxx)
  - `STRIPE_PRICE_ID`: Price ID for premium subscription (price_xxx)
- Events:
  - `checkout.session.completed`: Subscription created → upgrade user to premium
  - `customer.subscription.deleted`: Subscription cancelled → downgrade to free
- Webhook Security: Verify signature using stripe-signature header

**3. SQLite Database (evento + SQLx)**
- evento Event Store: Managed automatically by evento library
- Read Model: Manual migrations in `migrations/001_create_users_table.sql`
- Connection: SQLite file at path specified by `DATABASE_URL` env var
- Configuration:
  - WAL mode: `PRAGMA journal_mode=WAL`
  - Foreign keys: `PRAGMA foreign_keys=ON`

## Acceptance Criteria (Authoritative)

### Story 1: User Registration

**AC-1.1**: Given a new user visits /register, when they submit a valid email and password (min 8 chars), then a user account is created and they are logged in with a JWT cookie

**AC-1.2**: Given a user submits registration with an existing email, when the form is processed, then an error "Email already registered" is displayed

**AC-1.3**: Given a user submits registration with password < 8 characters, when the form is validated, then an error "Password must be at least 8 characters" is displayed

**AC-1.4**: Given a user submits registration with non-matching password confirmation, when validated, then an error "Passwords do not match" is displayed

**AC-1.5**: Given a user successfully registers, when UserCreated event is committed, then users table contains new row with tier="free", recipe_count=0

### Story 2: User Login

**AC-2.1**: Given an existing user visits /login, when they submit correct email and password, then JWT cookie is set and they are redirected to /dashboard

**AC-2.2**: Given a user submits login with incorrect password, when verified, then error "Invalid email or password" is displayed

**AC-2.3**: Given a user submits login with non-existent email, when queried, then error "Invalid email or password" is displayed (same message to prevent enumeration)

**AC-2.4**: Given a user logs in successfully, when JWT is generated, then token contains claims: user_id, email, tier, exp (7 days from now)

**AC-2.5**: Given a logged-in user's JWT expires after 7 days, when they access protected routes, then they are redirected to /login

### Story 3: User Logout

**AC-3.1**: Given a logged-in user clicks logout button, when POST /logout is processed, then auth_token cookie is cleared (Max-Age=0)

**AC-3.2**: Given a user logs out, when they attempt to access /dashboard, then they are redirected to /login

### Story 4: Password Reset Request

**AC-4.1**: Given a user visits /password-reset, when they submit their email, then a reset email is sent with a token link (if email exists)

**AC-4.2**: Given a user requests password reset for non-existent email, when processed, then success message is shown (prevent enumeration)

**AC-4.3**: Given a reset email is sent, when PasswordResetRequested event is committed, then users table stores reset_token and reset_token_expires_at (+1 hour)

**AC-4.4**: Given a reset token is generated, when it's inserted into read model, then it's a 32-character random string

### Story 5: Password Reset Completion

**AC-5.1**: Given a user clicks reset link with valid token, when they access /password-reset/:token, then reset form is displayed

**AC-5.2**: Given a user submits new password (min 8 chars), when PasswordChanged event is committed, then password_hash in users table is updated

**AC-5.3**: Given a user submits reset with expired token (>1 hour), when validated, then error "Reset link expired" is displayed

**AC-5.4**: Given a user submits reset with invalid token, when queried, then error "Invalid reset link" is displayed

**AC-5.5**: Given a password reset completes successfully, when user is redirected to /login, then reset_token and reset_token_expires_at are cleared in read model

### Story 6: Profile Viewing

**AC-6.1**: Given a logged-in user visits /profile, when page loads, then their email, dietary restrictions, household size, skill level, and availability are displayed

**AC-6.2**: Given a user has not set profile fields, when /profile loads, then optional fields show placeholder text or default values

### Story 7: Profile Editing

**AC-7.1**: Given a logged-in user edits dietary restrictions (comma-separated), when form is submitted, then ProfileUpdated event is committed with dietary_restrictions array

**AC-7.2**: Given a user updates household_size to 5, when ProfileUpdated event projects, then users.household_size = 5

**AC-7.3**: Given a user selects skill_level "intermediate", when saved, then users.skill_level = "intermediate"

**AC-7.4**: Given a user sets weeknight_availability JSON {"start": "18:00", "duration_minutes": 45}, when saved, then users.weeknight_availability stores JSON string

**AC-7.5**: Given a user submits profile with household_size > 20, when validated, then error "Household size must be between 1 and 20" is displayed

### Story 8: Freemium Enforcement - Recipe Creation

**AC-8.1**: Given a free-tier user has 9 recipes, when they create recipe #10, then creation succeeds and users.recipe_count = 10

**AC-8.2**: Given a free-tier user has 10 recipes, when they attempt to create recipe #11, then error "Recipe limit reached. Upgrade to premium for unlimited recipes" is displayed

**AC-8.3**: Given a premium user has 50 recipes, when they create recipe #51, then creation succeeds (no limit)

**AC-8.4**: Given recipe creation command validation calls user::validate_recipe_creation, when user is free-tier with 10 recipes, then UserError::RecipeLimitReached is returned

### Story 9: Premium Subscription Upgrade

**AC-9.1**: Given a free-tier user visits /subscription, when they click "Upgrade to Premium", then they are redirected to Stripe Checkout

**AC-9.2**: Given a user completes Stripe Checkout, when checkout.session.completed webhook is received, then SubscriptionUpgraded event is committed with tier=Premium

**AC-9.3**: Given SubscriptionUpgraded event projects, when users table is updated, then tier="premium", stripe_customer_id and stripe_subscription_id are set

**AC-9.4**: Given a newly upgraded premium user, when they access /profile, then tier badge shows "Premium Member"

**AC-9.5**: Given a premium user, when they create recipe #11, then validation passes and recipe is created

### Story 10: Subscription Cancellation

**AC-10.1**: Given a premium user cancels subscription in Stripe, when customer.subscription.deleted webhook is received, then SubscriptionUpgraded event with tier=Free is committed

**AC-10.2**: Given subscription cancellation event projects, when users table updates, then tier="free", stripe_customer_id and stripe_subscription_id are cleared

**AC-10.3**: Given a downgraded user has 15 recipes, when they attempt to create recipe #16, then error "Recipe limit reached" is displayed

### Story 11: Auth Middleware

**AC-11.1**: Given a user accesses /dashboard without auth_token cookie, when middleware runs, then they are redirected to /login with 401 status

**AC-11.2**: Given a user accesses /dashboard with expired JWT, when middleware validates token, then they are redirected to /login

**AC-11.3**: Given a user accesses /dashboard with valid JWT, when middleware extracts claims, then request.extensions contains Auth struct with user_id and tier

**AC-11.4**: Given a protected route requires premium tier, when free user accesses it, then 403 Forbidden is returned

### Story 12: Security Validation

**AC-12.1**: Given a password is hashed with Argon2, when stored in database, then hash starts with "$argon2id$" prefix

**AC-12.2**: Given a JWT cookie is set, when inspected in browser, then HttpOnly=true, Secure=true, SameSite=Lax

**AC-12.3**: Given a Stripe webhook is received, when signature is invalid, then 401 Unauthorized is returned and event is ignored

**AC-12.4**: Given user input in registration form, when rendered in error messages, then Askama auto-escaping prevents XSS

**AC-12.5**: Given SQL queries in read model projections, when executed, then all use parameterized queries (no string concatenation)

## Traceability Mapping

### Epic 1 Traceability Matrix

| Acceptance Criteria | Spec Section | Component | Test Approach |
|---------------------|--------------|-----------|---------------|
| AC-1.1 - AC-1.5 (Registration) | APIs/POST /register, Commands/register_user | user crate, src/routes/auth.rs | Integration test: POST /register with valid/invalid inputs, verify users table, JWT cookie |
| AC-2.1 - AC-2.5 (Login) | APIs/POST /login, JWT generation | user crate, src/routes/auth.rs | Integration test: POST /login with valid/invalid credentials, verify JWT claims, redirect |
| AC-3.1 - AC-3.2 (Logout) | APIs/POST /logout | src/routes/auth.rs | Integration test: POST /logout, verify cookie cleared, access /dashboard fails |
| AC-4.1 - AC-4.4 (Password Reset Request) | APIs/POST /password-reset, Events/PasswordResetRequested | user crate, src/routes/auth.rs, SMTP integration | Integration test: POST /password-reset, verify reset_token in users table, mock email send |
| AC-5.1 - AC-5.5 (Password Reset Completion) | APIs/POST /password-reset/:token, Events/PasswordChanged | user crate, src/routes/auth.rs | Integration test: POST /password-reset/:token with valid/expired/invalid tokens, verify password_hash updated |
| AC-6.1 - AC-6.2 (Profile Viewing) | APIs/GET /profile | src/routes/profile.rs | Integration test: GET /profile, verify template renders user data |
| AC-7.1 - AC-7.5 (Profile Editing) | APIs/PUT /profile, Commands/update_profile | user crate, src/routes/profile.rs | Integration test: PUT /profile with various field combinations, verify ProfileUpdated event, read model updated |
| AC-8.1 - AC-8.4 (Freemium Enforcement) | Commands/validate_recipe_creation, subscription.rs | user crate | Unit test: validate_recipe_creation with free/premium users at various recipe counts |
| AC-9.1 - AC-9.5 (Premium Upgrade) | APIs/POST /subscription/upgrade, Stripe integration, Events/SubscriptionUpgraded | src/routes/profile.rs, user crate | Integration test: POST /subscription/upgrade redirects to Stripe, mock webhook, verify tier=premium |
| AC-10.1 - AC-10.3 (Subscription Cancellation) | APIs/POST /webhooks/stripe, Events/SubscriptionUpgraded | src/routes/auth.rs, user crate | Integration test: Mock customer.subscription.deleted webhook, verify tier=free, recipe creation fails at limit |
| AC-11.1 - AC-11.4 (Auth Middleware) | Middleware/auth.rs | src/middleware/auth.rs | Integration test: Access protected routes without/with expired/valid JWT, verify redirects and Auth extraction |
| AC-12.1 - AC-12.5 (Security) | password.rs, JWT generation, Stripe webhooks, Askama templates | user crate, src/routes/auth.rs | Unit tests: Verify Argon2 hash format, JWT cookie flags; Integration tests: Invalid webhook signature rejection, XSS prevention in error messages |

### Component Dependency Map

```
src/routes/auth.rs (Registration, Login, Logout, Password Reset, Stripe Webhooks)
  ↓ invokes
user crate (Commands: register_user, update_profile, validate_recipe_creation)
  ↓ writes
evento Event Store (Events: UserCreated, ProfileUpdated, SubscriptionUpgraded, PasswordResetRequested, PasswordChanged)
  ↓ triggers
evento Subscriptions (Read Model Projections)
  ↓ updates
SQLite users table (Read Model)
  ↑ queries
src/routes/profile.rs (Profile viewing/editing)
  ↑ reads
Auth Middleware (JWT validation, user_id extraction)
```

### Test Coverage Strategy

**Unit Tests (crates/user/tests/)**:
- Aggregate behavior: UserAggregate event handlers (user_created, profile_updated, subscription_upgraded, password_changed)
- Business logic: validate_recipe_creation with various tier/count combinations
- Password utilities: hash_password, verify_password, generate_reset_token
- JWT utilities: generate_jwt, validate_jwt with valid/expired/invalid tokens

**Integration Tests (tests/auth_tests.rs)**:
- Registration flow: Valid registration, duplicate email, password validation failures
- Login flow: Valid login, invalid credentials, JWT cookie verification
- Password reset flow: Request (valid/invalid email), completion (valid/expired/invalid token)
- Profile flow: View profile, update profile with various field combinations
- Subscription flow: Upgrade via Stripe, webhook handling (upgrade/cancellation)
- Auth middleware: Protected route access with no/invalid/valid JWT
- Security: Webhook signature verification, XSS prevention, Argon2 hash format

**E2E Tests (e2e/tests/auth.spec.ts)**:
- Complete onboarding flow: Register → View profile → Update profile → Create 10 recipes → Hit limit → Upgrade → Create recipe #11 succeeds
- Password reset flow: Request reset → Receive email → Click link → Reset password → Login with new password
- Subscription flow: Upgrade to premium → Cancel in Stripe → Recipe creation limited again

## Risks, Assumptions, Open Questions

### Risks

**R-1: Argon2 Hashing Performance**
- **Risk**: Argon2 default params (memory=65536 KB) may cause high CPU usage under load
- **Likelihood**: Medium
- **Impact**: High (degraded response times, server resource exhaustion)
- **Mitigation**: Load test with 100 concurrent registrations, tune Argon2 params if needed, consider async password hashing in background task
- **Contingency**: Reduce memory parameter to 32768 KB if performance issues occur

**R-2: JWT Secret Compromise**
- **Risk**: JWT secret leaked → attacker generates valid tokens for any user
- **Likelihood**: Low
- **Impact**: Critical (full authentication bypass)
- **Mitigation**: Store secret in Kubernetes secret (never commit), rotate secret periodically, implement token revocation list (future)
- **Contingency**: Immediate secret rotation, invalidate all existing tokens, force re-authentication

**R-3: Stripe Webhook Signature Bypass**
- **Risk**: Attacker sends fake webhook to upgrade users to premium without payment
- **Likelihood**: Low
- **Impact**: High (revenue loss, unauthorized premium access)
- **Mitigation**: Always verify stripe-signature header, log all webhook attempts, monitor for signature verification failures
- **Contingency**: Audit all subscription upgrades against Stripe dashboard, downgrade suspicious accounts

**R-4: Email Enumeration via Timing Attacks**
- **Risk**: Attacker measures response time differences to determine if email exists
- **Likelihood**: Medium
- **Impact**: Low (privacy leak, targeted phishing)
- **Mitigation**: Always return same response/timing for password reset requests regardless of email existence, use constant-time password verification
- **Contingency**: Rate limit password reset requests per IP, implement CAPTCHA on password reset form

**R-5: SMTP Service Downtime**
- **Risk**: SMTP service unavailable → password reset emails not delivered
- **Likelihood**: Medium
- **Impact**: Medium (user frustration, support tickets)
- **Mitigation**: Use reliable SMTP provider (SendGrid, Mailgun), implement retry logic with exponential backoff
- **Contingency**: Manual password reset via admin interface (future), status page showing email service issues

**R-6: evento Projection Lag**
- **Risk**: Read model not updated before redirect → user sees stale data
- **Likelihood**: Low (evento subscriptions typically < 100ms)
- **Impact**: Medium (user confusion, inconsistent UI)
- **Mitigation**: Profile pages query read model (already projected), critical flows wait for projection confirmation
- **Contingency**: Add polling or redirect delay if stale data issues observed in production

### Assumptions

**A-1**: Users accept 7-day JWT expiration without refresh token pattern (re-authentication acceptable)

**A-2**: Email verification on registration not required for MVP (reduces onboarding friction, acceptable spam risk)

**A-3**: Argon2 default parameters provide sufficient security without performance issues at 10K concurrent users

**A-4**: Stripe Checkout hosted page provides sufficient UX (no custom payment form needed)

**A-5**: Single JWT secret sufficient for MVP (no key rotation required initially)

**A-6**: HTTP-only, SameSite=Lax cookies provide sufficient CSRF protection without additional CSRF tokens

**A-7**: Password reset emails delivered within 5 minutes (acceptable latency for users)

**A-8**: evento projection lag < 100ms sufficient for all user flows (no explicit wait logic needed)

### Open Questions

**Q-1**: Should we implement account deletion/anonymization in Epic 1 or defer to compliance epic?
- **Decision Needed By**: Sprint planning for Epic 1
- **Impact**: Scope/timeline if included in Epic 1
- **Recommendation**: Defer to separate compliance epic (GDPR, data export/deletion together)

**Q-2**: Should premium subscription be monthly ($9.99) or offer annual discount ($99/year)?
- **Decision Needed By**: Before Stripe price creation
- **Impact**: Revenue model, subscription management complexity
- **Recommendation**: Start with monthly only for MVP, add annual option post-launch

**Q-3**: Should we support social authentication (Google, Facebook) in Epic 1?
- **Decision Needed By**: Sprint planning for Epic 1
- **Impact**: Scope/timeline, complexity (OAuth flows)
- **Recommendation**: Defer to post-MVP (email/password sufficient for MVP)

**Q-4**: Should password reset tokens be stored in evento event stream or read model only?
- **Decision Needed By**: Before implementation
- **Impact**: Architecture consistency, audit trail completeness
- **Recommendation**: Store PasswordResetRequested event in evento stream (audit trail), project to read model for fast lookup

**Q-5**: Should we implement rate limiting on login/registration endpoints?
- **Decision Needed By**: Before production deployment
- **Impact**: Security (brute force protection), user experience (legitimate users rate limited)
- **Recommendation**: Implement basic rate limiting (10 attempts per IP per minute) via Axum middleware before production launch

**Q-6**: Should we send welcome emails after registration?
- **Decision Needed By**: Sprint planning for Epic 1
- **Impact**: Scope, SMTP dependency, user experience
- **Recommendation**: Defer to post-MVP (focus on core auth flows first)

## Test Strategy Summary

### Unit Testing Strategy

**Scope**: Domain logic in `crates/user/` (aggregates, commands, password utilities, JWT utilities)

**Approach**:
- TDD enforced: Write failing test → Implement feature → Test passes
- In-memory evento executor for aggregate tests (no database required)
- Mock external dependencies (SMTP, Stripe) using test doubles

**Coverage Target**: 90% for user crate (higher than 80% project minimum due to security criticality)

**Key Test Cases**:
- Aggregate: UserAggregate rebuilds correct state from event stream
- Commands: register_user enforces email uniqueness, validates password length
- Freemium: validate_recipe_creation rejects free users at 10 recipes, allows premium users unlimited
- Passwords: hash_password produces Argon2id hashes, verify_password accepts correct passwords only
- JWT: generate_jwt creates valid tokens with correct claims, validate_jwt rejects expired/invalid tokens

### Integration Testing Strategy

**Scope**: HTTP routes in `src/routes/auth.rs` and `src/routes/profile.rs`, evento projections, database interactions

**Approach**:
- Spin up in-memory SQLite database for each test
- Run evento migrations + read model migrations
- Use reqwest client to call HTTP endpoints
- Verify database state after operations

**Coverage Target**: 85% for route handlers and middleware

**Key Test Cases**:
- Registration: POST /register creates user, sets JWT cookie, redirects to /dashboard
- Login: POST /login with invalid credentials returns error, valid credentials sets cookie
- Password reset: POST /password-reset sends email (mocked), POST /password-reset/:token updates password
- Profile: GET /profile renders user data, PUT /profile updates read model via projection
- Subscription: POST /subscription/upgrade redirects to Stripe, webhook handler upgrades tier
- Auth middleware: Protected routes reject missing/invalid JWT, extract user_id from valid JWT

### E2E Testing Strategy

**Scope**: Critical user flows end-to-end using real browser (Playwright)

**Approach**:
- Run full imkitchen server in test mode
- Use Playwright to automate browser interactions
- Mock external services (Stripe Checkout redirects, SMTP sends)

**Coverage Target**: 100% of critical user flows

**Key Test Cases**:
- Onboarding: Register → Profile setup → Create 10 recipes → Upgrade to premium → Create recipe #11
- Password reset: Request reset → Click email link → Reset password → Login with new password
- Freemium enforcement: Create 10 recipes as free user → Attempt #11 fails → Upgrade → Attempt #11 succeeds

### Test Execution

**Local Development**:
```bash
# Unit tests
cargo test -p user

# Integration tests
cargo test --test auth_tests

# E2E tests
cd e2e && npm test
```

**CI Pipeline** (GitHub Actions):
```yaml
- name: Unit Tests
  run: cargo test --all --lib

- name: Integration Tests
  run: cargo test --all --test '*'

- name: E2E Tests
  run: |
    cargo build --release
    ./target/release/imkitchen serve &
    cd e2e && npm ci && npx playwright test
```

### Test Data Management

**Fixtures** (tests/common/fixtures.rs):
```rust
pub fn create_test_user() -> User {
    User {
        id: "test-user-123".to_string(),
        email: "test@example.com".to_string(),
        password_hash: hash_password("password123").unwrap(),
        tier: SubscriptionTier::Free,
        recipe_count: 0,
        ..Default::default()
    }
}

pub fn create_premium_user() -> User {
    User {
        tier: SubscriptionTier::Premium,
        stripe_customer_id: Some("cus_test123".to_string()),
        ..create_test_user()
    }
}
```

### Performance Testing (Post-MVP)

**Load Testing** (k6):
- Registration: 100 concurrent users/second
- Login: 200 concurrent users/second
- Profile updates: 50 concurrent users/second
- Target: < 500ms response time at 95th percentile

**Stress Testing**:
- Argon2 hashing under load: Verify CPU usage stays < 80%
- JWT validation: 1000 requests/second, verify < 10ms middleware overhead

---

## Summary

This technical specification provides a comprehensive blueprint for implementing Epic 1: User Authentication & Profile Management for the imkitchen platform. The specification aligns with the event-sourced architecture defined in solution-architecture.md, leveraging evento for complete audit trails and CQRS for optimized read performance.

Key implementation highlights:
- **User domain crate** with evento aggregates, commands, events, and read model projections
- **JWT cookie-based authentication** with Argon2 password hashing and secure HTTP-only cookies
- **Freemium tier enforcement** at domain level (10 recipe limit for free users)
- **Stripe integration** for premium subscription upgrades via Checkout and webhook handlers
- **Password reset flow** with secure token-based verification and SMTP email delivery
- **Comprehensive test coverage** (90% unit, 85% integration, 100% E2E for critical flows)

All 12 stories are fully specified with authoritative acceptance criteria, traceability mapping to components, and detailed test strategies. The implementation follows TDD principles with clear component boundaries, security best practices (OWASP compliance), and observability through OpenTelemetry instrumentation.

**Next Steps**:
1. Review and approve technical specification
2. Create Jira tickets for 12 user stories with acceptance criteria
3. Begin TDD implementation starting with user crate unit tests
4. Implement evento projections and route handlers
5. Execute integration and E2E tests
6. Deploy to staging for QA validation

---

**Epic 1 tech-spec generated at docs/tech-spec-epic-1.md**
