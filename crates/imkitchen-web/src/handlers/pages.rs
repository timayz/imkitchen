use crate::SharedState;
use askama::Template;
use axum::{
    extract::Form,
    extract::State,
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use imkitchen_core::models::user::{CreateUserRequest, LoginRequest};
use imkitchen_core::services::AuthService;
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Template)]
#[template(path = "pages/hello.html")]
pub struct HelloTemplate {}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct LoginTemplate {}

#[derive(Template)]
#[template(path = "pages/register.html")]
pub struct RegisterTemplate {}

#[derive(Template)]
#[template(path = "fragments/login-success.html")]
pub struct LoginSuccessTemplate {}

#[derive(Template)]
#[template(path = "fragments/login-error.html")]
pub struct LoginErrorTemplate {
    pub email: String,
    pub email_error: String,
    pub password_error: String,
    pub error_message: String,
}

#[derive(Template)]
#[template(path = "fragments/register-success.html")]
pub struct RegisterSuccessTemplate {}

#[derive(Template)]
#[template(path = "fragments/register-error.html")]
pub struct RegisterErrorTemplate {
    pub name: String,
    pub email: String,
    pub family_size: String,
    pub name_error: String,
    pub email_error: String,
    pub password_error: String,
    pub family_size_error: String,
    pub error_message: String,
}

#[derive(Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterFormData {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "familySize")]
    pub family_size: Option<String>,
}

/// Serve the hello world page
pub async fn hello_page() -> impl IntoResponse {
    info!("Hello world page requested");

    let template = HelloTemplate {};
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("Failed to render hello template: {}", e);
            Html("<h1>Hello World!</h1>".to_string())
        }
    }
}

/// Serve the login page
pub async fn login_page() -> impl IntoResponse {
    info!("Login page requested");

    let template = LoginTemplate {};
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("Failed to render login template: {}", e);
            Html("<h1>Error loading page</h1>".to_string())
        }
    }
}

/// Serve the registration page
pub async fn register_page() -> impl IntoResponse {
    info!("Register page requested");

    let template = RegisterTemplate {};
    match template.render() {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("Failed to render register template: {}", e);
            Html("<h1>Error loading page</h1>".to_string())
        }
    }
}

/// Handle login form submission (TwinSpark HTML endpoint)
pub async fn login_form_handler(
    State(shared_state): State<SharedState>,
    Form(form_data): Form<LoginFormData>,
) -> Response {
    info!("Login form submitted for: {}", form_data.email);

    let app_state = shared_state.read().await;
    let db_pool = match app_state.db.as_ref() {
        Some(pool) => pool.clone(),
        None => {
            let template = LoginErrorTemplate {
                email: form_data.email,
                email_error: "".to_string(),
                password_error: "".to_string(),
                error_message: "Database not available".to_string(),
            };
            return match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => Html("<div class='error'>System error</div>".to_string()).into_response(),
            };
        }
    };
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    let login_request = LoginRequest {
        email: form_data.email.clone(),
        password: form_data.password.clone(),
    };

    match auth_service
        .login_user(login_request, "127.0.0.1".to_string())
        .await
    {
        Ok(_user) => {
            let template = LoginSuccessTemplate {};
            match template.render() {
                Ok(html) => {
                    let mut headers = HeaderMap::new();
                    headers.insert("ts-location", "/".parse().unwrap());
                    (headers, Html(html)).into_response()
                }
                Err(_) => {
                    let mut headers = HeaderMap::new();
                    headers.insert("ts-location", "/".parse().unwrap());
                    (
                        headers,
                        Html("<div class='success'>Login successful!</div>".to_string()),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            warn!("Login failed for {}: {}", form_data.email, e);
            let template = LoginErrorTemplate {
                email: form_data.email,
                email_error: "".to_string(),
                password_error: "".to_string(),
                error_message: "Invalid email or password".to_string(),
            };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => {
                    Html("<div class='error'>Invalid credentials</div>".to_string()).into_response()
                }
            }
        }
    }
}

/// Handle registration form submission (TwinSpark HTML endpoint)
pub async fn register_form_handler(
    State(shared_state): State<SharedState>,
    Form(form_data): Form<RegisterFormData>,
) -> Response {
    info!("Registration form submitted for: {}", form_data.email);

    let app_state = shared_state.read().await;
    let db_pool = match app_state.db.as_ref() {
        Some(pool) => pool.clone(),
        None => {
            let template = RegisterErrorTemplate {
                name: form_data.name,
                email: form_data.email,
                family_size: form_data.family_size.unwrap_or_default(),
                name_error: "".to_string(),
                email_error: "".to_string(),
                password_error: "".to_string(),
                family_size_error: "".to_string(),
                error_message: "Database not available".to_string(),
            };
            return match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => Html("<div class='error'>System error</div>".to_string()).into_response(),
            };
        }
    };
    drop(app_state);

    let auth_service = AuthService::new(db_pool);

    let family_size = form_data
        .family_size
        .as_deref()
        .and_then(|f| f.parse::<i32>().ok())
        .unwrap_or(1);

    let create_request = CreateUserRequest {
        email: form_data.email.clone(),
        password: form_data.password.clone(),
        name: form_data.name.clone(),
        family_size: Some(family_size),
    };

    match auth_service
        .register_user(create_request, "127.0.0.1".to_string())
        .await
    {
        Ok(_) => {
            let template = RegisterSuccessTemplate {};
            match template.render() {
                Ok(html) => {
                    let mut headers = HeaderMap::new();
                    headers.insert("ts-location", "/".parse().unwrap());
                    (headers, Html(html)).into_response()
                }
                Err(_) => {
                    let mut headers = HeaderMap::new();
                    headers.insert("ts-location", "/".parse().unwrap());
                    (
                        headers,
                        Html(
                            "<div class='success'>Account created successfully!</div>".to_string(),
                        ),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            warn!("Registration failed for {}: {}", form_data.email, e);
            let error_message = if e.to_string().contains("already exists") {
                "Email already registered"
            } else if e.to_string().contains("Password validation") {
                "Password does not meet requirements"
            } else {
                "Registration failed"
            };

            let template = RegisterErrorTemplate {
                name: form_data.name,
                email: form_data.email,
                family_size: form_data.family_size.unwrap_or_default(),
                name_error: "".to_string(),
                email_error: if error_message.contains("Email") {
                    error_message.to_string()
                } else {
                    "".to_string()
                },
                password_error: if error_message.contains("Password") {
                    error_message.to_string()
                } else {
                    "".to_string()
                },
                family_size_error: "".to_string(),
                error_message: if !error_message.contains("Email")
                    && !error_message.contains("Password")
                {
                    error_message.to_string()
                } else {
                    "".to_string()
                },
            };
            match template.render() {
                Ok(html) => Html(html).into_response(),
                Err(_) => {
                    Html("<div class='error'>Registration failed</div>".to_string()).into_response()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login_page_renders() {
        let _response = login_page().await;
        // Test passes if no panic occurs during rendering
    }

    #[tokio::test]
    async fn test_register_page_renders() {
        let _response = register_page().await;
        // Test passes if no panic occurs during rendering
    }
}
