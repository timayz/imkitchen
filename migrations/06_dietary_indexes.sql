-- Migration 06: Dietary Tags Performance Indexes - ALL Combinations
-- Created: 2025-10-22
-- Purpose: Add indexes for ALL dietary tag combinations to improve recipe discovery performance
-- Strategy: Create partial indexes for all possible combinations of supported dietary tags

-- Supported dietary tags (from DietaryTagDetector in crates/recipe/src/tagging.rs):
-- 1. vegetarian
-- 2. vegan
-- 3. gluten-free

-- =============================================================================
-- SINGLE TAG INDEXES (3 indexes) - C(3,1)
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_vegetarian
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%vegetarian%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_vegan
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%vegan%';

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_gluten_free
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL AND dietary_tags LIKE '%gluten-free%';

-- =============================================================================
-- PAIR COMBINATIONS (3 indexes) - C(3,2)
-- =============================================================================

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

-- =============================================================================
-- TRIPLE COMBINATION (1 index) - C(3,3)
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_recipes_dietary_all_three
ON recipes(is_shared, deleted_at)
WHERE is_shared = 1 AND deleted_at IS NULL
  AND dietary_tags LIKE '%vegetarian%'
  AND dietary_tags LIKE '%vegan%'
  AND dietary_tags LIKE '%gluten-free%';

-- =============================================================================
-- SUMMARY: 7 total indexes covering ALL combinations of 3 supported dietary tags
-- - 3 single tag indexes
-- - 3 pair combinations
-- - 1 all-three combination
--
-- Benefits:
-- - Partial indexes (only shared, non-deleted recipes) minimize disk space
-- - SQLite query planner will use appropriate index based on WHERE clause
-- - Covers 100% of possible dietary filter combinations (2^3 - 1 = 7)
-- - Fast lookups even with 100K+ recipes in database
-- =============================================================================
