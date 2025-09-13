-- Initial database schema for ImKitchen
-- This migration sets up the foundational tables

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users table with cooking preferences
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    dietary_preferences TEXT[] DEFAULT '{}', 
    skill_level VARCHAR(20) CHECK (skill_level IN ('beginner', 'intermediate', 'advanced')) DEFAULT 'beginner',
    household_size INTEGER DEFAULT 1,
    kitchen_equipment JSONB DEFAULT '[]',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Recipes table with detailed recipe information
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL,
    instructions JSONB NOT NULL,
    prep_time INTEGER NOT NULL,
    cook_time INTEGER NOT NULL,
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    servings INTEGER DEFAULT 1,
    difficulty VARCHAR(20) CHECK (difficulty IN ('beginner', 'intermediate', 'advanced')) DEFAULT 'beginner',
    cuisine_type VARCHAR(100),
    tags TEXT[] DEFAULT '{}',
    nutritional_info JSONB,
    source_url TEXT,
    image_path VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for optimized queries
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_difficulty ON recipes(difficulty);
CREATE INDEX idx_recipes_total_time ON recipes(total_time);
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine_type);
CREATE INDEX idx_recipes_tags ON recipes USING GIN(tags);

-- Full-text search for recipes
ALTER TABLE recipes ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
) STORED;
CREATE INDEX idx_recipes_fts ON recipes USING GIN(search_vector);