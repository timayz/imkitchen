-- Create user_email_uniqueness table for consistent email validation
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

CREATE INDEX idx_user_email_uniqueness_user_id ON user_email_uniqueness(user_id);
