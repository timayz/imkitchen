-- Migration 04: Course-Based Meal Planning Model (v0.5)
-- Change from breakfast/lunch/dinner to appetizer/main_course/dessert
-- Date: 2025-10-20
--
-- ALPHA PROJECT: Simple add/drop column migration

-- Add recipe_type column to recipes table
ALTER TABLE recipes ADD COLUMN recipe_type TEXT NOT NULL DEFAULT 'main_course';

-- Add course_type column to meal_assignments table
ALTER TABLE meal_assignments ADD COLUMN course_type TEXT NOT NULL DEFAULT 'main_course';

-- Drop old meal_type column and its index
DROP INDEX IF EXISTS idx_meal_assignments_date;
ALTER TABLE meal_assignments DROP COLUMN meal_type;

-- Create indexes for course-based queries
CREATE INDEX IF NOT EXISTS idx_recipes_type ON recipes(recipe_type);
CREATE INDEX IF NOT EXISTS idx_recipes_user_type ON recipes(user_id, recipe_type) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipes_favorite_type ON recipes(user_id, is_favorite, recipe_type) WHERE is_favorite = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_meal_assignments_course ON meal_assignments(meal_plan_id, course_type);
