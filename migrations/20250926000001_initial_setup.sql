-- Initial database setup for IMKitchen
-- This is a placeholder migration file

-- Enable foreign key constraints in SQLite
PRAGMA foreign_keys = ON;

-- Create a simple health check table for now
CREATE TABLE IF NOT EXISTS system_health (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    check_time TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'OK'
);

-- Insert initial health check record
INSERT INTO system_health (status) VALUES ('INITIALIZED');