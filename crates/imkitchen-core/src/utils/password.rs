use bcrypt::{hash, verify, DEFAULT_COST};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password validation failed: {0}")]
    ValidationFailed(String),
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
    #[error("Password verification failed: {0}")]
    VerificationFailed(String),
}

/// OWASP Authentication Cheat Sheet compliant password validation
pub fn validate_password(password: &str) -> Result<(), PasswordError> {
    if password.len() < 8 {
        return Err(PasswordError::ValidationFailed(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(PasswordError::ValidationFailed(
            "Password must be no longer than 128 characters".to_string(),
        ));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password
        .chars()
        .any(|c| "!@#$%^&*()_+-=[]{}|;':\",./<>?".contains(c));

    let complexity_count = [has_lowercase, has_uppercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if complexity_count < 3 {
        return Err(PasswordError::ValidationFailed(
            "Password must contain at least 3 of: lowercase, uppercase, digits, special characters"
                .to_string(),
        ));
    }

    // Check for common weak patterns
    let lower_password = password.to_lowercase();
    let weak_patterns = [
        "password", "123456", "qwerty", "admin", "user", "login", "welcome", "test", "guest",
        "default", "root", "master",
    ];

    for pattern in &weak_patterns {
        if lower_password.contains(pattern) {
            return Err(PasswordError::ValidationFailed(format!(
                "Password cannot contain common weak pattern: {}",
                pattern
            )));
        }
    }

    Ok(())
}

/// Hash password using bcrypt with appropriate cost
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    validate_password(password)?;

    hash(password, DEFAULT_COST).map_err(|e| PasswordError::HashingFailed(e.to_string()))
}

/// Verify password against hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    verify(password, hash).map_err(|e| PasswordError::VerificationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_validation_success() {
        // Valid passwords with 3+ complexity types
        assert!(validate_password("MySecret123!").is_ok());
        assert!(validate_password("SecurePass1$").is_ok());
        assert!(validate_password("ComplexKey@1").is_ok());

        // Valid with lowercase, uppercase, digit (no special)
        assert!(validate_password("MySecret123").is_ok());

        // Valid with lowercase, uppercase, special (no digit)
        assert!(validate_password("MySecret!@#").is_ok());
    }

    #[test]
    fn test_password_validation_failure() {
        // Too short
        assert!(validate_password("short").is_err());

        // Too long
        let long_password = "a".repeat(129);
        assert!(validate_password(&long_password).is_err());

        // Not enough complexity (only lowercase)
        assert!(validate_password("alllowercase").is_err());

        // Not enough complexity (only 2 types)
        assert!(validate_password("lowercase123").is_err());

        // Contains weak pattern
        assert!(validate_password("MyPassword123").is_err()); // contains "password"
        assert!(validate_password("Admin123!").is_err()); // contains "admin"
    }

    #[tokio::test]
    async fn test_password_hashing_and_verification() {
        let password = "MySecureKey@123";

        // Hash the password
        let hash = hash_password(password).expect("Failed to hash password");

        // Verify correct password
        assert!(verify_password(password, &hash).expect("Verification failed"));

        // Verify incorrect password
        assert!(!verify_password("WrongPassword", &hash).expect("Verification failed"));
    }

    #[test]
    fn test_hash_invalid_password() {
        // Should fail validation before hashing
        let result = hash_password("short");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PasswordError::ValidationFailed(_)
        ));
    }
}
