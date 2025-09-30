// Evento projections for recipe views

use chrono::{DateTime, Utc};
use imkitchen_shared::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Ingredient, Instruction, RecipeCategory};

/// Read model for recipe list view - optimized for browsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeListView {
    pub recipe_id: Uuid,
    pub title: String,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub rating: f32,
    pub review_count: u32,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub image_url: Option<String>, // For future image support
}

impl RecipeListView {
    pub fn total_time_minutes(&self) -> u32 {
        self.prep_time_minutes + self.cook_time_minutes
    }

    pub fn has_rating(&self) -> bool {
        self.review_count > 0
    }
}

/// Read model for detailed recipe view - includes full recipe data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDetailView {
    pub recipe_id: Uuid,
    pub title: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub rating: f32,
    pub review_count: u32,
    pub created_by: Uuid,
    pub is_public: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub image_url: Option<String>,
    pub nutritional_info: Option<ProjectionNutritionalInfo>,
}

impl RecipeDetailView {
    pub fn total_time_minutes(&self) -> u32 {
        self.prep_time_minutes + self.cook_time_minutes
    }

    pub fn ingredient_count(&self) -> usize {
        self.ingredients.len()
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn estimated_servings(&self) -> u32 {
        // Simple heuristic based on ingredient quantities
        // This could be enhanced with better logic
        match self.ingredient_count() {
            1..=3 => 1,
            4..=6 => 2,
            7..=10 => 4,
            _ => 6,
        }
    }
}

/// Simplified nutritional information for projections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionNutritionalInfo {
    pub calories: f64,
    pub protein_grams: f64,
    pub carbs_grams: f64,
    pub fat_grams: f64,
}

/// Search index projection for full-text search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSearchIndex {
    pub recipe_id: Uuid,
    pub title: String,
    pub search_text: String, // Combined searchable text
    pub category: RecipeCategory,
    pub difficulty: Difficulty,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub tags: Vec<String>,
    pub ingredients_text: String,  // Searchable ingredient names
    pub instructions_text: String, // Searchable instruction text
    pub is_public: bool,
    pub created_by: Uuid,
    pub rating: f32,
    pub review_count: u32,
    pub created_at: DateTime<Utc>,
}

impl RecipeSearchIndex {
    pub fn build_search_text(
        title: &str,
        ingredients: &[Ingredient],
        instructions: &[Instruction],
        tags: &[String],
    ) -> String {
        let mut search_text = String::new();

        // Add title
        search_text.push_str(title);
        search_text.push(' ');

        // Add ingredient names
        for ingredient in ingredients {
            search_text.push_str(&ingredient.name);
            search_text.push(' ');
        }

        // Add instruction text (first few words)
        for instruction in instructions {
            let words: Vec<&str> = instruction.text.split_whitespace().take(5).collect();
            search_text.push_str(&words.join(" "));
            search_text.push(' ');
        }

        // Add tags
        for tag in tags {
            search_text.push_str(tag);
            search_text.push(' ');
        }

        search_text.to_lowercase()
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.search_text.contains(&query_lower)
            || self.title.to_lowercase().contains(&query_lower)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_lowercase().contains(&query_lower))
    }
}

/// Summary projection for recipe cards and quick views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub recipe_id: Uuid,
    pub title: String,
    pub difficulty: Difficulty,
    pub category: RecipeCategory,
    pub total_time_minutes: u32,
    pub rating: f32,
    pub review_count: u32,
    pub ingredient_count: usize,
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub is_public: bool,
    pub image_url: Option<String>,
}

impl RecipeSummary {
    pub fn is_quick_recipe(&self) -> bool {
        self.total_time_minutes <= 30
    }

    pub fn is_highly_rated(&self) -> bool {
        self.rating >= 4.0 && self.review_count >= 5
    }
}

/// User's recipe collection view - personalized for user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecipeCollection {
    pub user_id: Uuid,
    pub recipes: Vec<RecipeSummary>,
    pub total_count: usize,
    pub public_count: usize,
    pub private_count: usize,
    pub categories: std::collections::HashMap<RecipeCategory, usize>,
    pub average_rating: f32,
}

impl UserRecipeCollection {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            recipes: Vec::new(),
            total_count: 0,
            public_count: 0,
            private_count: 0,
            categories: std::collections::HashMap::new(),
            average_rating: 0.0,
        }
    }

    pub fn add_recipe(&mut self, recipe: RecipeSummary) {
        self.total_count += 1;

        if recipe.is_public {
            self.public_count += 1;
        } else {
            self.private_count += 1;
        }

        *self.categories.entry(recipe.category).or_insert(0) += 1;

        // Recalculate average rating
        let total_rating: f32 = self
            .recipes
            .iter()
            .filter(|r| r.review_count > 0)
            .map(|r| r.rating)
            .sum::<f32>()
            + if recipe.review_count > 0 {
                recipe.rating
            } else {
                0.0
            };

        let rated_count = self.recipes.iter().filter(|r| r.review_count > 0).count()
            + if recipe.review_count > 0 { 1 } else { 0 };

        self.average_rating = if rated_count > 0 {
            total_rating / rated_count as f32
        } else {
            0.0
        };

        self.recipes.push(recipe);
    }
}

// Collection projections

use crate::domain::collection::CollectionPrivacy;

/// Read model for collection list view - optimized for browsing collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionListView {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub recipe_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
}

impl CollectionListView {
    pub fn is_empty(&self) -> bool {
        self.recipe_count == 0
    }

    pub fn is_private(&self) -> bool {
        matches!(self.privacy, CollectionPrivacy::Private)
    }

    pub fn is_public(&self) -> bool {
        matches!(self.privacy, CollectionPrivacy::Public)
    }

    pub fn is_shared(&self) -> bool {
        matches!(self.privacy, CollectionPrivacy::Shared)
    }
}

/// Read model for detailed collection view - includes recipes and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDetailView {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub recipes: Vec<RecipeSummary>,
    pub recipe_count: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
}

impl CollectionDetailView {
    pub fn is_empty(&self) -> bool {
        self.recipes.is_empty()
    }

    pub fn average_difficulty(&self) -> Option<Difficulty> {
        if self.recipes.is_empty() {
            return None;
        }

        let difficulty_sum: u32 = self
            .recipes
            .iter()
            .map(|r| match r.difficulty {
                Difficulty::Easy => 1,
                Difficulty::Medium => 2,
                Difficulty::Hard => 3,
            })
            .sum();

        let average = difficulty_sum as f32 / self.recipes.len() as f32;

        match average {
            x if x <= 1.5 => Some(Difficulty::Easy),
            x if x <= 2.5 => Some(Difficulty::Medium),
            _ => Some(Difficulty::Hard),
        }
    }

    pub fn average_cook_time(&self) -> u32 {
        if self.recipes.is_empty() {
            return 0;
        }

        let total_time: u32 = self.recipes.iter().map(|r| r.total_time_minutes).sum();
        total_time / self.recipes.len() as u32
    }

    pub fn categories(&self) -> std::collections::HashMap<RecipeCategory, usize> {
        let mut categories = std::collections::HashMap::new();
        for recipe in &self.recipes {
            *categories.entry(recipe.category).or_insert(0) += 1;
        }
        categories
    }
}

/// Search index projection for collection full-text search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSearchIndex {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub search_text: String, // Combined searchable text
    pub recipe_count: usize,
    pub recipe_titles: Vec<String>, // For recipe-based searches
    pub created_at: DateTime<Utc>,
    pub is_archived: bool,
}

impl CollectionSearchIndex {
    pub fn build_search_text(
        name: &str,
        description: &Option<String>,
        recipe_titles: &[String],
    ) -> String {
        let mut search_text = String::new();

        // Add collection name
        search_text.push_str(name);
        search_text.push(' ');

        // Add description if available
        if let Some(desc) = description {
            search_text.push_str(desc);
            search_text.push(' ');
        }

        // Add recipe titles
        for title in recipe_titles {
            search_text.push_str(title);
            search_text.push(' ');
        }

        search_text.to_lowercase()
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.search_text.contains(&query_lower)
            || self.name.to_lowercase().contains(&query_lower)
            || self
                .description
                .as_ref()
                .is_some_and(|desc| desc.to_lowercase().contains(&query_lower))
            || self
                .recipe_titles
                .iter()
                .any(|title| title.to_lowercase().contains(&query_lower))
    }
}

/// User favorites view - optimized for quick favorite recipe access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFavoritesView {
    pub user_id: Uuid,
    pub recipes: Vec<RecipeSummary>,
    pub total_count: usize,
    pub last_updated: DateTime<Utc>,
}

impl UserFavoritesView {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            recipes: Vec::new(),
            total_count: 0,
            last_updated: Utc::now(),
        }
    }

    pub fn add_favorite(&mut self, recipe: RecipeSummary) {
        if !self.recipes.iter().any(|r| r.recipe_id == recipe.recipe_id) {
            self.recipes.push(recipe);
            self.total_count += 1;
            self.last_updated = Utc::now();
        }
    }

    pub fn remove_favorite(&mut self, recipe_id: Uuid) {
        if let Some(pos) = self.recipes.iter().position(|r| r.recipe_id == recipe_id) {
            self.recipes.remove(pos);
            self.total_count -= 1;
            self.last_updated = Utc::now();
        }
    }

    pub fn is_favorited(&self, recipe_id: Uuid) -> bool {
        self.recipes.iter().any(|r| r.recipe_id == recipe_id)
    }

    pub fn quick_recipes(&self) -> Vec<&RecipeSummary> {
        self.recipes
            .iter()
            .filter(|r| r.is_quick_recipe())
            .collect()
    }

    pub fn highly_rated_favorites(&self) -> Vec<&RecipeSummary> {
        self.recipes
            .iter()
            .filter(|r| r.is_highly_rated())
            .collect()
    }
}

/// Projection showing which collections contain a specific recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeInCollectionsView {
    pub recipe_id: Uuid,
    pub collections: Vec<CollectionReference>,
    pub total_collection_count: usize,
    pub public_collection_count: usize,
    pub private_collection_count: usize,
}

/// Reference to a collection (lightweight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionReference {
    pub collection_id: Uuid,
    pub name: String,
    pub privacy: CollectionPrivacy,
    pub user_id: Uuid,
    pub added_at: DateTime<Utc>,
}

impl RecipeInCollectionsView {
    pub fn new(recipe_id: Uuid) -> Self {
        Self {
            recipe_id,
            collections: Vec::new(),
            total_collection_count: 0,
            public_collection_count: 0,
            private_collection_count: 0,
        }
    }

    pub fn add_collection(&mut self, collection: CollectionReference) {
        if !self
            .collections
            .iter()
            .any(|c| c.collection_id == collection.collection_id)
        {
            match collection.privacy {
                CollectionPrivacy::Public => self.public_collection_count += 1,
                CollectionPrivacy::Private => self.private_collection_count += 1,
                CollectionPrivacy::Shared => {} // Could add shared count if needed
            }

            self.collections.push(collection);
            self.total_collection_count += 1;
        }
    }

    pub fn remove_collection(&mut self, collection_id: Uuid) {
        if let Some(pos) = self
            .collections
            .iter()
            .position(|c| c.collection_id == collection_id)
        {
            let collection = &self.collections[pos];
            match collection.privacy {
                CollectionPrivacy::Public => self.public_collection_count -= 1,
                CollectionPrivacy::Private => self.private_collection_count -= 1,
                CollectionPrivacy::Shared => {}
            }

            self.collections.remove(pos);
            self.total_collection_count -= 1;
        }
    }

    pub fn collections_for_user(&self, user_id: Uuid) -> Vec<&CollectionReference> {
        self.collections
            .iter()
            .filter(|c| c.user_id == user_id)
            .collect()
    }

    pub fn public_collections(&self) -> Vec<&CollectionReference> {
        self.collections
            .iter()
            .filter(|c| matches!(c.privacy, CollectionPrivacy::Public))
            .collect()
    }
}
