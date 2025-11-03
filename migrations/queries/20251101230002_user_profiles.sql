-- User profiles projection table for meal planning preferences
CREATE TABLE IF NOT EXISTS user_profiles (
    user_id TEXT PRIMARY KEY,
    dietary_restrictions TEXT,  -- JSON array: ["gluten-free", "vegan"]
    cuisine_variety_weight REAL NOT NULL DEFAULT 0.7,
    household_size INTEGER,
    is_premium_active BOOLEAN NOT NULL DEFAULT 0,
    premium_bypass BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Index for user_id lookups
CREATE INDEX IF NOT EXISTS idx_user_profiles_user_id ON user_profiles(user_id);
