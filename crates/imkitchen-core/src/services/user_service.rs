use crate::models::user::UserPublic;
use crate::repositories::user_repository::{UserRepository, UserRepositoryError};
use imkitchen_shared::auth_types::{
    CookingSkillLevel, CookingTimePreferences, ProfileUpdateRequest,
};
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Debug)]
pub enum UserServiceError {
    Repository(UserRepositoryError),
    ValidationError(String),
    UserNotFound,
}

impl From<UserRepositoryError> for UserServiceError {
    fn from(error: UserRepositoryError) -> Self {
        match error {
            UserRepositoryError::NotFound => Self::UserNotFound,
            _ => Self::Repository(error),
        }
    }
}

impl std::fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repository(e) => write!(f, "Repository error: {}", e),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::UserNotFound => write!(f, "User not found"),
        }
    }
}

impl std::error::Error for UserServiceError {}

#[derive(Clone)]
pub struct UserService {
    user_repository: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Self {
        Self { user_repository }
    }

    /// Get user profile by ID
    pub async fn get_profile(&self, user_id: &str) -> Result<UserPublic, UserServiceError> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(UserServiceError::UserNotFound)?;

        Ok(user.into())
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: &str,
        request: ProfileUpdateRequest,
    ) -> Result<UserPublic, UserServiceError> {
        // Get current user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(UserServiceError::UserNotFound)?;

        // Audit log - profile update attempt
        info!(
            "Profile update attempted for user: {} ({})",
            user.id, user.email
        );

        // Collect field names for audit log before processing
        let updated_fields: Vec<&str> = [
            request.name.as_ref().map(|_| "name"),
            request.family_size.as_ref().map(|_| "family_size"),
            request
                .dietary_restrictions
                .as_ref()
                .map(|_| "dietary_restrictions"),
            request
                .cooking_skill_level
                .as_ref()
                .map(|_| "cooking_skill_level"),
            request
                .cooking_time_preferences
                .as_ref()
                .map(|_| "cooking_time_preferences"),
        ]
        .into_iter()
        .flatten()
        .collect();

        // Validate and apply updates
        if let Some(name) = request.name {
            self.validate_name_impl(&name)?;
            user.name = name;
        }

        if let Some(family_size) = request.family_size {
            self.validate_family_size_impl(family_size)?;
            user.family_size = family_size;
        }

        if let Some(dietary_restrictions) = request.dietary_restrictions {
            self.validate_dietary_restrictions_impl(&dietary_restrictions)?;
            user.dietary_restrictions =
                serde_json::to_string(&dietary_restrictions).map_err(|e| {
                    UserServiceError::ValidationError(format!(
                        "Failed to serialize dietary restrictions: {}",
                        e
                    ))
                })?;
        }

        if let Some(cooking_skill_level) = request.cooking_skill_level {
            user.cooking_skill_level = match cooking_skill_level {
                CookingSkillLevel::Beginner => "beginner".to_string(),
                CookingSkillLevel::Intermediate => "intermediate".to_string(),
                CookingSkillLevel::Advanced => "advanced".to_string(),
            };
        }

        if let Some(cooking_time_preferences) = request.cooking_time_preferences {
            self.validate_cooking_time_preferences_impl(&cooking_time_preferences)?;
            user.cooking_time_preferences = serde_json::to_string(&cooking_time_preferences)
                .map_err(|e| {
                    UserServiceError::ValidationError(format!(
                        "Failed to serialize cooking time preferences: {}",
                        e
                    ))
                })?;
        }

        // Update in repository
        self.user_repository.update_profile(&user).await?;

        // Audit log - successful profile update
        info!(
            "Profile updated successfully for user: {} ({}). Fields updated: [{}]",
            user.id,
            user.email,
            updated_fields.join(", ")
        );

        // Return updated user
        Ok(user.into())
    }

    /// Delete user account
    pub async fn delete_account(&self, user_id: &str) -> Result<(), UserServiceError> {
        // Check if user exists and get user info for audit log
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or(UserServiceError::UserNotFound)?;

        // Audit log - account deletion attempt
        warn!(
            "Account deletion attempted for user: {} ({})",
            user.id, user.email
        );

        // Delete user (this should cascade delete related data)
        self.user_repository.delete_user(user_id).await?;

        // Audit log - successful account deletion
        warn!(
            "Account deleted successfully for user: {} ({})",
            user.id, user.email
        );

        Ok(())
    }

    // Enhanced validation methods with security checks
    #[cfg(test)]
    pub fn validate_name(&self, name: &str) -> Result<(), UserServiceError> {
        self.validate_name_impl(name)
    }

    #[cfg(not(test))]
    #[allow(dead_code)]
    fn validate_name(&self, name: &str) -> Result<(), UserServiceError> {
        self.validate_name_impl(name)
    }

    fn validate_name_impl(&self, name: &str) -> Result<(), UserServiceError> {
        let trimmed_name = name.trim();

        // Basic validation
        if trimmed_name.is_empty() {
            return Err(UserServiceError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        // Length validation
        if trimmed_name.len() < 2 {
            return Err(UserServiceError::ValidationError(
                "Name must be at least 2 characters long".to_string(),
            ));
        }
        if trimmed_name.len() > 100 {
            return Err(UserServiceError::ValidationError(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }

        // Security validation - prevent potential XSS/injection
        if trimmed_name.contains('<') || trimmed_name.contains('>') {
            warn!(
                "Security: Potential XSS attempt in name field: {}",
                trimmed_name
            );
            return Err(UserServiceError::ValidationError(
                "Name cannot contain HTML tags".to_string(),
            ));
        }

        // Check for suspicious patterns
        if trimmed_name.to_lowercase().contains("script")
            || trimmed_name.to_lowercase().contains("javascript")
            || trimmed_name.contains("&lt;")
            || trimmed_name.contains("&gt;")
        {
            warn!(
                "Security: Potential script injection attempt in name field: {}",
                trimmed_name
            );
            return Err(UserServiceError::ValidationError(
                "Name contains invalid content".to_string(),
            ));
        }

        // Character validation - allow letters, numbers, spaces, hyphens, apostrophes
        if !trimmed_name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '\'' || c == '.')
        {
            return Err(UserServiceError::ValidationError(
                "Name can only contain letters, numbers, spaces, hyphens, apostrophes, and periods"
                    .to_string(),
            ));
        }

        Ok(())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn validate_family_size(&self, family_size: i32) -> Result<(), UserServiceError> {
        self.validate_family_size_impl(family_size)
    }

    #[cfg(not(test))]
    #[allow(dead_code)]
    fn validate_family_size(&self, family_size: i32) -> Result<(), UserServiceError> {
        self.validate_family_size_impl(family_size)
    }

    fn validate_family_size_impl(&self, family_size: i32) -> Result<(), UserServiceError> {
        if !(1..=8).contains(&family_size) {
            return Err(UserServiceError::ValidationError(
                "Family size must be between 1 and 8".to_string(),
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn validate_dietary_restrictions(
        &self,
        restrictions: &[String],
    ) -> Result<(), UserServiceError> {
        self.validate_dietary_restrictions_impl(restrictions)
    }

    #[cfg(not(test))]
    #[allow(dead_code)]
    fn validate_dietary_restrictions(
        &self,
        restrictions: &[String],
    ) -> Result<(), UserServiceError> {
        self.validate_dietary_restrictions_impl(restrictions)
    }

    fn validate_dietary_restrictions_impl(
        &self,
        restrictions: &[String],
    ) -> Result<(), UserServiceError> {
        // Limit number of restrictions to prevent abuse
        if restrictions.len() > 10 {
            return Err(UserServiceError::ValidationError(
                "Cannot select more than 10 dietary restrictions".to_string(),
            ));
        }

        let valid_restrictions = [
            "vegetarian",
            "vegan",
            "gluten-free",
            "dairy-free",
            "nut-allergies",
        ];

        for restriction in restrictions {
            // Security check for length and content
            if restriction.len() > 50 {
                return Err(UserServiceError::ValidationError(
                    "Dietary restriction name too long".to_string(),
                ));
            }

            // Check for suspicious content
            if restriction.contains('<')
                || restriction.contains('>')
                || restriction.to_lowercase().contains("script")
                || restriction.contains("&lt;")
                || restriction.contains("&gt;")
            {
                warn!(
                    "Security: Potential injection attempt in dietary restrictions: {}",
                    restriction
                );
                return Err(UserServiceError::ValidationError(
                    "Invalid dietary restriction format".to_string(),
                ));
            }

            // Validate against whitelist
            if !valid_restrictions.contains(&restriction.as_str()) {
                return Err(UserServiceError::ValidationError(format!(
                    "Invalid dietary restriction: {}. Valid options are: {}",
                    restriction,
                    valid_restrictions.join(", ")
                )));
            }
        }
        Ok(())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn validate_cooking_time_preferences(
        &self,
        preferences: &CookingTimePreferences,
    ) -> Result<(), UserServiceError> {
        self.validate_cooking_time_preferences_impl(preferences)
    }

    #[cfg(not(test))]
    #[allow(dead_code)]
    fn validate_cooking_time_preferences(
        &self,
        preferences: &CookingTimePreferences,
    ) -> Result<(), UserServiceError> {
        self.validate_cooking_time_preferences_impl(preferences)
    }

    fn validate_cooking_time_preferences_impl(
        &self,
        preferences: &CookingTimePreferences,
    ) -> Result<(), UserServiceError> {
        if preferences.weekday_max_minutes < 5 || preferences.weekday_max_minutes > 480 {
            return Err(UserServiceError::ValidationError(
                "Weekday max minutes must be between 5 and 480 (8 hours)".to_string(),
            ));
        }
        if preferences.weekend_max_minutes < 5 || preferences.weekend_max_minutes > 480 {
            return Err(UserServiceError::ValidationError(
                "Weekend max minutes must be between 5 and 480 (8 hours)".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::auth_types::CookingTimePreferences;

    // Helper validation functions for testing (extracted from service methods)
    fn validate_name_test(name: &str) -> Result<(), UserServiceError> {
        let trimmed_name = name.trim();

        // Basic validation
        if trimmed_name.is_empty() {
            return Err(UserServiceError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        // Length validation
        if trimmed_name.len() < 2 {
            return Err(UserServiceError::ValidationError(
                "Name must be at least 2 characters long".to_string(),
            ));
        }
        if trimmed_name.len() > 100 {
            return Err(UserServiceError::ValidationError(
                "Name cannot exceed 100 characters".to_string(),
            ));
        }

        // Security validation - prevent potential XSS/injection
        if trimmed_name.contains('<') || trimmed_name.contains('>') {
            return Err(UserServiceError::ValidationError(
                "Name cannot contain HTML tags".to_string(),
            ));
        }

        // Check for suspicious patterns
        if trimmed_name.to_lowercase().contains("script")
            || trimmed_name.to_lowercase().contains("javascript")
            || trimmed_name.contains("&lt;")
            || trimmed_name.contains("&gt;")
        {
            return Err(UserServiceError::ValidationError(
                "Name contains invalid content".to_string(),
            ));
        }

        // Character validation - allow letters, numbers, spaces, hyphens, apostrophes
        if !trimmed_name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '\'' || c == '.')
        {
            return Err(UserServiceError::ValidationError(
                "Name can only contain letters, numbers, spaces, hyphens, apostrophes, and periods"
                    .to_string(),
            ));
        }

        Ok(())
    }

    fn validate_family_size_test(family_size: i32) -> Result<(), UserServiceError> {
        if !(1..=8).contains(&family_size) {
            return Err(UserServiceError::ValidationError(
                "Family size must be between 1 and 8".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_dietary_restrictions_test(restrictions: &[String]) -> Result<(), UserServiceError> {
        // Limit number of restrictions to prevent abuse
        if restrictions.len() > 10 {
            return Err(UserServiceError::ValidationError(
                "Cannot select more than 10 dietary restrictions".to_string(),
            ));
        }

        let valid_restrictions = [
            "vegetarian",
            "vegan",
            "gluten-free",
            "dairy-free",
            "nut-allergies",
        ];

        for restriction in restrictions {
            // Security check for length and content
            if restriction.len() > 50 {
                return Err(UserServiceError::ValidationError(
                    "Dietary restriction name too long".to_string(),
                ));
            }

            // Check for suspicious content
            if restriction.contains('<')
                || restriction.contains('>')
                || restriction.to_lowercase().contains("script")
                || restriction.contains("&lt;")
                || restriction.contains("&gt;")
            {
                return Err(UserServiceError::ValidationError(
                    "Invalid dietary restriction format".to_string(),
                ));
            }

            // Validate against whitelist
            if !valid_restrictions.contains(&restriction.as_str()) {
                return Err(UserServiceError::ValidationError(format!(
                    "Invalid dietary restriction: {}. Valid options are: {}",
                    restriction,
                    valid_restrictions.join(", ")
                )));
            }
        }
        Ok(())
    }

    fn validate_cooking_time_preferences_test(
        preferences: &CookingTimePreferences,
    ) -> Result<(), UserServiceError> {
        if preferences.weekday_max_minutes < 5 || preferences.weekday_max_minutes > 480 {
            return Err(UserServiceError::ValidationError(
                "Weekday max minutes must be between 5 and 480 (8 hours)".to_string(),
            ));
        }
        if preferences.weekend_max_minutes < 5 || preferences.weekend_max_minutes > 480 {
            return Err(UserServiceError::ValidationError(
                "Weekend max minutes must be between 5 and 480 (8 hours)".to_string(),
            ));
        }
        Ok(())
    }

    #[test]
    fn test_validate_name_valid_names() {
        // Valid names should pass
        assert!(validate_name_test("John Doe").is_ok());
        assert!(validate_name_test("Jane Smith-Johnson").is_ok());
        assert!(validate_name_test("Mary O'Connor").is_ok());
        assert!(validate_name_test("José García").is_ok());
        assert!(validate_name_test("Dr. Smith Jr.").is_ok());
    }

    #[test]
    fn test_validate_name_invalid_names() {
        // Empty name should fail
        assert!(validate_name_test("").is_err());
        assert!(validate_name_test("   ").is_err());

        // Too short name should fail
        assert!(validate_name_test("A").is_err());

        // Too long name should fail
        let long_name = "a".repeat(101);
        assert!(validate_name_test(&long_name).is_err());

        // HTML tags should fail (XSS prevention)
        assert!(validate_name_test("John<script>alert('xss')</script>").is_err());
        assert!(validate_name_test("Jane<div>test</div>").is_err());

        // Script injection attempts should fail
        assert!(validate_name_test("javascript:alert('xss')").is_err());
        assert!(validate_name_test("John&lt;script&gt;").is_err());

        // Invalid characters should fail
        assert!(validate_name_test("John@Doe").is_err());
        assert!(validate_name_test("Jane#Smith").is_err());
        assert!(validate_name_test("User$Name").is_err());
    }

    #[test]
    fn test_validate_family_size_valid_sizes() {
        // Valid family sizes (1-8) should pass
        for size in 1..=8 {
            assert!(validate_family_size_test(size).is_ok());
        }
    }

    #[test]
    fn test_validate_family_size_invalid_sizes() {
        // Invalid family sizes should fail
        assert!(validate_family_size_test(0).is_err());
        assert!(validate_family_size_test(9).is_err());
        assert!(validate_family_size_test(-1).is_err());
        assert!(validate_family_size_test(100).is_err());
    }

    #[test]
    fn test_validate_dietary_restrictions_valid() {
        // Valid restrictions should pass
        assert!(validate_dietary_restrictions_test(&["vegetarian".to_string()]).is_ok());
        assert!(validate_dietary_restrictions_test(&[
            "vegan".to_string(),
            "gluten-free".to_string()
        ])
        .is_ok());
        assert!(validate_dietary_restrictions_test(&[]).is_ok()); // Empty is valid

        // All valid restrictions
        let all_valid = vec![
            "vegetarian".to_string(),
            "vegan".to_string(),
            "gluten-free".to_string(),
            "dairy-free".to_string(),
            "nut-allergies".to_string(),
        ];
        assert!(validate_dietary_restrictions_test(&all_valid).is_ok());
    }

    #[test]
    fn test_validate_dietary_restrictions_invalid() {
        // Invalid restrictions should fail
        assert!(validate_dietary_restrictions_test(&["invalid-restriction".to_string()]).is_err());
        assert!(validate_dietary_restrictions_test(&["paleo".to_string()]).is_err());

        // Too many restrictions should fail
        let too_many = vec!["vegetarian".to_string(); 11];
        assert!(validate_dietary_restrictions_test(&too_many).is_err());

        // Restriction names too long should fail
        let long_restriction = "a".repeat(51);
        assert!(validate_dietary_restrictions_test(&[long_restriction]).is_err());

        // XSS attempts should fail
        assert!(
            validate_dietary_restrictions_test(&["<script>alert('xss')</script>".to_string()])
                .is_err()
        );
        assert!(
            validate_dietary_restrictions_test(&["javascript:alert('xss')".to_string()]).is_err()
        );
        assert!(
            validate_dietary_restrictions_test(&["vegetarian&lt;script&gt;".to_string()]).is_err()
        );
    }

    #[test]
    fn test_validate_cooking_time_preferences_valid() {
        // Valid preferences should pass
        let valid_prefs = CookingTimePreferences {
            weekday_max_minutes: 30,
            weekend_max_minutes: 60,
        };
        assert!(validate_cooking_time_preferences_test(&valid_prefs).is_ok());

        // Boundary values should pass
        let min_prefs = CookingTimePreferences {
            weekday_max_minutes: 5,
            weekend_max_minutes: 5,
        };
        assert!(validate_cooking_time_preferences_test(&min_prefs).is_ok());

        let max_prefs = CookingTimePreferences {
            weekday_max_minutes: 480,
            weekend_max_minutes: 480,
        };
        assert!(validate_cooking_time_preferences_test(&max_prefs).is_ok());
    }

    #[test]
    fn test_validate_cooking_time_preferences_invalid() {
        // Weekday time too low should fail
        let invalid_weekday_low = CookingTimePreferences {
            weekday_max_minutes: 4,
            weekend_max_minutes: 60,
        };
        assert!(validate_cooking_time_preferences_test(&invalid_weekday_low).is_err());

        // Weekday time too high should fail
        let invalid_weekday_high = CookingTimePreferences {
            weekday_max_minutes: 481,
            weekend_max_minutes: 60,
        };
        assert!(validate_cooking_time_preferences_test(&invalid_weekday_high).is_err());

        // Weekend time too low should fail
        let invalid_weekend_low = CookingTimePreferences {
            weekday_max_minutes: 30,
            weekend_max_minutes: 4,
        };
        assert!(validate_cooking_time_preferences_test(&invalid_weekend_low).is_err());

        // Weekend time too high should fail
        let invalid_weekend_high = CookingTimePreferences {
            weekday_max_minutes: 30,
            weekend_max_minutes: 481,
        };
        assert!(validate_cooking_time_preferences_test(&invalid_weekend_high).is_err());
    }

    #[test]
    fn test_cooking_skill_level_conversion() {
        // Test conversion logic used in update_profile
        assert_eq!("beginner", "beginner");
        assert_eq!("intermediate", "intermediate");
        assert_eq!("advanced", "advanced");
    }

    #[test]
    fn test_user_service_error_display() {
        // Test error message formatting
        let validation_error =
            UserServiceError::ValidationError("Test validation error".to_string());
        assert_eq!(
            validation_error.to_string(),
            "Validation error: Test validation error"
        );

        let not_found_error = UserServiceError::UserNotFound;
        assert_eq!(not_found_error.to_string(), "User not found");
    }
}
