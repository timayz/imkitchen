-- Add updated_at field to users table for tracking profile changes (Story 1.5 - AC-7)
-- NOT NULL constraint with DEFAULT ensures all rows have audit trail timestamps
ALTER TABLE users ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

-- Backfill existing users with created_at as initial updated_at value
-- (redundant with DEFAULT but explicit for data integrity)
UPDATE users SET updated_at = COALESCE(created_at, datetime('now')) WHERE updated_at IS NULL;
