-- Migration 007 Rollback: Drop user preference tables
-- Generated: 2025-09-08
-- Description: Remove user recipe favorites and weekly patterns tables

-- Drop triggers first
DROP TRIGGER IF EXISTS update_user_recipe_favorites_updated_at ON user_recipe_favorites;
DROP TRIGGER IF EXISTS update_user_weekly_patterns_updated_at ON user_weekly_patterns;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_user_recipe_favorites_user_id;
DROP INDEX IF EXISTS idx_user_recipe_favorites_recipe_id;
DROP INDEX IF EXISTS idx_user_recipe_favorites_favorited_at;

DROP INDEX IF EXISTS idx_user_weekly_patterns_user_id;
DROP INDEX IF EXISTS idx_user_weekly_patterns_day_of_week;
DROP INDEX IF EXISTS idx_user_weekly_patterns_weekend;

-- Drop tables (order matters due to foreign keys)
DROP TABLE IF EXISTS user_weekly_patterns;
DROP TABLE IF EXISTS user_recipe_favorites;

-- Remove preference columns from users table if they were added
ALTER TABLE users DROP COLUMN IF EXISTS max_cook_time;
ALTER TABLE users DROP COLUMN IF EXISTS preferred_meal_complexity;