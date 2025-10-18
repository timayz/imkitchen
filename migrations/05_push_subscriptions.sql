-- Story 4.6: Advance Preparation Reminder System
-- Create push_subscriptions table for Web Push API

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
