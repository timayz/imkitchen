-- Fix foreign key reference in recipes table
-- The recipes table references users(user_id) but should reference user_profiles(id)

-- Enable foreign key constraints
PRAGMA foreign_keys = OFF;

-- Create temporary table with correct foreign key reference
CREATE TABLE recipes_temp (
    recipe_id TEXT PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) >= 1 AND length(title) <= 200),
    prep_time_minutes INTEGER NOT NULL CHECK (prep_time_minutes > 0),
    cook_time_minutes INTEGER NOT NULL CHECK (cook_time_minutes > 0),
    difficulty TEXT NOT NULL CHECK (difficulty IN ('Easy', 'Medium', 'Hard')),
    category TEXT NOT NULL CHECK (category IN ('Appetizer', 'Main', 'Dessert', 'Beverage', 'Bread', 'Soup', 'Salad')),
    rating REAL DEFAULT 0.0 CHECK (rating >= 0.0 AND rating <= 5.0),
    review_count INTEGER DEFAULT 0 CHECK (review_count >= 0),
    created_by TEXT NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME,
    
    -- Correct foreign key to user_profiles table
    FOREIGN KEY (created_by) REFERENCES user_profiles(id) ON DELETE CASCADE
);

-- Copy data from original table
INSERT INTO recipes_temp SELECT * FROM recipes;

-- Drop original table
DROP TABLE recipes;

-- Rename temp table to original name
ALTER TABLE recipes_temp RENAME TO recipes;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_recipes_created_by ON recipes(created_by);
CREATE INDEX IF NOT EXISTS idx_recipes_category ON recipes(category);
CREATE INDEX IF NOT EXISTS idx_recipes_difficulty ON recipes(difficulty);
CREATE INDEX IF NOT EXISTS idx_recipes_is_public ON recipes(is_public);
CREATE INDEX IF NOT EXISTS idx_recipes_rating ON recipes(rating DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_created_at ON recipes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_recipes_prep_time ON recipes(prep_time_minutes);
CREATE INDEX IF NOT EXISTS idx_recipes_cook_time ON recipes(cook_time_minutes);

-- Re-enable foreign key constraints
PRAGMA foreign_keys = ON;