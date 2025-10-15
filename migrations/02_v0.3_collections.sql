-- Migration v0.3 - Recipe Collections
-- Created: 2025-10-14
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
