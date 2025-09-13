use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::services::AuthService;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthError {
    pub message: String,
}

impl AuthError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

pub async fn require_authentication(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check if it's a Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Check if token is blacklisted
    if auth_service.is_token_blacklisted(token).await.unwrap_or(true) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Validate the JWT token
    let claims = auth_service
        .validate_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Add the user claims to the request extensions
    request.extensions_mut().insert(claims);

    // Continue with the request
    Ok(next.run(request).await)
}

pub async fn optional_authentication(
    State(auth_service): State<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract the Authorization header
    if let Some(auth_header) = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
    {
        // Check if it's a Bearer token
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..]; // Remove "Bearer " prefix

            // Check if token is not blacklisted and validate
            if !auth_service.is_token_blacklisted(token).await.unwrap_or(true) {
                if let Ok(claims) = auth_service.validate_token(token).await {
                    // Add the user claims to the request extensions
                    request.extensions_mut().insert(claims);
                }
            }
        }
    }

    // Continue with the request regardless of authentication status
    next.run(request).await
}

// CSRF protection middleware
pub async fn csrf_protection(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // For state-changing operations (POST, PUT, DELETE, PATCH)
    if matches!(
        request.method().as_str(),
        "POST" | "PUT" | "DELETE" | "PATCH"
    ) {
        // Check for CSRF token in header
        let csrf_token = request
            .headers()
            .get("X-CSRF-Token")
            .and_then(|header| header.to_str().ok());

        // For API endpoints, require CSRF token
        if request.uri().path().starts_with("/api/") {
            csrf_token.ok_or(StatusCode::FORBIDDEN)?;
            
            // Here you would validate the CSRF token against a stored value
            // For this implementation, we'll accept any non-empty token
            if csrf_token.unwrap().is_empty() {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

    Ok(next.run(request).await)
}