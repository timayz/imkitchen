use evento::Executor;
use imkitchen_types::recipe::{
    CuisineType, DietaryRestriction, Imported, Ingredient, Instruction, RecipeType,
};
use validator::Validate;

use super::UpdateInput;

#[derive(Validate, Clone)]
pub struct ImportInput {
    pub recipe_type: RecipeType,
    #[validate(length(min = 3, max = 100))]
    pub name: String,
    #[validate(url, length(min = 10, max = 255))]
    pub origin: Option<String>,
    #[validate(length(min = 3, max = 2000))]
    pub description: String,
    #[validate(range(min = 1))]
    pub household_size: u16,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub cuisine_type: CuisineType,
    #[validate(length(max = 2000))]
    pub advance_prep: String,
    pub accepts_accompaniment: bool,
    pub dietary_restrictions: Vec<DietaryRestriction>,
}

impl<E: Executor + Clone> super::Module<E> {
    pub async fn import(
        &self,
        input: ImportInput,
        request_by: impl Into<String>,
        owner_name: impl Into<Option<String>>,
    ) -> crate::Result<String> {
        input.validate()?;
        let request_by = request_by.into();

        if let Some(existing_id) = self
            .find_user_to_upsert(&request_by, input.origin.as_deref(), &input.name)
            .await?
        {
            if let Some(existing) = self.load(&existing_id).await? {
                if existing.owner_id == request_by {
                    self.update(
                        UpdateInput {
                            id: existing_id.clone(),
                            recipe_type: input.recipe_type,
                            name: input.name,
                            origin: input.origin,
                            description: input.description,
                            household_size: input.household_size,
                            prep_time: input.prep_time,
                            cook_time: input.cook_time,
                            ingredients: input.ingredients,
                            instructions: input.instructions,
                            dietary_restrictions: input.dietary_restrictions,
                            cuisine_type: input.cuisine_type,
                            accepts_accompaniment: input.accepts_accompaniment,
                            advance_prep: input.advance_prep,
                        },
                        &request_by,
                    )
                    .await?;

                    return Ok(existing_id);
                }
            }
        }

        Ok(evento::create()
            .event(&Imported {
                owner_name: owner_name.into(),
                name: input.name,
                origin: input.origin,
                description: input.description,
                recipe_type: input.recipe_type,
                cuisine_type: input.cuisine_type,
                household_size: input.household_size,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
                advance_prep: input.advance_prep,
                ingredients: input.ingredients,
                instructions: input.instructions,
                accepts_accompaniment: input.accepts_accompaniment,
                dietary_restrictions: input.dietary_restrictions,
            })
            .requested_by(request_by)
            .commit(&self.executor)
            .await?)
    }
}
