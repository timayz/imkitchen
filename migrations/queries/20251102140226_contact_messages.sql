-- Contact messages projection table for admin inbox
CREATE TABLE IF NOT EXISTS contact_messages (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT NOT NULL,
    message TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new',  -- 'new' | 'read' | 'resolved'
    created_at INTEGER NOT NULL
);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_contact_messages_status ON contact_messages(status);

-- Index for sorting by creation date (newest first)
CREATE INDEX IF NOT EXISTS idx_contact_messages_created_at ON contact_messages(created_at DESC);
