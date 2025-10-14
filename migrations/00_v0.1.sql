-- Migration v0.1
-- Created: 2025-10-14

-- Users table for read model
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    tier TEXT NOT NULL DEFAULT 'free',
    recipe_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    dietary_restrictions TEXT,
    household_size INTEGER,
    skill_level TEXT,
    weeknight_availability TEXT,
    onboarding_completed INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    stripe_customer_id TEXT,
    stripe_subscription_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_stripe_customer ON users(stripe_customer_id);

-- User email uniqueness table for consistent email validation
-- This table is used by the command handler to ensure email uniqueness
-- before committing events to the event store.
--
-- Using a separate table (not the read model) ensures:
-- 1. Consistency - validation happens in the same transaction as event commit
-- 2. No race conditions - unique constraint enforced at database level
-- 3. Separation of concerns - read model can be eventually consistent
CREATE TABLE IF NOT EXISTS user_email_uniqueness (
    email TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    created_at TEXT NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_user_email_uniqueness_user_id ON user_email_uniqueness(user_id);
