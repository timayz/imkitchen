-- Initial schema for imkitchen application
-- Based on data models from architecture/data-models.md

-- Users table
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, -- UUID v4
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    family_size INTEGER,
    dietary_restrictions TEXT, -- JSON array of restrictions
    cooking_skill_level TEXT CHECK(cooking_skill_level IN ('beginner', 'intermediate', 'advanced')),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Recipes table
CREATE TABLE recipes (
    id TEXT PRIMARY KEY NOT NULL, -- UUID v4
    title TEXT NOT NULL,
    description TEXT,
    prep_time_minutes INTEGER,
    cook_time_minutes INTEGER,
    total_time_minutes INTEGER GENERATED ALWAYS AS (prep_time_minutes + cook_time_minutes) STORED,
    difficulty TEXT CHECK(difficulty IN ('easy', 'medium', 'hard')),
    servings INTEGER NOT NULL,
    ingredients TEXT NOT NULL, -- JSON array of ingredients
    instructions TEXT NOT NULL, -- JSON array of instructions
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Meal plans table
CREATE TABLE meal_plans (
    id TEXT PRIMARY KEY NOT NULL, -- UUID v4
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    week_start_date DATE NOT NULL,
    meals TEXT NOT NULL, -- JSON array of meals for the week
    status TEXT CHECK(status IN ('draft', 'active', 'completed', 'cancelled')) DEFAULT 'draft',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Shopping lists table
CREATE TABLE shopping_lists (
    id TEXT PRIMARY KEY NOT NULL, -- UUID v4
    meal_plan_id TEXT NOT NULL REFERENCES meal_plans(id) ON DELETE CASCADE,
    items TEXT NOT NULL, -- JSON array of shopping items
    sharing_config TEXT, -- JSON object for sharing configuration
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_meal_plans_user_id ON meal_plans(user_id);
CREATE INDEX idx_meal_plans_week_start ON meal_plans(week_start_date);
CREATE INDEX idx_shopping_lists_meal_plan_id ON shopping_lists(meal_plan_id);

-- Triggers to update updated_at timestamps
CREATE TRIGGER trigger_users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW
BEGIN
    UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER trigger_recipes_updated_at
BEFORE UPDATE ON recipes
FOR EACH ROW
BEGIN
    UPDATE recipes SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER trigger_meal_plans_updated_at
BEFORE UPDATE ON meal_plans
FOR EACH ROW
BEGIN
    UPDATE meal_plans SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER trigger_shopping_lists_updated_at
BEFORE UPDATE ON shopping_lists
FOR EACH ROW
BEGIN
    UPDATE shopping_lists SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;