-- Migration v0.6 - Recipe Soft Delete Support
-- Created: 2025-10-15
-- Story: 2.7 Share Recipe to Community (AC-12 compliance)
-- Description: Add deleted_at column to recipes table for soft delete support

-- Add deleted_at column to recipes table
ALTER TABLE recipes ADD COLUMN deleted_at TEXT DEFAULT NULL;

-- Create index for non-deleted shared recipes (optimizes /discover query)
-- Replaces idx_recipes_shared to filter both is_shared=1 AND deleted_at IS NULL
DROP INDEX IF EXISTS idx_recipes_shared;
CREATE INDEX IF NOT EXISTS idx_recipes_shared ON recipes(is_shared, deleted_at) WHERE is_shared = 1 AND deleted_at IS NULL;

-- Note: This migration enables proper soft delete for recipes.
-- The delete_recipe command will now UPDATE deleted_at instead of DELETE.
-- The /discover community feed will filter: WHERE is_shared = 1 AND deleted_at IS NULL
