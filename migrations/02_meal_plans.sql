-- Migration 02: Meal Plans and Assignments Tables
-- Story 3.1: Generate Initial Meal Plan
-- Creates meal_plans and meal_assignments tables for meal planning functionality

-- Meal plans table
CREATE TABLE IF NOT EXISTS meal_plans (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    start_date TEXT NOT NULL,               -- ISO 8601 date (Monday of the week)
    status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
    rotation_state TEXT,                    -- JSON: {"cycle_number": 1, "used_recipe_ids": ["id1", "id2"]}
    created_at TEXT NOT NULL,               -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Indexes for meal_plans
CREATE INDEX IF NOT EXISTS idx_meal_plans_user_id ON meal_plans(user_id);
CREATE INDEX IF NOT EXISTS idx_meal_plans_user_status ON meal_plans(user_id, status);

-- Meal assignments table (7 days × 3 meals = 21 assignments per plan)
CREATE TABLE IF NOT EXISTS meal_assignments (
    id TEXT PRIMARY KEY NOT NULL,
    meal_plan_id TEXT NOT NULL,
    date TEXT NOT NULL,                     -- ISO 8601 date (YYYY-MM-DD)
    meal_type TEXT NOT NULL CHECK(meal_type IN ('breakfast', 'lunch', 'dinner')),
    recipe_id TEXT NOT NULL,
    prep_required INTEGER NOT NULL DEFAULT 0, -- SQLite boolean (0 = false, 1 = true)
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);

-- Indexes for meal_assignments
CREATE INDEX IF NOT EXISTS idx_meal_assignments_meal_plan_id ON meal_assignments(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_meal_assignments_date ON meal_assignments(meal_plan_id, date);
CREATE INDEX IF NOT EXISTS idx_meal_assignments_recipe_id ON meal_assignments(recipe_id);
