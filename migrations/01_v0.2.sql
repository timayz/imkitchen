-- Migration v0.2 - Complete Application Schema
-- Created: 2025-10-15
-- Stories: 2.1 Create Recipe, 2.4 Organize Recipes into Collections, 2.5 Automatic Recipe Tagging,
--          2.6 Mark Recipe as Favorite, 2.7 Share Recipe to Community, 2.9 Rate and Review Community Recipes,
--          2.10 Copy Community Recipe to Personal Library
-- Description: Merged migration containing all application tables and indexes

-- ============================================================================
-- RECIPES TABLE
-- ============================================================================
-- Stories: 2.1 Create Recipe, 2.10 Copy Community Recipe
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    ingredients TEXT NOT NULL,      -- JSON array of {name, quantity, unit}
    instructions TEXT NOT NULL,     -- JSON array of {step_number, instruction_text, timer_minutes}
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    advance_prep_hours INTEGER,
    serving_size INTEGER,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    is_shared INTEGER NOT NULL DEFAULT 0,
    complexity TEXT,
    cuisine TEXT,
    dietary_tags TEXT,              -- JSON array (Story 2.5)
    original_recipe_id TEXT,        -- ID of original recipe if copied (Story 2.10)
    original_author TEXT,           -- User ID of original creator if copied (Story 2.10)
    deleted_at TEXT,                -- Soft delete (Story 2.7)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_recipes_user_id ON recipes(user_id);
CREATE INDEX IF NOT EXISTS idx_recipes_favorite ON recipes(user_id, is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_recipes_shared ON recipes(is_shared, deleted_at) WHERE is_shared = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipes_complexity ON recipes(complexity) WHERE complexity IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_recipes_cuisine ON recipes(cuisine) WHERE cuisine IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_recipes_original ON recipes(user_id, original_recipe_id) WHERE original_recipe_id IS NOT NULL AND deleted_at IS NULL;

-- ============================================================================
-- RECIPE COLLECTIONS
-- ============================================================================
-- Story: 2.4 Organize Recipes into Collections

CREATE TABLE IF NOT EXISTS recipe_collections (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_recipe_collections_user_id ON recipe_collections(user_id, deleted_at) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS recipe_collection_assignments (
    collection_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    PRIMARY KEY (collection_id, recipe_id),
    FOREIGN KEY (collection_id) REFERENCES recipe_collections(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recipe_collection_assignments_recipe ON recipe_collection_assignments(recipe_id);

-- ============================================================================
-- FAVORITE COUNT OPTIMIZATION
-- ============================================================================
-- Story: 2.6 Mark Recipe as Favorite

ALTER TABLE users ADD COLUMN favorite_count INTEGER NOT NULL DEFAULT 0;

UPDATE users
SET favorite_count = (
    SELECT COUNT(*)
    FROM recipes
    WHERE user_id = users.id AND is_favorite = 1
);

-- ============================================================================
-- RATINGS AND REVIEWS
-- ============================================================================
-- Story: 2.9 Rate and Review Community Recipes

CREATE TABLE IF NOT EXISTS ratings (
    id TEXT PRIMARY KEY,
    recipe_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    stars INTEGER NOT NULL CHECK(stars BETWEEN 1 AND 5),
    review_text TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(recipe_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_ratings_recipe_created ON ratings(recipe_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ratings_user ON ratings(user_id);
