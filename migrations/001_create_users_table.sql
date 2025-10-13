-- Create users table for read model
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    tier TEXT NOT NULL DEFAULT 'free',
    recipe_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Index for email lookups (uniqueness check)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
