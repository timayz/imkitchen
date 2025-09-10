-- Rollback Recipe Search Performance Enhancement Indices
-- Migration: 013_recipe_search_performance_indices.down.sql

-- Drop materialized view and its index
DROP MATERIALIZED VIEW IF EXISTS recipe_search_stats CASCADE;

-- Reset statistics targets
ALTER TABLE recipes ALTER COLUMN cuisine_type SET STATISTICS DEFAULT;
ALTER TABLE recipes ALTER COLUMN dietary_labels SET STATISTICS DEFAULT;
ALTER TABLE recipes ALTER COLUMN prep_time SET STATISTICS DEFAULT;
ALTER TABLE recipes ALTER COLUMN average_rating SET STATISTICS DEFAULT;

-- Drop performance optimization indices
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_prep_total_time;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_search_performance;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_gluten_free_fast;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_vegan_fast;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_vegetarian_fast;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_count_optimization;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_trending_optimized;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_pagination_optimal;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_combined_filters;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_fulltext_ranked;
DROP INDEX CONCURRENTLY IF EXISTS idx_recipes_cuisine_diet_preptime;