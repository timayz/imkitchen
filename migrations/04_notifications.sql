-- Story 4.6: Advance Preparation Reminder System
-- Create notifications table for reminder read model

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY NOT NULL,                 -- notification_id (UUID)
    user_id TEXT NOT NULL,                        -- Owner of the notification
    recipe_id TEXT NOT NULL,                      -- Recipe requiring advance prep
    meal_date TEXT NOT NULL,                      -- ISO 8601 date of the meal
    scheduled_time TEXT NOT NULL,                 -- RFC3339 timestamp when reminder should fire
    status TEXT NOT NULL DEFAULT 'pending',       -- 'pending', 'sent', 'failed', 'dismissed'
    reminder_type TEXT NOT NULL,                  -- 'advance_prep', 'morning', 'day_of'
    prep_hours INTEGER NOT NULL,                  -- Hours of advance prep required (from recipe)
    prep_task TEXT,                               -- Specific task: 'marinate', 'rise', 'chill', etc.
    sent_at TEXT,                                 -- RFC3339 timestamp when notification was sent
    delivery_status TEXT,                         -- 'sent', 'failed', 'endpoint_invalid'
    dismissed_at TEXT                             -- RFC3339 timestamp when user dismissed
);

-- Index for background worker query: pending notifications scheduled in the past
CREATE INDEX IF NOT EXISTS idx_notifications_pending_scheduled
    ON notifications(status, scheduled_time)
    WHERE status = 'pending';

-- Index for user query: list pending notifications for a user
CREATE INDEX IF NOT EXISTS idx_notifications_user_pending
    ON notifications(user_id, status);
