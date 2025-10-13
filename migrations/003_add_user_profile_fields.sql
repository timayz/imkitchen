-- Add profile fields to users table for onboarding (Story 1.4)
ALTER TABLE users ADD COLUMN dietary_restrictions TEXT;
ALTER TABLE users ADD COLUMN household_size INTEGER;
ALTER TABLE users ADD COLUMN skill_level TEXT;
ALTER TABLE users ADD COLUMN weeknight_availability TEXT;
ALTER TABLE users ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0;
