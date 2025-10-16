-- Migration v0.2 - Complete Application Schema
-- Created: 2025-10-15
-- Stories: 2.1 Create Recipe, 2.4 Organize Recipes into Collections, 2.5 Automatic Recipe Tagging,
--          2.6 Mark Recipe as Favorite, 2.7 Share Recipe to Community, 2.9 Rate and Review Community Recipes
-- Description: Merged migration containing all application tables and indexes

-- ============================================================================
-- RECIPES TABLE
-- ============================================================================
-- Story: 2.1 Create Recipe
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    ingredients TEXT NOT NULL, -- JSON array of {name, quantity, unit}
    instructions TEXT NOT NULL, -- JSON array of {step_number, instruction_text, timer_minutes}
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    advance_prep_hours INTEGER, -- NULL if no advance prep required
    serving_size INTEGER,
    is_favorite INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false, 1=true
    is_shared INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false (private), 1=true (shared to community)
    complexity TEXT, -- "simple", "moderate", "complex"
    cuisine TEXT, -- "italian", "chinese", etc.
    dietary_tags TEXT, -- JSON array of dietary tags e.g. ["vegetarian", "vegan"] (Story 2.5)
    deleted_at TEXT DEFAULT NULL, -- Soft delete timestamp (NULL if not deleted) (Story 2.7)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Recipe indexes
CREATE INDEX IF NOT EXISTS idx_recipes_user_id ON recipes(user_id);
CREATE INDEX IF NOT EXISTS idx_recipes_favorite ON recipes(user_id, is_favorite);
CREATE INDEX IF NOT EXISTS idx_recipes_shared ON recipes(is_shared, deleted_at) WHERE is_shared = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipes_complexity ON recipes(complexity);
CREATE INDEX IF NOT EXISTS idx_recipes_cuisine ON recipes(cuisine);

-- ============================================================================
-- RECIPE COLLECTIONS
-- ============================================================================
-- Story: 2.4 Organize Recipes into Collections

-- Recipe collections table for read model
CREATE TABLE IF NOT EXISTS recipe_collections (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT, -- Optional description
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT, -- Soft delete timestamp (NULL if not deleted)
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_recipe_collections_user_id ON recipe_collections(user_id);
CREATE INDEX IF NOT EXISTS idx_recipe_collections_not_deleted ON recipe_collections(user_id, deleted_at) WHERE deleted_at IS NULL;

-- Recipe collection assignments table (many-to-many relationship)
-- Enables recipes to belong to multiple collections simultaneously
CREATE TABLE IF NOT EXISTS recipe_collection_assignments (
    collection_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    PRIMARY KEY (collection_id, recipe_id),
    FOREIGN KEY (collection_id) REFERENCES recipe_collections(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recipe_collection_assignments_collection ON recipe_collection_assignments(collection_id);
CREATE INDEX IF NOT EXISTS idx_recipe_collection_assignments_recipe ON recipe_collection_assignments(recipe_id);

-- ============================================================================
-- FAVORITE COUNT OPTIMIZATION
-- ============================================================================
-- Story: 2.6 Mark Recipe as Favorite

-- Add favorite_count column to track favorited recipes
-- This is updated via evento subscription when RecipeFavorited events are emitted
ALTER TABLE users ADD COLUMN favorite_count INTEGER NOT NULL DEFAULT 0;

-- Backfill favorite_count for existing users
UPDATE users SET favorite_count = (
    SELECT COUNT(*)
    FROM recipes
    WHERE recipes.user_id = users.id
      AND recipes.is_favorite != 0
);

-- ============================================================================
-- RATINGS AND REVIEWS
-- ============================================================================
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

-- Rating indexes
CREATE INDEX IF NOT EXISTS idx_ratings_recipe ON ratings(recipe_id);
CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_ratings_created ON ratings(recipe_id, created_at DESC);
