-- Add cooking time preferences to user profiles
-- Extends the user profile system to track preferred cooking times

-- Add cooking time columns to user_profiles table
ALTER TABLE user_profiles ADD COLUMN weekday_cooking_minutes INTEGER DEFAULT 30;
ALTER TABLE user_profiles ADD COLUMN weekend_cooking_minutes INTEGER DEFAULT 60;

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_profiles_cooking_times ON user_profiles(weekday_cooking_minutes, weekend_cooking_minutes);