-- Story 4.10: Push Notification Permission Flow
-- Add notification permission tracking fields to users table

-- Add notification permission status field
-- Values: 'not_asked' | 'granted' | 'denied' | 'skipped'
ALTER TABLE users ADD COLUMN notification_permission_status TEXT NOT NULL DEFAULT 'not_asked';

-- Add last_permission_denial_at field for grace period tracking (AC #8)
-- Tracks timestamp when user last denied permission to prevent re-prompting within 30 days
ALTER TABLE users ADD COLUMN last_permission_denial_at TEXT;

-- Index for grace period queries (check if user can be re-prompted)
CREATE INDEX IF NOT EXISTS idx_users_notification_permission
    ON users(notification_permission_status, last_permission_denial_at);
