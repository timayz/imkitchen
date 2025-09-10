-- Recipe Search Performance Enhancement Indices
-- Migration: 013_recipe_search_performance_indices.up.sql
-- Story: 3.4 - Database Query Performance Optimization

-- Enhanced compound indices for common search patterns
-- Targets sub-200ms search performance for 10,000+ recipes

-- Optimized index for cuisine + dietary restrictions + prep time queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_cuisine_diet_preptime 
ON recipes(cuisine_type, prep_time) 
INCLUDE (average_rating, total_time, complexity)
WHERE deleted_at IS NULL AND prep_time <= 120;

-- Enhanced full-text search with ranking optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_fulltext_ranked 
ON recipes USING GIN(
    to_tsvector('english', 
        setweight(to_tsvector('english', title), 'A') ||
        setweight(to_tsvector('english', COALESCE(description, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(cuisine_type, '')), 'C')
    )
) 
WHERE deleted_at IS NULL;

-- Multi-column index for combined filters (cuisine + time + dietary)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_combined_filters 
ON recipes USING GIN(dietary_labels) 
INCLUDE (cuisine_type, prep_time, cook_time, average_rating, complexity)
WHERE deleted_at IS NULL;

-- Index for pagination optimization with consistent ordering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_pagination_optimal 
ON recipes(created_at DESC, id)
WHERE deleted_at IS NULL;

-- Specialized index for popular/trending recipes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_trending_optimized 
ON recipes(average_rating DESC NULLS LAST, total_ratings DESC, updated_at DESC)
WHERE deleted_at IS NULL 
AND average_rating >= 3.5 
AND total_ratings >= 5;

-- Index for quick count queries (avoid full table scans)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_count_optimization 
ON recipes(deleted_at)
WHERE deleted_at IS NULL;

-- Partial indices for common dietary restrictions
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_vegetarian_fast 
ON recipes(prep_time, cook_time, average_rating DESC)
WHERE deleted_at IS NULL 
AND dietary_labels @> '["vegetarian"]';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_vegan_fast 
ON recipes(prep_time, cook_time, average_rating DESC)
WHERE deleted_at IS NULL 
AND dietary_labels @> '["vegan"]';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_gluten_free_fast 
ON recipes(prep_time, cook_time, average_rating DESC)
WHERE deleted_at IS NULL 
AND dietary_labels @> '["gluten-free"]';

-- Performance monitoring indices
-- Index for query performance tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_search_performance 
ON recipes(updated_at DESC)
WHERE deleted_at IS NULL
AND updated_at >= CURRENT_DATE - INTERVAL '30 days';

-- Add statistics targets for better query planning
ALTER TABLE recipes ALTER COLUMN cuisine_type SET STATISTICS 1000;
ALTER TABLE recipes ALTER COLUMN dietary_labels SET STATISTICS 1000;
ALTER TABLE recipes ALTER COLUMN prep_time SET STATISTICS 500;
ALTER TABLE recipes ALTER COLUMN average_rating SET STATISTICS 500;

-- Create expression index for common search patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_recipes_prep_total_time 
ON recipes((prep_time + cook_time), complexity, average_rating DESC)
WHERE deleted_at IS NULL;

-- Materialized view for recipe search statistics (for caching)
CREATE MATERIALIZED VIEW IF NOT EXISTS recipe_search_stats AS
SELECT 
    cuisine_type,
    COUNT(*) as recipe_count,
    AVG(prep_time) as avg_prep_time,
    AVG(average_rating) as avg_rating,
    MAX(updated_at) as last_updated
FROM recipes 
WHERE deleted_at IS NULL
GROUP BY cuisine_type
WITH DATA;

-- Index for the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_recipe_search_stats_cuisine 
ON recipe_search_stats(cuisine_type);

-- Refresh the materialized view
REFRESH MATERIALIZED VIEW recipe_search_stats;