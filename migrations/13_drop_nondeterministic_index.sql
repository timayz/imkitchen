-- Drop non-deterministic index that uses DATE('now') in WHERE clause
-- This causes "non-deterministic use of date() in an index" errors
DROP INDEX IF EXISTS idx_dashboard_meals_today;

-- The idx_dashboard_meals_user_date index is sufficient for queries
-- filtering by user_id and date
