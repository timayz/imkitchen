// CQRS commands for recipe operations

use crate::domain::{Ingredient, Instruction, RecipeCategory};
use imkitchen_shared::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Parameters for creating a new recipe
#[derive(Debug, Clone)]
pub struct CreateRecipeParams {
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub ingredients: Vec<Ingredient>,
    #[validate(length(min = 1))]
    pub instructions: Vec<Instruction>,
    #[validate(range(min = 1))]
    pub prep_time_minutes: u32,
    #[validate(range(min = 1))]
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
}

impl CreateRecipeCommand {
    pub fn new(params: CreateRecipeParams) -> Result<Self, validator::ValidationErrors> {
        let command = Self {
            title: params.title,
            ingredients: params.ingredients,
            instructions: params.instructions,
            prep_time_minutes: params.prep_time_minutes,
            cook_time_minutes: params.cook_time_minutes,
            difficulty: params.difficulty,
            category: params.category,
            created_by: params.created_by,
            is_public: params.is_public,
            tags: params.tags,
        };

        command.validate()?;
        Ok(command)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateRecipeCommand {
    pub recipe_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub ingredients: Option<Vec<Ingredient>>,
    pub instructions: Option<Vec<Instruction>>,
    #[validate(range(min = 1))]
    pub prep_time_minutes: Option<u32>,
    #[validate(range(min = 1))]
    pub cook_time_minutes: Option<u32>,
    pub difficulty: Option<Difficulty>,
    pub category: Option<RecipeCategory>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub updated_by: Uuid,
}

impl UpdateRecipeCommand {
    pub fn new(recipe_id: Uuid, updated_by: Uuid) -> Self {
        Self {
            recipe_id,
            title: None,
            ingredients: None,
            instructions: None,
            prep_time_minutes: None,
            cook_time_minutes: None,
            difficulty: None,
            category: None,
            is_public: None,
            tags: None,
            updated_by,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_ingredients(mut self, ingredients: Vec<Ingredient>) -> Self {
        self.ingredients = Some(ingredients);
        self
    }

    pub fn with_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.instructions = Some(instructions);
        self
    }

    pub fn with_prep_time(mut self, prep_time_minutes: u32) -> Self {
        self.prep_time_minutes = Some(prep_time_minutes);
        self
    }

    pub fn with_cook_time(mut self, cook_time_minutes: u32) -> Self {
        self.cook_time_minutes = Some(cook_time_minutes);
        self
    }

    pub fn with_difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = Some(difficulty);
        self
    }

    pub fn with_category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn with_visibility(mut self, is_public: bool) -> Self {
        self.is_public = Some(is_public);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn validate_and_build(self) -> Result<Self, validator::ValidationErrors> {
        self.validate()?;
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRecipeCommand {
    pub recipe_id: Uuid,
    pub deleted_by: Uuid,
}

impl DeleteRecipeCommand {
    pub fn new(recipe_id: Uuid, deleted_by: Uuid) -> Self {
        Self {
            recipe_id,
            deleted_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddIngredientCommand {
    pub recipe_id: Uuid,
    pub ingredient: Ingredient,
    pub added_by: Uuid,
}

impl AddIngredientCommand {
    pub fn new(
        recipe_id: Uuid,
        ingredient: Ingredient,
        added_by: Uuid,
    ) -> Result<Self, validator::ValidationErrors> {
        let command = Self {
            recipe_id,
            ingredient,
            added_by,
        };

        command.validate()?;
        Ok(command)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ModifyInstructionCommand {
    pub recipe_id: Uuid,
    pub step_number: u32,
    pub instruction: Instruction,
    pub modified_by: Uuid,
}

impl ModifyInstructionCommand {
    pub fn new(
        recipe_id: Uuid,
        step_number: u32,
        instruction: Instruction,
        modified_by: Uuid,
    ) -> Result<Self, validator::ValidationErrors> {
        let command = Self {
            recipe_id,
            step_number,
            instruction,
            modified_by,
        };

        command.validate()?;
        Ok(command)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRecipeCommand {
    pub recipe_id: Uuid,
    pub archived_by: Uuid,
}

impl ArchiveRecipeCommand {
    pub fn new(recipe_id: Uuid, archived_by: Uuid) -> Self {
        Self {
            recipe_id,
            archived_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRecipeCommand {
    pub recipe_id: Uuid,
    pub restored_by: Uuid,
}

impl RestoreRecipeCommand {
    pub fn new(recipe_id: Uuid, restored_by: Uuid) -> Self {
        Self {
            recipe_id,
            restored_by,
        }
    }
}

// Collection commands

use crate::domain::collection::CollectionPrivacy;

/// Parameters for creating a new collection
#[derive(Debug, Clone)]
pub struct CreateCollectionParams {
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCollectionCommand {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub created_by: Uuid,
}

impl CreateCollectionCommand {
    pub fn new(params: CreateCollectionParams) -> Result<Self, validator::ValidationErrors> {
        let command = Self {
            name: params.name,
            description: params.description,
            privacy: params.privacy,
            created_by: params.created_by,
        };

        command.validate()?;
        Ok(command)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCollectionCommand {
    pub collection_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<Option<String>>,
    pub privacy: Option<CollectionPrivacy>,
    pub updated_by: Uuid,
}

impl UpdateCollectionCommand {
    pub fn new(collection_id: Uuid, updated_by: Uuid) -> Self {
        Self {
            collection_id,
            name: None,
            description: None,
            privacy: None,
            updated_by,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_privacy(mut self, privacy: CollectionPrivacy) -> Self {
        self.privacy = Some(privacy);
        self
    }

    pub fn validate_and_build(self) -> Result<Self, validator::ValidationErrors> {
        self.validate()?;
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCollectionCommand {
    pub collection_id: Uuid,
    pub deleted_by: Uuid,
}

impl DeleteCollectionCommand {
    pub fn new(collection_id: Uuid, deleted_by: Uuid) -> Self {
        Self {
            collection_id,
            deleted_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRecipeToCollectionCommand {
    pub collection_id: Uuid,
    pub recipe_id: Uuid,
    pub added_by: Uuid,
}

impl AddRecipeToCollectionCommand {
    pub fn new(collection_id: Uuid, recipe_id: Uuid, added_by: Uuid) -> Self {
        Self {
            collection_id,
            recipe_id,
            added_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveRecipeFromCollectionCommand {
    pub collection_id: Uuid,
    pub recipe_id: Uuid,
    pub removed_by: Uuid,
}

impl RemoveRecipeFromCollectionCommand {
    pub fn new(collection_id: Uuid, recipe_id: Uuid, removed_by: Uuid) -> Self {
        Self {
            collection_id,
            recipe_id,
            removed_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAddRecipesToCollectionCommand {
    pub collection_id: Uuid,
    pub recipe_ids: Vec<Uuid>,
    pub added_by: Uuid,
}

impl BulkAddRecipesToCollectionCommand {
    pub fn new(collection_id: Uuid, recipe_ids: Vec<Uuid>, added_by: Uuid) -> Self {
        Self {
            collection_id,
            recipe_ids,
            added_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRemoveRecipesFromCollectionCommand {
    pub collection_id: Uuid,
    pub recipe_ids: Vec<Uuid>,
    pub removed_by: Uuid,
}

impl BulkRemoveRecipesFromCollectionCommand {
    pub fn new(collection_id: Uuid, recipe_ids: Vec<Uuid>, removed_by: Uuid) -> Self {
        Self {
            collection_id,
            recipe_ids,
            removed_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveCollectionCommand {
    pub collection_id: Uuid,
    pub archived_by: Uuid,
}

impl ArchiveCollectionCommand {
    pub fn new(collection_id: Uuid, archived_by: Uuid) -> Self {
        Self {
            collection_id,
            archived_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreCollectionCommand {
    pub collection_id: Uuid,
    pub restored_by: Uuid,
}

impl RestoreCollectionCommand {
    pub fn new(collection_id: Uuid, restored_by: Uuid) -> Self {
        Self {
            collection_id,
            restored_by,
        }
    }
}
