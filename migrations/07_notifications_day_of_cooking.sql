-- Story 4.8: Day-of Cooking Reminders
-- Add snoozed_until column to notifications table for snooze functionality

ALTER TABLE notifications ADD COLUMN snoozed_until TEXT;

-- Index for day-of cooking reminders: query meals by date for cooking reminder scheduling
CREATE INDEX IF NOT EXISTS idx_meal_assignments_date
    ON meal_assignments(date, meal_type);

-- Index for snoozed notifications: background worker needs to find snoozed notifications due for redelivery
CREATE INDEX IF NOT EXISTS idx_notifications_snoozed
    ON notifications(status, snoozed_until)
    WHERE status = 'snoozed';
