-- Performance Optimization Indices for Recipe Search and Meal Planning
-- Migration: 011_performance_optimization_indices.up.sql

-- Recipe Search Performance Indices
-- Composite index for common recipe search patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_search_composite 
ON recipes(deleted_at, total_time, complexity, average_rating) 
WHERE deleted_at IS NULL;

-- Optimized index for meal type + dietary restrictions queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_meal_diet_composite 
ON recipes USING GIN(meal_type, dietary_labels) 
WHERE deleted_at IS NULL;

-- Enhanced full-text search index with improved performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_fulltext_enhanced 
ON recipes USING GIN(to_tsvector('english', title || ' ' || COALESCE(description, '') || ' ' || COALESCE(cuisine_type, ''))) 
WHERE deleted_at IS NULL;

-- Time-based filtering optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_time_filters 
ON recipes(prep_time, cook_time, total_time, complexity) 
WHERE deleted_at IS NULL;

-- Rating and popularity optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_popularity 
ON recipes(average_rating DESC, total_ratings DESC, created_at DESC) 
WHERE deleted_at IS NULL AND average_rating > 0;

-- Ingredient-based search optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_ingredients_enhanced 
ON recipes USING GIN(ingredients) 
WHERE deleted_at IS NULL;

-- Nutritional filtering optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_nutrition_enhanced 
ON recipes USING GIN(nutrition) 
WHERE deleted_at IS NULL AND nutrition IS NOT NULL;

-- User-specific recipe access optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_user_access 
ON recipes(user_id, is_public, deleted_at, created_at DESC) 
WHERE deleted_at IS NULL;

-- Meal Plan Performance Indices
-- Optimized user meal plan queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_user_active 
ON meal_plans(user_id, status, week_start DESC) 
WHERE status = 'active';

-- Enhanced meal plan date range queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_date_range 
ON meal_plans(week_start, week_end, status, user_id) 
WHERE status IN ('active', 'draft');

-- Meal plan meals content optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_meals_enhanced 
ON meal_plans USING GIN(meals) 
WHERE status = 'active';

-- Generation performance tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_generation_perf 
ON meal_plans(generation_duration_ms, generation_algorithm_version, created_at DESC) 
WHERE generation_duration_ms IS NOT NULL;

-- Recipe Rating Performance Indices
-- User rating lookup optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_ratings_user_recipe 
ON recipe_ratings(user_id, recipe_id, overall_rating, created_at DESC);

-- Recipe popularity calculation optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_ratings_recipe_stats 
ON recipe_ratings(recipe_id, overall_rating, created_at DESC) 
WHERE overall_rating IS NOT NULL;

-- Recent ratings for trending calculations
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_ratings_trending 
ON recipe_ratings(created_at DESC, overall_rating, recipe_id) 
WHERE created_at > NOW() - INTERVAL '30 days';

-- Shopping List Performance Indices
-- User shopping list queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_shopping_lists_user_active 
ON shopping_lists(user_id, status, created_at DESC) 
WHERE status = 'active';

-- Meal plan association optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_shopping_lists_meal_plan 
ON shopping_lists(meal_plan_id, status, created_at DESC) 
WHERE meal_plan_id IS NOT NULL;

-- Shopping list items search
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_shopping_lists_items 
ON shopping_lists USING GIN(items) 
WHERE status = 'active';

-- User Preferences Performance Indices
-- Preference learning optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_pref_history_learning 
ON user_preference_history(user_id, created_at DESC, preference_type, confidence_score) 
WHERE confidence_score > 0.5;

-- Recipe Performance Metrics Indices
-- Performance analytics optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_perf_analytics 
ON recipe_performance_metrics(date DESC, recipe_id, algorithm_score DESC) 
WHERE date > CURRENT_DATE - INTERVAL '90 days';

-- Algorithm scoring optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_perf_scoring 
ON recipe_performance_metrics(recipe_id, algorithm_score DESC, date DESC) 
WHERE algorithm_score > 0;

-- Usage pattern analysis
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipe_perf_usage 
ON recipe_performance_metrics(date DESC, times_included_in_meal_plans DESC, completion_rate DESC) 
WHERE times_included_in_meal_plans > 0;

-- Specialized Function-Based Indices
-- Recipe search relevance scoring
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_search_score 
ON recipes((
    COALESCE(average_rating, 0) * 0.4 + 
    LEAST(total_ratings / 10.0, 5.0) * 0.3 + 
    CASE complexity 
        WHEN 'simple' THEN 5.0 
        WHEN 'moderate' THEN 3.0 
        ELSE 1.0 
    END * 0.2 +
    CASE 
        WHEN total_time <= 30 THEN 5.0
        WHEN total_time <= 60 THEN 3.0
        ELSE 1.0
    END * 0.1
)) 
WHERE deleted_at IS NULL;

-- User engagement scoring for meal plans
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_meal_plans_engagement 
ON meal_plans((completion_percentage * 0.6 + 
    CASE 
        WHEN jsonb_array_length(COALESCE(user_feedback, '[]'::jsonb)) > 0 THEN 40.0 
        ELSE 0.0 
    END)) 
WHERE status = 'active' AND completion_percentage > 0;

-- Partial Indices for Common Queries
-- Active recipes only
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_active_only 
ON recipes(created_at DESC, average_rating DESC) 
WHERE deleted_at IS NULL AND is_public = true;

-- User's own recipes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_user_own 
ON recipes(user_id, created_at DESC, average_rating DESC) 
WHERE deleted_at IS NULL;

-- High-rated recipes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_high_rated 
ON recipes(average_rating DESC, total_ratings DESC, created_at DESC) 
WHERE deleted_at IS NULL AND average_rating >= 4.0 AND total_ratings >= 5;

-- Quick recipes (30 minutes or less)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_quick 
ON recipes(total_time ASC, average_rating DESC) 
WHERE deleted_at IS NULL AND total_time <= 30;

-- Vegetarian/Vegan recipes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_vegetarian 
ON recipes(created_at DESC, average_rating DESC) 
WHERE deleted_at IS NULL AND dietary_labels && ARRAY['vegetarian', 'vegan'];

-- Performance Monitoring Views
-- Create materialized view for recipe search statistics
CREATE MATERIALIZED VIEW IF NOT EXISTS recipe_search_stats AS
SELECT 
    date_trunc('day', created_at) as date,
    COUNT(*) as total_recipes,
    AVG(average_rating) as avg_rating,
    AVG(total_time) as avg_cook_time,
    COUNT(*) FILTER (WHERE dietary_labels && ARRAY['vegetarian']) as vegetarian_count,
    COUNT(*) FILTER (WHERE dietary_labels && ARRAY['vegan']) as vegan_count,
    COUNT(*) FILTER (WHERE complexity = 'simple') as simple_count,
    COUNT(*) FILTER (WHERE complexity = 'moderate') as moderate_count,
    COUNT(*) FILTER (WHERE complexity = 'complex') as complex_count
FROM recipes 
WHERE deleted_at IS NULL 
GROUP BY date_trunc('day', created_at)
ORDER BY date DESC;

-- Index for the materialized view
CREATE INDEX IF NOT EXISTS idx_recipe_search_stats_date 
ON recipe_search_stats(date DESC);

-- Refresh schedule comment (would be implemented via cron or application scheduler)
-- REFRESH MATERIALIZED VIEW CONCURRENTLY recipe_search_stats;

-- Query performance tracking table
CREATE TABLE IF NOT EXISTS query_performance_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query_type VARCHAR(50) NOT NULL,
    query_hash VARCHAR(64) NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    rows_returned INTEGER DEFAULT 0,
    cache_hit BOOLEAN DEFAULT false,
    user_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Index for query performance analysis
CREATE INDEX IF NOT EXISTS idx_query_perf_log_analysis 
ON query_performance_log(query_type, created_at DESC, execution_time_ms DESC);

CREATE INDEX IF NOT EXISTS idx_query_perf_log_slow 
ON query_performance_log(created_at DESC, execution_time_ms DESC) 
WHERE execution_time_ms > 100;

-- Comments for maintenance
COMMENT ON INDEX idx_recipes_search_composite IS 'Optimizes common recipe search patterns with multiple filters';
COMMENT ON INDEX idx_recipes_fulltext_enhanced IS 'Enhanced full-text search including cuisine type for better relevance';
COMMENT ON INDEX idx_meal_plans_user_active IS 'Optimizes user meal plan queries for active plans only';
COMMENT ON MATERIALIZED VIEW recipe_search_stats IS 'Aggregated statistics for recipe search analytics and reporting';

-- Analyze tables to update statistics after index creation
ANALYZE recipes;
ANALYZE meal_plans;
ANALYZE recipe_ratings;
ANALYZE shopping_lists;
ANALYZE user_preference_history;
ANALYZE recipe_performance_metrics;