-- Migration: 20250930005_discovery_schema_enhancements
-- Purpose: Complete database schema for Recipe Discovery and Browsing System
-- Story: 2.4 Recipe Discovery and Browsing
-- Acceptance Criteria: AC 1, 2, 4, 7

-- Enable necessary extensions for advanced features
PRAGMA foreign_keys = ON;

-- ===================================================================
-- DISCOVERY METRICS AND ANALYTICS TABLES
-- ===================================================================

-- Table: recipe_discovery_metrics
-- Purpose: Core metrics for recipe discovery algorithms
CREATE TABLE IF NOT EXISTS recipe_discovery_metrics (
    recipe_id TEXT PRIMARY KEY REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    base_popularity_score REAL NOT NULL DEFAULT 0.0,
    trending_score_24h REAL NOT NULL DEFAULT 0.0,
    trending_score_7d REAL NOT NULL DEFAULT 0.0,
    trending_score_30d REAL NOT NULL DEFAULT 0.0,
    view_count_total INTEGER NOT NULL DEFAULT 0,
    view_count_24h INTEGER NOT NULL DEFAULT 0,
    view_count_7d INTEGER NOT NULL DEFAULT 0,
    view_count_30d INTEGER NOT NULL DEFAULT 0,
    bookmark_count INTEGER NOT NULL DEFAULT 0,
    bookmark_velocity_24h REAL NOT NULL DEFAULT 0.0,
    search_mention_count INTEGER NOT NULL DEFAULT 0,
    search_rank_average REAL,
    last_viewed_at DATETIME,
    last_bookmarked_at DATETIME,
    metrics_updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for fast popularity queries
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_popularity ON recipe_discovery_metrics(base_popularity_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_trending_24h ON recipe_discovery_metrics(trending_score_24h DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_trending_7d ON recipe_discovery_metrics(trending_score_7d DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_views ON recipe_discovery_metrics(view_count_total DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_recent_activity ON recipe_discovery_metrics(last_viewed_at DESC);

-- ===================================================================
-- USER DISCOVERY PREFERENCES AND PERSONALIZATION
-- ===================================================================

-- Table: user_discovery_preferences
-- Purpose: Store user preferences for personalized discovery
CREATE TABLE IF NOT EXISTS user_discovery_preferences (
    user_id TEXT NOT NULL,
    preference_type TEXT NOT NULL, -- 'category', 'difficulty', 'prep_time', 'dietary', 'meal_type'
    preference_value TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0, -- Learning weight (0.0 to 1.0)
    interaction_count INTEGER NOT NULL DEFAULT 1,
    last_interaction_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, preference_type, preference_value)
);

-- Index for fast user preference lookups
CREATE INDEX IF NOT EXISTS idx_user_discovery_prefs_user ON user_discovery_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_user_discovery_prefs_type ON user_discovery_preferences(preference_type);
CREATE INDEX IF NOT EXISTS idx_user_discovery_prefs_weight ON user_discovery_preferences(weight DESC);

-- ===================================================================
-- RECIPE SIMILARITY AND RECOMMENDATIONS
-- ===================================================================

-- Table: recipe_similarity_cache (enhance existing table from migration 004)
-- Purpose: Add enhanced fields to existing recipe similarity cache
-- Note: Table already exists from migration 004, we're adding new columns

-- Add new columns to existing recipe_similarity_cache table
ALTER TABLE recipe_similarity_cache ADD COLUMN similarity_reasons TEXT DEFAULT '[]';
ALTER TABLE recipe_similarity_cache ADD COLUMN cache_valid_until DATETIME DEFAULT (datetime('now', '+7 days'));

-- Index for fast similarity lookups (add new indexes only)
CREATE INDEX IF NOT EXISTS idx_recipe_similarity_main ON recipe_similarity_cache(recipe_id, similarity_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_similarity_valid ON recipe_similarity_cache(cache_valid_until);

-- ===================================================================
-- DISCOVERY SEARCH ANALYTICS
-- ===================================================================

-- Table: discovery_search_sessions
-- Purpose: Track complete search sessions for analytics
CREATE TABLE IF NOT EXISTS discovery_search_sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT,
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME,
    total_queries INTEGER NOT NULL DEFAULT 0,
    total_results_viewed INTEGER NOT NULL DEFAULT 0,
    recipes_clicked INTEGER NOT NULL DEFAULT 0,
    recipes_bookmarked INTEGER NOT NULL DEFAULT 0,
    session_duration_ms INTEGER,
    conversion_rate REAL, -- recipes_clicked / total_results_viewed
    satisfaction_score REAL, -- Derived from user behavior
    user_agent TEXT,
    ip_address TEXT
);

-- Table: discovery_query_analytics
-- Purpose: Enhanced search query analytics
CREATE TABLE IF NOT EXISTS discovery_query_analytics (
    query_id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES discovery_search_sessions(session_id) ON DELETE SET NULL,
    user_id TEXT,
    query_text TEXT NOT NULL,
    query_type TEXT NOT NULL DEFAULT 'search', -- 'search', 'filter', 'suggestion'
    results_count INTEGER NOT NULL DEFAULT 0,
    results_clicked INTEGER NOT NULL DEFAULT 0,
    click_through_rate REAL,
    first_click_position INTEGER,
    query_duration_ms INTEGER NOT NULL,
    had_typos BOOLEAN NOT NULL DEFAULT FALSE,
    used_suggestions BOOLEAN NOT NULL DEFAULT FALSE,
    applied_filters TEXT, -- JSON of applied filters
    sort_order TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for search analytics
CREATE INDEX IF NOT EXISTS idx_discovery_query_text ON discovery_query_analytics(query_text);
CREATE INDEX IF NOT EXISTS idx_discovery_query_user ON discovery_query_analytics(user_id);
CREATE INDEX IF NOT EXISTS idx_discovery_query_timestamp ON discovery_query_analytics(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_discovery_query_ctr ON discovery_query_analytics(click_through_rate DESC);

-- ===================================================================
-- RECIPE DISCOVERY EVENTS
-- ===================================================================

-- Table: recipe_discovery_events
-- Purpose: Track all discovery-related events for analytics and learning
CREATE TABLE IF NOT EXISTS recipe_discovery_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL, -- 'view', 'click', 'bookmark', 'share', 'rate', 'similar_click'
    user_id TEXT,
    recipe_id TEXT REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    session_id TEXT REFERENCES discovery_search_sessions(session_id) ON DELETE SET NULL,
    source_context TEXT NOT NULL, -- 'search', 'trending', 'similar', 'popular', 'featured'
    source_query TEXT, -- Original search query if applicable
    position_in_results INTEGER, -- Position where recipe appeared in results
    event_metadata TEXT, -- JSON with additional context
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for event analytics
CREATE INDEX IF NOT EXISTS idx_discovery_events_recipe ON recipe_discovery_events(recipe_id);
CREATE INDEX IF NOT EXISTS idx_discovery_events_user ON recipe_discovery_events(user_id);
CREATE INDEX IF NOT EXISTS idx_discovery_events_type ON recipe_discovery_events(event_type);
CREATE INDEX IF NOT EXISTS idx_discovery_events_context ON recipe_discovery_events(source_context);
CREATE INDEX IF NOT EXISTS idx_discovery_events_timestamp ON recipe_discovery_events(timestamp DESC);

-- ===================================================================
-- DISCOVERY FEATURE TOGGLES AND CONFIGURATION
-- ===================================================================

-- Table: discovery_feature_config
-- Purpose: Runtime configuration for discovery features
CREATE TABLE IF NOT EXISTS discovery_feature_config (
    config_key TEXT PRIMARY KEY,
    config_value TEXT NOT NULL,
    value_type TEXT NOT NULL DEFAULT 'string', -- 'string', 'number', 'boolean', 'json'
    description TEXT,
    category TEXT NOT NULL DEFAULT 'general', -- 'search', 'trending', 'similarity', 'ui'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by TEXT
);

-- Insert default configuration values
INSERT OR IGNORE INTO discovery_feature_config (config_key, config_value, value_type, description, category) VALUES
-- Search Configuration
('search_fts_enabled', 'true', 'boolean', 'Enable Full-Text Search indexing', 'search'),
('search_max_results', '50', 'number', 'Maximum search results per page', 'search'),
('search_suggestion_limit', '8', 'number', 'Maximum number of search suggestions', 'search'),
('search_typo_tolerance', '2', 'number', 'Maximum edit distance for typo correction', 'search'),

-- Trending Configuration
('trending_min_views_24h', '10', 'number', 'Minimum views for 24h trending eligibility', 'trending'),
('trending_min_views_7d', '50', 'number', 'Minimum views for 7d trending eligibility', 'trending'),
('trending_velocity_threshold', '1.5', 'number', 'Minimum velocity multiplier for trending', 'trending'),
('trending_max_results', '20', 'number', 'Maximum trending recipes to display', 'trending'),

-- Similarity Configuration
('similarity_cache_ttl_hours', '24', 'number', 'Hours before similarity cache expires', 'similarity'),
('similarity_min_score', '0.3', 'number', 'Minimum similarity score to include', 'similarity'),
('similarity_max_results', '12', 'number', 'Maximum similar recipes to display', 'similarity'),

-- Popularity Configuration
('popularity_view_weight', '1.0', 'number', 'Weight for view count in popularity scoring', 'popularity'),
('popularity_rating_weight', '3.0', 'number', 'Weight for ratings in popularity scoring', 'popularity'),
('popularity_bookmark_weight', '5.0', 'number', 'Weight for bookmarks in popularity scoring', 'popularity'),
('popularity_recency_decay_days', '30', 'number', 'Days for recency decay in popularity', 'popularity'),

-- UI Configuration
('ui_infinite_scroll_enabled', 'true', 'boolean', 'Enable infinite scroll for recipe lists', 'ui'),
('ui_cards_per_row_mobile', '1', 'number', 'Recipe cards per row on mobile', 'ui'),
('ui_cards_per_row_tablet', '2', 'number', 'Recipe cards per row on tablet', 'ui'),
('ui_cards_per_row_desktop', '3', 'number', 'Recipe cards per row on desktop', 'ui');

-- ===================================================================
-- RECIPE CONTENT ENHANCEMENT FOR DISCOVERY
-- ===================================================================

-- Table: recipe_discovery_content
-- Purpose: Enhanced content metadata for better discovery
CREATE TABLE IF NOT EXISTS recipe_discovery_content (
    recipe_id TEXT PRIMARY KEY REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    search_keywords TEXT, -- Additional searchable keywords
    cooking_techniques TEXT, -- JSON array of detected techniques
    flavor_profile TEXT, -- JSON object with flavor characteristics
    season_relevance TEXT, -- JSON object with seasonal scores
    skill_level_computed REAL, -- Computed skill level (0.0-1.0)
    time_to_make_total INTEGER, -- Total time including prep + cook + rest
    dietary_tags TEXT, -- JSON array of detected dietary properties
    cuisine_style TEXT, -- Detected cuisine style
    meal_occasions TEXT, -- JSON array of suitable occasions
    ingredient_complexity_score REAL, -- Complexity based on ingredients
    equipment_required TEXT, -- JSON array of required equipment
    content_updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for content-based discovery
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_content_skill ON recipe_discovery_content(skill_level_computed);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_content_time ON recipe_discovery_content(time_to_make_total);
CREATE INDEX IF NOT EXISTS idx_recipe_discovery_content_complexity ON recipe_discovery_content(ingredient_complexity_score);

-- ===================================================================
-- TRIGGERS FOR AUTOMATIC METRIC UPDATES
-- ===================================================================

-- Trigger: Update metrics when recipe views occur
CREATE TRIGGER IF NOT EXISTS update_recipe_view_metrics
AFTER INSERT ON recipe_discovery_events
WHEN NEW.event_type = 'view'
BEGIN
    INSERT OR REPLACE INTO recipe_discovery_metrics (
        recipe_id, 
        view_count_total, 
        last_viewed_at, 
        metrics_updated_at
    ) VALUES (
        NEW.recipe_id,
        COALESCE((SELECT view_count_total FROM recipe_discovery_metrics WHERE recipe_id = NEW.recipe_id), 0) + 1,
        NEW.timestamp,
        CURRENT_TIMESTAMP
    );
END;

-- Trigger: Update metrics when recipe bookmarks occur
CREATE TRIGGER IF NOT EXISTS update_recipe_bookmark_metrics
AFTER INSERT ON recipe_discovery_events
WHEN NEW.event_type = 'bookmark'
BEGIN
    INSERT OR REPLACE INTO recipe_discovery_metrics (
        recipe_id, 
        bookmark_count, 
        last_bookmarked_at, 
        metrics_updated_at
    ) VALUES (
        NEW.recipe_id,
        COALESCE((SELECT bookmark_count FROM recipe_discovery_metrics WHERE recipe_id = NEW.recipe_id), 0) + 1,
        NEW.timestamp,
        CURRENT_TIMESTAMP
    );
END;

-- ===================================================================
-- VIEWS FOR COMMON DISCOVERY QUERIES
-- ===================================================================

-- View: Popular recipes with enhanced metrics
CREATE VIEW IF NOT EXISTS v_popular_recipes AS
SELECT 
    r.recipe_id,
    r.title,
    r.prep_time_minutes,
    r.cook_time_minutes,
    r.difficulty,
    r.category,
    dm.base_popularity_score,
    dm.view_count_total,
    dm.bookmark_count,
    COALESCE(AVG(rr.rating), 0) as average_rating,
    COUNT(rr.review_id) as review_count,
    dm.last_viewed_at,
    r.created_at
FROM recipes r
LEFT JOIN recipe_discovery_metrics dm ON r.recipe_id = dm.recipe_id
LEFT JOIN recipe_reviews rr ON r.recipe_id = rr.recipe_id AND rr.status = 'approved'
GROUP BY r.recipe_id
ORDER BY dm.base_popularity_score DESC NULLS LAST;

-- View: Trending recipes with velocity metrics
CREATE VIEW IF NOT EXISTS v_trending_recipes AS
SELECT 
    r.recipe_id,
    r.title,
    r.prep_time_minutes,
    r.difficulty,
    r.category,
    dm.trending_score_24h,
    dm.trending_score_7d,
    dm.view_count_24h,
    dm.view_count_7d,
    dm.bookmark_velocity_24h,
    CASE 
        WHEN dm.view_count_7d > 0 THEN 
            CAST(dm.view_count_24h AS REAL) / CAST(dm.view_count_7d AS REAL) * 7.0
        ELSE 0.0
    END as velocity_24h,
    dm.last_viewed_at,
    r.created_at
FROM recipes r
INNER JOIN recipe_discovery_metrics dm ON r.recipe_id = dm.recipe_id
WHERE dm.trending_score_24h > 0 OR dm.trending_score_7d > 0
ORDER BY dm.trending_score_24h DESC;

-- View: Recipe discovery analytics summary
CREATE VIEW IF NOT EXISTS v_discovery_analytics_summary AS
SELECT 
    DATE(timestamp) as analytics_date,
    COUNT(*) as total_queries,
    COUNT(DISTINCT user_id) as unique_users,
    AVG(results_count) as avg_results_per_query,
    AVG(click_through_rate) as avg_click_through_rate,
    AVG(query_duration_ms) as avg_query_duration_ms,
    SUM(CASE WHEN used_suggestions THEN 1 ELSE 0 END) as queries_with_suggestions,
    SUM(CASE WHEN had_typos THEN 1 ELSE 0 END) as queries_with_typos
FROM discovery_query_analytics
GROUP BY DATE(timestamp)
ORDER BY analytics_date DESC;

-- ===================================================================
-- DISCOVERY SEARCH OPTIMIZATION
-- ===================================================================

-- Enhanced FTS virtual table for recipe search
DROP TABLE IF EXISTS recipe_search_fts;
CREATE VIRTUAL TABLE IF NOT EXISTS recipe_search_fts USING fts5(
    recipe_id UNINDEXED,
    title,
    description,
    ingredients,
    instructions,
    tags,
    search_keywords,
    cooking_techniques,
    content='recipes',
    content_rowid='rowid'
);

-- Trigger to keep FTS table synchronized
CREATE TRIGGER IF NOT EXISTS recipe_search_fts_insert AFTER INSERT ON recipes BEGIN
    INSERT INTO recipe_search_fts(recipe_id, title, description, ingredients, instructions, tags)
    VALUES (new.recipe_id, new.title, new.description, new.ingredients, new.instructions, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_fts_delete AFTER DELETE ON recipes BEGIN
    DELETE FROM recipe_search_fts WHERE recipe_id = old.recipe_id;
END;

CREATE TRIGGER IF NOT EXISTS recipe_search_fts_update AFTER UPDATE ON recipes BEGIN
    DELETE FROM recipe_search_fts WHERE recipe_id = old.recipe_id;
    INSERT INTO recipe_search_fts(recipe_id, title, description, ingredients, instructions, tags)
    VALUES (new.recipe_id, new.title, new.description, new.ingredients, new.instructions, new.tags);
END;

-- ===================================================================
-- PERFORMANCE INDEXES FOR DISCOVERY QUERIES
-- ===================================================================

-- Composite indexes for common discovery query patterns
CREATE INDEX IF NOT EXISTS idx_recipes_category_difficulty ON recipes(category, difficulty);
CREATE INDEX IF NOT EXISTS idx_recipes_prep_time_category ON recipes(prep_time_minutes, category);
CREATE INDEX IF NOT EXISTS idx_recipes_created_category ON recipes(created_at DESC, category);

-- Indexes for user preference queries
CREATE INDEX IF NOT EXISTS idx_user_prefs_interaction ON user_discovery_preferences(last_interaction_at DESC);
CREATE INDEX IF NOT EXISTS idx_user_prefs_count ON user_discovery_preferences(interaction_count DESC);

-- Indexes for session analytics
CREATE INDEX IF NOT EXISTS idx_discovery_sessions_user_started ON discovery_search_sessions(user_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_discovery_sessions_duration ON discovery_search_sessions(session_duration_ms DESC);

-- ===================================================================
-- DISCOVERY SYSTEM HEALTH MONITORING
-- ===================================================================

-- Table: discovery_system_health
-- Purpose: Monitor discovery system performance and health
CREATE TABLE IF NOT EXISTS discovery_system_health (
    check_id TEXT PRIMARY KEY,
    check_type TEXT NOT NULL, -- 'search_performance', 'cache_hit_rate', 'recommendation_accuracy'
    metric_value REAL NOT NULL,
    metric_unit TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'healthy', -- 'healthy', 'warning', 'critical'
    threshold_warning REAL,
    threshold_critical REAL,
    details TEXT, -- JSON with additional context
    checked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default health check configurations
INSERT OR IGNORE INTO discovery_system_health (check_id, check_type, metric_value, metric_unit, threshold_warning, threshold_critical, details) VALUES
('search_avg_response_time', 'search_performance', 0, 'milliseconds', 500, 1000, '{"description": "Average search response time"}'),
('similarity_cache_hit_rate', 'cache_hit_rate', 0, 'percentage', 70, 50, '{"description": "Similarity cache hit rate"}'),
('trending_calculation_time', 'performance', 0, 'milliseconds', 1000, 2000, '{"description": "Time to calculate trending recipes"}');

-- Index for health monitoring
CREATE INDEX IF NOT EXISTS idx_discovery_health_type ON discovery_system_health(check_type);
CREATE INDEX IF NOT EXISTS idx_discovery_health_status ON discovery_system_health(status);
CREATE INDEX IF NOT EXISTS idx_discovery_health_checked ON discovery_system_health(checked_at DESC);

-- ===================================================================
-- FINAL VERIFICATION QUERIES
-- ===================================================================

-- Verify all tables were created successfully
SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%discovery%' OR name LIKE 'recipe_similarity%' OR name = 'recipe_search_fts';

-- Verify all indexes were created successfully  
SELECT name FROM sqlite_master WHERE type='index' AND name LIKE '%discovery%' OR name LIKE '%similarity%';

-- Verify configuration was inserted
SELECT config_key, config_value, category FROM discovery_feature_config ORDER BY category, config_key;