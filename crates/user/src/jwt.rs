use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub tier: String,
    pub exp: usize, // Expiration time (as UTC timestamp)
    pub iat: usize, // Issued at (as UTC timestamp)
}

/// Generate a JWT token for a user
/// Token expires in 7 days
/// Uses HS256 algorithm with secret from config
pub fn generate_jwt(user_id: String, email: String, tier: String, secret: &str) -> Result<String> {
    generate_jwt_with_expiration(user_id, email, tier, secret, 7 * 24 * 60 * 60)
}

/// Generate a password reset JWT token
/// Token expires in 1 hour (3600 seconds)
/// Uses HS256 algorithm with secret from config
pub fn generate_reset_token(user_id: String, email: String, secret: &str) -> Result<String> {
    generate_jwt_with_expiration(user_id, email, "reset".to_string(), secret, 60 * 60)
}

/// Generate a JWT token with custom expiration
/// Internal helper function to support different token types
fn generate_jwt_with_expiration(
    user_id: String,
    email: String,
    tier: String,
    secret: &str,
    expiration_seconds: u64,
) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current time")?
        .as_secs() as usize;

    let expiration = now + expiration_seconds as usize;

    let claims = Claims {
        sub: user_id,
        email,
        tier,
        exp: expiration,
        iat: now,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("Failed to encode JWT")?;

    Ok(token)
}

/// Validate and decode a JWT token
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .context("Failed to decode JWT")?;

    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_jwt() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let email = "test@example.com".to_string();
        let tier = "free".to_string();
        let secret = "test_secret_key_minimum_32_characters_long";

        let token = generate_jwt(user_id.clone(), email.clone(), tier.clone(), secret).unwrap();

        let claims = validate_jwt(&token, secret).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
        assert_eq!(claims.tier, tier);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_invalid_secret_fails_validation() {
        let user_id = "550e8400-e29b-41d4-a716-446655440000".to_string();
        let email = "test@example.com".to_string();
        let tier = "free".to_string();
        let secret = "test_secret_key_minimum_32_characters_long";

        let token = generate_jwt(user_id, email, tier, secret).unwrap();

        let result = validate_jwt(&token, "wrong_secret");
        assert!(result.is_err());
    }
}
