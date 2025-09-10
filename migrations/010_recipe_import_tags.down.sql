-- Rollback migration for recipe imports and community recipe enhancements
-- Story 3.2: Community Recipe Discovery & Import

-- Drop triggers first
DROP TRIGGER IF EXISTS trigger_community_recipes_total_time ON community_recipes;
DROP TRIGGER IF EXISTS trigger_recipe_rating_aggregates ON recipe_ratings;

-- Drop functions
DROP FUNCTION IF EXISTS update_total_time();
DROP FUNCTION IF EXISTS update_recipe_rating_aggregates();
DROP FUNCTION IF EXISTS update_trending_scores();

-- Drop indexes on recipes table
DROP INDEX IF EXISTS idx_recipes_user_tags;
DROP INDEX IF EXISTS idx_recipes_trending_score;
DROP INDEX IF EXISTS idx_recipes_import_count;

-- Remove added columns from recipes table
ALTER TABLE recipes DROP COLUMN IF EXISTS user_tags;
ALTER TABLE recipes DROP COLUMN IF EXISTS trending_score;
ALTER TABLE recipes DROP COLUMN IF EXISTS import_count;

-- Drop community recipes table
DROP TABLE IF EXISTS community_recipes CASCADE;

-- Drop recipe imports table
DROP TABLE IF EXISTS recipe_imports CASCADE;