-- Rollback migration for enhanced recipe attribution and community metrics
-- Story 3.2: Task 5 - Enhanced Recipe Attribution & Community Metrics

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_track_recipe_engagement ON recipe_engagement_metrics;
DROP TRIGGER IF EXISTS trigger_check_achievements ON recipe_engagement_metrics;

-- Drop functions
DROP FUNCTION IF EXISTS update_recipe_popularity_metrics();
DROP FUNCTION IF EXISTS track_recipe_engagement();
DROP FUNCTION IF EXISTS check_and_award_achievements();

-- Remove columns from recipes table
ALTER TABLE recipes DROP COLUMN IF EXISTS total_views;
ALTER TABLE recipes DROP COLUMN IF EXISTS total_saves;
ALTER TABLE recipes DROP COLUMN IF EXISTS social_shares;

-- Drop indexes
DROP INDEX IF EXISTS idx_recipes_total_views;
DROP INDEX IF EXISTS idx_recipes_total_saves;
DROP INDEX IF EXISTS idx_recipes_social_shares;

-- Drop tables (in reverse order of creation to handle foreign key constraints)
DROP TABLE IF EXISTS feature_highlights;
DROP TABLE IF EXISTS attribution_reports;
DROP TABLE IF EXISTS attribution_preferences;
DROP TABLE IF EXISTS recipe_popularity_metrics;
DROP TABLE IF EXISTS recipe_engagement_metrics;
DROP TABLE IF EXISTS contributor_achievements;
DROP TABLE IF EXISTS contributor_badges;
DROP TABLE IF EXISTS contributor_profiles;
DROP TABLE IF EXISTS recipe_attributions;