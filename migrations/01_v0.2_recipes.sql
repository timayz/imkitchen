-- Migration v0.2 - Recipe Management
-- Created: 2025-10-14
-- Story: 2.1 Create Recipe

-- Recipes table for read model
CREATE TABLE IF NOT EXISTS recipes (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    ingredients TEXT NOT NULL, -- JSON array of {name, quantity, unit}
    instructions TEXT NOT NULL, -- JSON array of {step_number, instruction_text, timer_minutes}
    prep_time_min INTEGER,
    cook_time_min INTEGER,
    advance_prep_hours INTEGER, -- NULL if no advance prep required
    serving_size INTEGER,
    is_favorite INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false, 1=true
    is_shared INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false (private), 1=true (shared to community)
    complexity TEXT, -- "simple", "moderate", "complex" (future enhancement)
    cuisine TEXT, -- "italian", "chinese", etc. (future enhancement)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_recipes_user_id ON recipes(user_id);
CREATE INDEX IF NOT EXISTS idx_recipes_favorite ON recipes(user_id, is_favorite);
CREATE INDEX IF NOT EXISTS idx_recipes_shared ON recipes(is_shared) WHERE is_shared = 1;
