-- Account Deletion and Audit Schema
-- Adds support for account deletion, audit trails, and GDPR compliance

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Add deletion fields to user_profiles table
ALTER TABLE user_profiles 
ADD COLUMN deleted_at TEXT NULL;

ALTER TABLE user_profiles 
ADD COLUMN deletion_reason TEXT NULL;

ALTER TABLE user_profiles 
ADD COLUMN data_purge_scheduled_at TEXT NULL;

-- Account deletion audit trail
CREATE TABLE IF NOT EXISTS account_deletion_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    deletion_reason TEXT NOT NULL,  -- JSON serialized DeletionReason
    initiated_by_user BOOLEAN NOT NULL,
    admin_user_id TEXT NULL,  -- UUID of admin who initiated deletion (if applicable)
    deleted_at TEXT NOT NULL,
    audit_created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (admin_user_id) REFERENCES user_profiles(id) ON DELETE SET NULL
);

-- Data purge audit trail for GDPR compliance
CREATE TABLE IF NOT EXISTS data_purge_audit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,  -- User ID that was purged
    admin_authorization TEXT NOT NULL,  -- UUID of admin who authorized purge
    purged_at TEXT NOT NULL,
    audit_created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- User preferences table for extended user settings
CREATE TABLE IF NOT EXISTS user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    weekday_cooking_minutes INTEGER NOT NULL DEFAULT 30,
    weekend_cooking_minutes INTEGER NOT NULL DEFAULT 60,
    notification_preferences TEXT DEFAULT '{}',  -- JSON preferences
    privacy_settings TEXT DEFAULT '{}',  -- JSON privacy settings
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TEXT NULL,  -- Soft delete support
    FOREIGN KEY (user_id) REFERENCES user_profiles(id) ON DELETE CASCADE,
    UNIQUE(user_id)
);

-- Cascading deletion tracking table
CREATE TABLE IF NOT EXISTS cascading_deletion_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    bounded_context TEXT NOT NULL,  -- Which bounded context was notified
    operation_type TEXT NOT NULL,   -- 'SOFT_DELETE', 'PURGE', etc.
    status TEXT NOT NULL DEFAULT 'PENDING',  -- 'PENDING', 'COMPLETED', 'FAILED'
    initiated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT NULL,
    error_message TEXT NULL
);

-- Indexes for performance and compliance queries
CREATE INDEX IF NOT EXISTS idx_user_profiles_deleted_at ON user_profiles(deleted_at);
CREATE INDEX IF NOT EXISTS idx_user_profiles_data_purge_scheduled ON user_profiles(data_purge_scheduled_at);
CREATE INDEX IF NOT EXISTS idx_account_deletion_audit_user_id ON account_deletion_audit(user_id);
CREATE INDEX IF NOT EXISTS idx_account_deletion_audit_deleted_at ON account_deletion_audit(deleted_at);
CREATE INDEX IF NOT EXISTS idx_data_purge_audit_user_id ON data_purge_audit(user_id);
CREATE INDEX IF NOT EXISTS idx_data_purge_audit_purged_at ON data_purge_audit(purged_at);
CREATE INDEX IF NOT EXISTS idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_user_preferences_deleted_at ON user_preferences(deleted_at);
CREATE INDEX IF NOT EXISTS idx_cascading_deletion_log_user_id ON cascading_deletion_log(user_id);
CREATE INDEX IF NOT EXISTS idx_cascading_deletion_log_status ON cascading_deletion_log(status);

-- Create view for active (non-deleted) users
CREATE VIEW IF NOT EXISTS active_users AS
SELECT * FROM user_profiles 
WHERE deleted_at IS NULL;

-- Create view for users scheduled for data purge
CREATE VIEW IF NOT EXISTS users_scheduled_for_purge AS
SELECT id, email, deleted_at, data_purge_scheduled_at, 
       julianday(data_purge_scheduled_at) - julianday('now') AS days_until_purge
FROM user_profiles 
WHERE deleted_at IS NOT NULL 
  AND data_purge_scheduled_at IS NOT NULL
  AND julianday('now') >= julianday(data_purge_scheduled_at);

-- Create trigger to automatically update updated_at in user_preferences
CREATE TRIGGER IF NOT EXISTS update_user_preferences_timestamp 
    AFTER UPDATE ON user_preferences
    FOR EACH ROW
BEGIN
    UPDATE user_preferences 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.id;
END;

-- Create trigger to ensure data consistency on user deletion
CREATE TRIGGER IF NOT EXISTS ensure_deletion_consistency
    AFTER UPDATE OF deleted_at ON user_profiles
    FOR EACH ROW
    WHEN NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL
BEGIN
    -- Mark user sessions as expired when user is deleted
    UPDATE user_sessions 
    SET expires_at = CURRENT_TIMESTAMP 
    WHERE user_id = NEW.id;
    
    -- Mark email verification tokens as used
    UPDATE email_verification_tokens 
    SET used = TRUE 
    WHERE user_id = NEW.id AND used = FALSE;
    
    -- Mark password reset tokens as used
    UPDATE password_reset_tokens 
    SET used = TRUE 
    WHERE user_id = NEW.id AND used = FALSE;
END;