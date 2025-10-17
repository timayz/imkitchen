use thiserror::Error;

#[derive(Error, Debug)]
pub enum MealPlanningError {
    #[error(
        "Insufficient recipes: need at least {minimum} favorite recipes, but only have {current}"
    )]
    InsufficientRecipes { minimum: usize, current: usize },

    #[error("No active meal plan found for user")]
    NoActiveMealPlan,

    #[error("Meal plan already exists for this week")]
    MealPlanAlreadyExists,

    #[error("Invalid meal type: {0}")]
    InvalidMealType(String),

    #[error("Invalid date format: {0}")]
    InvalidDate(String),

    #[error("Recipe not found: {0}")]
    RecipeNotFound(String),

    #[error("Algorithm failed to generate meal plan: {0}")]
    AlgorithmError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Event sourcing error: {0}")]
    EventoError(String),

    // Story 3.6: Meal replacement errors
    #[error("Meal plan not found: {0}")]
    MealPlanNotFound(String),

    #[error("Meal plan is not active: {0}")]
    MealPlanNotActive(String),

    #[error("Meal assignment not found for date {0}, meal type {1}")]
    MealAssignmentNotFound(String, String),

    #[error("Recipe {0} is already assigned to this meal slot")]
    RecipeAlreadyAssigned(String),

    #[error("Recipe {0} is already used in rotation cycle {1}")]
    RecipeAlreadyUsedInRotation(String, u32),

    #[error("Rotation state error: {0}")]
    RotationStateError(String),

    // Story 3.7: Regeneration errors
    #[error("Unauthorized access: user {0} cannot access meal plan {1}")]
    UnauthorizedAccess(String, String),
}
