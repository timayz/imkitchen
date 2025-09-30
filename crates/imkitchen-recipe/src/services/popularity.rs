use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PopularityError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Recipe not found: {0}")]
    RecipeNotFound(Uuid),
    #[error("Invalid popularity score: {0}")]
    InvalidScore(f64),
    #[error("Time window error: {0}")]
    TimeWindow(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityMetrics {
    pub recipe_id: Uuid,
    pub base_score: f64,
    pub trending_score: f64,
    pub view_count: i64,
    pub rating_average: f64,
    pub rating_count: i64,
    pub bookmark_count: i64,
    pub recent_activity_score: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingRecipe {
    pub recipe_id: Uuid,
    pub title: String,
    pub trending_score: f64,
    pub velocity: f64, // Rate of change in popularity
    pub recent_views: i64,
    pub time_window: String,
    pub trend_factors: Vec<TrendFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendFactor {
    ViewSpike { multiplier: f64, timeframe: String },
    RatingBoost { average_rating: f64, recent_ratings: i64 },
    BookmarkSurge { bookmark_velocity: f64 },
    SeasonalRelevance { relevance_score: f64, context: String },
    NewRecipe { days_old: i32, novelty_boost: f64 },
    SearchPopularity { query_frequency: i64, search_rank: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularityConfig {
    pub view_weight: f64,
    pub rating_weight: f64,
    pub bookmark_weight: f64,
    pub recency_decay_days: i64,
    pub trending_window_hours: i64,
    pub min_views_for_trending: i64,
    pub velocity_threshold: f64,
}

impl Default for PopularityConfig {
    fn default() -> Self {
        Self {
            view_weight: 1.0,
            rating_weight: 3.0,
            bookmark_weight: 5.0,
            recency_decay_days: 30,
            trending_window_hours: 24,
            min_views_for_trending: 50,
            velocity_threshold: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub label: String,
}

impl TimeWindow {
    pub fn last_24_hours() -> Self {
        let end = Utc::now();
        let start = end - Duration::hours(24);
        Self {
            start,
            end,
            label: "24h".to_string(),
        }
    }

    pub fn last_week() -> Self {
        let end = Utc::now();
        let start = end - Duration::days(7);
        Self {
            start,
            end,
            label: "7d".to_string(),
        }
    }

    pub fn last_month() -> Self {
        let end = Utc::now();
        let start = end - Duration::days(30);
        Self {
            start,
            end,
            label: "30d".to_string(),
        }
    }
}

pub struct PopularityService {
    config: PopularityConfig,
}

impl PopularityService {
    pub fn new() -> Self {
        Self {
            config: PopularityConfig::default(),
        }
    }

    pub fn with_config(config: PopularityConfig) -> Self {
        Self { config }
    }

    /// Calculate base popularity score for a recipe
    pub async fn calculate_base_score(
        &self,
        recipe_id: Uuid,
    ) -> Result<f64, PopularityError> {
        // In a real implementation, this would query the database
        // For now, we'll simulate the calculation
        
        let metrics = self.get_recipe_metrics(recipe_id).await?;
        
        let view_score = (metrics.view_count as f64).ln() * self.config.view_weight;
        let rating_score = metrics.rating_average * (metrics.rating_count as f64).sqrt() * self.config.rating_weight;
        let bookmark_score = (metrics.bookmark_count as f64) * self.config.bookmark_weight;
        
        // Apply recency decay
        let days_since_update = (Utc::now() - metrics.last_updated).num_days();
        let recency_factor = (-days_since_update as f64 / self.config.recency_decay_days as f64).exp();
        
        let base_score = (view_score + rating_score + bookmark_score) * recency_factor;
        
        // Normalize to 0-100 scale
        let normalized_score = (base_score * 10.0).min(100.0).max(0.0);
        
        Ok(normalized_score)
    }

    /// Calculate trending score with velocity analysis
    pub async fn calculate_trending_score(
        &self,
        recipe_id: Uuid,
        time_window: &TimeWindow,
    ) -> Result<f64, PopularityError> {
        let current_metrics = self.get_recipe_metrics_for_window(recipe_id, time_window).await?;
        let previous_window = TimeWindow {
            start: time_window.start - (time_window.end - time_window.start),
            end: time_window.start,
            label: format!("prev_{}", time_window.label),
        };
        let previous_metrics = self.get_recipe_metrics_for_window(recipe_id, &previous_window).await?;
        
        // Calculate velocity (rate of change)
        let view_velocity = if previous_metrics.view_count > 0 {
            current_metrics.view_count as f64 / previous_metrics.view_count as f64
        } else {
            current_metrics.view_count as f64
        };
        
        let rating_velocity = if previous_metrics.rating_count > 0 {
            (current_metrics.rating_average * current_metrics.rating_count as f64) /
            (previous_metrics.rating_average * previous_metrics.rating_count as f64)
        } else {
            current_metrics.rating_average
        };
        
        let bookmark_velocity = if previous_metrics.bookmark_count > 0 {
            current_metrics.bookmark_count as f64 / previous_metrics.bookmark_count as f64
        } else {
            current_metrics.bookmark_count as f64
        };
        
        // Weighted velocity score
        let velocity_score = (view_velocity * self.config.view_weight +
                             rating_velocity * self.config.rating_weight +
                             bookmark_velocity * self.config.bookmark_weight) /
                            (self.config.view_weight + self.config.rating_weight + self.config.bookmark_weight);
        
        // Apply minimum threshold
        if current_metrics.view_count < self.config.min_views_for_trending {
            return Ok(0.0);
        }
        
        // Boost for velocity above threshold
        let trending_multiplier = if velocity_score > self.config.velocity_threshold {
            velocity_score.powf(1.5)
        } else {
            velocity_score
        };
        
        let base_score = self.calculate_base_score(recipe_id).await?;
        let trending_score = base_score * trending_multiplier;
        
        Ok(trending_score.min(200.0)) // Cap at 200 for trending
    }

    /// Get trending recipes for a specific time window
    pub async fn get_trending_recipes(
        &self,
        time_window: &TimeWindow,
        limit: usize,
    ) -> Result<Vec<TrendingRecipe>, PopularityError> {
        // In a real implementation, this would query the database
        // For now, we'll simulate trending recipes
        
        let recipe_ids = self.get_active_recipes().await?;
        let mut trending_recipes = Vec::new();
        
        for recipe_id in recipe_ids.into_iter().take(limit * 2) { // Get more to filter
            let trending_score = self.calculate_trending_score(recipe_id, time_window).await?;
            
            if trending_score > 0.0 {
                let velocity = self.calculate_velocity(recipe_id, time_window).await?;
                let trend_factors = self.analyze_trend_factors(recipe_id, time_window).await?;
                
                trending_recipes.push(TrendingRecipe {
                    recipe_id,
                    title: format!("Recipe {}", recipe_id), // Would be fetched from DB
                    trending_score,
                    velocity,
                    recent_views: self.get_recent_views(recipe_id, time_window).await?,
                    time_window: time_window.label.clone(),
                    trend_factors,
                });
            }
        }
        
        // Sort by trending score and take limit
        trending_recipes.sort_by(|a, b| b.trending_score.partial_cmp(&a.trending_score).unwrap());
        trending_recipes.truncate(limit);
        
        Ok(trending_recipes)
    }

    /// Analyze factors contributing to a recipe's trending status
    pub async fn analyze_trend_factors(
        &self,
        recipe_id: Uuid,
        time_window: &TimeWindow,
    ) -> Result<Vec<TrendFactor>, PopularityError> {
        let mut factors = Vec::new();
        
        // Check for view spikes
        let recent_views = self.get_recent_views(recipe_id, time_window).await?;
        let historical_average = self.get_historical_view_average(recipe_id).await?;
        
        if recent_views > (historical_average * 2.0) as i64 {
            factors.push(TrendFactor::ViewSpike {
                multiplier: recent_views as f64 / historical_average,
                timeframe: time_window.label.clone(),
            });
        }
        
        // Check for rating boosts
        let recent_ratings = self.get_recent_ratings(recipe_id, time_window).await?;
        if recent_ratings.rating_count > 5 && recent_ratings.rating_average > 4.0 {
            factors.push(TrendFactor::RatingBoost {
                average_rating: recent_ratings.rating_average,
                recent_ratings: recent_ratings.rating_count,
            });
        }
        
        // Check for bookmark surges
        let bookmark_velocity = self.calculate_bookmark_velocity(recipe_id, time_window).await?;
        if bookmark_velocity > 2.0 {
            factors.push(TrendFactor::BookmarkSurge { bookmark_velocity });
        }
        
        // Check if it's a new recipe
        let recipe_age = self.get_recipe_age_days(recipe_id).await?;
        if recipe_age <= 7 {
            factors.push(TrendFactor::NewRecipe {
                days_old: recipe_age,
                novelty_boost: (8.0 - recipe_age as f64) / 8.0,
            });
        }
        
        // Check search popularity
        let search_metrics = self.get_search_metrics(recipe_id, time_window).await?;
        if search_metrics.query_frequency > 20 {
            factors.push(TrendFactor::SearchPopularity {
                query_frequency: search_metrics.query_frequency,
                search_rank: search_metrics.average_rank,
            });
        }
        
        Ok(factors)
    }

    /// Update popularity metrics for a recipe
    pub async fn update_popularity_metrics(
        &self,
        recipe_id: Uuid,
    ) -> Result<PopularityMetrics, PopularityError> {
        let base_score = self.calculate_base_score(recipe_id).await?;
        let trending_score = self.calculate_trending_score(recipe_id, &TimeWindow::last_24_hours()).await?;
        let metrics = self.get_recipe_metrics(recipe_id).await?;
        
        let updated_metrics = PopularityMetrics {
            recipe_id,
            base_score,
            trending_score,
            view_count: metrics.view_count,
            rating_average: metrics.rating_average,
            rating_count: metrics.rating_count,
            bookmark_count: metrics.bookmark_count,
            recent_activity_score: self.calculate_recent_activity_score(recipe_id).await?,
            last_updated: Utc::now(),
        };
        
        // In a real implementation, save to database
        self.save_popularity_metrics(&updated_metrics).await?;
        
        Ok(updated_metrics)
    }

    /// Get popularity rankings for recipes
    pub async fn get_popularity_rankings(
        &self,
        category: Option<String>,
        time_window: &TimeWindow,
        limit: usize,
    ) -> Result<Vec<(Uuid, f64)>, PopularityError> {
        // In a real implementation, this would use database queries with proper filtering
        let recipe_ids = if let Some(_category) = category {
            self.get_recipes_by_category(_category).await?
        } else {
            self.get_active_recipes().await?
        };
        
        let mut rankings = Vec::new();
        for recipe_id in recipe_ids {
            let score = match time_window.label.as_str() {
                "24h" => self.calculate_trending_score(recipe_id, time_window).await?,
                _ => self.calculate_base_score(recipe_id).await?,
            };
            rankings.push((recipe_id, score));
        }
        
        rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        rankings.truncate(limit);
        
        Ok(rankings)
    }

    // Helper methods for simulated data access
    async fn get_recipe_metrics(&self, recipe_id: Uuid) -> Result<PopularityMetrics, PopularityError> {
        // Simulate database query with mock data
        Ok(PopularityMetrics {
            recipe_id,
            base_score: 0.0,
            trending_score: 0.0,
            view_count: (recipe_id.as_u128() % 1000) as i64 + 100,
            rating_average: 3.5 + ((recipe_id.as_u128() % 15) as f64 / 10.0),
            rating_count: (recipe_id.as_u128() % 50) as i64 + 10,
            bookmark_count: (recipe_id.as_u128() % 25) as i64 + 5,
            recent_activity_score: 0.0,
            last_updated: Utc::now() - Duration::days((recipe_id.as_u128() % 30) as i64),
        })
    }

    async fn get_recipe_metrics_for_window(
        &self,
        recipe_id: Uuid,
        _time_window: &TimeWindow,
    ) -> Result<PopularityMetrics, PopularityError> {
        // Simulate windowed metrics
        let mut metrics = self.get_recipe_metrics(recipe_id).await?;
        metrics.view_count = (metrics.view_count as f64 * 0.3) as i64; // Simulate window portion
        metrics.rating_count = (metrics.rating_count as f64 * 0.2) as i64;
        metrics.bookmark_count = (metrics.bookmark_count as f64 * 0.25) as i64;
        Ok(metrics)
    }

    async fn get_active_recipes(&self) -> Result<Vec<Uuid>, PopularityError> {
        // Simulate getting active recipe IDs
        Ok((0..20).map(|_| Uuid::new_v4()).collect())
    }

    async fn get_recipes_by_category(&self, _category: String) -> Result<Vec<Uuid>, PopularityError> {
        // Simulate category filtering
        Ok((0..10).map(|_| Uuid::new_v4()).collect())
    }

    async fn calculate_velocity(&self, recipe_id: Uuid, time_window: &TimeWindow) -> Result<f64, PopularityError> {
        let current = self.get_recipe_metrics_for_window(recipe_id, time_window).await?;
        let historical = self.get_recipe_metrics(recipe_id).await?;
        
        if historical.view_count > 0 {
            Ok(current.view_count as f64 / historical.view_count as f64)
        } else {
            Ok(1.0)
        }
    }

    async fn get_recent_views(&self, recipe_id: Uuid, _time_window: &TimeWindow) -> Result<i64, PopularityError> {
        let metrics = self.get_recipe_metrics(recipe_id).await?;
        Ok((metrics.view_count as f64 * 0.3) as i64) // Simulate recent portion
    }

    async fn get_historical_view_average(&self, recipe_id: Uuid) -> Result<f64, PopularityError> {
        let metrics = self.get_recipe_metrics(recipe_id).await?;
        Ok(metrics.view_count as f64 / 30.0) // Simulate daily average
    }

    async fn get_recent_ratings(&self, recipe_id: Uuid, _time_window: &TimeWindow) -> Result<PopularityMetrics, PopularityError> {
        let mut metrics = self.get_recipe_metrics(recipe_id).await?;
        metrics.rating_count = (metrics.rating_count as f64 * 0.2) as i64; // Recent portion
        Ok(metrics)
    }

    async fn calculate_bookmark_velocity(&self, recipe_id: Uuid, time_window: &TimeWindow) -> Result<f64, PopularityError> {
        let current = self.get_recipe_metrics_for_window(recipe_id, time_window).await?;
        let historical = self.get_recipe_metrics(recipe_id).await?;
        
        if historical.bookmark_count > 0 {
            Ok(current.bookmark_count as f64 / historical.bookmark_count as f64)
        } else {
            Ok(1.0)
        }
    }

    async fn get_recipe_age_days(&self, _recipe_id: Uuid) -> Result<i32, PopularityError> {
        // Simulate recipe age
        Ok((chrono::Utc::now().timestamp() % 30) as i32)
    }

    async fn get_search_metrics(&self, recipe_id: Uuid, _time_window: &TimeWindow) -> Result<SearchMetrics, PopularityError> {
        Ok(SearchMetrics {
            query_frequency: (recipe_id.as_u128() % 100) as i64 + 5,
            average_rank: ((recipe_id.as_u128() % 10) + 1) as i32,
        })
    }

    async fn calculate_recent_activity_score(&self, recipe_id: Uuid) -> Result<f64, PopularityError> {
        let metrics = self.get_recipe_metrics(recipe_id).await?;
        let window = TimeWindow::last_24_hours();
        let recent_views = self.get_recent_views(recipe_id, &window).await?;
        
        // Combine recent activity indicators
        let activity_score = (recent_views as f64).ln() + 
                           (metrics.rating_count as f64 * 0.1) +
                           (metrics.bookmark_count as f64 * 0.2);
        
        Ok(activity_score.max(0.0).min(100.0))
    }

    async fn save_popularity_metrics(&self, _metrics: &PopularityMetrics) -> Result<(), PopularityError> {
        // Simulate saving to database
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct SearchMetrics {
    query_frequency: i64,
    average_rank: i32,
}

impl Default for PopularityService {
    fn default() -> Self {
        Self::new()
    }
}