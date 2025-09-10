-- Migration for enhanced recipe attribution and community metrics
-- Story 3.2: Task 5 - Enhanced Recipe Attribution & Community Metrics

-- Create recipe_attributions table for tracking recipe import attribution
CREATE TABLE IF NOT EXISTS recipe_attributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    original_recipe_id UUID NOT NULL,
    original_contributor_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    import_date TIMESTAMP NOT NULL DEFAULT NOW(),
    preserve_attribution BOOLEAN NOT NULL DEFAULT TRUE,
    customizations TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_recipe_attributions_recipe_id (recipe_id),
    INDEX idx_recipe_attributions_original_recipe_id (original_recipe_id),
    INDEX idx_recipe_attributions_original_contributor_id (original_contributor_id),
    INDEX idx_recipe_attributions_import_date (import_date DESC),
    
    -- Unique constraint to prevent duplicate attributions
    UNIQUE(recipe_id)
);

-- Create contributor_profiles table for enhanced contributor information
CREATE TABLE IF NOT EXISTS contributor_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    bio TEXT,
    location VARCHAR(255),
    website VARCHAR(255),
    social_links JSONB,
    specialties TEXT[],
    cooking_experience VARCHAR(50),
    preferred_cuisines TEXT[],
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for searching
    INDEX idx_contributor_profiles_location (location),
    INDEX idx_contributor_profiles_specialties USING GIN (specialties),
    INDEX idx_contributor_profiles_cuisines USING GIN (preferred_cuisines)
);

-- Create contributor_badges table for recognition system
CREATE TABLE IF NOT EXISTS contributor_badges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_type VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    emoji VARCHAR(10),
    earned_at TIMESTAMP NOT NULL DEFAULT NOW(),
    criteria_met JSONB,
    
    -- Indexes for performance
    INDEX idx_contributor_badges_user_id (user_id),
    INDEX idx_contributor_badges_badge_type (badge_type),
    INDEX idx_contributor_badges_earned_at (earned_at DESC),
    
    -- Prevent duplicate badges of same type for same user
    UNIQUE(user_id, badge_type)
);

-- Create contributor_achievements table for achievement system
CREATE TABLE IF NOT EXISTS contributor_achievements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    achievement_type VARCHAR(50) NOT NULL,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    emoji VARCHAR(10),
    category VARCHAR(50) NOT NULL,
    points INTEGER DEFAULT 0,
    earned_at TIMESTAMP NOT NULL DEFAULT NOW(),
    metadata JSONB,
    
    -- Indexes for performance
    INDEX idx_contributor_achievements_user_id (user_id),
    INDEX idx_contributor_achievements_achievement_type (achievement_type),
    INDEX idx_contributor_achievements_category (category),
    INDEX idx_contributor_achievements_points (points DESC),
    INDEX idx_contributor_achievements_earned_at (earned_at DESC),
    
    -- Allow multiple achievements of same type (e.g., multiple "popular recipe" achievements)
    INDEX idx_contributor_achievements_user_type (user_id, achievement_type)
);

-- Create recipe_engagement_metrics table for detailed analytics
CREATE TABLE IF NOT EXISTS recipe_engagement_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action_type VARCHAR(50) NOT NULL, -- 'view', 'save', 'share', 'import', 'rate', 'comment'
    action_date TIMESTAMP NOT NULL DEFAULT NOW(),
    session_id VARCHAR(255),
    referrer_source VARCHAR(255),
    user_agent TEXT,
    ip_address INET,
    country_code VARCHAR(3),
    city VARCHAR(100),
    metadata JSONB,
    
    -- Indexes for analytics queries
    INDEX idx_recipe_engagement_recipe_id (recipe_id),
    INDEX idx_recipe_engagement_user_id (user_id),
    INDEX idx_recipe_engagement_action_type (action_type),
    INDEX idx_recipe_engagement_action_date (action_date DESC),
    INDEX idx_recipe_engagement_country (country_code),
    INDEX idx_recipe_engagement_session (session_id),
    
    -- Composite indexes for common queries
    INDEX idx_recipe_engagement_recipe_action_date (recipe_id, action_type, action_date DESC),
    INDEX idx_recipe_engagement_user_action_date (user_id, action_type, action_date DESC)
);

-- Create recipe_popularity_metrics table for aggregated popularity data
CREATE TABLE IF NOT EXISTS recipe_popularity_metrics (
    recipe_id UUID PRIMARY KEY REFERENCES recipes(id) ON DELETE CASCADE,
    daily_views INTEGER DEFAULT 0,
    weekly_views INTEGER DEFAULT 0,
    monthly_views INTEGER DEFAULT 0,
    total_views INTEGER DEFAULT 0,
    daily_imports INTEGER DEFAULT 0,
    weekly_imports INTEGER DEFAULT 0,
    monthly_imports INTEGER DEFAULT 0,
    total_imports INTEGER DEFAULT 0,
    daily_saves INTEGER DEFAULT 0,
    weekly_saves INTEGER DEFAULT 0,
    monthly_saves INTEGER DEFAULT 0,
    total_saves INTEGER DEFAULT 0,
    social_shares INTEGER DEFAULT 0,
    trending_score DECIMAL(5,2) DEFAULT 0.0,
    virality_index DECIMAL(5,2) DEFAULT 0.0,
    weekly_rank INTEGER,
    monthly_rank INTEGER,
    last_calculated TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Indexes for ranking and trending queries
    INDEX idx_recipe_popularity_trending_score (trending_score DESC),
    INDEX idx_recipe_popularity_virality_index (virality_index DESC),
    INDEX idx_recipe_popularity_weekly_rank (weekly_rank ASC),
    INDEX idx_recipe_popularity_monthly_rank (monthly_rank ASC),
    INDEX idx_recipe_popularity_total_views (total_views DESC),
    INDEX idx_recipe_popularity_total_imports (total_imports DESC)
);

-- Create attribution_preferences table for user attribution settings
CREATE TABLE IF NOT EXISTS attribution_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    preserve_attribution BOOLEAN DEFAULT TRUE,
    allow_derivatives BOOLEAN DEFAULT TRUE,
    require_notification BOOLEAN DEFAULT FALSE,
    attribution_text TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create attribution_reports table for tracking attribution issues
CREATE TABLE IF NOT EXISTS attribution_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    issue_type VARCHAR(50) NOT NULL CHECK (issue_type IN ('missing_attribution', 'incorrect_attribution', 'unauthorized_use')),
    description TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'reviewing', 'resolved', 'dismissed')),
    resolution_notes TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP,
    resolved_by UUID REFERENCES users(id),
    
    -- Indexes for management queries
    INDEX idx_attribution_reports_reporter_id (reporter_id),
    INDEX idx_attribution_reports_recipe_id (recipe_id),
    INDEX idx_attribution_reports_status (status),
    INDEX idx_attribution_reports_created_at (created_at DESC),
    INDEX idx_attribution_reports_issue_type (issue_type)
);

-- Create feature_highlights table for showcasing popular recipes
CREATE TABLE IF NOT EXISTS feature_highlights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    feature_type VARCHAR(50) NOT NULL, -- 'trending_weekly', 'editor_pick', 'community_favorite'
    title VARCHAR(255) NOT NULL,
    description TEXT,
    featured_date TIMESTAMP NOT NULL DEFAULT NOW(),
    end_date TIMESTAMP,
    priority INTEGER DEFAULT 0,
    created_by UUID REFERENCES users(id),
    
    -- Indexes for feature queries
    INDEX idx_feature_highlights_recipe_id (recipe_id),
    INDEX idx_feature_highlights_feature_type (feature_type),
    INDEX idx_feature_highlights_featured_date (featured_date DESC),
    INDEX idx_feature_highlights_priority (priority DESC),
    INDEX idx_feature_highlights_active (featured_date, end_date)
);

-- Add missing columns to existing recipes table if they don't exist
DO $$ 
BEGIN
    -- Add columns for community metrics if they don't exist
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'total_views'
    ) THEN
        ALTER TABLE recipes ADD COLUMN total_views INTEGER DEFAULT 0;
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'total_saves'
    ) THEN
        ALTER TABLE recipes ADD COLUMN total_saves INTEGER DEFAULT 0;
    END IF;
    
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'recipes' AND column_name = 'social_shares'
    ) THEN
        ALTER TABLE recipes ADD COLUMN social_shares INTEGER DEFAULT 0;
    END IF;
END $$;

-- Create indexes for the new recipe columns
CREATE INDEX IF NOT EXISTS idx_recipes_total_views ON recipes (total_views DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_total_saves ON recipes (total_saves DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_social_shares ON recipes (social_shares DESC);

-- Create function to update recipe popularity metrics
CREATE OR REPLACE FUNCTION update_recipe_popularity_metrics()
RETURNS void AS $$
BEGIN
    -- Update aggregated metrics from engagement data
    INSERT INTO recipe_popularity_metrics (
        recipe_id,
        daily_views,
        weekly_views,
        monthly_views,
        total_views,
        daily_imports,
        weekly_imports,
        monthly_imports,
        total_imports,
        daily_saves,
        weekly_saves,
        monthly_saves,
        total_saves,
        social_shares,
        last_calculated
    )
    SELECT 
        r.id as recipe_id,
        COALESCE(daily_metrics.views, 0) as daily_views,
        COALESCE(weekly_metrics.views, 0) as weekly_views,
        COALESCE(monthly_metrics.views, 0) as monthly_views,
        COALESCE(total_metrics.views, 0) as total_views,
        COALESCE(daily_metrics.imports, 0) as daily_imports,
        COALESCE(weekly_metrics.imports, 0) as weekly_imports,
        COALESCE(monthly_metrics.imports, 0) as monthly_imports,
        COALESCE(total_metrics.imports, 0) as total_imports,
        COALESCE(daily_metrics.saves, 0) as daily_saves,
        COALESCE(weekly_metrics.saves, 0) as weekly_saves,
        COALESCE(monthly_metrics.saves, 0) as monthly_saves,
        COALESCE(total_metrics.saves, 0) as total_saves,
        COALESCE(total_metrics.shares, 0) as social_shares,
        NOW() as last_calculated
    FROM recipes r
    LEFT JOIN (
        -- Daily metrics
        SELECT 
            recipe_id,
            COUNT(CASE WHEN action_type = 'view' THEN 1 END) as views,
            COUNT(CASE WHEN action_type = 'import' THEN 1 END) as imports,
            COUNT(CASE WHEN action_type = 'save' THEN 1 END) as saves
        FROM recipe_engagement_metrics 
        WHERE action_date >= NOW() - INTERVAL '1 day'
        GROUP BY recipe_id
    ) daily_metrics ON r.id = daily_metrics.recipe_id
    LEFT JOIN (
        -- Weekly metrics
        SELECT 
            recipe_id,
            COUNT(CASE WHEN action_type = 'view' THEN 1 END) as views,
            COUNT(CASE WHEN action_type = 'import' THEN 1 END) as imports,
            COUNT(CASE WHEN action_type = 'save' THEN 1 END) as saves
        FROM recipe_engagement_metrics 
        WHERE action_date >= NOW() - INTERVAL '7 days'
        GROUP BY recipe_id
    ) weekly_metrics ON r.id = weekly_metrics.recipe_id
    LEFT JOIN (
        -- Monthly metrics
        SELECT 
            recipe_id,
            COUNT(CASE WHEN action_type = 'view' THEN 1 END) as views,
            COUNT(CASE WHEN action_type = 'import' THEN 1 END) as imports,
            COUNT(CASE WHEN action_type = 'save' THEN 1 END) as saves
        FROM recipe_engagement_metrics 
        WHERE action_date >= NOW() - INTERVAL '30 days'
        GROUP BY recipe_id
    ) monthly_metrics ON r.id = monthly_metrics.recipe_id
    LEFT JOIN (
        -- Total metrics
        SELECT 
            recipe_id,
            COUNT(CASE WHEN action_type = 'view' THEN 1 END) as views,
            COUNT(CASE WHEN action_type = 'import' THEN 1 END) as imports,
            COUNT(CASE WHEN action_type = 'save' THEN 1 END) as saves,
            COUNT(CASE WHEN action_type = 'share' THEN 1 END) as shares
        FROM recipe_engagement_metrics 
        GROUP BY recipe_id
    ) total_metrics ON r.id = total_metrics.recipe_id
    ON CONFLICT (recipe_id) 
    DO UPDATE SET
        daily_views = EXCLUDED.daily_views,
        weekly_views = EXCLUDED.weekly_views,
        monthly_views = EXCLUDED.monthly_views,
        total_views = EXCLUDED.total_views,
        daily_imports = EXCLUDED.daily_imports,
        weekly_imports = EXCLUDED.weekly_imports,
        monthly_imports = EXCLUDED.monthly_imports,
        total_imports = EXCLUDED.total_imports,
        daily_saves = EXCLUDED.daily_saves,
        weekly_saves = EXCLUDED.weekly_saves,
        monthly_saves = EXCLUDED.monthly_saves,
        total_saves = EXCLUDED.total_saves,
        social_shares = EXCLUDED.social_shares,
        last_calculated = EXCLUDED.last_calculated;

    -- Update trending scores and rankings
    UPDATE recipe_popularity_metrics 
    SET 
        trending_score = GREATEST(0, 
            (weekly_views * 0.3) + 
            (weekly_imports * 0.4) + 
            (weekly_saves * 0.2) + 
            (social_shares * 0.1) -
            -- Decay factor for older recipes
            (EXTRACT(EPOCH FROM (NOW() - (SELECT created_at FROM recipes WHERE id = recipe_id))) / 86400 * 0.01)
        ),
        virality_index = CASE 
            WHEN total_views > 0 THEN 
                LEAST(10.0, (social_shares::DECIMAL / total_views) * 100)
            ELSE 0.0 
        END;

    -- Update weekly rankings
    WITH weekly_rankings AS (
        SELECT 
            recipe_id,
            ROW_NUMBER() OVER (ORDER BY trending_score DESC) as rank
        FROM recipe_popularity_metrics
        WHERE trending_score > 0
    )
    UPDATE recipe_popularity_metrics rpm
    SET weekly_rank = wr.rank
    FROM weekly_rankings wr
    WHERE rpm.recipe_id = wr.recipe_id;

    -- Update monthly rankings based on monthly metrics
    WITH monthly_rankings AS (
        SELECT 
            recipe_id,
            ROW_NUMBER() OVER (ORDER BY monthly_views + monthly_imports + monthly_saves DESC) as rank
        FROM recipe_popularity_metrics
        WHERE monthly_views + monthly_imports + monthly_saves > 0
    )
    UPDATE recipe_popularity_metrics rpm
    SET monthly_rank = mr.rank
    FROM monthly_rankings mr
    WHERE rpm.recipe_id = mr.recipe_id;
END;
$$ LANGUAGE plpgsql;

-- Create function to track recipe engagement
CREATE OR REPLACE FUNCTION track_recipe_engagement()
RETURNS TRIGGER AS $$
BEGIN
    -- Update the aggregated counters in recipes table
    IF NEW.action_type = 'view' THEN
        UPDATE recipes SET total_views = total_views + 1 WHERE id = NEW.recipe_id;
    ELSIF NEW.action_type = 'save' THEN
        UPDATE recipes SET total_saves = total_saves + 1 WHERE id = NEW.recipe_id;
    ELSIF NEW.action_type = 'share' THEN
        UPDATE recipes SET social_shares = social_shares + 1 WHERE id = NEW.recipe_id;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for recipe engagement tracking
DROP TRIGGER IF EXISTS trigger_track_recipe_engagement ON recipe_engagement_metrics;
CREATE TRIGGER trigger_track_recipe_engagement
    AFTER INSERT ON recipe_engagement_metrics
    FOR EACH ROW
    EXECUTE FUNCTION track_recipe_engagement();

-- Create function to automatically award achievements
CREATE OR REPLACE FUNCTION check_and_award_achievements()
RETURNS TRIGGER AS $$
DECLARE
    recipe_creator_id UUID;
    import_count INTEGER;
    total_recipes INTEGER;
    avg_rating DECIMAL;
BEGIN
    -- Get recipe creator
    SELECT user_id INTO recipe_creator_id FROM recipes WHERE id = NEW.recipe_id;
    
    IF recipe_creator_id IS NULL THEN
        RETURN NEW;
    END IF;
    
    -- Check for achievements based on action type
    IF NEW.action_type = 'import' THEN
        -- Get total imports for this recipe
        SELECT COUNT(*) INTO import_count 
        FROM recipe_engagement_metrics 
        WHERE recipe_id = NEW.recipe_id AND action_type = 'import';
        
        -- Award "Popular Recipe" achievement at 100 imports
        IF import_count = 100 THEN
            INSERT INTO contributor_achievements (
                user_id, achievement_type, title, description, emoji, category, points
            ) VALUES (
                recipe_creator_id, 'popular_recipe', 'Popular Recipe', 
                'One of your recipes reached 100+ imports', '🔥', 'popularity', 500
            ) ON CONFLICT DO NOTHING;
        END IF;
        
        -- Award "Viral Recipe" achievement at 1000 imports
        IF import_count = 1000 THEN
            INSERT INTO contributor_achievements (
                user_id, achievement_type, title, description, emoji, category, points
            ) VALUES (
                recipe_creator_id, 'viral_recipe', 'Viral Recipe', 
                'One of your recipes went viral with 1000+ imports', '🚀', 'popularity', 2000
            ) ON CONFLICT DO NOTHING;
        END IF;
    END IF;
    
    -- Check milestone achievements for total recipes
    SELECT COUNT(*) INTO total_recipes FROM recipes WHERE user_id = recipe_creator_id;
    
    -- Award "Prolific Creator" at 50 recipes
    IF total_recipes = 50 THEN
        INSERT INTO contributor_achievements (
            user_id, achievement_type, title, description, emoji, category, points
        ) VALUES (
            recipe_creator_id, 'prolific_creator', 'Prolific Creator', 
            'Shared 50+ recipes with the community', '📚', 'milestone', 2000
        ) ON CONFLICT DO NOTHING;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic achievement awarding
DROP TRIGGER IF EXISTS trigger_check_achievements ON recipe_engagement_metrics;
CREATE TRIGGER trigger_check_achievements
    AFTER INSERT ON recipe_engagement_metrics
    FOR EACH ROW
    EXECUTE FUNCTION check_and_award_achievements();

-- Insert default attribution preferences for existing users
INSERT INTO attribution_preferences (user_id)
SELECT id FROM users 
WHERE NOT EXISTS (
    SELECT 1 FROM attribution_preferences WHERE user_id = users.id
);

-- Comments for documentation
COMMENT ON TABLE recipe_attributions IS 'Tracks attribution for imported community recipes';
COMMENT ON TABLE contributor_profiles IS 'Extended profile information for recipe contributors';
COMMENT ON TABLE contributor_badges IS 'Badge system for recognizing contributor achievements';
COMMENT ON TABLE contributor_achievements IS 'Achievement system for gamification and recognition';
COMMENT ON TABLE recipe_engagement_metrics IS 'Detailed tracking of user interactions with recipes';
COMMENT ON TABLE recipe_popularity_metrics IS 'Aggregated popularity and trending metrics for recipes';
COMMENT ON TABLE attribution_preferences IS 'User preferences for recipe attribution';
COMMENT ON TABLE attribution_reports IS 'Reports of attribution issues for moderation';
COMMENT ON TABLE feature_highlights IS 'Featured recipes and community highlights';

COMMENT ON COLUMN recipe_popularity_metrics.trending_score IS 'Calculated trending score based on recent engagement';
COMMENT ON COLUMN recipe_popularity_metrics.virality_index IS 'Measure of how viral a recipe is (shares/views ratio)';
COMMENT ON COLUMN contributor_achievements.points IS 'Points awarded for the achievement';

-- Grant permissions (adjust as needed for your setup)
GRANT SELECT, INSERT, UPDATE, DELETE ON recipe_attributions TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON contributor_profiles TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON contributor_badges TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON contributor_achievements TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON recipe_engagement_metrics TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON recipe_popularity_metrics TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON attribution_preferences TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON attribution_reports TO imkitchen_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON feature_highlights TO imkitchen_app;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO imkitchen_app;