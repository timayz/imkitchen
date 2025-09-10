-- Drop view
DROP VIEW IF EXISTS shopping_list_summaries;

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_shopping_lists_updated_at ON shopping_lists;
DROP TRIGGER IF EXISTS trigger_shopping_items_updated_at ON shopping_items;

-- Drop functions
DROP FUNCTION IF EXISTS update_shopping_lists_updated_at();
DROP FUNCTION IF EXISTS update_shopping_items_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_shopping_lists_user_id;
DROP INDEX IF EXISTS idx_shopping_lists_meal_plan_id;
DROP INDEX IF EXISTS idx_shopping_lists_status;
DROP INDEX IF EXISTS idx_shopping_lists_created_at;

DROP INDEX IF EXISTS idx_shopping_items_list_id;
DROP INDEX IF EXISTS idx_shopping_items_category;
DROP INDEX IF EXISTS idx_shopping_items_completed;
DROP INDEX IF EXISTS idx_shopping_items_ingredient_name;

-- Drop tables
DROP TABLE IF EXISTS shopping_items;
DROP TABLE IF EXISTS shopping_lists;