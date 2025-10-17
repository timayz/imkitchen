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
    EventoError(#[from] anyhow::Error),
}
