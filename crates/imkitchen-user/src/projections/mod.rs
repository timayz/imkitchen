// Projections for user domain read models

pub mod projection_maintenance;
pub mod user_preferences_view;
pub mod user_profile_view;

// Re-export projection types
pub use projection_maintenance::{
    MaintenanceConfig, MaintenanceStats, ProjectionCacheInfo, ProjectionMaintenanceManager,
};
pub use user_preferences_view::{
    ComplexityBalance, CookingRecommendations, DailySchedule, MealPlanningPreferences,
    ProjectionMaintenanceInfo, RecipeCriteriaSummary, UserPreferencesProjectionBuilder,
    UserPreferencesView,
};
pub use user_profile_view::{UserProfileProjectionBuilder, UserProfileView};
