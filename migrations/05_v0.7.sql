-- Migration 05: v0.7 - Contact Forms, Dietary Indexes, Meal Plan Tracking
-- Created: 2025-10-22
-- Merged from: 05_v0.7.sql, 06_dietary_indexes.sql, 07_remove_skill_level.sql, 08_add_meal_plan_updated_at.sql
-- Purpose: Contact form submissions, dietary tag performance indexes, meal plan regeneration tracking

-- =============================================================================
-- CONTACT FORM SUBMISSIONS (Original 05_v0.7.sql)
-- =============================================================================

-- Contact form submissions table
CREATE TABLE IF NOT EXISTS contact_submissions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT, -- Optional: NULL if not authenticated
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT NOT NULL, -- support, bug, feature, account, billing, feedback, other
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'pending', -- pending, read, responded, archived
    responded_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Index for querying submissions by status
CREATE INDEX IF NOT EXISTS idx_contact_submissions_status ON contact_submissions(status);

-- Index for querying submissions by user
CREATE INDEX IF NOT EXISTS idx_contact_submissions_user_id ON contact_submissions(user_id) WHERE user_id IS NOT NULL;

-- Index for querying submissions by created date
CREATE INDEX IF NOT EXISTS idx_contact_submissions_created_at ON contact_submissions(created_at DESC);

-- =============================================================================
-- DIETARY TAG PERFORMANCE INDEXES (From 06_dietary_indexes.sql)
-- =============================================================================
-- Purpose: Add indexes for ALL dietary tag combinations to improve recipe discovery performance
-- Strategy: Create partial indexes for all possible combinations of supported dietary tags
-- Supported dietary tags: vegetarian, vegan, gluten-free

-- SINGLE TAG INDEXES (3 indexes) - C(3,1)
CREATE INDEX IF NOT EXISTS idx_recipes_dietary_vegetarian
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%vegetarian%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_vegan
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%vegan%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_gluten_free
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%gluten-free%';

-- PAIR COMBINATIONS (3 indexes) - C(3,2)
CREATE INDEX IF NOT EXISTS idx_recipes_dietary_veg_vegan
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL
  AND dietary_tags LIKE '%vegetarian%'
  AND dietary_tags LIKE '%vegan%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_veg_gf
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL
  AND dietary_tags LIKE '%vegetarian%'
  AND dietary_tags LIKE '%gluten-free%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_vegan_gf
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL
  AND dietary_tags LIKE '%vegan%'
  AND dietary_tags LIKE '%gluten-free%';

-- TRIPLE COMBINATION (1 index) - C(3,3)
CREATE INDEX IF NOT EXISTS idx_recipes_dietary_all_three
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL
  AND dietary_tags LIKE '%vegetarian%'
  AND dietary_tags LIKE '%vegan%'
  AND dietary_tags LIKE '%gluten-free%';

-- SUMMARY: 7 total indexes covering ALL combinations of 3 supported dietary tags

-- =============================================================================
-- MEAL PLAN REGENERATION TRACKING (From 08_add_meal_plan_updated_at.sql)
-- =============================================================================
-- Add updated_at field to meal_plans table to track regeneration timestamp
-- This enables polling to verify the meal plan was actually regenerated

ALTER TABLE meal_plans ADD COLUMN updated_at TEXT;

-- =============================================================================
-- NOTE: Migration 07 (remove skill_level) is NOT needed
-- skill_level column was never added after migration 00_v0.1.sql
-- Migration 01_v0.2.sql added favorite_count without skill_level
-- Migration 03_v0.4.sql added notification fields without skill_level
-- Therefore no DROP COLUMN operation is required
-- =============================================================================
