use evento::{Executor, metadata::Metadata};
use sha3::{Digest, Sha3_224};
use validator::Validate;

use imkitchen_shared::recipe::{
    AdvancePrepChanged, BasicInformationChanged, CuisineType, CuisineTypeChanged,
    DietaryRestriction, DietaryRestrictionsChanged, Ingredient, IngredientsChanged, Instruction,
    InstructionsChanged, MainCourseOptionsChanged, RecipeType, RecipeTypeChanged,
};

#[derive(Validate, Clone)]
pub struct UpdateInput {
    pub id: String,
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
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_type: CuisineType,
    pub accepts_accompaniment: bool,
    #[validate(length(max = 2000))]
    pub advance_prep: String,
}

impl<'a, E: Executor + Clone> super::Command<'a, E> {
    pub async fn update(
        &self,
        input: UpdateInput,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        if self.is_deleted {
            imkitchen_shared::not_found!("recipe");
        }

        let request_by = request_by.into();
        if self.owner_id != request_by {
            imkitchen_shared::forbidden!("not owner of recipe");
        }

        let mut builder = self
            .aggregator()
            .metadata(&Metadata::new(request_by))
            .to_owned();

        let mut has_data = false;

        if self.recipe_type.0 != input.recipe_type {
            has_data = true;
            builder.event(&RecipeTypeChanged {
                recipe_type: input.recipe_type,
            });
        }

        if self.cuisine_type.0 != input.cuisine_type {
            has_data = true;
            builder.event(&CuisineTypeChanged {
                cuisine_type: input.cuisine_type,
            });
        }

        let mut hasher = Sha3_224::default();
        hasher.update(&input.name);
        hasher.update(&input.description);
        hasher.update(input.household_size.to_string());
        hasher.update(input.prep_time.to_string());
        hasher.update(input.cook_time.to_string());

        let basic_information_hash = hasher.finalize()[..].to_vec();

        if self.basic_information_hash != basic_information_hash {
            has_data = true;
            builder.event(&BasicInformationChanged {
                name: input.name,
                description: input.description,
                household_size: input.household_size,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
            });
        }

        let mut hasher = Sha3_224::default();

        for instruction in input.instructions.iter() {
            hasher.update(&instruction.description);
            hasher.update(instruction.time_next.to_string());
        }

        let instructions_hash = hasher.finalize()[..].to_vec();

        if self.instructions_hash != instructions_hash {
            has_data = true;
            builder.event(&InstructionsChanged {
                instructions: input.instructions.to_vec(),
            });
        }

        let mut hasher = Sha3_224::default();

        for ingredient in input.ingredients.iter() {
            hasher.update(&ingredient.name);
            hasher.update(ingredient.quantity.to_string());

            if let Some(unit) = &ingredient.unit {
                hasher.update(unit.to_string());
            }

            if let Some(catagory) = &ingredient.category {
                hasher.update(catagory.to_string());
            }
        }

        let ingredient_hash = hasher.finalize()[..].to_vec();

        if self.ingredients_hash != ingredient_hash {
            has_data = true;
            builder.event(&IngredientsChanged {
                ingredients: input.ingredients,
            });
        }

        let mut hasher = Sha3_224::default();

        for restriction in input.dietary_restrictions.iter() {
            hasher.update(restriction.to_string());
        }

        let dietary_restrictions_hash = hasher.finalize()[..].to_vec();

        if self.dietary_restrictions_hash != dietary_restrictions_hash {
            has_data = true;
            builder.event(&DietaryRestrictionsChanged {
                dietary_restrictions: input.dietary_restrictions,
            });
        }

        if self.accepts_accompaniment != input.accepts_accompaniment {
            has_data = true;
            builder.event(&MainCourseOptionsChanged {
                accepts_accompaniment: input.accepts_accompaniment,
            });
        }

        let mut hasher = Sha3_224::default();
        hasher.update(&input.advance_prep);

        let advance_prep_hash = hasher.finalize()[..].to_vec();
        if self.advance_prep_hash != advance_prep_hash {
            has_data = true;
            builder.event(&AdvancePrepChanged {
                advance_prep: input.advance_prep,
            });
        }
        if !has_data {
            return Ok(());
        }

        builder.commit(self.executor).await?;

        Ok(())
    }
}
