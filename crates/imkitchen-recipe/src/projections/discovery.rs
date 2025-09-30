use crate::domain::discovery::{DiscoveryFilters, SortingCriteria};
use chrono::{DateTime, Utc};
use imkitchen_shared::types::Difficulty;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Recipe browse view projection for efficient discovery page rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeBrowseView {
    pub recipes: Vec<RecipeCard>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
    pub applied_filters: DiscoveryFilters,
    pub sort_order: SortingCriteria,
}

/// Optimized recipe card for browse and search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCard {
    pub recipe_id: Uuid,
    pub title: String,
    pub prep_time_minutes: u32,
    pub cook_time_minutes: u32,
    pub difficulty: Difficulty,
    pub rating_average: f32,
    pub rating_count: u32,
    pub image_url: Option<String>,
    pub created_by: Uuid,
    pub tags: Vec<String>,
}

impl RecipeCard {
    /// Calculate total time for the recipe
    pub fn total_time_minutes(&self) -> u32 {
        self.prep_time_minutes + self.cook_time_minutes
    }

    /// Check if recipe is highly rated (4+ stars)
    pub fn is_highly_rated(&self) -> bool {
        self.rating_average >= 4.0
    }

    /// Get difficulty badge color for UI
    pub fn difficulty_color(&self) -> &'static str {
        match self.difficulty {
            Difficulty::Easy => "green",
            Difficulty::Medium => "yellow",
            Difficulty::Hard => "red",
        }
    }
}

/// Search results view projection with autocomplete and suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsView {
    pub results: Vec<SearchResult>,
    pub query_text: String,
    pub total_results: u32,
    pub page: u32,
    pub page_size: u32,
    pub suggestions: Vec<SearchSuggestion>,
    pub search_time_ms: u64,
}

/// Individual search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub recipe_id: Uuid,
    pub title: String,
    pub snippet: String,
    pub relevance_score: f64,
    pub match_type: String, // "title", "ingredient", "instruction"
}

/// Search suggestions view projection for autocomplete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestionsView {
    pub query_prefix: String,
    pub suggestions: Vec<SearchSuggestion>,
    pub generated_at: DateTime<Utc>,
}

impl SearchSuggestionsView {
    /// Sort suggestions by frequency (most popular first)
    pub fn sort_by_frequency(&mut self) {
        self.suggestions
            .sort_by(|a, b| b.frequency.cmp(&a.frequency));
    }

    /// Filter suggestions by type
    pub fn filter_by_type(&self, suggestion_type: &str) -> Vec<&SearchSuggestion> {
        self.suggestions
            .iter()
            .filter(|s| s.suggestion_type == suggestion_type)
            .collect()
    }
}

/// Individual search suggestion with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub suggestion_text: String,
    pub suggestion_type: String, // "ingredient", "category", "recipe"
    pub frequency: u32,
}

/// Trending recipes view projection with real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingRecipesView {
    pub time_window: String,
    pub trending_recipes: Vec<TrendingRecipe>,
    pub calculated_at: DateTime<Utc>,
}

impl TrendingRecipesView {
    /// Get the top trending recipe
    pub fn get_top_trending(&self) -> Option<&TrendingRecipe> {
        self.trending_recipes.first()
    }

    /// Get trending recipes above a certain score threshold
    pub fn get_above_score(&self, min_score: f64) -> Vec<&TrendingRecipe> {
        self.trending_recipes
            .iter()
            .filter(|r| r.popularity_score >= min_score)
            .collect()
    }
}

/// Individual trending recipe with popularity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingRecipe {
    pub recipe_id: Uuid,
    pub title: String,
    pub popularity_score: f64,
    pub trending_rank: u32,
    pub view_count_24h: u32,
    pub rating_average: f32,
    pub time_weighted_score: f64,
}

/// Similar recipes view projection for recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarRecipesView {
    pub original_recipe_id: Uuid,
    pub similar_recipes: Vec<SimilarRecipe>,
    pub similarity_threshold: f64,
    pub generated_at: DateTime<Utc>,
}

/// Individual similar recipe with similarity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarRecipe {
    pub recipe_id: Uuid,
    pub title: String,
    pub similarity_score: f64,
    pub similarity_reasons: Vec<String>,
    pub ingredient_overlap: f64,
    pub technique_similarity: f64,
}

impl SimilarRecipe {
    /// Check if recipe is highly similar (>= 0.8 score)
    pub fn is_highly_similar(&self) -> bool {
        self.similarity_score >= 0.8
    }

    /// Get similarity badge for UI display
    pub fn similarity_badge(&self) -> &'static str {
        if self.similarity_score >= 0.9 {
            "Very Similar"
        } else if self.similarity_score >= 0.7 {
            "Similar"
        } else {
            "Somewhat Similar"
        }
    }
}

/// User preferences view derived from discovery sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferencesView {
    pub user_id: Uuid,
    pub preferred_difficulties: Vec<Difficulty>,
    pub preferred_meal_types: Vec<String>,
    pub preferred_dietary_restrictions: Vec<String>,
    pub avg_prep_time_preference: u32,
    pub favorite_ingredients: Vec<String>,
    pub disliked_ingredients: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

impl UserPreferencesView {
    /// Check if user prefers quick recipes (< 30 min)
    pub fn prefers_quick_recipes(&self) -> bool {
        self.avg_prep_time_preference <= 30
    }

    /// Get difficulty preference as string
    pub fn difficulty_preference_text(&self) -> String {
        if self.preferred_difficulties.is_empty() {
            "No preference".to_string()
        } else {
            format!("{:?}", self.preferred_difficulties)
        }
    }
}

/// Discovery analytics view for tracking user behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAnalyticsView {
    pub time_period: String,
    pub total_searches: u32,
    pub total_filter_applications: u32,
    pub total_recipe_views: u32,
    pub popular_search_terms: Vec<String>,
    pub popular_filters: Vec<String>,
    pub conversion_rate: f64, // views to recipes ratio
    pub generated_at: DateTime<Utc>,
}

impl DiscoveryAnalyticsView {
    /// Calculate search success rate
    pub fn search_success_rate(&self) -> f64 {
        if self.total_searches == 0 {
            0.0
        } else {
            (self.total_recipe_views as f64) / (self.total_searches as f64)
        }
    }
}
