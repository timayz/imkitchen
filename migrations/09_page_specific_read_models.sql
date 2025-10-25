-- Migration 09: Page-Specific Read Models
-- Created: 2025-10-24
-- Story: Read Model Migration to Page-Specific Architecture
-- Strategy: MVP - Fresh start with page-specific tables (replaces domain-centric tables)

-- =============================================================================
-- DROP OLD DOMAIN-CENTRIC READ MODEL TABLES
-- =============================================================================

-- Drop old tables that are replaced by page-specific read models
DROP TABLE IF EXISTS recipes;
DROP TABLE IF EXISTS meal_assignments;
DROP TABLE IF EXISTS recipe_rotation_state;
DROP TABLE IF EXISTS ratings;
DROP TABLE IF EXISTS shopping_list_items;

-- Note: Keep these tables (not replaced by page-specific models):
-- - users (auth & profile - unchanged)
-- - meal_plans (plan metadata - still needed)
-- - shopping_lists (list headers - still needed)
-- - recipe_collections (already page-specific)
-- - recipe_collection_assignments (already page-specific)
-- - notifications (prep reminders - unchanged)
-- - push_subscriptions (web push - unchanged)
-- - contact_submissions (support - unchanged)
-- - user_email_uniqueness (command validation - unchanged)

-- =============================================================================
-- DASHBOARD PAGE READ MODELS
-- =============================================================================

-- Today's meal assignments with denormalized recipe data
CREATE TABLE IF NOT EXISTS dashboard_meals (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,                       -- ISO 8601 date (YYYY-MM-DD)
    course_type TEXT NOT NULL CHECK(course_type IN ('appetizer', 'main_course', 'dessert')),
    recipe_id TEXT NOT NULL,
    recipe_title TEXT NOT NULL,
    recipe_image_url TEXT,
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    prep_required INTEGER NOT NULL DEFAULT 0, -- Boolean: 0 = false, 1 = true
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dashboard_meals_user_date ON dashboard_meals(user_id, date);
CREATE INDEX IF NOT EXISTS idx_dashboard_meals_today ON dashboard_meals(user_id, date) WHERE date = DATE('now');

-- Today's prep tasks
CREATE TABLE IF NOT EXISTS dashboard_prep_tasks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    task_date TEXT NOT NULL,                  -- Date prep should be done
    meal_date TEXT NOT NULL,                  -- Date meal is scheduled
    recipe_id TEXT NOT NULL,
    recipe_title TEXT NOT NULL,
    prep_description TEXT,
    advance_prep_hours INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dashboard_prep_tasks_user_date ON dashboard_prep_tasks(user_id, task_date);
CREATE INDEX IF NOT EXISTS idx_dashboard_prep_tasks_today ON dashboard_prep_tasks(user_id, task_date) WHERE task_date = DATE('now');

-- User metrics (recipe variety, favorites, etc.)
CREATE TABLE IF NOT EXISTS dashboard_metrics (
    user_id TEXT PRIMARY KEY,
    recipe_count INTEGER NOT NULL DEFAULT 0,
    favorite_count INTEGER NOT NULL DEFAULT 0,
    cuisine_variety_count INTEGER NOT NULL DEFAULT 0,
    last_plan_generated_at TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- =============================================================================
-- MEAL CALENDAR PAGE READ MODELS
-- =============================================================================

-- Week view meal assignments (7 days × 3 courses = 21 rows per week)
CREATE TABLE IF NOT EXISTS calendar_view (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    meal_plan_id TEXT NOT NULL,
    date TEXT NOT NULL,                       -- ISO 8601 date
    course_type TEXT NOT NULL CHECK(course_type IN ('appetizer', 'main_course', 'dessert')),
    recipe_id TEXT NOT NULL,
    recipe_title TEXT NOT NULL,
    recipe_image_url TEXT,
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    assignment_reasoning TEXT,                -- Human-readable explanation
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_calendar_view_user_plan ON calendar_view(user_id, meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_calendar_view_user_date ON calendar_view(user_id, date);

-- =============================================================================
-- RECIPE LIBRARY PAGE READ MODELS
-- =============================================================================

-- Recipe cards for library list view
CREATE TABLE IF NOT EXISTS recipe_list (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    recipe_type TEXT NOT NULL,                -- "appetizer" | "main_course" | "dessert"
    image_url TEXT,
    complexity TEXT,                          -- "simple" | "moderate" | "complex"
    cuisine TEXT,
    dietary_tags TEXT,                        -- JSON array
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    is_favorite INTEGER NOT NULL DEFAULT 0,
    is_shared INTEGER NOT NULL DEFAULT 0,
    avg_rating REAL,                          -- Denormalized from ratings
    rating_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,                          -- Soft delete
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recipe_list_user ON recipe_list(user_id, deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_list_favorite ON recipe_list(user_id, is_favorite) WHERE is_favorite = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_list_shared ON recipe_list(is_shared) WHERE is_shared = 1 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_list_complexity ON recipe_list(complexity) WHERE complexity IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_list_cuisine ON recipe_list(cuisine) WHERE cuisine IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_list_type ON recipe_list(recipe_type);

-- Filter facet counts (e.g., "Simple: 12", "Moderate: 8")
CREATE TABLE IF NOT EXISTS recipe_filter_counts (
    user_id TEXT NOT NULL,
    filter_type TEXT NOT NULL,                -- "complexity" | "cuisine" | "recipe_type" | "dietary_tag"
    filter_value TEXT NOT NULL,               -- e.g., "simple", "Italian", "vegetarian"
    count INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (user_id, filter_type, filter_value),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recipe_filter_counts_user ON recipe_filter_counts(user_id);

-- =============================================================================
-- RECIPE DETAIL PAGE READ MODELS
-- =============================================================================

-- Full recipe data for detail view
CREATE TABLE IF NOT EXISTS recipe_detail (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    recipe_type TEXT NOT NULL,
    ingredients TEXT NOT NULL,                -- JSON array of {name, quantity, unit}
    instructions TEXT NOT NULL,               -- JSON array of {step_number, text, timer_minutes}
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    advance_prep_hours INTEGER,
    serving_size INTEGER,
    complexity TEXT,
    cuisine TEXT,
    dietary_tags TEXT,                        -- JSON array
    is_favorite INTEGER NOT NULL DEFAULT 0,
    is_shared INTEGER NOT NULL DEFAULT 0,
    original_recipe_id TEXT,                  -- If copied from community
    original_author TEXT,                     -- Original creator name
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_recipe_detail_user ON recipe_detail(user_id, deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_recipe_detail_shared ON recipe_detail(is_shared) WHERE is_shared = 1 AND deleted_at IS NULL;

-- Aggregated ratings for detail view
CREATE TABLE IF NOT EXISTS recipe_ratings (
    recipe_id TEXT PRIMARY KEY,
    avg_stars REAL NOT NULL DEFAULT 0.0,
    rating_count INTEGER NOT NULL DEFAULT 0,
    five_star_count INTEGER NOT NULL DEFAULT 0,
    four_star_count INTEGER NOT NULL DEFAULT 0,
    three_star_count INTEGER NOT NULL DEFAULT 0,
    two_star_count INTEGER NOT NULL DEFAULT 0,
    one_star_count INTEGER NOT NULL DEFAULT 0,
    recent_reviews TEXT,                      -- JSON array of 3 most recent reviews
    updated_at TEXT NOT NULL
);

-- =============================================================================
-- SHOPPING LIST PAGE READ MODELS
-- =============================================================================

-- Categorized shopping items with checkoff state
CREATE TABLE IF NOT EXISTS shopping_list_view (
    id TEXT PRIMARY KEY,
    shopping_list_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,            -- Monday of the week
    ingredient_name TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit TEXT NOT NULL,
    category TEXT,                            -- "produce" | "dairy" | "meat" | "pantry" | "frozen" | "bakery"
    is_collected INTEGER NOT NULL DEFAULT 0,
    source_recipes TEXT,                      -- JSON array of {recipe_id, recipe_title}
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_shopping_list_view_user_week ON shopping_list_view(user_id, week_start_date);
CREATE INDEX IF NOT EXISTS idx_shopping_list_view_category ON shopping_list_view(shopping_list_id, category);

-- Category totals and completion progress
CREATE TABLE IF NOT EXISTS shopping_list_summary (
    shopping_list_id TEXT NOT NULL,
    category TEXT NOT NULL,
    total_items INTEGER NOT NULL DEFAULT 0,
    collected_items INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (shopping_list_id, category)
);

CREATE INDEX IF NOT EXISTS idx_shopping_list_summary_list ON shopping_list_summary(shopping_list_id);
