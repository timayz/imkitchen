use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params,
};

use crate::error::{UserError, UserResult};

/// Hash a password using Argon2id with OWASP-recommended parameters
/// - Memory: 65536 KB (64 MB)
/// - Iterations: 3
/// - Parallelism: 4
/// - Target: ~100ms per operation
pub fn hash_password(password: &str) -> UserResult<String> {
    // OWASP recommended parameters for Argon2id
    let params =
        Params::new(65536, 3, 4, None).map_err(|e| UserError::HashingError(e.to_string()))?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| UserError::HashingError(e.to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verify a password against an Argon2 hash
pub fn verify_password(password: &str, hash: &str) -> UserResult<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| UserError::HashingError(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
