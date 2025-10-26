pub mod aggregate;
pub mod commands;
pub mod error;
pub mod events;
pub mod jwt;
pub mod password;
pub mod read_model;
pub mod types;

// Re-export main types
pub use aggregate::UserAggregate;
pub use commands::{
    complete_profile, register_user, reset_password, set_dietary_restrictions, set_household_size,
    set_weeknight_availability, update_profile, upgrade_subscription, CompleteProfileCommand,
    RegisterUserCommand, ResetPasswordCommand, SetDietaryRestrictionsCommand,
    SetHouseholdSizeCommand, SetWeeknightAvailabilityCommand, UpdateProfileCommand,
    UpgradeSubscriptionCommand,
};
pub use error::{UserError, UserResult};
pub use events::{
    DietaryRestrictionsSet, HouseholdSizeSet, PasswordChanged, ProfileCompleted, ProfileUpdated,
    RecipeCreated, RecipeDeleted, RecipeShared, SubscriptionUpgraded, UserCreated,
    UserMealPlanningPreferencesUpdated, WeeknightAvailabilitySet,
};
pub use jwt::{generate_jwt, generate_reset_token, validate_jwt, Claims};
pub use password::{hash_password, verify_password};
pub use read_model::{query_user_by_email, query_user_for_login, user_projection, UserLoginData};
pub use types::{DietaryRestriction, SkillLevel, SubscriptionTier, TimeRange, UserPreferences};
