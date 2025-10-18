-- Migration 02: Meal Planning System (v0.3)
-- Consolidates migrations 02-06: Meal Plans, Rotation, Reasoning, Indexes
-- Stories: 3.1, 3.3, 3.8, 3.9, 3.11
-- Creates complete meal planning schema with optimizations

-- =============================================================================
-- TABLES
-- =============================================================================

-- Meal plans table: one active plan per user
CREATE TABLE IF NOT EXISTS meal_plans (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    start_date TEXT NOT NULL,                   -- ISO 8601 date (Monday of the week)
    status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
    rotation_state TEXT,                        -- JSON: {"cycle_number": 1, "used_recipe_ids": [...]}
    created_at TEXT NOT NULL,                   -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Meal assignments table: 7 days × 3 meals = 21 assignments per plan
CREATE TABLE IF NOT EXISTS meal_assignments (
    id TEXT PRIMARY KEY NOT NULL,
    meal_plan_id TEXT NOT NULL,
    date TEXT NOT NULL,                         -- ISO 8601 date (YYYY-MM-DD)
    meal_type TEXT NOT NULL CHECK(meal_type IN ('breakfast', 'lunch', 'dinner')),
    recipe_id TEXT NOT NULL,
    prep_required INTEGER NOT NULL DEFAULT 0,   -- Boolean: 0 = false, 1 = true
    assignment_reasoning TEXT,                  -- Human-readable reasoning for meal assignment
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);

-- Recipe rotation state table: tracks recipe usage across rotation cycles
CREATE TABLE IF NOT EXISTS recipe_rotation_state (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL,                      -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id),
    UNIQUE (user_id, cycle_number, recipe_id)   -- Prevent duplicate rotation entries
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- Meal plans indexes
CREATE INDEX IF NOT EXISTS idx_meal_plans_user_status
    ON meal_plans(user_id, status);             -- Optimizes: get active plan per user

CREATE UNIQUE INDEX IF NOT EXISTS idx_meal_plans_unique_active
    ON meal_plans(user_id)
    WHERE status = 'active';                    -- Enforces: exactly one active plan per user

-- Meal assignments indexes (optimized for common queries)
CREATE INDEX IF NOT EXISTS idx_meal_assignments_plan_date
    ON meal_assignments(meal_plan_id, date);    -- Optimizes: get today's meals, week view

CREATE INDEX IF NOT EXISTS idx_meal_assignments_recipe
    ON meal_assignments(recipe_id);             -- Optimizes: find meals using specific recipe

-- Recipe rotation state indexes
CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_cycle
    ON recipe_rotation_state(user_id, cycle_number); -- Optimizes: get current cycle recipes
