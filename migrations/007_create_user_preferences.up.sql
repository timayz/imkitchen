-- Migration 007: Create user preference tables
-- Generated: 2025-09-08
-- Description: Add user recipe favorites and weekly patterns tables for preference management

-- User Recipe Favorites Junction Table
CREATE TABLE IF NOT EXISTS user_recipe_favorites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    favorited_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    weight_multiplier DECIMAL(3,2) DEFAULT 1.5 CHECK (weight_multiplier >= 0.1 AND weight_multiplier <= 5.0),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, recipe_id)
);

-- User Weekly Patterns Table
CREATE TABLE IF NOT EXISTS user_weekly_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    day_of_week INTEGER NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    max_prep_time INTEGER DEFAULT 60 CHECK (max_prep_time BETWEEN 15 AND 180),
    preferred_complexity VARCHAR(20) DEFAULT 'moderate' CHECK (preferred_complexity IN ('simple', 'moderate', 'complex')),
    is_weekend_pattern BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, day_of_week)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_user_id ON user_recipe_favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_recipe_id ON user_recipe_favorites(recipe_id);
CREATE INDEX IF NOT EXISTS idx_user_recipe_favorites_favorited_at ON user_recipe_favorites(favorited_at);

CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_user_id ON user_weekly_patterns(user_id);
CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_day_of_week ON user_weekly_patterns(day_of_week);
CREATE INDEX IF NOT EXISTS idx_user_weekly_patterns_weekend ON user_weekly_patterns(is_weekend_pattern);

-- Add updated_at trigger for user_recipe_favorites
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_recipe_favorites_updated_at
    BEFORE UPDATE ON user_recipe_favorites
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_weekly_patterns_updated_at
    BEFORE UPDATE ON user_weekly_patterns
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Ensure existing users table has preference fields (if not exists)
ALTER TABLE users 
    ADD COLUMN IF NOT EXISTS max_cook_time INTEGER DEFAULT 60 CHECK (max_cook_time BETWEEN 15 AND 180);

ALTER TABLE users
    ADD COLUMN IF NOT EXISTS preferred_meal_complexity VARCHAR(20) DEFAULT 'moderate' CHECK (preferred_meal_complexity IN ('simple', 'moderate', 'complex'));