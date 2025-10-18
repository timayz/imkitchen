-- Story 4.7: Morning Preparation Reminders
-- Add message_body column to notifications table for precomputed reminder messages
-- Add prep_task column to recipes table to store specific prep instructions

ALTER TABLE notifications ADD COLUMN message_body TEXT;
ALTER TABLE notifications ADD COLUMN created_at TEXT;

-- Add prep_task column to recipes table (e.g., "Marinate chicken", "Let dough rise")
ALTER TABLE recipes ADD COLUMN prep_task TEXT;

-- Index for auto-dismissal worker: query expired morning reminders
CREATE INDEX IF NOT EXISTS idx_notifications_morning_expired
    ON notifications(reminder_type, status, meal_date)
    WHERE reminder_type = 'morning' AND status = 'pending';
