use evento::{Executor, metadata::Metadata};
use validator::Validate;

use crate::{CuisineType, Imported, Ingredient, Instruction, RecipeType};

#[derive(Validate, Clone)]
pub struct ImportInput {
    pub recipe_type: RecipeType,
    #[validate(length(min = 3, max = 50))]
    pub name: String,
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
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn import(
        executor: &E,
        input: ImportInput,
        request_by: impl Into<String>,
        owner_name: impl Into<Option<String>>,
    ) -> imkitchen_shared::Result<String> {
        input.validate()?;
        let request_by = request_by.into();

        Ok(evento::create()
            .event(&Imported {
                owner_name: owner_name.into(),
                name: input.name,
                description: input.description,
                recipe_type: input.recipe_type,
                cuisine_type: input.cuisine_type,
                household_size: input.household_size,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
                advance_prep: input.advance_prep,
                ingredients: input.ingredients,
                instructions: input.instructions,
            })
            .metadata(&Metadata::new(request_by))
            .commit(executor)
            .await?)
    }
}
