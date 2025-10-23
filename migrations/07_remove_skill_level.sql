-- Migration: Remove skill_level column from users table
-- Reason: skill_level is not used by any constraint or business logic

-- SQLite doesn't support DROP COLUMN directly before version 3.35.0
-- We need to recreate the table without the skill_level column

-- 1. Create new users table without skill_level
CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    dietary_restrictions TEXT,
    household_size INTEGER,
    weeknight_availability TEXT,
    onboarding_completed INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login TEXT,
    tier TEXT NOT NULL DEFAULT 'free',
    recipe_count INTEGER NOT NULL DEFAULT 0,
    favorite_count INTEGER NOT NULL DEFAULT 0,
    notification_permission_status TEXT,
    last_permission_denial_at TEXT,
    stripe_customer_id TEXT,
    stripe_subscription_id TEXT
);

-- 2. Copy data from old table to new table (excluding skill_level)
INSERT INTO users_new (
    id,
    email,
    password_hash,
    created_at,
    dietary_restrictions,
    household_size,
    weeknight_availability,
    onboarding_completed,
    updated_at,
    last_login,
    tier,
    recipe_count,
    favorite_count,
    notification_permission_status,
    last_permission_denial_at,
    stripe_customer_id,
    stripe_subscription_id
)
SELECT
    id,
    email,
    password_hash,
    created_at,
    dietary_restrictions,
    household_size,
    weeknight_availability,
    onboarding_completed,
    updated_at,
    last_login,
    tier,
    recipe_count,
    favorite_count,
    notification_permission_status,
    last_permission_denial_at,
    stripe_customer_id,
    stripe_subscription_id
FROM users;

-- 3. Drop old table
DROP TABLE users;

-- 4. Rename new table to users
ALTER TABLE users_new RENAME TO users;

-- 5. Recreate indexes and triggers that were lost during table recreation

-- Index for email lookups (if it existed)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Index for tier queries (if it existed)
CREATE INDEX IF NOT EXISTS idx_users_tier ON users(tier);
