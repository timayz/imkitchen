-- Migration 03: Add unique constraint for single active meal plan per user
-- Story 3.11: Meal Plan Persistence and Activation (AC #2, #10)
-- Ensures exactly one meal plan with status='active' per user_id at database level

-- Create unique partial index to enforce single active meal plan constraint
-- This prevents multiple active plans for the same user, even under race conditions
-- WHERE clause ensures index only applies to active plans (archived plans exempt)
CREATE UNIQUE INDEX IF NOT EXISTS idx_meal_plans_unique_active
ON meal_plans(user_id)
WHERE status = 'active';
