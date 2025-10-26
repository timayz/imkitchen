-- Rollback Migration 06: v0.8 - Enhanced Meal Planning System
-- Purpose: Reverse all changes from 06_v0.8.sql migration
-- Note: SQLite doesn't support DROP COLUMN, so we use table recreation strategy

-- =============================================================================
-- PART 1: Drop Triggers
-- =============================================================================

DROP TRIGGER IF EXISTS prevent_locked_week_modification;
DROP TRIGGER IF EXISTS update_meal_plan_status;

-- =============================================================================
-- PART 2: Drop Rotation State Table
-- =============================================================================

DROP INDEX IF EXISTS idx_rotation_state_user;
DROP INDEX IF EXISTS idx_rotation_state_batch;
DROP TABLE IF EXISTS meal_plan_rotation_state;

-- =============================================================================
-- PART 3: Drop Indexes
-- =============================================================================

DROP INDEX IF EXISTS idx_meal_plans_user_batch;
DROP INDEX IF EXISTS idx_meal_plans_status;
DROP INDEX IF EXISTS idx_meal_plans_dates;
DROP INDEX IF EXISTS idx_meal_assignments_accompaniment;
DROP INDEX IF EXISTS idx_recipes_accompaniment_type;
DROP INDEX IF EXISTS idx_recipes_accompaniment_category;

-- =============================================================================
-- PART 4: Recreate Tables Without New Columns
-- =============================================================================

-- Recreate meal_plans table without new columns
CREATE TABLE meal_plans_backup AS
SELECT id, user_id, start_date, status, created_at, updated_at
FROM meal_plans;

DROP TABLE meal_plans;

CREATE TABLE meal_plans (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  start_date TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active', 'archived')),
  created_at TEXT NOT NULL,
  updated_at TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO meal_plans SELECT * FROM meal_plans_backup;
DROP TABLE meal_plans_backup;

-- Recreate the unique active constraint that was dropped in 06_v0.8
CREATE UNIQUE INDEX IF NOT EXISTS idx_meal_plans_unique_active
    ON meal_plans(user_id)
    WHERE status = 'active';

-- Recreate meal_assignments table without accompaniment_recipe_id
CREATE TABLE meal_assignments_backup AS
SELECT id, meal_plan_id, date, course_type, recipe_id, prep_required, assignment_reasoning
FROM meal_assignments;

DROP TABLE meal_assignments;

CREATE TABLE meal_assignments (
  id TEXT PRIMARY KEY,
  meal_plan_id TEXT NOT NULL,
  date TEXT NOT NULL,
  course_type TEXT NOT NULL,
  recipe_id TEXT NOT NULL,
  prep_required BOOLEAN DEFAULT FALSE,
  assignment_reasoning TEXT,
  FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
  FOREIGN KEY (recipe_id) REFERENCES recipes(id)
);

INSERT INTO meal_assignments SELECT * FROM meal_assignments_backup;
DROP TABLE meal_assignments_backup;

-- Recreate recipes table without new columns (keep cuisine and dietary_tags which existed before)
CREATE TABLE recipes_backup AS
SELECT id, user_id, title, ingredients, instructions, prep_time_min, cook_time_min,
       serving_size, recipe_type, is_favorite, created_at, updated_at, cuisine, dietary_tags
FROM recipes;

DROP TABLE recipes;

CREATE TABLE recipes (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  title TEXT NOT NULL,
  ingredients TEXT NOT NULL,
  instructions TEXT NOT NULL,
  prep_time_min INTEGER NOT NULL,
  cook_time_min INTEGER NOT NULL,
  serving_size INTEGER NOT NULL,
  recipe_type TEXT NOT NULL CHECK(recipe_type IN ('appetizer', 'main_course', 'dessert', 'accompaniment')),
  is_favorite BOOLEAN DEFAULT FALSE,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  cuisine TEXT,
  dietary_tags TEXT,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

INSERT INTO recipes SELECT * FROM recipes_backup;
DROP TABLE recipes_backup;

-- Recreate idx_recipes_cuisine that existed before this migration (from 01_v0.2.sql)
CREATE INDEX IF NOT EXISTS idx_recipes_cuisine ON recipes(cuisine) WHERE cuisine IS NOT NULL;

-- Recreate users table without new columns
CREATE TABLE users_backup AS
SELECT id, email, password_hash, tier, recipe_count, created_at, updated_at, onboarding_completed
FROM users;

DROP TABLE users;

CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  tier TEXT NOT NULL CHECK(tier IN ('free', 'pro')) DEFAULT 'free',
  recipe_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  onboarding_completed BOOLEAN DEFAULT FALSE
);

INSERT INTO users SELECT * FROM users_backup;
DROP TABLE users_backup;
