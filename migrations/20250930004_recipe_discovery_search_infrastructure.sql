-- Recipe Discovery Search Infrastructure Migration
-- Story 2.4: Recipe Discovery and Browsing - Task 6
-- Extends existing FTS5 search with advanced discovery features

-- Recipe Views Tracking Table
CREATE TABLE IF NOT EXISTS recipe_views (
    view_id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipe_id TEXT NOT NULL,
    user_id TEXT, -- NULL for anonymous views
    view_timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    view_duration_seconds INTEGER, -- How long user viewed recipe
    view_source TEXT CHECK (view_source IN ('search', 'browse', 'trending', 'similar', 'direct')),
    search_query TEXT, -- What search led to this view (if from search)
    referrer_recipe_id TEXT, -- If viewed from similar recipes section
    
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE SET NULL,
    FOREIGN KEY (referrer_recipe_id) REFERENCES recipes(recipe_id) ON DELETE SET NULL
);

-- Search History Table for Analytics and Suggestions
CREATE TABLE IF NOT EXISTS search_history (
    search_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT, -- NULL for anonymous searches
    search_query TEXT NOT NULL CHECK (length(trim(search_query)) > 0),
    search_timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    results_count INTEGER NOT NULL DEFAULT 0,
    clicked_recipe_id TEXT, -- Which recipe was clicked from results (if any)
    click_position INTEGER, -- Position in search results (1-based)
    search_filters TEXT, -- JSON representation of applied filters
    search_duration_ms INTEGER, -- How long search took to execute
    
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE SET NULL,
    FOREIGN KEY (clicked_recipe_id) REFERENCES recipes(recipe_id) ON DELETE SET NULL
);

-- Recipe Popularity Metrics (Materialized View for Performance)
CREATE TABLE IF NOT EXISTS recipe_popularity_metrics (
    recipe_id TEXT PRIMARY KEY,
    view_count_total INTEGER DEFAULT 0,
    view_count_24h INTEGER DEFAULT 0,
    view_count_7d INTEGER DEFAULT 0,
    view_count_30d INTEGER DEFAULT 0,
    search_appearances INTEGER DEFAULT 0, -- How often appears in search results
    search_clicks INTEGER DEFAULT 0, -- How often clicked from search
    click_through_rate REAL DEFAULT 0.0, -- search_clicks / search_appearances
    popularity_score REAL DEFAULT 0.0, -- Calculated weighted score
    trending_score REAL DEFAULT 0.0, -- Time-weighted trending score
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Recipe Similarity Cache for Performance
CREATE TABLE IF NOT EXISTS recipe_similarity_cache (
    recipe_id TEXT NOT NULL,
    similar_recipe_id TEXT NOT NULL,
    similarity_score REAL NOT NULL CHECK (similarity_score >= 0.0 AND similarity_score <= 1.0),
    similarity_type TEXT NOT NULL CHECK (similarity_type IN ('ingredient', 'technique', 'category', 'composite')),
    calculated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (recipe_id, similar_recipe_id, similarity_type),
    FOREIGN KEY (recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE,
    FOREIGN KEY (similar_recipe_id) REFERENCES recipes(recipe_id) ON DELETE CASCADE
);

-- Search Suggestions Cache
CREATE TABLE IF NOT EXISTS search_suggestions (
    suggestion_id INTEGER PRIMARY KEY AUTOINCREMENT,
    suggestion_text TEXT NOT NULL UNIQUE,
    suggestion_type TEXT NOT NULL CHECK (suggestion_type IN ('ingredient', 'recipe', 'tag', 'category', 'autocomplete')),
    search_count INTEGER DEFAULT 0, -- How many times this was searched
    success_rate REAL DEFAULT 0.0, -- Percentage of searches that resulted in clicks
    popularity_score REAL DEFAULT 0.0,
    last_used DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- User Search Preferences for Personalization
CREATE TABLE IF NOT EXISTS user_search_preferences (
    user_id TEXT PRIMARY KEY,
    preferred_categories TEXT, -- JSON array of preferred categories
    preferred_difficulty TEXT, -- JSON array of preferred difficulty levels
    preferred_prep_time_max INTEGER, -- Maximum prep time preference
    preferred_dietary_restrictions TEXT, -- JSON array of dietary restrictions
    preferred_meal_types TEXT, -- JSON array of meal types
    search_behavior_data TEXT, -- JSON with aggregated search patterns
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE CASCADE
);

-- Performance Indexes
CREATE INDEX IF NOT EXISTS idx_recipe_views_recipe_id ON recipe_views(recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipe_views_timestamp ON recipe_views(view_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_views_user_id ON recipe_views(user_id);
CREATE INDEX IF NOT EXISTS idx_recipe_views_source ON recipe_views(view_source);
CREATE INDEX IF NOT EXISTS idx_recipe_views_24h ON recipe_views(view_timestamp) WHERE view_timestamp > datetime('now', '-1 day');

CREATE INDEX IF NOT EXISTS idx_search_history_user_id ON search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_search_history_timestamp ON search_history(search_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_query ON search_history(search_query);
CREATE INDEX IF NOT EXISTS idx_search_history_clicked_recipe ON search_history(clicked_recipe_id);

CREATE INDEX IF NOT EXISTS idx_recipe_popularity_score ON recipe_popularity_metrics(popularity_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_popularity_trending ON recipe_popularity_metrics(trending_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_popularity_24h ON recipe_popularity_metrics(view_count_24h DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_popularity_updated ON recipe_popularity_metrics(last_updated);

CREATE INDEX IF NOT EXISTS idx_recipe_similarity_score ON recipe_similarity_cache(similarity_score DESC);
CREATE INDEX IF NOT EXISTS idx_recipe_similarity_type ON recipe_similarity_cache(similarity_type);
CREATE INDEX IF NOT EXISTS idx_recipe_similarity_calculated ON recipe_similarity_cache(calculated_at);

CREATE INDEX IF NOT EXISTS idx_search_suggestions_type ON search_suggestions(suggestion_type);
CREATE INDEX IF NOT EXISTS idx_search_suggestions_popularity ON search_suggestions(popularity_score DESC);
CREATE INDEX IF NOT EXISTS idx_search_suggestions_text ON search_suggestions(suggestion_text);

-- Triggers for Real-time Popularity Updates
CREATE TRIGGER IF NOT EXISTS update_recipe_popularity_on_view AFTER INSERT ON recipe_views
BEGIN
    INSERT OR REPLACE INTO recipe_popularity_metrics (
        recipe_id, view_count_total, view_count_24h, view_count_7d, view_count_30d,
        search_appearances, search_clicks, click_through_rate,
        popularity_score, trending_score, last_updated
    ) VALUES (
        NEW.recipe_id,
        COALESCE((SELECT view_count_total FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 0) + 1,
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-1 day')), 0),
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-7 days')), 0),
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-30 days')), 0),
        COALESCE((SELECT search_appearances FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 0),
        COALESCE((SELECT search_clicks FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 0),
        CASE 
            WHEN COALESCE((SELECT search_appearances FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 0) > 0 
            THEN CAST(COALESCE((SELECT search_clicks FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 0) AS REAL) / 
                 COALESCE((SELECT search_appearances FROM recipe_popularity_metrics WHERE recipe_id = NEW.recipe_id), 1)
            ELSE 0.0
        END,
        -- Simple popularity score: 24h views * 3 + 7d views * 2 + 30d views * 1 + rating * 10
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-1 day')), 0) * 3.0 +
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-7 days')), 0) * 2.0 +
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-30 days')), 0) * 1.0 +
        COALESCE((SELECT rating FROM recipes WHERE recipe_id = NEW.recipe_id), 0) * 10.0,
        -- Trending score: weighted heavily toward recent activity
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-1 day')), 0) * 10.0 +
        COALESCE((SELECT COUNT(*) FROM recipe_views WHERE recipe_id = NEW.recipe_id AND view_timestamp > datetime('now', '-7 days')), 0) * 3.0,
        CURRENT_TIMESTAMP
    );
END;

-- Trigger for Search Click Analytics
CREATE TRIGGER IF NOT EXISTS update_search_suggestions_on_search AFTER INSERT ON search_history
BEGIN
    INSERT OR REPLACE INTO search_suggestions (
        suggestion_text, suggestion_type, search_count, success_rate, popularity_score, last_used
    ) VALUES (
        NEW.search_query,
        'autocomplete',
        COALESCE((SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query), 0) + 1,
        CASE 
            WHEN NEW.clicked_recipe_id IS NOT NULL THEN
                CASE 
                    WHEN EXISTS (SELECT 1 FROM search_suggestions WHERE suggestion_text = NEW.search_query) THEN
                        ((SELECT success_rate FROM search_suggestions WHERE suggestion_text = NEW.search_query) * 
                         (SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query) + 1.0) /
                        ((SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query) + 1.0)
                    ELSE 1.0
                END
            ELSE 
                CASE 
                    WHEN EXISTS (SELECT 1 FROM search_suggestions WHERE suggestion_text = NEW.search_query) THEN
                        ((SELECT success_rate FROM search_suggestions WHERE suggestion_text = NEW.search_query) * 
                         (SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query)) /
                        ((SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query) + 1.0)
                    ELSE 0.0
                END
        END,
        COALESCE((SELECT search_count FROM search_suggestions WHERE suggestion_text = NEW.search_query), 0) + 1.0,
        CURRENT_TIMESTAMP
    );
END;

-- Initialize popularity metrics for existing recipes
INSERT OR IGNORE INTO recipe_popularity_metrics (recipe_id, last_updated)
SELECT recipe_id, CURRENT_TIMESTAMP FROM recipes WHERE is_public = TRUE;