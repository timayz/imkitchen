-- Migration 11: Recreate ratings table for projections
-- Created: 2025-10-24
-- Context: Migration 09 dropped the ratings table, but the recipe_projection handlers still write to it
--          This migration recreates it (without the foreign key to recipes table which no longer exists)

CREATE TABLE IF NOT EXISTS ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    stars INTEGER NOT NULL CHECK(stars BETWEEN 1 AND 5),
    review_text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_ratings_recipe_created ON ratings(recipe_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);
