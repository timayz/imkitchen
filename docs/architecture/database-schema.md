# Database Schema

## User Context Schema (SQLite)

```sql
-- User profile and preferences
CREATE TABLE user_profiles (
    user_id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    family_size INTEGER CHECK(family_size BETWEEN 1 AND 8),
    cooking_skill_level TEXT CHECK(cooking_skill_level IN ('Beginner', 'Intermediate', 'Advanced')),
    weekday_cooking_minutes INTEGER DEFAULT 30,
    weekend_cooking_minutes INTEGER DEFAULT 60,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Dietary restrictions (normalized)
CREATE TABLE dietary_restrictions (
    user_id TEXT REFERENCES user_profiles(user_id),
    restriction_type TEXT NOT NULL,
    severity TEXT DEFAULT 'Strict',
    PRIMARY KEY (user_id, restriction_type)
);

-- Evento manages event storage automatically through its own migrations
-- No manual event tables needed
```

## Recipe Context Schema (SQLite)

```sql
-- Recipe information
CREATE TABLE recipes (
    recipe_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_by TEXT NOT NULL,
    prep_time_minutes INTEGER NOT NULL,
    cook_time_minutes INTEGER NOT NULL,
    difficulty TEXT CHECK(difficulty IN ('Easy', 'Medium', 'Hard')),
    category TEXT NOT NULL,
    is_public BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Recipe ingredients (denormalized for performance)
CREATE TABLE recipe_ingredients (
    recipe_id TEXT REFERENCES recipes(recipe_id),
    ingredient_name TEXT NOT NULL,
    quantity DECIMAL(10,2),
    unit TEXT,
    notes TEXT,
    sort_order INTEGER
);

-- Recipe instructions
CREATE TABLE recipe_instructions (
    recipe_id TEXT REFERENCES recipes(recipe_id),
    step_number INTEGER NOT NULL,
    instruction_text TEXT NOT NULL,
    estimated_minutes INTEGER,
    PRIMARY KEY (recipe_id, step_number)
);

-- Full-text search index
CREATE VIRTUAL TABLE recipe_search USING fts5(
    recipe_id UNINDEXED,
    title,
    ingredients,
    instructions,
    tags
);

-- Evento manages event storage automatically
-- Only application-specific read models and projections needed
```

## Meal Planning Context Schema (SQLite)

```sql
-- Weekly meal plans
CREATE TABLE meal_plans (
    meal_plan_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    week_start_date DATE NOT NULL,
    generation_algorithm TEXT NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Individual meal slots (21 per week)
CREATE TABLE meal_slots (
    meal_plan_id TEXT REFERENCES meal_plans(meal_plan_id),
    day_of_week INTEGER CHECK(day_of_week BETWEEN 0 AND 6),
    meal_type TEXT CHECK(meal_type IN ('Breakfast', 'Lunch', 'Dinner')),
    recipe_id TEXT,
    requires_advance_prep BOOLEAN DEFAULT FALSE,
    prep_start_time TIME,
    PRIMARY KEY (meal_plan_id, day_of_week, meal_type)
);

-- Recipe rotation tracking
CREATE TABLE recipe_usage_history (
    user_id TEXT NOT NULL,
    recipe_id TEXT NOT NULL,
    last_cooked_date DATE,
    cook_count INTEGER DEFAULT 0,
    PRIMARY KEY (user_id, recipe_id)
);

-- Evento handles all event storage - no manual event tables required
```
