//! Login route handlers

use super::{render_template, AppState};
use crate::auth::{generate_token, AUTH_COOKIE_NAME};
use crate::queries::user::get_user_by_email;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{cookie::Cookie, CookieJar, Form};
use imkitchen_user::command::{Command, LoginUserInput};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

/// Login page template
#[derive(Template)]
#[template(path = "pages/auth/login.html")]
struct LoginPageTemplate {
    error: Option<String>,
}

/// Login form data
#[derive(Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

/// GET /auth/login - Show login form
pub async fn get_login() -> Response {
    render_template(LoginPageTemplate { error: None })
}

/// POST /auth/login - Handle login submission
pub async fn post_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> (CookieJar, Response) {
    info!(email = %form.email, "Processing login");

    // Look up user by email in query database
    let user = match get_user_by_email(&state.query_pool, &form.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!(email = %form.email, "User not found");
            return (
                jar,
                render_template(LoginPageTemplate {
                    error: Some("Invalid email or password".to_string()),
                }),
            );
        }
        Err(e) => {
            error!(error = %e, "Database error during login");
            return (
                jar,
                render_template(LoginPageTemplate {
                    error: Some("An error occurred. Please try again.".to_string()),
                }),
            );
        }
    };

    // Create login input
    let input = LoginUserInput {
        email: form.email.clone(),
        password: form.password,
    };

    let metadata = EventMetadata {
        user_id: Some(user.id.clone()),
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());

    // Verify password and emit login event
    if let Err(e) = command
        .login_user(
            input,
            user.id.clone(),
            user.hashed_password.clone(),
            metadata,
        )
        .await
    {
        error!(error = %e, "Login failed");
        return (
            jar,
            render_template(LoginPageTemplate {
                error: Some("Invalid email or password".to_string()),
            }),
        );
    }

    // Generate JWT token
    let token = match generate_token(
        user.id.clone(),
        user.is_admin,
        &state.jwt_secret,
        state.jwt_lifetime_seconds,
    ) {
        Ok(token) => token,
        Err(e) => {
            error!(error = %e, "Failed to generate token");
            return (
                jar,
                render_template(LoginPageTemplate {
                    error: Some("An error occurred. Please try again.".to_string()),
                }),
            );
        }
    };

    // Set HTTP-only cookie
    let cookie = Cookie::build((AUTH_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .build();

    let jar = jar.add(cookie);

    info!(user_id = %user.id, "User logged in successfully");

    // Use ts-location header for TwinSpark redirect
    let mut response = Html("").into_response();
    response
        .headers_mut()
        .insert("ts-location", "/".parse().unwrap());

    (jar, response)
}

/// POST /auth/logout - Clear session cookie
pub async fn post_logout(jar: CookieJar) -> (CookieJar, Redirect) {
    let jar = jar.remove(Cookie::from(AUTH_COOKIE_NAME));
    (jar, Redirect::to("/"))
}
