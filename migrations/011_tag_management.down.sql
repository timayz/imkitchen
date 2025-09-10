-- Rollback migration for user-generated tags and community-driven categorization
-- Story 3.2: Task 4 - User-Generated Tags & Categorization System

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_community_tag_confidence ON community_tag_votes;
DROP TRIGGER IF EXISTS trigger_recipe_tags_usage_update ON recipe_tags;
DROP TRIGGER IF EXISTS trigger_community_tags_usage_update ON community_tags;

-- Drop functions
DROP FUNCTION IF EXISTS update_tag_trending_scores();
DROP FUNCTION IF EXISTS update_community_tag_confidence();
DROP FUNCTION IF EXISTS update_tag_usage_on_insert();
DROP FUNCTION IF EXISTS cleanup_low_confidence_tags();

-- Drop indexes
DROP INDEX IF EXISTS idx_recipe_tags_tag_confidence;
DROP INDEX IF EXISTS idx_community_tags_tag_confidence;
DROP INDEX IF EXISTS idx_community_tag_votes_recipe_tag;
DROP INDEX IF EXISTS idx_recipe_tags_tag_search;
DROP INDEX IF EXISTS idx_community_tags_tag_search;

-- Drop tables (in reverse order of creation to handle foreign key constraints)
DROP TABLE IF EXISTS tag_usage_stats;
DROP TABLE IF EXISTS tag_categories;
DROP TABLE IF EXISTS community_tag_votes;
DROP TABLE IF EXISTS community_tags;
DROP TABLE IF EXISTS recipe_tags;