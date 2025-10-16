-- Migration v0.7 - Recipe Ratings and Reviews
-- Created: 2025-10-15
-- Story: 2.9 Rate and Review Community Recipes

-- Ratings table for recipe ratings and reviews read model
CREATE TABLE IF NOT EXISTS ratings (
    id TEXT PRIMARY KEY NOT NULL,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    stars INTEGER NOT NULL CHECK(stars >= 1 AND stars <= 5),
    review_text TEXT, -- Optional review text (max 500 chars, validated in application layer)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, user_id) -- One rating per user per recipe
);

-- Index for fast aggregation queries (average rating, review count)
CREATE INDEX IF NOT EXISTS idx_ratings_recipe ON ratings(recipe_id);

-- Index for querying user's ratings
CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);

-- Index for chronological sorting (most recent first)
CREATE INDEX IF NOT EXISTS idx_ratings_created ON ratings(recipe_id, created_at DESC);
