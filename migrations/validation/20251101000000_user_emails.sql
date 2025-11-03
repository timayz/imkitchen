-- User emails validation table for consistency checks in command handlers
CREATE TABLE IF NOT EXISTS user_emails (
    email TEXT PRIMARY KEY,
    user_id TEXT NOT NULL
);

-- Index for reverse lookups
CREATE INDEX IF NOT EXISTS idx_user_emails_user_id ON user_emails(user_id);
