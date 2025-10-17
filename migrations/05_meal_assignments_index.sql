-- Migration: Add index for today's meals query performance (Story 3.9 Review Action Item #1)
-- Optimizes the get_todays_meals() query which filters by meal_plan_id and date
CREATE INDEX IF NOT EXISTS idx_meal_assignments_plan_date
ON meal_assignments(meal_plan_id, date);
