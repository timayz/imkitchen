# 9. Database Schema

## PostgreSQL Schema Design

### Core Tables

**Users Table**
```sql
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
    
    -- Indexes
    CONSTRAINT email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Indexes for users table
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_dietary_restrictions ON users USING GIN(dietary_restrictions);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;
```

**Recipes Table**
```sql
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
    ingredients JSONB NOT NULL, -- [{"name": "flour", "amount": 2, "unit": "cups"}]
    instructions JSONB NOT NULL, -- [{"step": 1, "instruction": "Mix flour..."}]
    
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
```

**Meal Plans Table**
```sql
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
```

**Recipe Ratings Table**
```sql
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
```

### Supporting Tables

**Shopping Lists Table**
```sql
CREATE TABLE shopping_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    meal_plan_id UUID REFERENCES meal_plans(id) ON DELETE SET NULL,
    
    -- List Data
    name VARCHAR(255) NOT NULL DEFAULT 'Weekly Shopping List',
    items JSONB NOT NULL, -- [{"name": "flour", "quantity": 2, "unit": "lbs", "category": "baking", "checked": false}]
    
    -- Organization
    store_preferences JSONB, -- aisle ordering, preferred brands
    estimated_total_cost DECIMAL(10,2),
    
    -- Status
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'completed', 'archived')),
    completed_at TIMESTAMP WITH TIME ZONE,
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for shopping_lists table
CREATE INDEX idx_shopping_lists_user_id ON shopping_lists(user_id);
CREATE INDEX idx_shopping_lists_meal_plan_id ON shopping_lists(meal_plan_id);
CREATE INDEX idx_shopping_lists_status ON shopping_lists(status);
CREATE INDEX idx_shopping_lists_created_at ON shopping_lists(created_at DESC);
```

**User Preferences History Table**
```sql
CREATE TABLE user_preference_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Preference Tracking
    preference_type VARCHAR(50) NOT NULL, -- 'dietary_restriction_added', 'skill_level_updated'
    old_value JSONB,
    new_value JSONB,
    change_reason VARCHAR(255), -- 'onboarding', 'user_update', 'algorithm_learning'
    
    -- Context
    triggered_by VARCHAR(50), -- 'user_action', 'feedback_analysis', 'preference_quiz'
    confidence_score DECIMAL(5,4), -- algorithm confidence in learned preferences
    
    -- Metadata
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for user_preference_history table
CREATE INDEX idx_user_pref_history_user_id ON user_preference_history(user_id);
CREATE INDEX idx_user_pref_history_type ON user_preference_history(preference_type);
CREATE INDEX idx_user_pref_history_created_at ON user_preference_history(created_at DESC);
```

### Performance and Analytics Tables

**Recipe Performance Metrics**
```sql
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
```

### Views for Common Queries

**Active User Meal Plans View**
```sql
CREATE VIEW active_user_meal_plans AS
SELECT 
    mp.*,
    u.email,
    u.dietary_restrictions,
    u.cooking_skill_level,
    EXTRACT(EPOCH FROM (NOW() - mp.created_at))/3600 AS hours_since_creation,
    jsonb_array_length(mp.meals->'monday') + 
    jsonb_array_length(mp.meals->'tuesday') + 
    jsonb_array_length(mp.meals->'wednesday') + 
    jsonb_array_length(mp.meals->'thursday') + 
    jsonb_array_length(mp.meals->'friday') + 
    jsonb_array_length(mp.meals->'saturday') + 
    jsonb_array_length(mp.meals->'sunday') AS total_planned_meals
FROM meal_plans mp
JOIN users u ON mp.user_id = u.id
WHERE mp.status = 'active'
    AND u.deleted_at IS NULL
    AND mp.week_start <= CURRENT_DATE
    AND mp.week_end >= CURRENT_DATE;
```

**Recipe Recommendations View**
```sql
CREATE VIEW recipe_recommendations AS
SELECT 
    r.*,
    rpm.algorithm_score,
    rpm.completion_rate,
    CASE 
        WHEN r.total_ratings > 50 THEN r.average_rating
        WHEN r.total_ratings > 10 THEN (r.average_rating * r.total_ratings + 3.5 * 10) / (r.total_ratings + 10)
        ELSE 3.5 -- default rating for new recipes
    END AS weighted_rating
FROM recipes r
LEFT JOIN (
    SELECT 
        recipe_id,
        AVG(algorithm_score) as algorithm_score,
        AVG(completion_rate) as completion_rate
    FROM recipe_performance_metrics 
    WHERE date >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY recipe_id
) rpm ON r.id = rpm.recipe_id
WHERE r.deleted_at IS NULL;
```

### Database Functions

**Update Recipe Rating Aggregates Function**
```sql
CREATE OR REPLACE FUNCTION update_recipe_rating_aggregates()
RETURNS TRIGGER AS $$
BEGIN
    -- Update recipe aggregates when rating is inserted/updated
    UPDATE recipes 
    SET 
        average_rating = (
            SELECT ROUND(AVG(overall_rating)::numeric, 2)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        total_ratings = (
            SELECT COUNT(*)
            FROM recipe_ratings 
            WHERE recipe_id = COALESCE(NEW.recipe_id, OLD.recipe_id)
        ),
        updated_at = NOW()
    WHERE id = COALESCE(NEW.recipe_id, OLD.recipe_id);
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Trigger for automatic rating aggregation
CREATE TRIGGER trigger_update_recipe_ratings
    AFTER INSERT OR UPDATE OR DELETE ON recipe_ratings
    FOR EACH ROW
    EXECUTE FUNCTION update_recipe_rating_aggregates();
```

**Search Recipes Function**
```sql
CREATE OR REPLACE FUNCTION search_recipes(
    search_query TEXT DEFAULT NULL,
    dietary_restrictions TEXT[] DEFAULT '{}',
    max_cook_time INTEGER DEFAULT NULL,
    meal_types TEXT[] DEFAULT '{}',
    complexity_levels TEXT[] DEFAULT '{}',
    min_rating DECIMAL DEFAULT 0.0,
    limit_count INTEGER DEFAULT 50,
    offset_count INTEGER DEFAULT 0
)
RETURNS TABLE(
    recipe_id UUID,
    title VARCHAR,
    image_url TEXT,
    total_time INTEGER,
    complexity VARCHAR,
    average_rating DECIMAL,
    relevance_score REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        r.id,
        r.title,
        r.image_url,
        r.total_time,
        r.complexity,
        r.average_rating,
        CASE 
            WHEN search_query IS NOT NULL THEN 
                ts_rank(to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')), 
                        plainto_tsquery('english', search_query))
            ELSE 1.0
        END::REAL as relevance_score
    FROM recipes r
    WHERE r.deleted_at IS NULL
        AND (search_query IS NULL OR 
             to_tsvector('english', r.title || ' ' || COALESCE(r.description, '')) @@ 
             plainto_tsquery('english', search_query))
        AND (array_length(dietary_restrictions, 1) IS NULL OR 
             r.dietary_labels && dietary_restrictions)
        AND (max_cook_time IS NULL OR r.total_time <= max_cook_time)
        AND (array_length(meal_types, 1) IS NULL OR r.meal_type && meal_types)
        AND (array_length(complexity_levels, 1) IS NULL OR r.complexity = ANY(complexity_levels))
        AND r.average_rating >= min_rating
    ORDER BY relevance_score DESC, r.average_rating DESC
    LIMIT limit_count
    OFFSET offset_count;
END;
$$ LANGUAGE plpgsql;
```
