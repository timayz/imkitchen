# Database Schema

```sql
-- Users table with cooking preferences
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    dietary_preferences TEXT[], -- Array of dietary restrictions
    skill_level VARCHAR(20) CHECK (skill_level IN ('beginner', 'intermediate', 'advanced')),
    household_size INTEGER DEFAULT 1,
    kitchen_equipment JSONB DEFAULT '[]', -- Array of equipment objects
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Recipes table with detailed recipe information
CREATE TABLE recipes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    ingredients JSONB NOT NULL, -- Array of ingredient objects
    instructions JSONB NOT NULL, -- Array of instruction objects  
    prep_time INTEGER NOT NULL, -- minutes
    cook_time INTEGER NOT NULL, -- minutes
    total_time INTEGER GENERATED ALWAYS AS (prep_time + cook_time) STORED,
    servings INTEGER DEFAULT 1,
    difficulty VARCHAR(20) CHECK (difficulty IN ('beginner', 'intermediate', 'advanced')),
    cuisine_type VARCHAR(100),
    tags TEXT[] DEFAULT '{}',
    nutritional_info JSONB,
    source_url TEXT,
    image_path VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Meal plans for weekly planning
CREATE TABLE meal_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start DATE NOT NULL,
    meals JSONB NOT NULL, -- Weekly meal schedule object
    shopping_list JSONB NOT NULL, -- Array of shopping items
    generated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    
    UNIQUE(user_id, week_start)
);

-- Active cooking sessions with real-time state
CREATE TABLE cooking_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipe_id UUID NOT NULL REFERENCES recipes(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    start_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    end_time TIMESTAMP WITH TIME ZONE,
    current_step INTEGER DEFAULT 0,
    timers JSONB DEFAULT '[]', -- Array of active timer objects
    scaling_factor DECIMAL(3,2) DEFAULT 1.0,
    notes JSONB DEFAULT '[]', -- Array of cooking note objects
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    timing_accuracy INTEGER, -- Difference in minutes from predicted time
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Push notification subscriptions
CREATE TABLE push_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    endpoint TEXT NOT NULL,
    p256dh_key TEXT NOT NULL,
    auth_key TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(user_id, endpoint)
);

-- Indexes for query optimization
CREATE INDEX idx_recipes_user_id ON recipes(user_id);
CREATE INDEX idx_recipes_difficulty ON recipes(difficulty);
CREATE INDEX idx_recipes_total_time ON recipes(total_time);
CREATE INDEX idx_recipes_cuisine ON recipes(cuisine_type);
CREATE INDEX idx_recipes_tags ON recipes USING GIN(tags);
CREATE INDEX idx_recipes_search ON recipes USING GIN(to_tsvector('english', title || ' ' || description));

CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_start);
CREATE INDEX idx_cooking_sessions_user ON cooking_sessions(user_id);
CREATE INDEX idx_cooking_sessions_active ON cooking_sessions(user_id) WHERE end_time IS NULL;

-- Full-text search for recipes
ALTER TABLE recipes ADD COLUMN search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
) STORED;
CREATE INDEX idx_recipes_fts ON recipes USING GIN(search_vector);
```
