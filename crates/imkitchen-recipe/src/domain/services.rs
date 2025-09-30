use super::value_objects::{Difficulty, Ingredient, NutritionalInfo};
use regex::Regex;

/// Service for calculating recipe difficulty based on complexity factors
pub struct RecipeDifficultyCalculator;

impl RecipeDifficultyCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_difficulty(
        &self,
        ingredient_count: usize,
        instruction_count: usize,
        prep_time_minutes: u32,
        cook_time_minutes: u32,
    ) -> Difficulty {
        let total_time = prep_time_minutes + cook_time_minutes;

        // Simple scoring algorithm based on complexity factors
        let mut complexity_score = 0;

        // Ingredient complexity
        if ingredient_count >= 10 {
            complexity_score += 3;
        } else if ingredient_count >= 6 {
            complexity_score += 2;
        } else if ingredient_count >= 4 {
            complexity_score += 1;
        }

        // Instruction complexity
        if instruction_count >= 10 {
            complexity_score += 3;
        } else if instruction_count >= 6 {
            complexity_score += 2;
        } else if instruction_count >= 4 {
            complexity_score += 1;
        }

        // Time complexity
        if total_time >= 120 {
            complexity_score += 3;
        } else if total_time >= 60 {
            complexity_score += 2;
        } else if total_time >= 30 {
            complexity_score += 1;
        }

        // Determine difficulty based on total score
        match complexity_score {
            0..=3 => Difficulty::Easy,
            4..=6 => Difficulty::Medium,
            _ => Difficulty::Hard,
        }
    }
}

impl Default for RecipeDifficultyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for parsing ingredient text into structured ingredient objects
pub struct IngredientParser {
    // Regex patterns for parsing common ingredient formats
    ingredient_pattern: Regex,
    fraction_pattern: Regex,
}

impl IngredientParser {
    pub fn new() -> Self {
        Self {
            // Pattern: "2 cups flour" or "1/2 teaspoon salt" or "3.5 lbs chicken breast"
            ingredient_pattern: Regex::new(r"^(\d+(?:\.\d+)?|\d+/\d+)\s+(\w+)\s+(.+)$").unwrap(),
            fraction_pattern: Regex::new(r"^(\d+)/(\d+)$").unwrap(),
        }
    }

    pub fn parse_ingredient_text(&self, text: &str) -> Result<Ingredient, String> {
        if let Some(captures) = self.ingredient_pattern.captures(text.trim()) {
            let quantity_str = captures.get(1).unwrap().as_str();
            let unit = captures.get(2).unwrap().as_str().to_string();
            let name = captures.get(3).unwrap().as_str().to_string();

            let quantity = if let Some(fraction_captures) =
                self.fraction_pattern.captures(quantity_str)
            {
                // Parse fraction
                let numerator: f64 = fraction_captures.get(1).unwrap().as_str().parse().unwrap();
                let denominator: f64 = fraction_captures.get(2).unwrap().as_str().parse().unwrap();
                numerator / denominator
            } else {
                // Parse decimal
                quantity_str
                    .parse()
                    .map_err(|_| "Invalid quantity format")?
            };

            Ingredient::new(name, quantity, unit, None)
                .map_err(|_| "Invalid ingredient data".to_string())
        } else {
            Err("Invalid ingredient format".to_string())
        }
    }
}

impl Default for IngredientParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for calculating basic nutritional information from ingredients
pub struct NutritionalCalculator {
    // Basic nutritional database (simplified for demo)
    nutrition_db: std::collections::HashMap<String, NutritionalInfo>,
}

impl NutritionalCalculator {
    pub fn new() -> Self {
        let mut nutrition_db = std::collections::HashMap::new();

        // Add some basic nutritional data per 100g/100ml
        nutrition_db.insert(
            "flour".to_string(),
            NutritionalInfo {
                calories: 364.0,
                protein: 10.3,
                carbohydrates: 76.3,
                fat: 1.0,
                fiber: 2.7,
                sugar: 0.3,
                sodium: 2.0,
            },
        );

        nutrition_db.insert(
            "sugar".to_string(),
            NutritionalInfo {
                calories: 387.0,
                protein: 0.0,
                carbohydrates: 100.0,
                fat: 0.0,
                fiber: 0.0,
                sugar: 100.0,
                sodium: 0.0,
            },
        );

        nutrition_db.insert(
            "butter".to_string(),
            NutritionalInfo {
                calories: 717.0,
                protein: 0.9,
                carbohydrates: 0.1,
                fat: 81.1,
                fiber: 0.0,
                sugar: 0.1,
                sodium: 643.0,
            },
        );

        Self { nutrition_db }
    }

    pub fn estimate_nutrition(&self, ingredients: &[Ingredient]) -> NutritionalInfo {
        let mut total_nutrition = NutritionalInfo::default();

        for ingredient in ingredients {
            if let Some(base_nutrition) = self.nutrition_db.get(&ingredient.name.to_lowercase()) {
                // Simple calculation assuming standard conversion rates
                // This is a basic approximation - real implementation would have
                // proper unit conversions and more comprehensive database
                let quantity_factor =
                    self.get_quantity_factor(&ingredient.unit, ingredient.quantity);

                total_nutrition.calories += base_nutrition.calories * quantity_factor;
                total_nutrition.protein += base_nutrition.protein * quantity_factor;
                total_nutrition.carbohydrates += base_nutrition.carbohydrates * quantity_factor;
                total_nutrition.fat += base_nutrition.fat * quantity_factor;
                total_nutrition.fiber += base_nutrition.fiber * quantity_factor;
                total_nutrition.sugar += base_nutrition.sugar * quantity_factor;
                total_nutrition.sodium += base_nutrition.sodium * quantity_factor;
            }
        }

        total_nutrition
    }

    fn get_quantity_factor(&self, unit: &str, quantity: f64) -> f64 {
        // Basic unit conversions to 100g equivalent
        // This is simplified - real implementation would have comprehensive unit conversion
        match unit.to_lowercase().as_str() {
            "cup" | "cups" => quantity * 1.25, // Approximate 125g per cup for flour
            "teaspoon" | "teaspoons" => quantity * 0.04, // Approximate 4g per teaspoon
            "tablespoon" | "tablespoons" => quantity * 0.12, // Approximate 12g per tablespoon
            "lb" | "lbs" | "pound" | "pounds" => quantity * 4.54, // 454g per pound
            "oz" | "ounce" | "ounces" => quantity * 0.28, // 28g per ounce
            "g" | "gram" | "grams" => quantity * 0.01, // Direct conversion to 100g
            "kg" | "kilogram" | "kilograms" => quantity * 10.0, // 1000g per kg
            _ => quantity * 1.0,               // Default assumption
        }
    }
}

impl Default for NutritionalCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// Collection-related services

use super::collection::{CollectionPrivacy, RecipeCollection};
use uuid::Uuid;

/// Service for validating collection operations and constraints
pub struct CollectionValidationService;

impl CollectionValidationService {
    pub fn new() -> Self {
        Self
    }

    /// Validate that a user hasn't exceeded the maximum collection limit (50 collections)
    pub fn validate_user_collection_limit(&self, _user_id: Uuid, current_count: usize) -> bool {
        current_count < 50
    }

    /// Validate that a collection hasn't exceeded the maximum recipe limit (1000 recipes)
    pub fn validate_collection_recipe_limit(&self, collection: &RecipeCollection) -> bool {
        collection.recipes.len() < 1000
    }

    /// Validate collection name uniqueness for a user
    pub fn validate_collection_name_uniqueness(
        &self,
        _user_id: Uuid,
        _name: &str,
        _existing_collections: &[RecipeCollection],
    ) -> bool {
        // In a real implementation, this would check against existing collections
        // For now, we'll assume uniqueness validation happens at the database level
        true
    }
}

impl Default for CollectionValidationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple view model for collection list display (used by domain services)
#[derive(Debug, Clone)]
pub struct CollectionListItem {
    pub collection_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub recipe_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Simple view model for collection detail display (used by domain services)
#[derive(Debug, Clone)]
pub struct CollectionDetailItem {
    pub collection_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub privacy: CollectionPrivacy,
    pub recipe_ids: Vec<Uuid>,
    pub recipe_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Service for mapping collections to different view models
pub struct RecipeCollectionMapper;

impl RecipeCollectionMapper {
    pub fn new() -> Self {
        Self
    }

    /// Map collection to list item
    pub fn to_list_item(&self, collection: &RecipeCollection) -> CollectionListItem {
        CollectionListItem {
            collection_id: collection.collection_id,
            name: collection.name.clone(),
            description: collection.description.clone(),
            privacy: collection.privacy,
            recipe_count: collection.recipes.len(),
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        }
    }

    /// Map collection to detail item
    pub fn to_detail_item(&self, collection: &RecipeCollection) -> CollectionDetailItem {
        CollectionDetailItem {
            collection_id: collection.collection_id,
            user_id: collection.user_id,
            name: collection.name.clone(),
            description: collection.description.clone(),
            privacy: collection.privacy,
            recipe_ids: collection.recipes.iter().map(|r| r.recipe_id).collect(),
            recipe_count: collection.recipes.len(),
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        }
    }
}

impl Default for RecipeCollectionMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for searching and filtering collections
pub struct CollectionSearchService;

impl CollectionSearchService {
    pub fn new() -> Self {
        Self
    }

    /// Check if a collection matches a search query
    pub fn matches_search_query(&self, collection: &RecipeCollection, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Search in collection name
        if collection.name.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Search in collection description
        if let Some(description) = &collection.description {
            if description.to_lowercase().contains(&query_lower) {
                return true;
            }
        }

        false
    }

    /// Filter collections by privacy setting
    pub fn filter_by_privacy(
        &self,
        collections: Vec<RecipeCollection>,
        privacy: CollectionPrivacy,
    ) -> Vec<RecipeCollection> {
        collections
            .into_iter()
            .filter(|c| c.privacy == privacy)
            .collect()
    }

    /// Filter collections by user
    pub fn filter_by_user(
        &self,
        collections: Vec<RecipeCollection>,
        user_id: Uuid,
    ) -> Vec<RecipeCollection> {
        collections
            .into_iter()
            .filter(|c| c.user_id == user_id)
            .collect()
    }
}

impl Default for CollectionSearchService {
    fn default() -> Self {
        Self::new()
    }
}

// Rating and Review Services

use super::rating::{
    RatingStatistics, RecipeRating, RecipeReview, ReviewModerationStatus, StarRating,
};

/// Service for aggregating rating statistics and calculating weighted averages
pub struct RatingAggregationService;

impl RatingAggregationService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate weighted average rating with statistical significance weighting
    pub fn calculate_weighted_average(&self, ratings: &[RecipeRating]) -> f32 {
        if ratings.is_empty() {
            return 0.0;
        }

        let total_ratings = ratings.len() as f32;
        let sum: u32 = ratings.iter().map(|r| r.star_rating.value as u32).sum();
        let raw_average = sum as f32 / total_ratings;

        // Apply statistical significance weighting
        // Fewer ratings get pulled toward the global average (3.0)
        let global_average = 3.0;
        let weight = total_ratings / (total_ratings + 10.0); // Bayesian average weight

        raw_average * weight + global_average * (1.0 - weight)
    }

    /// Generate rating distribution for visualization
    pub fn calculate_rating_distribution(&self, ratings: &[RecipeRating]) -> [u32; 5] {
        let mut distribution = [0u32; 5];

        for rating in ratings {
            let index = (rating.star_rating.value - 1) as usize;
            distribution[index] += 1;
        }

        distribution
    }

    /// Update recipe statistics when new rating is added
    pub fn update_statistics_for_new_rating(
        &self,
        mut stats: RatingStatistics,
        rating: StarRating,
    ) -> RatingStatistics {
        stats.add_rating(rating);
        stats
    }

    /// Calculate confidence score for rating reliability
    pub fn calculate_confidence_score(&self, total_ratings: u32) -> f32 {
        // Simple confidence calculation based on sample size
        match total_ratings {
            0 => 0.0,
            1..=5 => 0.3,
            6..=15 => 0.6,
            16..=50 => 0.8,
            _ => 0.95,
        }
    }
}

impl Default for RatingAggregationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for automated review moderation and spam detection
pub struct ReviewModerationService {
    spam_keywords: Vec<String>,
    inappropriate_keywords: Vec<String>,
}

impl ReviewModerationService {
    pub fn new() -> Self {
        Self {
            spam_keywords: vec![
                "spam".to_string(),
                "click here".to_string(),
                "buy now".to_string(),
                "free money".to_string(),
                "visit my site".to_string(),
            ],
            inappropriate_keywords: vec![
                "offensive_word1".to_string(),
                "offensive_word2".to_string(),
                // Add actual inappropriate content keywords
            ],
        }
    }

    /// Automatically moderate review content
    pub fn moderate_review(&self, review: &RecipeReview) -> ReviewModerationStatus {
        let review_text_lower = review.review_text.to_lowercase();

        // Check for spam content
        if self.contains_spam_content(&review_text_lower) {
            return ReviewModerationStatus::Flagged;
        }

        // Check for inappropriate content
        if self.contains_inappropriate_content(&review_text_lower) {
            return ReviewModerationStatus::Rejected;
        }

        // Check review length and quality heuristics
        if self.is_low_quality_review(&review.review_text) {
            return ReviewModerationStatus::Pending;
        }

        ReviewModerationStatus::Approved
    }

    fn contains_spam_content(&self, text: &str) -> bool {
        self.spam_keywords
            .iter()
            .any(|keyword| text.contains(keyword))
    }

    fn contains_inappropriate_content(&self, text: &str) -> bool {
        self.inappropriate_keywords
            .iter()
            .any(|keyword| text.contains(keyword))
    }

    fn is_low_quality_review(&self, text: &str) -> bool {
        // Check for very short reviews that might not be helpful
        if text.len() < 20 {
            return true;
        }

        // Check for repetitive characters (like "aaaaaaa")
        let chars: Vec<char> = text.chars().collect();
        let mut consecutive_count = 1;
        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                consecutive_count += 1;
                if consecutive_count > 5 {
                    return true;
                }
            } else {
                consecutive_count = 1;
            }
        }

        false
    }

    /// Check if review requires manual moderation
    pub fn requires_manual_review(&self, review: &RecipeReview) -> bool {
        // Flag for manual review if contains certain patterns
        let text = &review.review_text.to_lowercase();

        // Check for borderline content that needs human judgment
        let borderline_keywords = ["maybe", "not sure", "controversial", "might be"];
        borderline_keywords
            .iter()
            .any(|keyword| text.contains(keyword))
    }
}

impl Default for ReviewModerationService {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for statistical weighting of ratings and reviews
pub struct StatisticalWeightingService;

impl StatisticalWeightingService {
    pub fn new() -> Self {
        Self
    }

    /// Calculate helpfulness score weighting based on user reputation
    pub fn calculate_helpfulness_weight(&self, _user_id: Uuid, base_score: i32) -> f32 {
        // In a real implementation, this would factor in user reputation,
        // review history, and other trust signals
        // For now, using a simple weighting
        let weight_factor = 1.0; // Could be adjusted based on user reputation
        base_score as f32 * weight_factor
    }

    /// Calculate time-decay factor for review relevance
    pub fn calculate_time_decay_factor(&self, review_age_days: u32) -> f32 {
        // More recent reviews have higher weight
        let decay_rate = 0.01; // 1% decay per day
        let max_age = 365.0; // Reviews older than 1 year get minimum weight

        if review_age_days as f32 > max_age {
            0.1 // Minimum weight of 10%
        } else {
            (1.0 - (review_age_days as f32 * decay_rate)).max(0.1)
        }
    }

    /// Calculate overall review quality score
    pub fn calculate_review_quality_score(&self, review: &RecipeReview) -> f32 {
        let mut score = 0.0;

        // Length factor (longer reviews generally more helpful)
        let length_score = (review.review_text.len() as f32 / 500.0).min(1.0);
        score += length_score * 0.3;

        // Helpfulness factor
        let helpfulness_score = if !review.helpfulness_votes.is_empty() {
            (review.helpfulness_score as f32 / review.helpfulness_votes.len() as f32)
                .clamp(0.0, 1.0)
        } else {
            0.5 // Neutral score for reviews without votes
        };
        score += helpfulness_score * 0.4;

        // Photo factor (reviews with photos can be more helpful)
        let photo_score = if !review.photos.is_empty() { 1.0 } else { 0.5 };
        score += photo_score * 0.3;

        score.min(1.0)
    }
}

impl Default for StatisticalWeightingService {
    fn default() -> Self {
        Self::new()
    }
}
