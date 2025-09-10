-- Migration for recipe imports and community recipe enhancements
-- Story 3.2: Community Recipe Discovery & Import

-- Create recipe_imports table for tracking imported community recipes
CREATE TABLE IF NOT EXISTS recipe_imports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    personal_recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    community_recipe_id UUID NOT NULL,
    imported_at TIMESTAMP NOT NULL DEFAULT NOW(),
    preserve_attribution BOOLEAN NOT NULL DEFAULT TRUE,
    original_contributor TEXT,
    import_date TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_recipe_imports_user_id (user_id),
    INDEX idx_recipe_imports_community_recipe_id (community_recipe_id),
    INDEX idx_recipe_imports_imported_at (imported_at),
    
    -- Unique constraint to prevent duplicate imports
    UNIQUE(user_id, community_recipe_id)
);

-- Create community_recipes table for community recipe data
CREATE TABLE IF NOT EXISTS community_recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    image_url TEXT,
    prep_time INTEGER NOT NULL,
    cook_time INTEGER NOT NULL,
    total_time INTEGER NOT NULL,
    meal_type TEXT[] NOT NULL,
    complexity VARCHAR(20) NOT NULL CHECK (complexity IN ('simple', 'moderate', 'complex')),
    cuisine_type VARCHAR(100),
    servings INTEGER NOT NULL,
    ingredients JSONB NOT NULL,
    instructions JSONB NOT NULL,
    dietary_labels TEXT[],
    
    -- Community-specific fields
    contributor_id UUID,
    contributor_name VARCHAR(255),
    average_rating DECIMAL(3,2) DEFAULT 0.0,
    total_ratings INTEGER DEFAULT 0,
    import_count INTEGER DEFAULT 0,
    user_tags TEXT[],
    trending_score DECIMAL(5,2) DEFAULT 0.0,
    is_popular BOOLEAN DEFAULT FALSE,
    is_trending BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_community_recipes_title (title),
    INDEX idx_community_recipes_meal_type USING GIN (meal_type),
    INDEX idx_community_recipes_complexity (complexity),
    INDEX idx_community_recipes_cuisine_type (cuisine_type),
    INDEX idx_community_recipes_average_rating (average_rating DESC),
    INDEX idx_community_recipes_import_count (import_count DESC),
    INDEX idx_community_recipes_trending_score (trending_score DESC),
    INDEX idx_community_recipes_created_at (created_at DESC),
    INDEX idx_community_recipes_user_tags USING GIN (user_tags),
    INDEX idx_community_recipes_dietary_labels USING GIN (dietary_labels),
    
    -- Full-text search index
    INDEX idx_community_recipes_search USING GIN (to_tsvector('english', title || ' ' || COALESCE(description, '')))
);

-- Add user_tags and trending_score columns to existing recipes table if they don't exist
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'user_tags'
    ) THEN
        ALTER TABLE recipes ADD COLUMN user_tags TEXT[];
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'trending_score'
    ) THEN
        ALTER TABLE recipes ADD COLUMN trending_score DECIMAL(5,2) DEFAULT 0.0;
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'import_count'
    ) THEN
        ALTER TABLE recipes ADD COLUMN import_count INTEGER DEFAULT 0;
    END IF;
END $$;

-- Create indexes for the new recipe columns
CREATE INDEX IF NOT EXISTS idx_recipes_user_tags ON recipes USING GIN (user_tags);
CREATE INDEX IF NOT EXISTS idx_recipes_trending_score ON recipes (trending_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_import_count ON recipes (import_count DESC);

-- Create function to update community recipe trending scores
CREATE OR REPLACE FUNCTION update_trending_scores()
RETURNS void AS $$
BEGIN
    -- Update trending scores based on recent activity (imports, ratings, etc.)
    -- This is a simplified algorithm - in practice, you'd want more sophisticated scoring
    UPDATE community_recipes 
    SET trending_score = (
        -- Weight recent imports more heavily
        (import_count * 0.4) + 
        (average_rating * total_ratings * 0.3) + 
        (EXTRACT(EPOCH FROM (NOW() - created_at)) / 86400 * -0.1) -- Decay over time
    ),
    is_trending = (
        -- Mark as trending if score is above threshold and has recent activity
        trending_score > 5.0 AND 
        created_at > NOW() - INTERVAL '7 days'
    ),
    updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Create function to automatically update total_time when prep_time or cook_time changes
CREATE OR REPLACE FUNCTION update_total_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.total_time := NEW.prep_time + NEW.cook_time;
    NEW.updated_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for automatic total_time calculation
DROP TRIGGER IF EXISTS trigger_community_recipes_total_time ON community_recipes;
CREATE TRIGGER trigger_community_recipes_total_time
    BEFORE INSERT OR UPDATE OF prep_time, cook_time ON community_recipes
    FOR EACH ROW
    EXECUTE FUNCTION update_total_time();

-- Create function to update recipe ratings aggregates
CREATE OR REPLACE FUNCTION update_recipe_rating_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    -- Update community recipe rating aggregates when ratings change
    -- This assumes you have a recipe_ratings table
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        UPDATE community_recipes
        SET 
            average_rating = COALESCE((
                SELECT AVG(overall_rating)
                FROM recipe_ratings rr
                WHERE rr.recipe_id = NEW.recipe_id
                  AND rr.moderation_status = 'approved'
            ), 0.0),
            total_ratings = COALESCE((
                SELECT COUNT(*)
                FROM recipe_ratings rr
                WHERE rr.recipe_id = NEW.recipe_id
                  AND rr.moderation_status = 'approved'
            ), 0),
            updated_at = NOW()
        WHERE id = NEW.recipe_id;
        
        RETURN NEW;
    END IF;
    
    IF TG_OP = 'DELETE' THEN
        UPDATE community_recipes
        SET 
            average_rating = COALESCE((
                SELECT AVG(overall_rating)
                FROM recipe_ratings rr
                WHERE rr.recipe_id = OLD.recipe_id
                  AND rr.moderation_status = 'approved'
            ), 0.0),
            total_ratings = COALESCE((
                SELECT COUNT(*)
                FROM recipe_ratings rr
                WHERE rr.recipe_id = OLD.recipe_id
                  AND rr.moderation_status = 'approved'
            ), 0),
            updated_at = NOW()
        WHERE id = OLD.recipe_id;
        
        RETURN OLD;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE recipe_imports IS 'Tracks when users import community recipes to their personal collections';
COMMENT ON TABLE community_recipes IS 'Community-shared recipes with rating and trending data';
COMMENT ON COLUMN community_recipes.trending_score IS 'Calculated score for trending algorithm based on recent activity';
COMMENT ON COLUMN community_recipes.user_tags IS 'User-generated tags for community categorization';
COMMENT ON COLUMN recipe_imports.preserve_attribution IS 'Whether to preserve original contributor attribution';

-- Grant permissions (adjust as needed for your setup)
GRANT SELECT, INSERT, UPDATE, DELETE ON recipe_imports TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON community_recipes TO imkitchen_app;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO imkitchen_app;