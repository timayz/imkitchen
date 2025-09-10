-- Migration for user-generated tags and community-driven categorization
-- Story 3.2: Task 4 - User-Generated Tags & Categorization System

-- Create recipe_tags table for tracking user-generated tags on recipes
CREATE TABLE IF NOT EXISTS recipe_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    tag VARCHAR(30) NOT NULL,
    confidence DECIMAL(3,2) DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_recipe_tags_recipe_id (recipe_id),
    INDEX idx_recipe_tags_tag (tag),
    INDEX idx_recipe_tags_confidence (confidence DESC),
    INDEX idx_recipe_tags_created_at (created_at DESC),
    
    -- Unique constraint to prevent duplicate tags per recipe
    UNIQUE(recipe_id, tag)
);

-- Create community_tags table for community-driven recipe tagging
CREATE TABLE IF NOT EXISTS community_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES community_recipes(id) ON DELETE CASCADE,
    tag VARCHAR(30) NOT NULL,
    confidence DECIMAL(3,2) DEFAULT 0.5 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_community_tags_recipe_id (recipe_id),
    INDEX idx_community_tags_tag (tag),
    INDEX idx_community_tags_confidence (confidence DESC),
    INDEX idx_community_tags_created_at (created_at DESC),
    
    -- Unique constraint to prevent duplicate community tags per recipe
    UNIQUE(recipe_id, tag)
);

-- Create community_tag_votes table for voting on community tags
CREATE TABLE IF NOT EXISTS community_tag_votes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES community_recipes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tag VARCHAR(30) NOT NULL,
    vote_type VARCHAR(10) NOT NULL CHECK (vote_type IN ('upvote', 'downvote')),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_community_tag_votes_recipe_id (recipe_id),
    INDEX idx_community_tag_votes_user_id (user_id),
    INDEX idx_community_tag_votes_tag (tag),
    INDEX idx_community_tag_votes_vote_type (vote_type),
    
    -- Unique constraint to prevent multiple votes per user per tag per recipe
    UNIQUE(recipe_id, user_id, tag)
);

-- Create tag_categories table for categorizing tags
CREATE TABLE IF NOT EXISTS tag_categories (
    tag VARCHAR(30) PRIMARY KEY,
    category VARCHAR(50) NOT NULL DEFAULT 'general',
    description TEXT,
    is_system_category BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Index for category filtering
    INDEX idx_tag_categories_category (category)
);

-- Create tag_usage_stats table for tracking tag popularity and trends
CREATE TABLE IF NOT EXISTS tag_usage_stats (
    tag VARCHAR(30) PRIMARY KEY,
    usage_count INTEGER DEFAULT 0,
    weekly_usage INTEGER DEFAULT 0,
    monthly_usage INTEGER DEFAULT 0,
    last_used TIMESTAMP,
    trending_score DECIMAL(5,2) DEFAULT 0.0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_tag_usage_stats_usage_count (usage_count DESC),
    INDEX idx_tag_usage_stats_trending_score (trending_score DESC),
    INDEX idx_tag_usage_stats_last_used (last_used DESC)
);

-- Insert default tag categories
INSERT INTO tag_categories (tag, category, description, is_system_category) VALUES
    ('vegetarian', 'dietary', 'Vegetarian diet compatible', true),
    ('vegan', 'dietary', 'Vegan diet compatible', true),
    ('gluten-free', 'dietary', 'Gluten-free diet compatible', true),
    ('dairy-free', 'dietary', 'Dairy-free diet compatible', true),
    ('low-carb', 'dietary', 'Low carbohydrate content', true),
    ('keto', 'dietary', 'Ketogenic diet compatible', true),
    ('paleo', 'dietary', 'Paleo diet compatible', true),
    ('quick', 'time', 'Quick to prepare (under 30 minutes)', true),
    ('slow-cook', 'time', 'Slow cooking method', true),
    ('make-ahead', 'time', 'Can be prepared in advance', true),
    ('one-pot', 'cooking-method', 'One pot or pan cooking', true),
    ('no-cook', 'cooking-method', 'No cooking required', true),
    ('grilling', 'cooking-method', 'Grilling method', true),
    ('baking', 'cooking-method', 'Baking method', true),
    ('comfort-food', 'style', 'Comfort food style', true),
    ('healthy', 'style', 'Health-focused recipe', true),
    ('kid-friendly', 'audience', 'Suitable for children', true),
    ('party', 'audience', 'Good for parties and gatherings', true),
    ('budget', 'cost', 'Budget-friendly ingredients', true),
    ('fancy', 'cost', 'Premium or special occasion', true)
ON CONFLICT (tag) DO NOTHING;

-- Create function to update tag trending scores based on recent usage
CREATE OR REPLACE FUNCTION update_tag_trending_scores()
RETURNS void AS $$
BEGIN
    -- Reset weekly and monthly usage counts
    UPDATE tag_usage_stats 
    SET 
        weekly_usage = COALESCE((
            SELECT COUNT(*)
            FROM recipe_tags rt
            WHERE rt.tag = tag_usage_stats.tag
              AND rt.created_at > NOW() - INTERVAL '7 days'
        ), 0) + COALESCE((
            SELECT COUNT(*)
            FROM community_tags ct
            WHERE ct.tag = tag_usage_stats.tag
              AND ct.created_at > NOW() - INTERVAL '7 days'
        ), 0),
        monthly_usage = COALESCE((
            SELECT COUNT(*)
            FROM recipe_tags rt
            WHERE rt.tag = tag_usage_stats.tag
              AND rt.created_at > NOW() - INTERVAL '30 days'
        ), 0) + COALESCE((
            SELECT COUNT(*)
            FROM community_tags ct
            WHERE ct.tag = tag_usage_stats.tag
              AND ct.created_at > NOW() - INTERVAL '30 days'
        ), 0);

    -- Calculate trending score: weight recent usage heavily, factor in total usage
    UPDATE tag_usage_stats 
    SET trending_score = (
        (weekly_usage * 5.0) +  -- Heavy weight on weekly usage
        (monthly_usage * 2.0) + -- Medium weight on monthly usage
        (usage_count * 0.1) +   -- Light weight on total usage
        -- Boost score if recently used
        (CASE WHEN last_used > NOW() - INTERVAL '1 day' THEN 10.0 ELSE 0.0 END) +
        -- Decay factor for older tags
        (CASE WHEN last_used < NOW() - INTERVAL '30 days' THEN -5.0 ELSE 0.0 END)
    ),
    updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Create function to automatically update tag confidence based on votes
CREATE OR REPLACE FUNCTION update_community_tag_confidence()
RETURNS TRIGGER AS $$
BEGIN
    -- Update confidence for the affected tag
    UPDATE community_tags 
    SET 
        confidence = GREATEST(0.0, LEAST(1.0, 
            0.5 + (
                COALESCE((
                    SELECT SUM(CASE WHEN vote_type = 'upvote' THEN 1 WHEN vote_type = 'downvote' THEN -1 ELSE 0 END)
                    FROM community_tag_votes 
                    WHERE recipe_id = NEW.recipe_id AND tag = NEW.tag
                ), 0) * 0.05  -- Each vote adjusts confidence by ±0.05
            )
        )),
        updated_at = NOW()
    WHERE recipe_id = NEW.recipe_id AND tag = NEW.tag;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic community tag confidence updates
DROP TRIGGER IF EXISTS trigger_update_community_tag_confidence ON community_tag_votes;
CREATE TRIGGER trigger_update_community_tag_confidence
    AFTER INSERT OR UPDATE OR DELETE ON community_tag_votes
    FOR EACH ROW
    EXECUTE FUNCTION update_community_tag_confidence();

-- Create function to update tag usage statistics when tags are added
CREATE OR REPLACE FUNCTION update_tag_usage_on_insert()
RETURNS TRIGGER AS $$
BEGIN
    -- Update or insert tag usage statistics
    INSERT INTO tag_usage_stats (tag, usage_count, last_used, created_at, updated_at)
    VALUES (NEW.tag, 1, NOW(), NOW(), NOW())
    ON CONFLICT (tag)
    DO UPDATE SET 
        usage_count = tag_usage_stats.usage_count + 1,
        last_used = NOW(),
        updated_at = NOW();
        
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for automatic tag usage statistics updates
DROP TRIGGER IF EXISTS trigger_recipe_tags_usage_update ON recipe_tags;
CREATE TRIGGER trigger_recipe_tags_usage_update
    AFTER INSERT ON recipe_tags
    FOR EACH ROW
    EXECUTE FUNCTION update_tag_usage_on_insert();

DROP TRIGGER IF EXISTS trigger_community_tags_usage_update ON community_tags;
CREATE TRIGGER trigger_community_tags_usage_update
    AFTER INSERT ON community_tags
    FOR EACH ROW
    EXECUTE FUNCTION update_tag_usage_on_insert();

-- Create function to clean up orphaned community tags (tags with no votes and low confidence)
CREATE OR REPLACE FUNCTION cleanup_low_confidence_tags()
RETURNS void AS $$
BEGIN
    -- Remove community tags with very low confidence and no recent votes
    DELETE FROM community_tags 
    WHERE confidence < 0.2 
      AND created_at < NOW() - INTERVAL '7 days'
      AND NOT EXISTS (
          SELECT 1 FROM community_tag_votes 
          WHERE community_tag_votes.recipe_id = community_tags.recipe_id 
            AND community_tag_votes.tag = community_tags.tag
            AND community_tag_votes.created_at > NOW() - INTERVAL '7 days'
      );
END;
$$ LANGUAGE plpgsql;

-- Add composite indexes for complex queries
CREATE INDEX IF NOT EXISTS idx_recipe_tags_tag_confidence ON recipe_tags (tag, confidence DESC);
CREATE INDEX IF NOT EXISTS idx_community_tags_tag_confidence ON community_tags (tag, confidence DESC);
CREATE INDEX IF NOT EXISTS idx_community_tag_votes_recipe_tag ON community_tag_votes (recipe_id, tag);

-- Create index for full-text search on tags
CREATE INDEX IF NOT EXISTS idx_recipe_tags_tag_search ON recipe_tags USING GIN (to_tsvector('english', tag));
CREATE INDEX IF NOT EXISTS idx_community_tags_tag_search ON community_tags USING GIN (to_tsvector('english', tag));

-- Comments for documentation
COMMENT ON TABLE recipe_tags IS 'User-generated tags for personal recipes';
COMMENT ON TABLE community_tags IS 'Community-driven tags for shared recipes with voting system';
COMMENT ON TABLE community_tag_votes IS 'Votes on community tags for quality control';
COMMENT ON TABLE tag_categories IS 'Categorization system for organizing tags';
COMMENT ON TABLE tag_usage_stats IS 'Statistics and trending data for tag popularity';

COMMENT ON COLUMN recipe_tags.confidence IS 'Confidence score (0.0-1.0) for tag relevance';
COMMENT ON COLUMN community_tags.confidence IS 'Community-validated confidence score based on votes';
COMMENT ON COLUMN tag_usage_stats.trending_score IS 'Calculated trending score based on recent usage patterns';

-- Grant permissions (adjust as needed for your setup)
GRANT SELECT, INSERT, UPDATE, DELETE ON recipe_tags TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON community_tags TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON community_tag_votes TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON tag_categories TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON tag_usage_stats TO imkitchen_app;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO imkitchen_app;