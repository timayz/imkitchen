-- Rollback Performance Optimization Indices
-- Migration: 011_performance_optimization_indices.down.sql

-- Drop query performance tracking
DROP TABLE IF EXISTS query_performance_log;

-- Drop materialized view
DROP MATERIALIZED VIEW IF EXISTS recipe_search_stats;

-- Drop function-based indices
DROP INDEX CONCURRENTLY IF EXISTS idx_meal_plans_engagement;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_search_score;

-- Drop partial indices
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_quick;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_vegetarian;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_high_rated;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_user_own;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_active_only;

-- Drop recipe performance metrics indices
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_perf_usage;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_perf_scoring;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_perf_analytics;

-- Drop user preferences indices
DROP INDEX CONCURRENTLY IF EXISTS idx_user_pref_history_learning;

-- Drop shopping list indices
DROP INDEX CONCURRENTLY IF EXISTS idx_shopping_lists_items;
DROP INDEX CONCURRENTLY IF EXISTS idx_shopping_lists_meal_plan;
DROP INDEX CONCURRENTLY IF EXISTS idx_shopping_lists_user_active;

-- Drop recipe rating indices
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_ratings_trending;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_ratings_recipe_stats;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipe_ratings_user_recipe;

-- Drop meal plan indices
DROP INDEX CONCURRENTLY IF EXISTS idx_meal_plans_generation_perf;
DROP INDEX CONCURRENTLY IF EXISTS idx_meal_plans_meals_enhanced;
DROP INDEX CONCURRENTLY IF EXISTS idx_meal_plans_date_range;
DROP INDEX CONCURRENTLY IF EXISTS idx_meal_plans_user_active;

-- Drop recipe search indices
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_user_access;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_nutrition_enhanced;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_ingredients_enhanced;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_popularity;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_time_filters;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_fulltext_enhanced;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_meal_diet_composite;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_search_composite;