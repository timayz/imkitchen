-- Add Stripe integration fields to users table
ALTER TABLE users ADD COLUMN stripe_customer_id TEXT;
ALTER TABLE users ADD COLUMN stripe_subscription_id TEXT;

-- Index for Stripe customer lookups (useful for webhooks and billing queries)
CREATE INDEX IF NOT EXISTS idx_users_stripe_customer ON users(stripe_customer_id);
