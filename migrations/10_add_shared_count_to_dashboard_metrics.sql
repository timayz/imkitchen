-- Migration 10: Add shared_count to dashboard_metrics
-- Created: 2025-10-24
-- Purpose: Add shared_count field to dashboard_metrics table to avoid COUNT(*) queries

ALTER TABLE dashboard_metrics ADD COLUMN shared_count INTEGER NOT NULL DEFAULT 0;
