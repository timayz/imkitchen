-- Migration 03: Shopping List System
-- Story 4.1: Generate Weekly Shopping List
-- Creates shopping list schema with categorization and aggregation support

-- =============================================================================
-- TABLES
-- =============================================================================

-- Shopping lists table: one list per meal plan/week
CREATE TABLE IF NOT EXISTS shopping_lists (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    meal_plan_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,                   -- ISO 8601 date (Monday of the week)
    generated_at TEXT NOT NULL,                       -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id)
);

-- Shopping list items table: aggregated and categorized ingredients
CREATE TABLE IF NOT EXISTS shopping_list_items (
    id TEXT PRIMARY KEY NOT NULL,
    shopping_list_id TEXT NOT NULL,
    ingredient_name TEXT NOT NULL,
    quantity REAL NOT NULL,                           -- Normalized quantity (e.g., 480.0 for 2 cups)
    unit TEXT NOT NULL,                               -- Normalized unit (e.g., "ml", "g", "item")
    category TEXT NOT NULL CHECK(category IN ('Produce', 'Dairy', 'Meat', 'Pantry', 'Frozen', 'Bakery', 'Other')),
    is_collected INTEGER NOT NULL DEFAULT 0,          -- Boolean: 0 = false, 1 = true (Story 4.2)
    FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists(id) ON DELETE CASCADE
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- Shopping lists indexes (optimized for common queries)
CREATE INDEX IF NOT EXISTS idx_shopping_lists_user
    ON shopping_lists(user_id);                       -- Optimizes: get user's shopping lists

CREATE INDEX IF NOT EXISTS idx_shopping_lists_meal_plan
    ON shopping_lists(meal_plan_id);                  -- Optimizes: get shopping list for meal plan

CREATE INDEX IF NOT EXISTS idx_shopping_lists_week
    ON shopping_lists(user_id, week_start_date);      -- Optimizes: get shopping list by week

-- Shopping list items indexes
CREATE INDEX IF NOT EXISTS idx_shopping_list_items_list
    ON shopping_list_items(shopping_list_id);         -- Optimizes: get all items for a list

CREATE INDEX IF NOT EXISTS idx_shopping_list_items_category
    ON shopping_list_items(shopping_list_id, category); -- Optimizes: group by category for display
