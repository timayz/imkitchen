-- Migration 06: v0.8 - Enhanced Meal Planning System
-- Created: 2025-10-25
-- Purpose: Multi-week meal plan generation, accompaniment recipe type, user preferences integration
-- Features:
--   1. Multi-week meal plan generation with week locking
--   2. Accompaniment recipe type (pasta, rice, fries, salad, bread, vegetables)
--   3. User preferences for algorithm (time constraints, complexity, cuisine variety)
--   4. Rotation state tracking for meal plan algorithm
--   5. Triggers for business rule enforcement

-- =============================================================================
-- PART 1: Multi-Week Meal Plan Support
-- =============================================================================

-- Note: SQLite doesn't support ALTER TABLE ... DROP CONSTRAINT, so we cannot modify the CHECK constraint
-- The existing CHECK constraint allows 'active' or 'archived'
-- We'll add new columns but keep using 'active'/'archived' for status to maintain compatibility
-- Future migrations can recreate the table with updated constraints if needed

ALTER TABLE meal_plans ADD COLUMN end_date TEXT NOT NULL DEFAULT '';
ALTER TABLE meal_plans ADD COLUMN is_locked BOOLEAN DEFAULT FALSE;
ALTER TABLE meal_plans ADD COLUMN generation_batch_id TEXT;

CREATE INDEX IF NOT EXISTS idx_meal_plans_user_batch ON meal_plans(user_id, generation_batch_id);
CREATE INDEX IF NOT EXISTS idx_meal_plans_status ON meal_plans(user_id, status);
CREATE INDEX IF NOT EXISTS idx_meal_plans_dates ON meal_plans(start_date, end_date);

-- =============================================================================
-- PART 2: Accompaniment Recipe Type
-- =============================================================================

ALTER TABLE recipes ADD COLUMN accepts_accompaniment BOOLEAN DEFAULT FALSE;
ALTER TABLE recipes ADD COLUMN preferred_accompaniments TEXT;
ALTER TABLE recipes ADD COLUMN accompaniment_category TEXT;

ALTER TABLE meal_assignments ADD COLUMN accompaniment_recipe_id TEXT;

CREATE INDEX IF NOT EXISTS idx_meal_assignments_accompaniment ON meal_assignments(accompaniment_recipe_id);
CREATE INDEX IF NOT EXISTS idx_recipes_accompaniment_type ON recipes(recipe_type) WHERE recipe_type = 'accompaniment';
CREATE INDEX IF NOT EXISTS idx_recipes_accompaniment_category ON recipes(accompaniment_category) WHERE accompaniment_category IS NOT NULL;

-- =============================================================================
-- PART 3: User Preferences for Algorithm
-- =============================================================================

ALTER TABLE users ADD COLUMN max_prep_time_weeknight INTEGER DEFAULT 30;
ALTER TABLE users ADD COLUMN max_prep_time_weekend INTEGER DEFAULT 90;
ALTER TABLE users ADD COLUMN avoid_consecutive_complex BOOLEAN DEFAULT TRUE;
ALTER TABLE users ADD COLUMN cuisine_variety_weight REAL DEFAULT 0.7;

-- Note: cuisine and dietary_tags columns already exist in recipes table from migration 01_v0.2.sql
-- Note: idx_recipes_cuisine index already exists from migration 01_v0.2.sql

-- =============================================================================
-- PART 4: Rotation State Tracking
-- =============================================================================

CREATE TABLE IF NOT EXISTS meal_plan_rotation_state (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  generation_batch_id TEXT NOT NULL,
  used_main_course_ids TEXT NOT NULL,
  used_appetizer_ids TEXT NOT NULL,
  used_dessert_ids TEXT NOT NULL,
  cuisine_usage_count TEXT NOT NULL,
  last_complex_meal_date TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_rotation_state_user ON meal_plan_rotation_state(user_id);
CREATE INDEX IF NOT EXISTS idx_rotation_state_batch ON meal_plan_rotation_state(generation_batch_id);

-- =============================================================================
-- PART 5: Triggers for Business Rules
-- =============================================================================

-- Prevent modification of locked weeks
CREATE TRIGGER IF NOT EXISTS prevent_locked_week_modification
BEFORE UPDATE ON meal_plans
WHEN OLD.is_locked = TRUE
BEGIN
    SELECT RAISE(FAIL, 'Cannot modify locked meal plan week');
END;

-- Auto-update meal plan status based on dates
-- Note: We use 'active'/'archived' to comply with existing CHECK constraint
CREATE TRIGGER IF NOT EXISTS update_meal_plan_status
AFTER UPDATE ON meal_plans
WHEN NEW.start_date != OLD.start_date OR NEW.end_date != OLD.end_date
BEGIN
    UPDATE meal_plans
    SET status = CASE
        -- Mark as archived if the week has ended
        WHEN date(NEW.end_date) < date('now') THEN 'archived'
        -- Otherwise keep as active (covers current and future weeks)
        ELSE 'active'
    END
    WHERE id = NEW.id;
END;

-- =============================================================================
-- Data Migration: Update Existing Meal Plans
-- =============================================================================

-- Update existing meal plans with end_date
UPDATE meal_plans
SET end_date = date(start_date, '+6 days')
WHERE end_date = '';

-- Mark current/past weeks as locked
UPDATE meal_plans
SET is_locked = TRUE
WHERE date(start_date) <= date('now');

-- Update status for existing meal plans (using 'active'/'archived' only)
UPDATE meal_plans
SET status = CASE
    -- Archive past weeks
    WHEN date(end_date) < date('now') THEN 'archived'
    -- Keep current and future weeks as active
    ELSE 'active'
END;
