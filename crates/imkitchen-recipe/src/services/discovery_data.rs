use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum DiscoveryDataError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Recipe not found: {0}")]
    RecipeNotFound(Uuid),
    #[error("User not found: {0}")]
    UserNotFound(String),
    #[error("Invalid metric value: {0}")]
    InvalidMetric(String),
    #[error("Cache error: {0}")]
    Cache(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDiscoveryMetrics {
    pub recipe_id: Uuid,
    pub base_popularity_score: f64,
    pub trending_score_24h: f64,
    pub trending_score_7d: f64,
    pub trending_score_30d: f64,
    pub view_count_total: i64,
    pub view_count_24h: i64,
    pub view_count_7d: i64,
    pub view_count_30d: i64,
    pub bookmark_count: i64,
    pub bookmark_velocity_24h: f64,
    pub search_mention_count: i64,
    pub search_rank_average: Option<f64>,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub last_bookmarked_at: Option<DateTime<Utc>>,
    pub metrics_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDiscoveryPreference {
    pub user_id: String,
    pub preference_type: PreferenceType,
    pub preference_value: String,
    pub weight: f64,
    pub interaction_count: i64,
    pub last_interaction_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferenceType {
    Category,
    Difficulty,
    PrepTime,
    Dietary,
    MealType,
    Cuisine,
    Technique,
}

impl PreferenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreferenceType::Category => "category",
            PreferenceType::Difficulty => "difficulty",
            PreferenceType::PrepTime => "prep_time",
            PreferenceType::Dietary => "dietary",
            PreferenceType::MealType => "meal_type",
            PreferenceType::Cuisine => "cuisine",
            PreferenceType::Technique => "technique",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "category" => Some(PreferenceType::Category),
            "difficulty" => Some(PreferenceType::Difficulty),
            "prep_time" => Some(PreferenceType::PrepTime),
            "dietary" => Some(PreferenceType::Dietary),
            "meal_type" => Some(PreferenceType::MealType),
            "cuisine" => Some(PreferenceType::Cuisine),
            "technique" => Some(PreferenceType::Technique),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSimilarityCache {
    pub recipe_id: Uuid,
    pub similar_recipe_id: Uuid,
    pub similarity_score: f64,
    pub similarity_type: SimilarityType,
    pub similarity_reasons: Vec<String>,
    pub cache_valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityType {
    Ingredient,
    Technique,
    Category,
    Difficulty,
    PrepTime,
    Flavor,
    Equipment,
}

impl SimilarityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SimilarityType::Ingredient => "ingredient",
            SimilarityType::Technique => "technique", 
            SimilarityType::Category => "category",
            SimilarityType::Difficulty => "difficulty",
            SimilarityType::PrepTime => "prep_time",
            SimilarityType::Flavor => "flavor",
            SimilarityType::Equipment => "equipment",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySearchSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_queries: i32,
    pub total_results_viewed: i32,
    pub recipes_clicked: i32,
    pub recipes_bookmarked: i32,
    pub session_duration_ms: Option<i64>,
    pub conversion_rate: Option<f64>,
    pub satisfaction_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQueryAnalytics {
    pub query_id: String,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub query_text: String,
    pub query_type: QueryType,
    pub results_count: i32,
    pub results_clicked: i32,
    pub click_through_rate: Option<f64>,
    pub first_click_position: Option<i32>,
    pub query_duration_ms: i64,
    pub had_typos: bool,
    pub used_suggestions: bool,
    pub applied_filters: HashMap<String, Vec<String>>,
    pub sort_order: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Search,
    Filter,
    Suggestion,
    Trending,
    Popular,
    Similar,
}

impl QueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueryType::Search => "search",
            QueryType::Filter => "filter",
            QueryType::Suggestion => "suggestion",
            QueryType::Trending => "trending",
            QueryType::Popular => "popular",
            QueryType::Similar => "similar",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDiscoveryEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub user_id: Option<String>,
    pub recipe_id: Uuid,
    pub session_id: Option<String>,
    pub source_context: SourceContext,
    pub source_query: Option<String>,
    pub position_in_results: Option<i32>,
    pub event_metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    View,
    Click,
    Bookmark,
    Share,
    Rate,
    SimilarClick,
    FilterApply,
    SearchRefine,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::View => "view",
            EventType::Click => "click",
            EventType::Bookmark => "bookmark",
            EventType::Share => "share",
            EventType::Rate => "rate",
            EventType::SimilarClick => "similar_click",
            EventType::FilterApply => "filter_apply",
            EventType::SearchRefine => "search_refine",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceContext {
    Search,
    Trending,
    Similar,
    Popular,
    Featured,
    Recommendation,
    Category,
    Random,
}

impl SourceContext {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceContext::Search => "search",
            SourceContext::Trending => "trending",
            SourceContext::Similar => "similar",
            SourceContext::Popular => "popular",
            SourceContext::Featured => "featured",
            SourceContext::Recommendation => "recommendation",
            SourceContext::Category => "category",
            SourceContext::Random => "random",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDiscoveryContent {
    pub recipe_id: Uuid,
    pub search_keywords: Vec<String>,
    pub cooking_techniques: Vec<String>,
    pub flavor_profile: FlavorProfile,
    pub season_relevance: HashMap<String, f64>,
    pub skill_level_computed: f64,
    pub time_to_make_total: i32,
    pub dietary_tags: Vec<String>,
    pub cuisine_style: Option<String>,
    pub meal_occasions: Vec<String>,
    pub ingredient_complexity_score: f64,
    pub equipment_required: Vec<String>,
    pub content_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlavorProfile {
    pub sweet: f64,
    pub salty: f64,
    pub sour: f64,
    pub bitter: f64,
    pub umami: f64,
    pub spicy: f64,
    pub richness: f64,
    pub freshness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryFeatureConfig {
    pub config_key: String,
    pub config_value: String,
    pub value_type: ConfigValueType,
    pub description: Option<String>,
    pub category: String,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigValueType {
    String,
    Number,
    Boolean,
    Json,
}

impl ConfigValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigValueType::String => "string",
            ConfigValueType::Number => "number",
            ConfigValueType::Boolean => "boolean",
            ConfigValueType::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularRecipe {
    pub recipe_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub cook_time: Option<i32>,
    pub difficulty: Option<String>,
    pub category: Option<String>,
    pub base_popularity_score: f64,
    pub view_count_total: i64,
    pub bookmark_count: i64,
    pub average_rating: f64,
    pub review_count: i64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingRecipe {
    pub recipe_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub prep_time: Option<i32>,
    pub difficulty: Option<String>,
    pub category: Option<String>,
    pub trending_score_24h: f64,
    pub trending_score_7d: f64,
    pub view_count_24h: i64,
    pub view_count_7d: i64,
    pub bookmark_velocity_24h: f64,
    pub velocity_24h: f64,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryAnalyticsSummary {
    pub analytics_date: String,
    pub total_queries: i64,
    pub unique_users: i64,
    pub avg_results_per_query: f64,
    pub avg_click_through_rate: f64,
    pub avg_query_duration_ms: f64,
    pub queries_with_suggestions: i64,
    pub queries_with_typos: i64,
}

pub struct DiscoveryDataService {
    // In a real implementation, this would contain database connection
    // For now, we'll simulate database operations
}

impl DiscoveryDataService {
    pub fn new() -> Self {
        Self {}
    }

    /// Get or create discovery metrics for a recipe
    pub async fn get_recipe_metrics(&self, recipe_id: Uuid) -> Result<RecipeDiscoveryMetrics, DiscoveryDataError> {
        // Simulate database query
        Ok(RecipeDiscoveryMetrics {
            recipe_id,
            base_popularity_score: (recipe_id.as_u128() % 100) as f64,
            trending_score_24h: ((recipe_id.as_u128() % 50) as f64) * 2.0,
            trending_score_7d: ((recipe_id.as_u128() % 30) as f64) * 3.0,
            trending_score_30d: ((recipe_id.as_u128() % 20) as f64) * 4.0,
            view_count_total: (recipe_id.as_u128() % 1000) as i64 + 100,
            view_count_24h: (recipe_id.as_u128() % 50) as i64 + 10,
            view_count_7d: (recipe_id.as_u128() % 200) as i64 + 50,
            view_count_30d: (recipe_id.as_u128() % 800) as i64 + 200,
            bookmark_count: (recipe_id.as_u128() % 25) as i64 + 5,
            bookmark_velocity_24h: ((recipe_id.as_u128() % 10) as f64) / 5.0 + 1.0,
            search_mention_count: (recipe_id.as_u128() % 100) as i64,
            search_rank_average: Some(((recipe_id.as_u128() % 10) as f64) + 1.0),
            last_viewed_at: Some(Utc::now()),
            last_bookmarked_at: Some(Utc::now()),
            metrics_updated_at: Utc::now(),
        })
    }

    /// Update recipe metrics
    pub async fn update_recipe_metrics(&self, metrics: &RecipeDiscoveryMetrics) -> Result<(), DiscoveryDataError> {
        // Simulate database update
        Ok(())
    }

    /// Get user discovery preferences
    pub async fn get_user_preferences(&self, user_id: &str) -> Result<Vec<UserDiscoveryPreference>, DiscoveryDataError> {
        // Simulate database query
        Ok(vec![
            UserDiscoveryPreference {
                user_id: user_id.to_string(),
                preference_type: PreferenceType::Category,
                preference_value: "pasta".to_string(),
                weight: 0.8,
                interaction_count: 15,
                last_interaction_at: Utc::now(),
            },
            UserDiscoveryPreference {
                user_id: user_id.to_string(),
                preference_type: PreferenceType::Difficulty,
                preference_value: "easy".to_string(),
                weight: 0.6,
                interaction_count: 8,
                last_interaction_at: Utc::now(),
            },
        ])
    }

    /// Update user preference
    pub async fn update_user_preference(
        &self,
        user_id: &str,
        preference_type: PreferenceType,
        preference_value: &str,
        interaction_weight: f64,
    ) -> Result<(), DiscoveryDataError> {
        // Simulate database upsert
        Ok(())
    }

    /// Get cached recipe similarities
    pub async fn get_recipe_similarities(
        &self, 
        recipe_id: Uuid, 
        limit: usize
    ) -> Result<Vec<RecipeSimilarityCache>, DiscoveryDataError> {
        // Simulate database query
        let mut similarities = Vec::new();
        for i in 0..limit.min(10) {
            similarities.push(RecipeSimilarityCache {
                recipe_id,
                similar_recipe_id: Uuid::new_v4(),
                similarity_score: 0.9 - (i as f64 * 0.1),
                similarity_type: SimilarityType::Ingredient,
                similarity_reasons: vec![
                    "Common ingredients: tomato, basil".to_string(),
                    "Similar cooking technique: sautéing".to_string(),
                ],
                cache_valid_until: Utc::now() + chrono::Duration::hours(24),
            });
        }
        Ok(similarities)
    }

    /// Cache recipe similarities
    pub async fn cache_recipe_similarities(
        &self,
        similarities: &[RecipeSimilarityCache],
    ) -> Result<(), DiscoveryDataError> {
        // Simulate database insert/update
        Ok(())
    }

    /// Record discovery event
    pub async fn record_discovery_event(&self, event: &RecipeDiscoveryEvent) -> Result<(), DiscoveryDataError> {
        // Simulate database insert
        Ok(())
    }

    /// Record query analytics
    pub async fn record_query_analytics(&self, analytics: &DiscoveryQueryAnalytics) -> Result<(), DiscoveryDataError> {
        // Simulate database insert
        Ok(())
    }

    /// Get popular recipes
    pub async fn get_popular_recipes(&self, limit: usize, category: Option<&str>) -> Result<Vec<PopularRecipe>, DiscoveryDataError> {
        // Simulate database query with view
        let mut recipes = Vec::new();
        for i in 0..limit.min(20) {
            recipes.push(PopularRecipe {
                recipe_id: Uuid::new_v4(),
                title: format!("Popular Recipe {}", i + 1),
                description: Some(format!("A very popular recipe #{}", i + 1)),
                prep_time: Some(20 + (i as i32 * 5)),
                cook_time: Some(30 + (i as i32 * 3)),
                difficulty: Some("easy".to_string()),
                category: category.map(|c| c.to_string()).or_else(|| Some("main".to_string())),
                base_popularity_score: 100.0 - (i as f64 * 5.0),
                view_count_total: 1000 - (i as i64 * 50),
                bookmark_count: 100 - (i as i64 * 5),
                average_rating: 4.8 - (i as f64 * 0.1),
                review_count: 50 - (i as i64 * 2),
                last_viewed_at: Some(Utc::now()),
                created_at: Utc::now() - chrono::Duration::days(i as i64),
            });
        }
        Ok(recipes)
    }

    /// Get trending recipes  
    pub async fn get_trending_recipes(&self, limit: usize, time_window: &str) -> Result<Vec<TrendingRecipe>, DiscoveryDataError> {
        // Simulate database query with view
        let mut recipes = Vec::new();
        for i in 0..limit.min(15) {
            let trending_score = match time_window {
                "24h" => 150.0 - (i as f64 * 10.0),
                "7d" => 120.0 - (i as f64 * 8.0),
                "30d" => 100.0 - (i as f64 * 6.0),
                _ => 80.0 - (i as f64 * 5.0),
            };
            
            recipes.push(TrendingRecipe {
                recipe_id: Uuid::new_v4(),
                title: format!("Trending Recipe {}", i + 1),
                description: Some(format!("A trending recipe for {}", time_window)),
                prep_time: Some(15 + (i as i32 * 3)),
                difficulty: Some("medium".to_string()),
                category: Some("trending".to_string()),
                trending_score_24h: if time_window == "24h" { trending_score } else { trending_score * 0.8 },
                trending_score_7d: if time_window == "7d" { trending_score } else { trending_score * 0.9 },
                view_count_24h: 200 - (i as i64 * 20),
                view_count_7d: 800 - (i as i64 * 50),
                bookmark_velocity_24h: 2.5 - (i as f64 * 0.2),
                velocity_24h: 3.0 - (i as f64 * 0.3),
                last_viewed_at: Some(Utc::now()),
                created_at: Utc::now() - chrono::Duration::days(i as i64),
            });
        }
        Ok(recipes)
    }

    /// Get discovery analytics summary
    pub async fn get_analytics_summary(&self, days: i32) -> Result<Vec<DiscoveryAnalyticsSummary>, DiscoveryDataError> {
        // Simulate analytics query
        let mut summaries = Vec::new();
        for i in 0..days.min(30) {
            let date = Utc::now() - chrono::Duration::days(i as i64);
            summaries.push(DiscoveryAnalyticsSummary {
                analytics_date: date.format("%Y-%m-%d").to_string(),
                total_queries: 1000 - (i as i64 * 20),
                unique_users: 200 - (i as i64 * 5),
                avg_results_per_query: 12.5 + (i as f64 * 0.2),
                avg_click_through_rate: 0.35 - (i as f64 * 0.01),
                avg_query_duration_ms: 250.0 + (i as f64 * 10.0),
                queries_with_suggestions: 400 - (i as i64 * 10),
                queries_with_typos: 50 - (i as i64 * 2),
            });
        }
        Ok(summaries)
    }

    /// Get feature configuration
    pub async fn get_feature_config(&self, category: Option<&str>) -> Result<Vec<DiscoveryFeatureConfig>, DiscoveryDataError> {
        // Simulate configuration query
        let mut configs = vec![
            DiscoveryFeatureConfig {
                config_key: "search_max_results".to_string(),
                config_value: "50".to_string(),
                value_type: ConfigValueType::Number,
                description: Some("Maximum search results per page".to_string()),
                category: "search".to_string(),
                is_active: true,
                updated_at: Utc::now(),
                updated_by: Some("system".to_string()),
            },
            DiscoveryFeatureConfig {
                config_key: "trending_min_views_24h".to_string(),
                config_value: "10".to_string(),
                value_type: ConfigValueType::Number,
                description: Some("Minimum views for 24h trending eligibility".to_string()),
                category: "trending".to_string(),
                is_active: true,
                updated_at: Utc::now(),
                updated_by: Some("system".to_string()),
            },
        ];

        if let Some(cat) = category {
            configs.retain(|c| c.category == cat);
        }

        Ok(configs)
    }

    /// Update feature configuration
    pub async fn update_feature_config(
        &self,
        config_key: &str,
        config_value: &str,
        updated_by: Option<&str>,
    ) -> Result<(), DiscoveryDataError> {
        // Simulate database update
        Ok(())
    }

    /// Full-text search recipes
    pub async fn search_recipes_fts(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<(Uuid, f64)>, DiscoveryDataError> {
        // Simulate FTS search with relevance scores
        let mut results = Vec::new();
        for i in 0..limit.min(20) {
            results.push((
                Uuid::new_v4(),
                1.0 - (i as f64 * 0.05), // Decreasing relevance
            ));
        }
        Ok(results)
    }
}

impl Default for DiscoveryDataService {
    fn default() -> Self {
        Self::new()
    }
}