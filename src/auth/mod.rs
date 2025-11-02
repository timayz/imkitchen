//! Authentication module
//!
//! Provides JWT-based authentication with HTTP-only cookies

pub mod jwt;
pub mod middleware;

pub use jwt::{generate_token, validate_token, AuthUser, Claims};
pub use middleware::{auth_middleware, get_auth_user, AuthState};

/// Cookie name for JWT token
pub const AUTH_COOKIE_NAME: &str = "auth_token";
