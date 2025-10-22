-- Migration 05: Contact Form Submissions (v0.7)
-- Created: 2025-10-22
-- Purpose: Add contact form submission tracking

-- Contact form submissions table
CREATE TABLE IF NOT EXISTS contact_submissions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT, -- Optional: NULL if not authenticated
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    subject TEXT NOT NULL, -- support, bug, feature, account, billing, feedback, other
    message TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'pending', -- pending, read, responded, archived
    responded_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Index for querying submissions by status
CREATE INDEX IF NOT EXISTS idx_contact_submissions_status ON contact_submissions(status);

-- Index for querying submissions by user
CREATE INDEX IF NOT EXISTS idx_contact_submissions_user_id ON contact_submissions(user_id) WHERE user_id IS NOT NULL;

-- Index for querying submissions by created date
CREATE INDEX IF NOT EXISTS idx_contact_submissions_created_at ON contact_submissions(created_at DESC);
