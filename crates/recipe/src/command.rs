use evento::{Executor, LoadResult};
use imkitchen_shared::Metadata;
use sha3::{Digest, Sha3_224};
use sqlx::SqlitePool;
use validator::Validate;

use crate::{
    AccompanimentType, AdvancePrepChanged, BasicInformationChanged, Created, CuisineType,
    CuisineTypeChanged, DietaryRestriction, DietaryRestrictionsChanged, Ingredient,
    IngredientsChanged, Instruction, InstructionsChanged, MainCourseOptionsChanged, Recipe,
    RecipeType, RecipeTypeChanged,
};

#[derive(Validate, Clone)]
pub struct UpdateInput {
    pub id: String,
    pub recipe_type: RecipeType,
    #[validate(length(min = 3, max = 30))]
    pub name: String,
    #[validate(length(min = 3, max = 2000))]
    pub description: String,
    pub prep_time: u16,
    pub cook_time: u16,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_type: CuisineType,
    pub accepts_accompaniment: bool,
    pub preferred_accompaniment_types: Vec<AccompanimentType>,
    #[validate(length(min = 3, max = 2000))]
    pub advance_prep: String,
}

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<Recipe>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<Recipe>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn create(&self, metadata: Metadata) -> imkitchen_shared::Result<String> {
        Ok(evento::create::<Recipe>()
            .data(&Created {
                name: "".to_owned(),
            })?
            .metadata(&metadata)?
            .commit(&self.0)
            .await?)
    }

    pub async fn update(
        &self,
        input: UpdateInput,
        metadata: Metadata,
    ) -> imkitchen_shared::Result<()> {
        input.validate()?;

        let recipe = self.load(&input.id).await?;
        let mut builder = evento::save_with::<Recipe>(recipe.clone()).metadata(&metadata)?;
        let mut has_data = false;

        if recipe.item.recipe_type != input.recipe_type {
            has_data = true;
            builder = builder.data(&RecipeTypeChanged {
                recipe_type: input.recipe_type,
            })?;
        }

        if recipe.item.cuisine_type != input.cuisine_type {
            has_data = true;
            builder = builder.data(&CuisineTypeChanged {
                cuisine_type: input.cuisine_type,
            })?;
        }

        let mut hasher = Sha3_224::default();
        hasher.update(&input.name);
        hasher.update(&input.description);
        hasher.update(input.prep_time.to_string());
        hasher.update(input.cook_time.to_string());

        let basic_information_hash = hasher.finalize()[..].to_vec();

        if recipe.item.basic_information_hash != basic_information_hash {
            has_data = true;
            builder = builder.data(&BasicInformationChanged {
                name: input.name,
                description: input.description,
                prep_time: input.prep_time,
                cook_time: input.cook_time,
            })?;
        }

        let mut hasher = Sha3_224::default();

        for instruction in input.instructions.iter() {
            hasher.update(&instruction.description);
            hasher.update(instruction.time_before_next.to_string());
        }

        let instructions_hash = hasher.finalize()[..].to_vec();

        if recipe.item.instructions_hash != instructions_hash {
            has_data = true;
            builder = builder.data(&InstructionsChanged {
                instructions: input.instructions.to_vec(),
            })?;
        }

        let mut hasher = Sha3_224::default();

        for ingredient in input.ingredients.iter() {
            hasher.update(&ingredient.name);
            hasher.update(ingredient.unit.to_string());
            hasher.update(&ingredient.unit_type);
        }

        let ingredient_hash = hasher.finalize()[..].to_vec();

        if recipe.item.ingredients_hash != ingredient_hash {
            has_data = true;
            builder = builder.data(&IngredientsChanged {
                ingredients: input.ingredients,
            })?;
        }

        let mut hasher = Sha3_224::default();

        for restriction in input.dietary_restrictions.iter() {
            hasher.update(restriction.to_string());
        }

        let dietary_restrictions_hash = hasher.finalize()[..].to_vec();

        if recipe.item.dietary_restrictions_hash != dietary_restrictions_hash {
            has_data = true;
            builder = builder.data(&DietaryRestrictionsChanged {
                dietary_restrictions: input.dietary_restrictions,
            })?;
        }

        let mut hasher = Sha3_224::default();
        hasher.update(input.accepts_accompaniment.to_string());

        for preferred in input.preferred_accompaniment_types.iter() {
            hasher.update(preferred.to_string());
        }

        let main_option_hash = hasher.finalize()[..].to_vec();
        if recipe.item.main_option_hash != main_option_hash {
            has_data = true;
            builder = builder.data(&MainCourseOptionsChanged {
                accepts_accompaniment: input.accepts_accompaniment,
                preferred_accompaniment_types: input.preferred_accompaniment_types,
            })?;
        }

        let mut hasher = Sha3_224::default();
        hasher.update(&input.advance_prep);

        let advance_prep_hash = hasher.finalize()[..].to_vec();
        if recipe.item.advance_prep_hash != advance_prep_hash {
            has_data = true;
            builder = builder.data(&AdvancePrepChanged {
                description: input.advance_prep,
            })?;
        }
        if !has_data {
            return Ok(());
        }

        builder.commit(&self.0).await?;

        Ok(())
    }
}
