-- Migration v0.5
-- Created: 2025-10-15
-- Add favorite_count to users table for performance optimization

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
