# Story 6.1: Database Schema Migration

Status: Approved

## Story

As a **backend developer**,
I want **to create and test database migrations for enhanced meal planning**,
so that **the schema supports multi-week plans, accompaniments, and user preferences**.

## Acceptance Criteria

1. Migration SQL file created per section 9.1 (Database Migration Strategy) of architecture-update-meal-planning-enhancements.md
2. Migration adds columns to `meal_plans`: end_date, is_locked, generation_batch_id
3. Migration adds columns to `recipes`: accepts_accompaniment, preferred_accompaniments, accompaniment_category, cuisine, dietary_tags
4. Migration adds column to `meal_assignments`: accompaniment_recipe_id
5. Migration adds columns to `users`: max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight
6. Migration creates table `meal_plan_rotation_state` with all fields
7. Migration creates indexes per section 4 (Database Schema Changes)
8. Migration creates triggers: prevent_locked_week_modification, update_meal_plan_status
9. Migration updates existing data (calculates end_date, sets is_locked based on dates)
10. Rollback migration created and tested

## Tasks / Subtasks

- [ ] Create migration file structure (AC: 1)
  - [ ] Create `migrations/XXX_enhanced_meal_planning.sql` with header comment
  - [ ] Structure migration into logical PART 1-5 sections per architecture doc

- [ ] Implement PART 1: Multi-Week Meal Plan Support (AC: 2, 7, 8)
  - [ ] Add `end_date TEXT NOT NULL DEFAULT ''` to meal_plans table
  - [ ] Add `is_locked BOOLEAN DEFAULT FALSE` to meal_plans table
  - [ ] Add `generation_batch_id TEXT` to meal_plans table
  - [ ] Create index `idx_meal_plans_user_batch ON meal_plans(user_id, generation_batch_id)`
  - [ ] Create index `idx_meal_plans_status ON meal_plans(user_id, status)`
  - [ ] Create index `idx_meal_plans_dates ON meal_plans(start_date, end_date)`
  - [ ] Create trigger `prevent_locked_week_modification` (BEFORE UPDATE, raise error if is_locked=TRUE)
  - [ ] Create trigger `update_meal_plan_status` (AFTER UPDATE, auto-set status based on dates)

- [ ] Implement PART 2: Accompaniment Recipe Type (AC: 3, 4, 7)
  - [ ] Add `accepts_accompaniment BOOLEAN DEFAULT FALSE` to recipes table
  - [ ] Add `preferred_accompaniments TEXT` to recipes table (JSON array storage)
  - [ ] Add `accompaniment_category TEXT` to recipes table
  - [ ] Add `accompaniment_recipe_id TEXT` to meal_assignments table
  - [ ] Create index `idx_meal_assignments_accompaniment ON meal_assignments(accompaniment_recipe_id)`
  - [ ] Create partial index `idx_recipes_accompaniment_type ON recipes(recipe_type) WHERE recipe_type = 'accompaniment'`
  - [ ] Create partial index `idx_recipes_accompaniment_category ON recipes(accompaniment_category) WHERE accompaniment_category IS NOT NULL`

- [ ] Implement PART 3: User Preferences for Algorithm (AC: 5, 7)
  - [ ] Add `max_prep_time_weeknight INTEGER DEFAULT 30` to users table
  - [ ] Add `max_prep_time_weekend INTEGER DEFAULT 90` to users table
  - [ ] Add `avoid_consecutive_complex BOOLEAN DEFAULT TRUE` to users table
  - [ ] Add `cuisine_variety_weight REAL DEFAULT 0.7` to users table
  - [ ] Add `cuisine TEXT` to recipes table
  - [ ] Add `dietary_tags TEXT` to recipes table (JSON array storage)
  - [ ] Create index `idx_recipes_cuisine ON recipes(cuisine)`

- [ ] Implement PART 4: Rotation State Tracking (AC: 6, 7)
  - [ ] Create table `meal_plan_rotation_state` with columns: id, user_id, generation_batch_id, used_main_course_ids (JSON TEXT), used_appetizer_ids (JSON TEXT), used_dessert_ids (JSON TEXT), cuisine_usage_count (JSON TEXT), last_complex_meal_date, created_at, updated_at
  - [ ] Add foreign key constraint `FOREIGN KEY (user_id) REFERENCES users(id)` to meal_plan_rotation_state
  - [ ] Create index `idx_rotation_state_user ON meal_plan_rotation_state(user_id)`
  - [ ] Create index `idx_rotation_state_batch ON meal_plan_rotation_state(generation_batch_id)`

- [ ] Implement existing data migration (AC: 9)
  - [ ] Write UPDATE statement: `SET end_date = date(start_date, '+6 days') WHERE end_date = ''` for meal_plans
  - [ ] Write UPDATE statement: `SET is_locked = TRUE WHERE date(start_date) <= date('now')` for meal_plans
  - [ ] Write UPDATE statement: Set status to 'current'/'past'/'future' based on date comparisons with CASE expression

- [ ] Create rollback migration script (AC: 10)
  - [ ] DROP both triggers (prevent_locked_week_modification, update_meal_plan_status)
  - [ ] DROP meal_plan_rotation_state table
  - [ ] DROP all 10 new indexes in reverse order
  - [ ] ALTER TABLE to drop all new columns from users, recipes, meal_assignments, meal_plans (reverse order)
  - [ ] Test rollback on development database with sample data

- [ ] Test migration execution (AC: 1-10)
  - [ ] Run migration on fresh SQLite database, verify all tables/columns/indexes/triggers created
  - [ ] Run migration on database with existing test data (100 users, 500 recipes, 200 meal plans)
  - [ ] Verify existing meal plans get correct end_date (start_date + 6 days)
  - [ ] Verify is_locked=TRUE for any meal plan where start_date <= today
  - [ ] Verify status field correctly set (current/past/future) based on dates
  - [ ] Test trigger: attempt to UPDATE locked meal plan, confirm RAISE(FAIL) error
  - [ ] Test trigger: UPDATE meal plan dates, confirm status auto-updates
  - [ ] Measure execution time (target: <5 seconds on development dataset)
  - [ ] Run rollback migration, verify database returns to original schema
  - [ ] Re-run forward migration successfully (idempotence check)

## Dev Notes

### Architecture Context

This story implements the foundational database schema changes required for Epic 6: Enhanced Meal Planning System, as defined in `docs/architecture-update-meal-planning-enhancements.md`.

**Three Major Features Enabled:**
1. **Multi-Week Meal Plan Generation** - Generate up to 5 weeks of meal plans with week locking (current week cannot be regenerated)
2. **Accompaniment Recipe Type** - Support pasta, rice, fries, salad, bread, vegetables as optional sides to main courses
3. **User Preferences Integration** - Store time constraints, complexity preferences, and cuisine variety weights for algorithm use

### Database Technology

- **SQLite 3.45+** with WAL mode enabled
- **JSON Storage**: All array/object fields stored as TEXT with application-layer validation (SQLite JSON functions available but not required)
- **Migration System**: SQLx migrations (`migrations/` directory)
- **Evento Integration**: This schema works alongside evento's automatic `events` table

### Key Design Decisions

**Week Locking Logic:**
- `is_locked = TRUE` when `date(start_date) <= date('now')` (current or past weeks)
- Trigger `prevent_locked_week_modification` blocks ANY update to locked meal plan rows
- Protects users from accidentally regenerating in-progress meal plans

**Status Auto-Update:**
- Trigger `update_meal_plan_status` fires AFTER UPDATE when start_date or end_date changes
- Automatically recalculates status: 'current' (today falls within week), 'past' (week ended), 'future' (week hasn't started)
- Ensures UI always displays accurate week state without manual status management

**Accompaniment Optionality:**
- `accepts_accompaniment BOOLEAN` on recipes (not meal_assignments) - recipe creator controls whether dish accepts sides
- No "required" accompaniment concept - always optional per architecture decision
- `preferred_accompaniments` filters which categories (pasta, rice, etc.) pair well

**Cuisine Preferences Inference:**
- No explicit `preferred_cuisines` field - inferred from user's favorite recipe selection
- `cuisine_variety_weight REAL` (0.0-1.0) slider controls variety vs. repetition
- Algorithm tracks `cuisine_usage_count` in rotation state to spread variety

**Default Values Rationale:**
- `max_prep_time_weeknight = 30` minutes (typical weeknight availability)
- `max_prep_time_weekend = 90` minutes (more time on weekends)
- `avoid_consecutive_complex = TRUE` (most users prefer spacing)
- `cuisine_variety_weight = 0.7` (moderate variety, allows some repetition)

### Migration Execution Strategy

**Forward Migration:**
1. Add all new columns with DEFAULT values (backwards compatible)
2. Create all new tables and indexes
3. Create triggers for business rule enforcement
4. Backfill existing data with calculated values (end_date, is_locked, status)

**Backward Compatibility:**
- All new columns have sensible defaults
- Existing queries continue to work (new columns ignored)
- Evento event sourcing unaffected (aggregate state reconstruction still valid)

**Rollback Strategy:**
- Drop triggers first (remove enforcement before removing columns)
- Drop new tables
- Drop all new indexes
- Remove new columns in reverse order (SQLite ALTER TABLE limitations)
- **Critical**: Test rollback on staging before production deployment

### Testing Standards

**Test Data Volumes:**
- 100 users with varying preferences
- 500 recipes across all types (appetizer, main_course, dessert, accompaniment)
- 200 meal plans (mix of current, past, future weeks)
- Verify migration completes in <5 seconds on this dataset

**Critical Test Cases:**
1. **Trigger Enforcement**: Attempt `UPDATE meal_plans SET start_date = '2025-11-01' WHERE is_locked = TRUE` → Must fail with RAISE(FAIL)
2. **Status Auto-Update**: `UPDATE meal_plans SET start_date = date('now')` → status must become 'current'
3. **Idempotence**: Run migration twice → second run should succeed (use IF NOT EXISTS where applicable)
4. **Rollback Round-Trip**: Forward migration → rollback → forward migration → all succeed

### Performance Considerations

**Index Selection Rationale:**
- `idx_meal_plans_user_batch`: Multi-week query `SELECT * FROM meal_plans WHERE user_id = ? AND generation_batch_id = ?`
- `idx_meal_plans_status`: Dashboard queries `WHERE user_id = ? AND status = 'current'`
- `idx_meal_plans_dates`: Week locking logic `WHERE date(start_date) <= date('now')`
- `idx_recipes_cuisine`: Algorithm preference filtering
- Partial indexes on `recipe_type = 'accompaniment'` and `accompaniment_category IS NOT NULL` (smaller index size for rare values)

**Query Optimization:**
- Composite index on `(user_id, generation_batch_id)` supports batch retrieval without full scan
- Partial indexes reduce index size for sparse data (accompaniments are minority of recipes)
- REAL datatype for `cuisine_variety_weight` supports floating-point algorithm calculations

### Project Structure Notes

**File Location:** `migrations/XXX_enhanced_meal_planning.sql` where XXX is sequential migration number

**Migration Naming Convention:**
- SQLx expects format: `<version>_<description>.sql`
- Example: `008_enhanced_meal_planning.sql` (if previous migration was 007)
- Auto-detected by `sqlx::migrate!("./migrations")` macro

**Related Domain Crates (Future Stories):**
- `crates/meal_planning/` - Multi-week algorithm implementation (Story 6.2+)
- `crates/recipe/` - Recipe aggregate updates with new fields (Story 6.2)
- `crates/user/` - User preferences aggregate extension (Story 6.3)

### References

- [Source: docs/architecture-update-meal-planning-enhancements.md#4-database-schema-changes] - Complete schema specification
- [Source: docs/architecture-update-meal-planning-enhancements.md#9-migration-strategy] - Migration SQL template and rollback plan
- [Source: docs/architecture-update-meal-planning-enhancements.md#1-multi-week-meal-plan-generation] - Multi-week feature context
- [Source: docs/architecture-update-meal-planning-enhancements.md#2-accompaniment-recipe-type-system] - Accompaniment design rationale
- [Source: docs/architecture-update-meal-planning-enhancements.md#3-user-preferences-integration] - Preferences design decisions
- [Source: docs/solution-architecture-compact.md#3-data-architecture] - Existing database conventions (SQLite WAL mode, SQLx migrations)

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML/JSON will be added here by context workflow -->

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
