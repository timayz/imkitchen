-- Migration 03: Epic 4 - Shopping & Preparation Orchestration (v0.4)
-- Consolidated migration from files 03-09
-- Stories: 4.1, 4.2, 4.6, 4.7, 4.8, 4.9, 4.10
-- Date: 2025-10-18

-- =============================================================================
-- SHOPPING LIST SYSTEM (Story 4.1, 4.2)
-- =============================================================================

-- Shopping lists table: one list per meal plan/week
CREATE TABLE IF NOT EXISTS shopping_lists (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    meal_plan_id TEXT NOT NULL,
    week_start_date TEXT NOT NULL,                   -- ISO 8601 date (Monday of the week)
    generated_at TEXT NOT NULL,                       -- RFC3339 formatted timestamp
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id)
);

-- Shopping list items table: aggregated and categorized ingredients
CREATE TABLE IF NOT EXISTS shopping_list_items (
    id TEXT PRIMARY KEY NOT NULL,
    shopping_list_id TEXT NOT NULL,
    ingredient_name TEXT NOT NULL,
    quantity REAL NOT NULL,                           -- Normalized quantity (e.g., 480.0 for 2 cups)
    unit TEXT NOT NULL,                               -- Normalized unit (e.g., "ml", "g", "item")
    category TEXT NOT NULL CHECK(category IN ('Produce', 'Dairy', 'Meat', 'Pantry', 'Frozen', 'Bakery', 'Other')),
    is_collected INTEGER NOT NULL DEFAULT 0,          -- Boolean: 0 = false, 1 = true (Story 4.2)
    FOREIGN KEY (shopping_list_id) REFERENCES shopping_lists(id) ON DELETE CASCADE
);

-- Shopping lists indexes (optimized for common queries)
CREATE INDEX IF NOT EXISTS idx_shopping_lists_user
    ON shopping_lists(user_id);

CREATE INDEX IF NOT EXISTS idx_shopping_lists_meal_plan
    ON shopping_lists(meal_plan_id);

CREATE INDEX IF NOT EXISTS idx_shopping_lists_week
    ON shopping_lists(user_id, week_start_date);

-- Shopping list items indexes
CREATE INDEX IF NOT EXISTS idx_shopping_list_items_list
    ON shopping_list_items(shopping_list_id);

CREATE INDEX IF NOT EXISTS idx_shopping_list_items_category
    ON shopping_list_items(shopping_list_id, category);

-- =============================================================================
-- NOTIFICATION SYSTEM (Story 4.6, 4.7, 4.8, 4.9)
-- Merged: All columns from migrations 04, 06, 07, 08 combined into CREATE TABLE
-- =============================================================================

-- Notifications table for reminder read model
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY NOT NULL,                 -- notification_id (UUID)
    user_id TEXT NOT NULL,                        -- Owner of the notification
    recipe_id TEXT NOT NULL,                      -- Recipe requiring advance prep
    meal_date TEXT NOT NULL,                      -- ISO 8601 date of the meal
    scheduled_time TEXT NOT NULL,                 -- RFC3339 timestamp when reminder should fire
    status TEXT NOT NULL DEFAULT 'pending',       -- 'pending', 'sent', 'failed', 'dismissed', 'snoozed'
    reminder_type TEXT NOT NULL,                  -- 'advance_prep', 'morning', 'day_of'
    prep_hours INTEGER NOT NULL,                  -- Hours of advance prep required (from recipe)
    prep_task TEXT,                               -- Specific task: 'marinate', 'rise', 'chill', etc.
    sent_at TEXT,                                 -- RFC3339 timestamp when notification was sent
    delivery_status TEXT,                         -- 'sent', 'failed', 'endpoint_invalid'
    dismissed_at TEXT,                            -- RFC3339 timestamp when user dismissed
    message_body TEXT,                            -- Precomputed reminder message (Story 4.7)
    created_at TEXT,                              -- RFC3339 timestamp when notification was created (Story 4.7)
    snoozed_until TEXT,                           -- RFC3339 timestamp when snoozed notification should reappear (Story 4.8)
    completed_at TEXT,                            -- RFC3339 timestamp when prep task was marked complete (Story 4.9)
    reminder_count INTEGER DEFAULT 0 NOT NULL,    -- Number of times reminder has been sent (Story 4.9)
    max_reminder_count INTEGER DEFAULT 3 NOT NULL -- Maximum reminder attempts before auto-dismiss (Story 4.9)
);

-- Index for background worker query: pending notifications scheduled in the past
CREATE INDEX IF NOT EXISTS idx_notifications_pending_scheduled
    ON notifications(status, scheduled_time)
    WHERE status = 'pending';

-- Index for user query: list pending notifications for a user
CREATE INDEX IF NOT EXISTS idx_notifications_user_pending
    ON notifications(user_id, status);

-- Index for auto-dismissal worker: query expired morning reminders (Story 4.7)
CREATE INDEX IF NOT EXISTS idx_notifications_morning_expired
    ON notifications(reminder_type, status, meal_date)
    WHERE reminder_type = 'morning' AND status = 'pending';

-- Index for snoozed notifications: background worker needs to find snoozed notifications due for redelivery (Story 4.8)
CREATE INDEX IF NOT EXISTS idx_notifications_snoozed
    ON notifications(status, snoozed_until)
    WHERE status = 'snoozed';

-- Index for dashboard prep tasks query: user_id + notification_type + meal_date + status (Story 4.9)
CREATE INDEX IF NOT EXISTS idx_notifications_user_prep_tasks
    ON notifications(user_id, reminder_type, meal_date, status)
    WHERE reminder_type IN ('advance_prep', 'morning');

-- =============================================================================
-- PUSH SUBSCRIPTIONS (Story 4.6)
-- =============================================================================

-- Push subscriptions table for Web Push API
CREATE TABLE IF NOT EXISTS push_subscriptions (
    id TEXT PRIMARY KEY NOT NULL,        -- subscription_id (UUID)
    user_id TEXT NOT NULL,               -- Owner of the subscription
    endpoint TEXT NOT NULL UNIQUE,       -- Web Push endpoint URL
    p256dh_key TEXT NOT NULL,            -- Base64-encoded public key
    auth_key TEXT NOT NULL,              -- Base64-encoded auth secret
    created_at TEXT NOT NULL             -- RFC3339 timestamp
);

-- Index for user query: get push subscription by user_id
CREATE INDEX IF NOT EXISTS idx_push_subscriptions_user
    ON push_subscriptions(user_id);

-- =============================================================================
-- RECIPE ENHANCEMENTS (Story 4.7)
-- =============================================================================

-- Add prep_task column to recipes table (e.g., "Marinate chicken", "Let dough rise")
ALTER TABLE recipes ADD COLUMN prep_task TEXT;

-- =============================================================================
-- MEAL ASSIGNMENTS INDEX (Story 4.8)
-- =============================================================================

-- Index for day-of cooking reminders: query meals by date for cooking reminder scheduling
CREATE INDEX IF NOT EXISTS idx_meal_assignments_date
    ON meal_assignments(date, meal_type);

-- =============================================================================
-- USER NOTIFICATION PERMISSIONS (Story 4.10)
-- Merged: Both ALTER TABLE statements combined for users table
-- =============================================================================

-- Add notification permission status field
-- Values: 'not_asked' | 'granted' | 'denied' | 'skipped'
ALTER TABLE users ADD COLUMN notification_permission_status TEXT NOT NULL DEFAULT 'not_asked';

-- Add last_permission_denial_at field for grace period tracking (AC #8)
-- Tracks timestamp when user last denied permission to prevent re-prompting within 30 days
ALTER TABLE users ADD COLUMN last_permission_denial_at TEXT;

-- Index for grace period queries (check if user can be re-prompted)
CREATE INDEX IF NOT EXISTS idx_users_notification_permission
    ON users(notification_permission_status, last_permission_denial_at);
