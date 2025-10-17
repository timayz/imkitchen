-- Migration 03: Recipe Rotation State Table
-- Story 3.3: Recipe Rotation System
-- Tracks which recipes have been used in each rotation cycle

-- Recipe rotation state table
CREATE TABLE IF NOT EXISTS recipe_rotation_state (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL,                      -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (recipe_id) REFERENCES recipes(id),
    UNIQUE (user_id, cycle_number, recipe_id)  -- Prevent duplicate rotation entries
);

-- Indexes for recipe_rotation_state
CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_cycle ON recipe_rotation_state(user_id, cycle_number);
CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_id ON recipe_rotation_state(user_id);
