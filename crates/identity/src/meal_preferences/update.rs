use evento::{Executor, ProjectionAggregate};
use imkitchen_types::meal_preferences::{Changed, RecipeTypesChanged};
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use validator::Validate;

#[derive(Validate)]
pub struct UpdateInput {
    #[validate(range(min = 1))]
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    #[validate(range(min = 0.1, max = 1.0))]
    pub cuisine_variety_weight: f32,
    /// Optional courses to generate. `MainCourse` is stripped — it is always
    /// generated — and an empty selection is valid.
    pub recipe_types: Vec<RecipeType>,
}

impl<E: Executor> super::Module<E> {
    pub async fn update(
        &self,
        id: impl Into<String>,
        input: UpdateInput,
    ) -> imkitchen_core::Result<()> {
        input.validate()?;

        let id = id.into();
        let preferences = self.load(&id).await?;

        // Canonicalise: drop MainCourse and anything duplicated, and impose a
        // stable order so stored events are comparable across writes.
        let recipe_types = RecipeType::OPTIONAL_VARIANTS
            .iter()
            .filter(|t| input.recipe_types.contains(t))
            .cloned()
            .collect::<Vec<_>>();

        preferences
            .write()?
            .event(&Changed {
                dietary_restrictions: input.dietary_restrictions,
                household_size: input.household_size,
                cuisine_variety_weight: input.cuisine_variety_weight,
            })
            .event(&RecipeTypesChanged { recipe_types })
            .requested_by(id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
