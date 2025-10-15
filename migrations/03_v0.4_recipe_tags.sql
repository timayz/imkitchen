-- Migration: Add dietary_tags column to recipes table for automatic recipe tagging
-- Story: 2.5 - Automatic Recipe Tagging
-- Date: 2025-10-14
-- Note: complexity and cuisine columns already exist from migration 01_v0.2_recipes.sql

-- Add dietary_tags column to recipes table
ALTER TABLE recipes ADD COLUMN dietary_tags TEXT; -- JSON array of dietary tags e.g. ["vegetarian", "vegan"]

-- Create indexes for tag filtering performance
-- Note: complexity and cuisine will be indexed when they are populated with data
CREATE INDEX IF NOT EXISTS idx_recipes_complexity ON recipes(complexity);
CREATE INDEX IF NOT EXISTS idx_recipes_cuisine ON recipes(cuisine);
