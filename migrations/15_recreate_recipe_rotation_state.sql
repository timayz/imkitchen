-- Recreate recipe_rotation_state table
-- This table tracks which recipes have been used in each rotation cycle

CREATE TABLE IF NOT EXISTS recipe_rotation_state (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    cycle_number INTEGER NOT NULL,
    recipe_id TEXT NOT NULL,
    used_at TEXT NOT NULL,                      -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    -- Note: No FK to recipes table (dropped in migration 09)
    UNIQUE (user_id, cycle_number, recipe_id)   -- Prevent duplicate rotation entries
);

CREATE INDEX IF NOT EXISTS idx_recipe_rotation_user_cycle ON recipe_rotation_state(user_id, cycle_number);
