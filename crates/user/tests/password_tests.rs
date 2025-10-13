use user::{hash_password, verify_password};

#[test]
fn test_hash_password_produces_argon2_hash() {
    let password = "test_password_123";
    let hash = hash_password(password).expect("Failed to hash password");

    // Argon2 hashes start with $argon2
    assert!(hash.starts_with("$argon2"));
}

#[test]
fn test_verify_password_correct() {
    let password = "test_password_123";
    let hash = hash_password(password).expect("Failed to hash password");

    let result = verify_password(password, &hash).expect("Failed to verify password");
    assert!(result, "Correct password should verify");
}

#[test]
fn test_verify_password_incorrect() {
    let password = "test_password_123";
    let wrong_password = "wrong_password";
    let hash = hash_password(password).expect("Failed to hash password");

    let result = verify_password(wrong_password, &hash).expect("Failed to verify password");
    assert!(!result, "Incorrect password should not verify");
}

#[test]
fn test_hash_produces_unique_hashes() {
    let password = "test_password_123";
    let hash1 = hash_password(password).expect("Failed to hash password");
    let hash2 = hash_password(password).expect("Failed to hash password");

    // Same password should produce different hashes due to salt
    assert_ne!(hash1, hash2);
}
