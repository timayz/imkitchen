-- Recreate meal_assignments table
-- This table stores all meal assignments for a meal plan (not just today's)
-- It serves as the source of truth for meal plan composition

CREATE TABLE IF NOT EXISTS meal_assignments (
    id TEXT PRIMARY KEY,
    meal_plan_id TEXT NOT NULL,
    date TEXT NOT NULL,
    course_type TEXT NOT NULL CHECK(course_type IN ('appetizer', 'main_course', 'dessert')),
    recipe_id TEXT NOT NULL,
    prep_required INTEGER NOT NULL DEFAULT 0,
    assignment_reasoning TEXT,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE
    -- Note: No FK to recipes table (dropped in migration 09)
);

CREATE INDEX IF NOT EXISTS idx_meal_assignments_plan ON meal_assignments(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_meal_assignments_date ON meal_assignments(meal_plan_id, date);
