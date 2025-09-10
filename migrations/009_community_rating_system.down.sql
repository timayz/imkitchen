-- Rollback Community Rating System Migration

-- Drop views
DROP VIEW IF EXISTS user_rating_history;
DROP VIEW IF EXISTS community_recipes_ranked;

-- Drop functions
DROP FUNCTION IF EXISTS calculate_recipe_recommendation_score(DECIMAL, INTEGER, BOOLEAN);
DROP FUNCTION IF EXISTS update_recipe_rating_aggregates();

-- Drop trigger
DROP TRIGGER IF EXISTS trigger_update_recipe_ratings ON recipe_ratings;

-- Remove columns from recipe_ratings
ALTER TABLE recipe_ratings DROP COLUMN IF EXISTS review_text_length;
ALTER TABLE recipe_ratings DROP COLUMN IF EXISTS moderation_status;
ALTER TABLE recipe_ratings DROP COLUMN IF EXISTS flagged_reason;
ALTER TABLE recipe_ratings DROP CONSTRAINT IF EXISTS review_text_max_length;

-- Drop indexes
DROP INDEX IF EXISTS idx_recipe_ratings_moderation;

-- Remove columns from recipes
ALTER TABLE recipes DROP COLUMN IF EXISTS is_public;
ALTER TABLE recipes DROP COLUMN IF EXISTS is_community;
ALTER TABLE recipes DROP COLUMN IF EXISTS rating_distribution;

-- Drop indexes
DROP INDEX IF EXISTS idx_recipes_is_public;
DROP INDEX IF EXISTS idx_recipes_is_community;
DROP INDEX IF EXISTS idx_recipes_community_public;