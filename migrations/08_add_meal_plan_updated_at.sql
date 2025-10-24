-- Add updated_at field to meal_plans table to track regeneration timestamp
-- This enables polling to verify the meal plan was actually regenerated

ALTER TABLE meal_plans ADD COLUMN updated_at TEXT;

-- Backfill existing records with created_at value
UPDATE meal_plans SET updated_at = created_at WHERE updated_at IS NULL;
