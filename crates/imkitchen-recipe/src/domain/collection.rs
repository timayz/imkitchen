use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Privacy settings for recipe collections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionPrivacy {
    Private,
    Shared,
    Public,
}

impl std::fmt::Display for CollectionPrivacy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectionPrivacy::Private => write!(f, "Private"),
            CollectionPrivacy::Shared => write!(f, "Shared"),
            CollectionPrivacy::Public => write!(f, "Public"),
        }
    }
}

/// Represents the membership of a recipe in a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCollectionMembership {
    pub recipe_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub sort_order: u32,
}

/// Recipe collection aggregate for organizing recipes
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RecipeCollection {
    pub collection_id: Uuid,
    pub user_id: Uuid,

    #[validate(length(
        min = 1,
        max = 100,
        message = "Collection name must be between 1 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(
        max = 500,
        message = "Collection description must be 500 characters or less"
    ))]
    pub description: Option<String>,

    pub privacy: CollectionPrivacy,
    pub recipes: Vec<RecipeCollectionMembership>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RecipeCollection {
    /// Create a new recipe collection
    pub fn new(
        collection_id: Uuid,
        user_id: Uuid,
        name: String,
        description: Option<String>,
        privacy: CollectionPrivacy,
    ) -> Self {
        let now = Utc::now();
        Self {
            collection_id,
            user_id,
            name,
            description,
            privacy,
            recipes: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a recipe to the collection
    pub fn add_recipe(&mut self, recipe_id: Uuid) {
        // Check if recipe already exists in collection
        if !self.recipes.iter().any(|r| r.recipe_id == recipe_id) {
            let sort_order = self.recipes.len() as u32 + 1;
            let membership = RecipeCollectionMembership {
                recipe_id,
                added_at: Utc::now(),
                sort_order,
            };
            self.recipes.push(membership);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a recipe from the collection
    pub fn remove_recipe(&mut self, recipe_id: Uuid) {
        if let Some(pos) = self.recipes.iter().position(|r| r.recipe_id == recipe_id) {
            self.recipes.remove(pos);
            // Reorder remaining recipes
            for (index, recipe) in self.recipes.iter_mut().enumerate() {
                recipe.sort_order = index as u32 + 1;
            }
            self.updated_at = Utc::now();
        }
    }

    /// Get the count of recipes in this collection
    pub fn recipe_count(&self) -> usize {
        self.recipes.len()
    }

    /// Check if a recipe is in this collection
    pub fn contains_recipe(&self, recipe_id: Uuid) -> bool {
        self.recipes.iter().any(|r| r.recipe_id == recipe_id)
    }

    /// Update collection details
    pub fn update_details(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        privacy: Option<CollectionPrivacy>,
    ) {
        if let Some(new_name) = name {
            self.name = new_name;
        }
        if let Some(new_description) = description {
            self.description = new_description;
        }
        if let Some(new_privacy) = privacy {
            self.privacy = new_privacy;
        }
        self.updated_at = Utc::now();
    }
}

/// Special favorites collection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFavorites {
    pub user_id: Uuid,
    pub recipe_ids: Vec<Uuid>,
    pub updated_at: DateTime<Utc>,
}

impl UserFavorites {
    /// Create new user favorites
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            recipe_ids: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Add recipe to favorites
    pub fn add_favorite(&mut self, recipe_id: Uuid) {
        if !self.recipe_ids.contains(&recipe_id) {
            self.recipe_ids.push(recipe_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove recipe from favorites
    pub fn remove_favorite(&mut self, recipe_id: Uuid) {
        if let Some(pos) = self.recipe_ids.iter().position(|&id| id == recipe_id) {
            self.recipe_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Check if recipe is favorited
    pub fn is_favorited(&self, recipe_id: Uuid) -> bool {
        self.recipe_ids.contains(&recipe_id)
    }

    /// Get count of favorites
    pub fn count(&self) -> usize {
        self.recipe_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_creation() {
        let collection_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let name = "Test Collection".to_string();
        let description = Some("Test description".to_string());
        let privacy = CollectionPrivacy::Private;

        let collection = RecipeCollection::new(
            collection_id,
            user_id,
            name.clone(),
            description.clone(),
            privacy.clone(),
        );

        assert_eq!(collection.collection_id, collection_id);
        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.name, name);
        assert_eq!(collection.description, description);
        assert_eq!(collection.privacy, privacy);
        assert!(collection.recipes.is_empty());
    }

    #[test]
    fn test_add_remove_recipes() {
        let collection_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let mut collection = RecipeCollection::new(
            collection_id,
            user_id,
            "Test Collection".to_string(),
            None,
            CollectionPrivacy::Private,
        );

        let recipe_id = Uuid::new_v4();

        // Add recipe
        collection.add_recipe(recipe_id);
        assert_eq!(collection.recipe_count(), 1);
        assert!(collection.contains_recipe(recipe_id));

        // Remove recipe
        collection.remove_recipe(recipe_id);
        assert_eq!(collection.recipe_count(), 0);
        assert!(!collection.contains_recipe(recipe_id));
    }

    #[test]
    fn test_user_favorites() {
        let user_id = Uuid::new_v4();
        let mut favorites = UserFavorites::new(user_id);

        let recipe_id = Uuid::new_v4();

        assert!(!favorites.is_favorited(recipe_id));

        favorites.add_favorite(recipe_id);
        assert!(favorites.is_favorited(recipe_id));
        assert_eq!(favorites.count(), 1);

        favorites.remove_favorite(recipe_id);
        assert!(!favorites.is_favorited(recipe_id));
        assert_eq!(favorites.count(), 0);
    }
}
