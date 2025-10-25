-- Rollback Migration 06: v0.8 - Enhanced Meal Planning System
-- Created: 2025-10-25
-- Purpose: Rollback all changes from 06_v0.8.sql migration
-- WARNING: This will remove all multi-week meal plans, accompaniments, and user preferences data

-- =============================================================================
-- PART 1: Drop Triggers
-- =============================================================================

DROP TRIGGER IF EXISTS prevent_locked_week_modification;
DROP TRIGGER IF EXISTS update_meal_plan_status;

-- =============================================================================
-- PART 2: Drop Tables
-- =============================================================================

DROP TABLE IF EXISTS meal_plan_rotation_state;

-- =============================================================================
-- PART 3: Drop Indexes (in reverse order of creation)
-- =============================================================================

DROP INDEX IF EXISTS idx_rotation_state_batch;
DROP INDEX IF EXISTS idx_rotation_state_user;
-- Note: idx_recipes_cuisine NOT dropped - it existed before this migration (01_v0.2.sql)
DROP INDEX IF EXISTS idx_recipes_accompaniment_category;
DROP INDEX IF EXISTS idx_recipes_accompaniment_type;
DROP INDEX IF EXISTS idx_meal_assignments_accompaniment;
DROP INDEX IF EXISTS idx_meal_plans_dates;
DROP INDEX IF EXISTS idx_meal_plans_status;
DROP INDEX IF EXISTS idx_meal_plans_user_batch;

-- =============================================================================
-- PART 4: Drop Columns (in reverse order of creation)
-- =============================================================================

-- PART 3 columns (User Preferences only - NOT Recipe metadata which existed before)
-- Note: cuisine and dietary_tags columns NOT dropped - they existed before this migration (01_v0.2.sql)
ALTER TABLE users DROP COLUMN cuisine_variety_weight;
ALTER TABLE users DROP COLUMN avoid_consecutive_complex;
ALTER TABLE users DROP COLUMN max_prep_time_weekend;
ALTER TABLE users DROP COLUMN max_prep_time_weeknight;

-- PART 2 columns (Accompaniment support)
ALTER TABLE meal_assignments DROP COLUMN accompaniment_recipe_id;
ALTER TABLE recipes DROP COLUMN accompaniment_category;
ALTER TABLE recipes DROP COLUMN preferred_accompaniments;
ALTER TABLE recipes DROP COLUMN accepts_accompaniment;

-- PART 1 columns (Multi-week support)
ALTER TABLE meal_plans DROP COLUMN generation_batch_id;
ALTER TABLE meal_plans DROP COLUMN is_locked;
ALTER TABLE meal_plans DROP COLUMN end_date;
