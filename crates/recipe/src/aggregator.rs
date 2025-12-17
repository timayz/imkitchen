use bincode::{Decode, Encode};
use imkitchen_shared::Event;
use sha3::{Digest, Sha3_224};

use crate::{
    AdvancePrepChanged, BasicInformationChanged, Created, CuisineType, CuisineTypeChanged, Deleted,
    DietaryRestrictionsChanged, Imported, IngredientsChanged, InstructionsChanged, MadePrivate,
    MainCourseOptionsChanged, RecipeType, RecipeTypeChanged, SharedToCommunity,
};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct Recipe {
    pub user_id: String,
    pub recipe_type: RecipeType,
    pub cuisine_type: CuisineType,
    pub basic_information_hash: Vec<u8>,
    pub ingredients_hash: Vec<u8>,
    pub instructions_hash: Vec<u8>,
    pub dietary_restrictions_hash: Vec<u8>,
    pub main_option_hash: Vec<u8>,
    pub advance_prep_hash: Vec<u8>,
    pub is_shared: bool,
    pub deleted: bool,
}

#[evento::aggregator]
impl Recipe {
    async fn handle_created(&mut self, event: Event<Created>) -> anyhow::Result<()> {
        self.user_id = event.metadata.trigger_by()?;

        Ok(())
    }

    async fn handle_imported(&mut self, event: Event<Imported>) -> anyhow::Result<()> {
        self.user_id = event.metadata.trigger_by()?;
        self.recipe_type = event.data.recipe_type;
        self.cuisine_type = event.data.cuisine_type;

        let mut hasher = Sha3_224::default();
        hasher.update(event.data.name);
        hasher.update(event.data.description);
        hasher.update(event.data.household_size.to_string());
        hasher.update(event.data.prep_time.to_string());
        hasher.update(event.data.cook_time.to_string());

        self.basic_information_hash = hasher.finalize()[..].to_vec();

        let mut hasher = Sha3_224::default();

        for instruction in event.data.instructions {
            hasher.update(instruction.description);
            hasher.update(instruction.time_next.to_string());
        }

        self.instructions_hash = hasher.finalize()[..].to_vec();

        let mut hasher = Sha3_224::default();

        for ingredient in event.data.ingredients {
            hasher.update(ingredient.name);
            hasher.update(ingredient.quantity.to_string());

            if let Some(unit) = ingredient.unit {
                hasher.update(unit.to_string());
            }

            if let Some(category) = ingredient.category {
                hasher.update(category.to_string());
            }
        }

        self.ingredients_hash = hasher.finalize()[..].to_vec();

        let mut hasher = Sha3_224::default();
        hasher.update(event.data.advance_prep);

        self.advance_prep_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_recipe_type_changed(
        &mut self,
        event: Event<RecipeTypeChanged>,
    ) -> anyhow::Result<()> {
        self.recipe_type = event.data.recipe_type;

        Ok(())
    }

    async fn handle_basic_iformation_changed(
        &mut self,
        event: Event<BasicInformationChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();
        hasher.update(event.data.name);
        hasher.update(event.data.description);
        hasher.update(event.data.household_size.to_string());
        hasher.update(event.data.prep_time.to_string());
        hasher.update(event.data.cook_time.to_string());

        self.basic_information_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_instructions_hanged(
        &mut self,
        event: Event<InstructionsChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();

        for instruction in event.data.instructions {
            hasher.update(instruction.description);
            hasher.update(instruction.time_next.to_string());
        }
        self.instructions_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_ingredients_changed(
        &mut self,
        event: Event<IngredientsChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();

        for ingredient in event.data.ingredients {
            hasher.update(ingredient.name);
            hasher.update(ingredient.quantity.to_string());

            if let Some(unit) = ingredient.unit {
                hasher.update(unit.to_string());
            }

            if let Some(category) = ingredient.category {
                hasher.update(category.to_string());
            }
        }

        self.ingredients_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_dietary_restrictions_changed(
        &mut self,
        event: Event<DietaryRestrictionsChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();

        for restriction in event.data.dietary_restrictions {
            hasher.update(restriction.to_string());
        }
        self.dietary_restrictions_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_cuisine_type_changed(
        &mut self,
        event: Event<CuisineTypeChanged>,
    ) -> anyhow::Result<()> {
        self.cuisine_type = event.data.cuisine_type;
        Ok(())
    }

    async fn handle_main_course_options_changed(
        &mut self,
        event: Event<MainCourseOptionsChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();
        hasher.update(event.data.accepts_accompaniment.to_string());

        self.main_option_hash = hasher.finalize()[..].to_vec();

        Ok(())
    }

    async fn handle_advance_prep_changed(
        &mut self,
        event: Event<AdvancePrepChanged>,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha3_224::default();
        hasher.update(event.data.advance_prep);

        self.advance_prep_hash = hasher.finalize()[..].to_vec();
        Ok(())
    }

    async fn handle_shared_to_community(
        &mut self,
        event: Event<SharedToCommunity>,
    ) -> anyhow::Result<()> {
        self.is_shared = event.data.shared;

        Ok(())
    }

    async fn handle_made_private(&mut self, event: Event<MadePrivate>) -> anyhow::Result<()> {
        self.is_shared = event.data.shared;

        Ok(())
    }

    async fn handle_deleted(&mut self, event: Event<Deleted>) -> anyhow::Result<()> {
        self.deleted = event.data.deleted;

        Ok(())
    }
}
