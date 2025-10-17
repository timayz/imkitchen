-- Migration 04: Add assignment_reasoning column to meal_assignments
-- Story 3.8: Algorithm Transparency (Show Reasoning)
-- Adds human-readable reasoning text explaining why each meal was assigned

-- Add assignment_reasoning column to meal_assignments table
ALTER TABLE meal_assignments ADD COLUMN assignment_reasoning TEXT;
