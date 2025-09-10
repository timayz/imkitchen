-- imkitchen Database Initial Schema Migration
-- This migration creates all the core tables, indexes, views, and functions

-- Create extension for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    email_verified BOOLEAN DEFAULT false,
    encrypted_password VARCHAR(255),
    
    -- Profile Information
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    avatar_url TEXT,
    
    -- Preferences
    dietary_restrictions TEXT[] DEFAULT '{}',
    allergies TEXT[] DEFAULT '{}',
    cooking_skill_level VARCHAR(20) CHECK (cooking_skill_level IN ('beginner', 'intermediate', 'advanced')),
    preferred_meal_complexity VARCHAR(20) CHECK (preferred_meal_complexity IN ('simple', 'moderate', 'complex')),
    max_cook_time INTEGER DEFAULT 60, -- minutes
    
    -- Learning Algorithm Data
    rotation_reset_count INTEGER DEFAULT 0,
    preference_learning_data JSONB DEFAULT '{}',
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active_at TIMESTAMP WITH TIME ZONE,
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    -- Constraints
    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Indexes for users table
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_dietary_restrictions ON users USING GIN(dietary_restrictions);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;

-- Recipes table
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255), -- from Spoonacular/Edamam
    external_source VARCHAR(50), -- 'spoonacular', 'edamam', 'user_generated'
    
    -- Basic Recipe Info
    title VARCHAR(255) NOT NULL,
    description TEXT,
    image_url TEXT,
    source_url TEXT,
    
    -- Timing
    prep_time INTEGER NOT NULL, -- minutes
    cook_time INTEGER NOT NULL, -- minutes
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    
    -- Classification
    meal_type VARCHAR(20)[] CHECK (meal_type <@ ARRAY['breakfast', 'lunch', 'dinner', 'snack']),
    complexity VARCHAR(20) CHECK (complexity IN ('simple', 'moderate', 'complex')),
    cuisine_type VARCHAR(50),
    
    -- Recipe Data
    servings INTEGER DEFAULT 4,
    ingredients JSONB NOT NULL, -- [{"name": "flour", "amount": 2, "unit": "cups", "category": "pantry"}]
    instructions JSONB NOT NULL, -- [{"step": 1, "instruction": "Mix flour...", "estimatedMinutes": 5}]
    
    -- Nutritional Information
    nutrition JSONB, -- calories, protein, carbs, fat, fiber, etc.
    dietary_labels TEXT[] DEFAULT '{}', -- vegetarian, vegan, gluten-free, etc.
    
    -- Quality Metrics
    average_rating DECIMAL(3,2) DEFAULT 0.0,
    total_ratings INTEGER DEFAULT 0,
    difficulty_score INTEGER CHECK (difficulty_score BETWEEN 1 AND 10),
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for recipes table
CREATE INDEX idx_recipes_meal_type ON recipes USING GIN(meal_type);
CREATE INDEX idx_recipes_complexity ON recipes(complexity);
CREATE INDEX idx_recipes_total_time ON recipes(total_time);
CREATE INDEX idx_recipes_dietary_labels ON recipes USING GIN(dietary_labels);
CREATE INDEX idx_recipes_average_rating ON recipes(average_rating DESC);
CREATE INDEX idx_recipes_external_source_id ON recipes(external_source, external_id);
CREATE INDEX idx_recipes_nutrition ON recipes USING GIN(nutrition);
CREATE INDEX idx_recipes_fulltext ON recipes USING GIN(to_tsvector('english', title || ' ' || COALESCE(description, '')));

-- Meal Plans table
CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Planning Period
    week_start DATE NOT NULL,
    week_end DATE GENERATED ALWAYS AS (week_start + INTERVAL '6 days') STORED,
    
    -- Meal Plan Data
    meals JSONB NOT NULL, -- {"monday": [{"mealType": "breakfast", "recipeId": "...", "servings": 2}]}
    
    -- Generation Metadata
    generation_algorithm_version VARCHAR(20) DEFAULT 'v1.0',
    generation_parameters JSONB, -- user preferences at generation time
    generation_duration_ms INTEGER,
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('draft', 'active', 'archived', 'deleted')),
    
    -- User Interaction
    user_feedback JSONB DEFAULT '{}', -- ratings, modifications, notes
    completion_percentage DECIMAL(5,2) DEFAULT 0.0, -- % of meals actually prepared
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    archived_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for meal_plans table
CREATE INDEX idx_meal_plans_user_id ON meal_plans(user_id);
CREATE INDEX idx_meal_plans_week_start ON meal_plans(week_start DESC);
CREATE INDEX idx_meal_plans_status ON meal_plans(status);
CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_start DESC);
CREATE INDEX idx_meal_plans_meals ON meal_plans USING GIN(meals);

-- Recipe Ratings table
CREATE TABLE recipe_ratings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Rating Data
    overall_rating INTEGER CHECK (overall_rating BETWEEN 1 AND 5),
    difficulty_rating INTEGER CHECK (difficulty_rating BETWEEN 1 AND 5),
    taste_rating INTEGER CHECK (taste_rating BETWEEN 1 AND 5),
    
    -- Feedback
    review_text TEXT,
    would_make_again BOOLEAN,
    actual_prep_time INTEGER, -- minutes (vs. estimated)
    actual_cook_time INTEGER, -- minutes (vs. estimated)
    
    -- Context
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    cooking_context VARCHAR(50), -- 'weeknight', 'weekend', 'special_occasion'
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(recipe_id, user_id)
);

-- Indexes for recipe_ratings table
CREATE INDEX idx_recipe_ratings_recipe_id ON recipe_ratings(recipe_id);
CREATE INDEX idx_recipe_ratings_user_id ON recipe_ratings(user_id);
CREATE INDEX idx_recipe_ratings_overall_rating ON recipe_ratings(overall_rating);
CREATE INDEX idx_recipe_ratings_created_at ON recipe_ratings(created_at DESC);