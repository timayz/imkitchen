-- Migration 12: Fix recipe_collection_assignments foreign key
-- Created: 2025-10-24
-- Context: Migration 09 dropped the recipes table, but recipe_collection_assignments still has a FK to it
--          This causes "no such table: main.recipes" errors when inserting assignments
--          Solution: Drop and recreate the table without the FK to recipes (recipe IDs are validated in command layer)

DROP TABLE IF EXISTS recipe_collection_assignments;

CREATE TABLE IF NOT EXISTS recipe_collection_assignments (
    collection_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    assigned_at TEXT NOT NULL,
    PRIMARY KEY (collection_id, recipe_id),
    FOREIGN KEY (collection_id) REFERENCES recipe_collections(id) ON DELETE CASCADE
    -- Note: No FK to recipes table (dropped in migration 09) - recipe existence validated in command layer
);

CREATE INDEX IF NOT EXISTS idx_recipe_collection_assignments_recipe ON recipe_collection_assignments(recipe_id);
