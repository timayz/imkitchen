-- Performance and Analytics Tables Migration
-- This migration creates tables for tracking recipe performance and analytics

-- Recipe Performance Metrics table
CREATE TABLE recipe_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    
    -- Usage Statistics (daily aggregation)
    date DATE NOT NULL,
    times_included_in_meal_plans INTEGER DEFAULT 0,
    times_rated INTEGER DEFAULT 0,
    average_rating_that_day DECIMAL(3,2),
    
    -- Performance Metrics
    successful_generations INTEGER DEFAULT 0, -- times algorithm successfully used this recipe
    user_modifications INTEGER DEFAULT 0, -- times users changed/replaced this recipe
    completion_rate DECIMAL(5,2) DEFAULT 0.0, -- % of times users actually cooked this
    
    -- Algorithm Learning
    algorithm_score DECIMAL(8,6) DEFAULT 0.0, -- internal scoring for meal plan generation
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(recipe_id, date)
);

-- Indexes for recipe_performance_metrics table
CREATE INDEX idx_recipe_perf_recipe_date ON recipe_performance_metrics(recipe_id, date DESC);
CREATE INDEX idx_recipe_perf_date ON recipe_performance_metrics(date DESC);
CREATE INDEX idx_recipe_perf_algorithm_score ON recipe_performance_metrics(algorithm_score DESC);