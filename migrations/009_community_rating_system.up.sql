-- Community Rating System Migration
-- Adds community features, rating aggregation, and moderation capabilities

-- Extend recipes table for community features
ALTER TABLE recipes ADD COLUMN IF NOT EXISTS is_public BOOLEAN DEFAULT false;
ALTER TABLE recipes ADD COLUMN IF NOT EXISTS is_community BOOLEAN DEFAULT false;
ALTER TABLE recipes ADD COLUMN IF NOT EXISTS rating_distribution JSONB DEFAULT '{"1": 0, "2": 0, "3": 0, "4": 0, "5": 0}';

-- Create indexes for new recipe columns
CREATE INDEX IF NOT EXISTS idx_recipes_is_public ON recipes(is_public) WHERE is_public = true;
CREATE INDEX IF NOT EXISTS idx_recipes_is_community ON recipes(is_community) WHERE is_community = true;
CREATE INDEX IF NOT EXISTS idx_recipes_community_public ON recipes(is_community, is_public) WHERE is_community = true AND is_public = true;

-- Extend recipe_ratings for community features
ALTER TABLE recipe_ratings ADD COLUMN IF NOT EXISTS review_text_length INTEGER GENERATED ALWAYS AS (CASE WHEN review_text IS NOT NULL THEN length(review_text) ELSE 0 END) STORED;
ALTER TABLE recipe_ratings ADD COLUMN IF NOT EXISTS moderation_status VARCHAR(20) DEFAULT 'approved' CHECK (moderation_status IN ('pending', 'approved', 'rejected', 'flagged'));
ALTER TABLE recipe_ratings ADD COLUMN IF NOT EXISTS flagged_reason TEXT;

-- Add constraint for review text length (max 500 characters)
ALTER TABLE recipe_ratings ADD CONSTRAINT review_text_max_length CHECK (length(review_text) <= 500);

-- Create index for moderation status
CREATE INDEX IF NOT EXISTS idx_recipe_ratings_moderation ON recipe_ratings(moderation_status);

-- Function to update recipe rating aggregates
CREATE OR REPLACE FUNCTION update_recipe_rating_aggregates()
RETURNS TRIGGER AS $$
DECLARE
    recipe_uuid UUID;
    avg_rating DECIMAL(3,2);
    total_count INTEGER;
    rating_dist JSONB;
BEGIN
    -- Get the recipe ID from either NEW or OLD record
    recipe_uuid := COALESCE(NEW.recipe_id, OLD.recipe_id);
    
    -- Calculate new averages and counts (only approved ratings)
    SELECT 
        ROUND(AVG(overall_rating)::numeric, 2),
        COUNT(*),
        jsonb_build_object(
            '1', COUNT(*) FILTER (WHERE overall_rating = 1),
            '2', COUNT(*) FILTER (WHERE overall_rating = 2),
            '3', COUNT(*) FILTER (WHERE overall_rating = 3),
            '4', COUNT(*) FILTER (WHERE overall_rating = 4),
            '5', COUNT(*) FILTER (WHERE overall_rating = 5)
        )
    INTO avg_rating, total_count, rating_dist
    FROM recipe_ratings 
    WHERE recipe_id = recipe_uuid 
      AND moderation_status = 'approved'
      AND overall_rating IS NOT NULL;
    
    -- Update recipe aggregates
    UPDATE recipes 
    SET 
        average_rating = COALESCE(avg_rating, 0.0),
        total_ratings = COALESCE(total_count, 0),
        rating_distribution = COALESCE(rating_dist, '{"1": 0, "2": 0, "3": 0, "4": 0, "5": 0}'::jsonb),
        updated_at = NOW()
    WHERE id = recipe_uuid;
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic rating aggregation
DROP TRIGGER IF EXISTS trigger_update_recipe_ratings ON recipe_ratings;
CREATE TRIGGER trigger_update_recipe_ratings
    AFTER INSERT OR UPDATE OR DELETE ON recipe_ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_recipe_rating_aggregates();

-- Function for recipe recommendation scoring
CREATE OR REPLACE FUNCTION calculate_recipe_recommendation_score(
    avg_rating DECIMAL,
    total_ratings INTEGER,
    is_community BOOLEAN DEFAULT false
) RETURNS DECIMAL AS $$
DECLARE
    base_score DECIMAL := 0.0;
    rating_confidence DECIMAL;
    community_bonus DECIMAL := 0.0;
BEGIN
    -- Base score from average rating
    base_score := COALESCE(avg_rating, 0.0);
    
    -- Apply confidence adjustment based on number of ratings
    -- Uses Wilson score interval for better ranking
    IF total_ratings > 0 THEN
        rating_confidence := CASE 
            WHEN total_ratings >= 20 THEN 1.0
            WHEN total_ratings >= 10 THEN 0.8
            WHEN total_ratings >= 5 THEN 0.6
            WHEN total_ratings >= 3 THEN 0.4
            ELSE 0.2
        END;
        
        base_score := base_score * rating_confidence;
    ELSE
        base_score := 2.5; -- Neutral score for unrated recipes
    END IF;
    
    -- Community recipes get slight boost for discovery
    IF is_community THEN
        community_bonus := 0.1;
    END IF;
    
    RETURN ROUND((base_score + community_bonus)::numeric, 4);
END;
$$ LANGUAGE plpgsql;

-- Create view for community recipes with recommendation scores
CREATE OR REPLACE VIEW community_recipes_ranked AS
SELECT 
    r.*,
    calculate_recipe_recommendation_score(r.average_rating, r.total_ratings, r.is_community) as recommendation_score,
    CASE 
        WHEN r.total_ratings >= 3 THEN true 
        ELSE false 
    END as eligible_for_recommendations
FROM recipes r
WHERE r.is_community = true 
  AND r.is_public = true 
  AND r.deleted_at IS NULL
ORDER BY recommendation_score DESC, r.created_at DESC;

-- Create view for user rating history
CREATE OR REPLACE VIEW user_rating_history AS
SELECT 
    rr.id,
    rr.user_id,
    rr.recipe_id,
    r.title as recipe_title,
    r.image_url as recipe_image_url,
    rr.overall_rating,
    rr.review_text,
    rr.would_make_again,
    rr.moderation_status,
    rr.created_at,
    rr.updated_at
FROM recipe_ratings rr
JOIN recipes r ON rr.recipe_id = r.id
WHERE r.deleted_at IS NULL
ORDER BY rr.created_at DESC;

-- Add comments for documentation
COMMENT ON COLUMN recipes.is_public IS 'Whether recipe is visible to other users';
COMMENT ON COLUMN recipes.is_community IS 'Whether recipe is part of community database vs personal collection';
COMMENT ON COLUMN recipes.rating_distribution IS 'JSONB object with count of each star rating: {"1": 0, "2": 0, "3": 0, "4": 0, "5": 0}';
COMMENT ON COLUMN recipe_ratings.moderation_status IS 'Review moderation status: pending, approved, rejected, flagged';
COMMENT ON COLUMN recipe_ratings.flagged_reason IS 'Reason for flagging review (inappropriate content, spam, etc.)';
COMMENT ON FUNCTION update_recipe_rating_aggregates() IS 'Automatically updates recipe rating aggregates when ratings are added/modified';
COMMENT ON FUNCTION calculate_recipe_recommendation_score(DECIMAL, INTEGER, BOOLEAN) IS 'Calculates recommendation score based on ratings and community status';
COMMENT ON VIEW community_recipes_ranked IS 'Community recipes ordered by recommendation score';
COMMENT ON VIEW user_rating_history IS 'User rating history with recipe details';