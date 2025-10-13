use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use user::{
    generate_jwt, query_user_for_login, register_user, verify_password, RegisterUserCommand,
    UserError,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub evento_executor: evento::Sqlite,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub email: String,
    pub password: String,
    pub password_confirm: String,
}

#[derive(Template)]
#[template(path = "pages/register.html")]
pub struct RegisterPageTemplate {
    pub error: String,
    pub user: Option<()>, // None for public pages
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginPageTemplate {
    pub error: String,
    pub user: Option<()>, // None for public pages
}

/// GET /register - Display registration form
#[tracing::instrument]
pub async fn get_register() -> impl IntoResponse {
    let template = RegisterPageTemplate {
        error: String::new(),
        user: None,
    };
    Html(template.render().unwrap())
}

/// POST /register - Handle registration form submission
#[tracing::instrument(skip(state, form), fields(email = %form.email))]
pub async fn post_register(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Response {
    // Validate password confirmation
    if form.password != form.password_confirm {
        let template = RegisterPageTemplate {
            error: "Passwords do not match".to_string(),
            user: None,
        };
        // Return 200 OK for TwinSpark form swap (progressive enhancement)
        return Html(template.render().unwrap()).into_response();
    }

    // Create command
    let command = RegisterUserCommand {
        email: form.email.clone(),
        password: form.password,
    };

    // Execute registration (evento event sourcing)
    match register_user(command, &state.evento_executor, &state.db_pool).await {
        Ok(aggregator_id) => {
            // Generate JWT token
            let token = match generate_jwt(
                aggregator_id,
                form.email,
                "free".to_string(),
                &state.jwt_secret,
            ) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to generate JWT token: {:?}", e);
                    let template = RegisterPageTemplate {
                        error: "Registration succeeded but failed to generate session token. Please try logging in.".to_string(),
                        user: None,
                    };
                    // Return 200 OK for TwinSpark form swap (progressive enhancement)
                    return Html(template.render().unwrap()).into_response();
                }
            };

            // Set HTTP-only secure cookie with CSRF protection
            let cookie = format!(
                "auth_token={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}; Path=/",
                token,
                7 * 24 * 60 * 60 // 7 days in seconds
            );

            // Redirect to dashboard using TwinSpark (progressive enhancement)
            // Returns 200 OK for proper form swap, ts-location triggers client-side navigation
            (
                StatusCode::OK,
                [
                    ("Set-Cookie", cookie.as_str()),
                    ("ts-location", "/dashboard"),
                ],
                (),
            )
                .into_response()
        }
        Err(e) => {
            let error_message = match e {
                UserError::EmailAlreadyExists => "Email already registered".to_string(),
                UserError::InvalidEmail => "Invalid email format".to_string(),
                UserError::PasswordTooShort => "Password must be at least 8 characters".to_string(),
                UserError::ValidationError(msg) => msg,
                _ => {
                    tracing::error!("Registration error: {:?}", e);
                    "Registration failed".to_string()
                }
            };

            // Return 200 OK for TwinSpark form swap (progressive enhancement)
            let template = RegisterPageTemplate {
                error: error_message,
                user: None,
            };

            Html(template.render().unwrap()).into_response()
        }
    }
}

/// GET /login - Display login form
#[tracing::instrument]
pub async fn get_login() -> impl IntoResponse {
    let template = LoginPageTemplate {
        error: String::new(),
        user: None,
    };
    Html(template.render().unwrap())
}

/// POST /login - Handle login form submission
#[tracing::instrument(skip(state, form), fields(email = %form.email))]
pub async fn post_login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    // Query user by email from read model
    let user = match query_user_for_login(&form.email, &state.db_pool).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // User not found - return generic error (AC: 4 - no user enumeration)
            tracing::warn!("Failed login attempt for email: {}", form.email);
            let template = LoginPageTemplate {
                error: "Invalid credentials".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
        Err(e) => {
            tracing::error!("Database error during login: {:?}", e);
            let template = LoginPageTemplate {
                error: "An error occurred. Please try again.".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    // Verify password (AC: 2)
    let password_valid = match verify_password(&form.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            tracing::error!("Password verification error: {:?}", e);
            let template = LoginPageTemplate {
                error: "An error occurred. Please try again.".to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    if !password_valid {
        // Password incorrect - return generic error (AC: 4 - no user enumeration)
        tracing::warn!(
            "Failed login attempt (incorrect password) for email: {}",
            form.email
        );
        let template = LoginPageTemplate {
            error: "Invalid credentials".to_string(),
            user: None,
        };
        return Html(template.render().unwrap()).into_response();
    }

    // Generate JWT token (AC: 3, 7)
    let token = match generate_jwt(user.id, user.email, user.tier, &state.jwt_secret) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to generate JWT token: {:?}", e);
            let template = LoginPageTemplate {
                error: "Login succeeded but failed to generate session token. Please try again."
                    .to_string(),
                user: None,
            };
            return Html(template.render().unwrap()).into_response();
        }
    };

    // Set HTTP-only secure cookie with CSRF protection (AC: 3, 6, 8)
    let cookie = format!(
        "auth_token={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}; Path=/",
        token,
        7 * 24 * 60 * 60 // 7 days in seconds (AC: 8)
    );

    // Redirect to dashboard using TwinSpark (progressive enhancement) (AC: 5)
    // Returns 200 OK for proper form swap, ts-location triggers client-side navigation
    (
        StatusCode::OK,
        [
            ("Set-Cookie", cookie.as_str()),
            ("ts-location", "/dashboard"),
        ],
        (),
    )
        .into_response()
}
