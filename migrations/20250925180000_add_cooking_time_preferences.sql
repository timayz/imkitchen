-- Add cooking time preferences to users table
ALTER TABLE users ADD COLUMN cooking_time_preferences TEXT DEFAULT '{"weekdayMaxMinutes": 30, "weekendMaxMinutes": 60}';