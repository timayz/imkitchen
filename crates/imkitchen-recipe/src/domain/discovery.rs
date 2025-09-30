use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;
use imkitchen_shared::types::{Difficulty, DietaryRestriction, MealType};

/// Recipe discovery aggregate for managing search and filtering state
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RecipeDiscovery {
    pub discovery_id: Uuid,
    pub user_id: Option<Uuid>,
    #[validate(nested)]
    pub search_criteria: SearchCriteria,
    #[validate(nested)]
    pub filters: DiscoveryFilters,
    pub sort_order: SortingCriteria,
    #[validate(range(min = 1))]
    pub page: u32,
    #[validate(range(min = 1, max = 100))]
    pub page_size: u32,
    pub created_at: DateTime<Utc>,
}

/// Filters for recipe discovery with comprehensive filtering options
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DiscoveryFilters {
    #[validate(range(min = 1.0, max = 5.0))]
    pub rating_threshold: Option<f32>,
    pub difficulty_levels: Vec<Difficulty>,
    #[validate(range(min = 1))]
    pub max_prep_time: Option<u32>,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub meal_types: Vec<MealType>,
}

/// Search criteria for recipe discovery
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchCriteria {
    #[validate(length(min = 1, max = 500))]
    pub query_text: String,
    pub search_type: SearchType,
    pub include_suggestions: bool,
}

/// Types of search available
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchType {
    FullText,
    Ingredient,
    Title,
}

/// Sorting criteria for recipe discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortingCriteria {
    Newest,
    Popular,
    HighestRated,
    QuickestPrep,
}

impl fmt::Display for SortingCriteria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortingCriteria::Newest => write!(f, "Newest"),
            SortingCriteria::Popular => write!(f, "Most Popular"),
            SortingCriteria::HighestRated => write!(f, "Highest Rated"),
            SortingCriteria::QuickestPrep => write!(f, "Quickest Prep"),
        }
    }
}

/// Trending score tracking for recipe popularity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TrendingScore {
    pub recipe_id: Uuid,
    #[validate(range(min = 0.0))]
    pub popularity_score: f64,
    #[validate(range(min = 1))]
    pub trending_rank: u32,
    pub calculated_at: DateTime<Utc>,
    #[validate(range(min = 0.0))]
    pub time_weighted_score: f64,
}

/// Recipe search service for full-text search, autocomplete, and typo tolerance
pub struct RecipeSearchService {
    // Service implementation will be added when implementing search infrastructure
}

impl RecipeSearchService {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Perform full-text search on recipes
    pub async fn search_recipes(&self, _criteria: &SearchCriteria) -> Result<Vec<Uuid>, SearchError> {
        // Implementation will be added in search infrastructure task
        Ok(vec![])
    }
    
    /// Generate autocomplete suggestions
    pub async fn get_suggestions(&self, _partial_query: &str) -> Result<Vec<String>, SearchError> {
        // Implementation will be added in search infrastructure task
        Ok(vec![])
    }
}

/// Recipe trending service for popularity tracking
pub struct RecipeTrendingService {
    // Service implementation will be added when implementing trending system
}

impl RecipeTrendingService {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Calculate trending score for a recipe
    pub async fn calculate_trending_score(&self, recipe_id: Uuid) -> Result<TrendingScore, TrendingError> {
        // Implementation will be added in trending system task
        Ok(TrendingScore {
            recipe_id,
            popularity_score: 0.0,
            trending_rank: 1,
            calculated_at: Utc::now(),
            time_weighted_score: 0.0,
        })
    }
    
    /// Update trending rankings
    pub async fn update_trending_rankings(&self) -> Result<(), TrendingError> {
        // Implementation will be added in trending system task
        Ok(())
    }
}

/// Random recipe selector with preference-aware selection
pub struct RandomRecipeSelector {
    // Service implementation will be added
}

impl RandomRecipeSelector {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Select random recipe with user preference filtering
    pub async fn select_random_recipe(&self, _user_id: Option<Uuid>, _filters: &DiscoveryFilters) -> Result<Option<Uuid>, SelectionError> {
        // Implementation will be added in random recipe feature
        Ok(None)
    }
}

/// Search error types
#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid search query: {0}")]
    InvalidQuery(String),
}

/// Trending calculation error types
#[derive(Debug, thiserror::Error)]
pub enum TrendingError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Calculation error: {0}")]
    Calculation(String),
}

/// Random selection error types
#[derive(Debug, thiserror::Error)]
pub enum SelectionError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("No recipes available")]
    NoRecipesAvailable,
}

impl Default for DiscoveryFilters {
    fn default() -> Self {
        Self {
            rating_threshold: None,
            difficulty_levels: vec![],
            max_prep_time: None,
            dietary_restrictions: vec![],
            meal_types: vec![],
        }
    }
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            query_text: String::new(),
            search_type: SearchType::FullText,
            include_suggestions: true,
        }
    }
}