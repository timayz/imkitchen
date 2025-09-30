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
