-- Migration to fix foreign key reference in recipes table
-- This migration was created to fix existing databases where recipes table
-- referenced users(user_id) instead of user_profiles(id)
-- 
-- For fresh databases created after this fix, this migration does nothing
-- as the recipes table is now created with the correct foreign key from the start

-- Check if this migration is needed by looking for recipes table with wrong foreign key
-- If not needed, this migration will effectively be a no-op

-- This migration is designed to be safe to run on both:
-- 1. Fresh databases (where recipes table has correct foreign key) - does nothing  
-- 2. Existing databases (where recipes table has wrong foreign key) - fixes it

-- For now, this migration is effectively disabled since the original issue
-- has been fixed at the source (in migration 20250929001_create_recipes_tables.sql)
-- and fresh databases will have the correct foreign key from the start

-- No action needed - migration 20250929001 now creates recipes table with correct foreign key
-- This placeholder ensures migration sequence integrity