// Projections for user domain read models

pub mod user_profile_view;
pub mod user_preferences_view;
pub mod projection_maintenance;

// Re-export projection types
pub use user_profile_view::{UserProfileView, UserProfileProjectionBuilder};
pub use user_preferences_view::{
    UserPreferencesView, 
    UserPreferencesProjectionBuilder,
    RecipeCriteriaSummary,
    MealPlanningPreferences,
    CookingRecommendations,
    DailySchedule,
    ComplexityBalance,
    ProjectionMaintenanceInfo,
};
pub use projection_maintenance::{
    ProjectionMaintenanceManager,
    MaintenanceConfig,
    MaintenanceStats,
    ProjectionCacheInfo,
};
