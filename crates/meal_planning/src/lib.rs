pub mod aggregate;
pub mod algorithm;
pub mod commands;
pub mod error;
pub mod events;
pub mod read_model;
pub mod rotation;

pub use aggregate::MealPlanAggregate;
pub use algorithm::{MealPlanningAlgorithm, RecipeComplexityCalculator};
pub use commands::GenerateMealPlanCommand;
pub use error::MealPlanningError;
pub use events::{MealPlanGenerated, RecipeUsedInRotation};
pub use read_model::{
    meal_plan_projection, MealAssignmentReadModel, MealPlanQueries, MealPlanReadModel,
};
pub use rotation::{RotationState, RotationSystem};
