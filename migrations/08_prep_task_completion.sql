-- Story 4.9: Prep Task Completion Tracking
-- Add completion tracking fields to notifications table

-- Add completion timestamp
ALTER TABLE notifications ADD COLUMN completed_at TEXT;

-- Add reminder carry-over tracking
ALTER TABLE notifications ADD COLUMN reminder_count INTEGER DEFAULT 0 NOT NULL;
ALTER TABLE notifications ADD COLUMN max_reminder_count INTEGER DEFAULT 3 NOT NULL;

-- Index for dashboard prep tasks query: user_id + notification_type + meal_date + status
CREATE INDEX IF NOT EXISTS idx_notifications_user_prep_tasks
    ON notifications(user_id, reminder_type, meal_date, status)
    WHERE reminder_type IN ('advance_prep', 'morning');
