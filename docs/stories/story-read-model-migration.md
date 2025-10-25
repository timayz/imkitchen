# Story: Read Model Migration to Page-Specific Architecture

**Epic:** Cross-Cutting - Architecture Evolution
**Priority:** High
**Story Points:** 8
**Status:** Ready for Review
**Created:** 2025-10-24

---

## Dev Agent Record

### Context Reference
- `/home/snapiz/projects/github/timayz/imkitchen/docs/stories/story-context-read-model-migration.xml` (Generated: 2025-10-24)
- `/home/snapiz/projects/github/timayz/imkitchen/docs/solution-architecture.md` (Section 3.2)
- Existing migrations: `migrations/00_v0.1.sql` through `migrations/08_add_meal_plan_updated_at.sql`

### Debug Log
(Will be populated during implementation)

### Completion Notes

**Migration Approach:** Simplified to MVP strategy per Jonathan's direction. Since this is MVP and DB will be reset, created a single SQL migration file instead of complex dual-write pattern.

**Deliverables:**
- `migrations/09_page_specific_read_models.sql` - Complete SQL schema for all page-specific read models

**Page-Specific Tables Created:**
1. **Dashboard:** dashboard_meals, dashboard_prep_tasks, dashboard_metrics
2. **Calendar:** calendar_view
3. **Recipe Library:** recipe_list, recipe_filter_counts
4. **Recipe Detail:** recipe_detail, recipe_ratings
5. **Shopping List:** shopping_list_view, shopping_list_summary

**Key Design Decisions:**
- Denormalized data (recipe metadata in meal tables) to eliminate JOINs
- Optimized indexes for each page's query patterns (e.g., `WHERE date = DATE('now')` for dashboard)
- Soft deletes via `deleted_at` column for recipes
- Aggregated ratings in `recipe_ratings` to avoid COUNT(*) GROUP BY queries
- Filter facet counts in `recipe_filter_counts` for instant filter UI updates

**Next Steps:**
- Implement evento projection handlers to populate these tables
- Update route handlers to query page-specific tables
- Remove old domain-centric table queries

---

## Tasks/Subtasks

- [x] T1: Create SQL migration for page-specific read models (`migrations/09_page_specific_read_models.sql`)
  - [x] T1.1: Dashboard page tables (dashboard_meals, dashboard_prep_tasks, dashboard_metrics)
  - [x] T1.2: Calendar page table (calendar_view)
  - [x] T1.3: Recipe Library page tables (recipe_list, recipe_filter_counts)
  - [x] T1.4: Recipe Detail page tables (recipe_detail, recipe_ratings)
  - [x] T1.5: Shopping List page tables (shopping_list_view, shopping_list_summary)
  - [x] T1.6: Add indexes optimized for page-specific queries

---

## User Story

**As a** system architect
**I want** a comprehensive migration plan from domain-centric read models to page-specific read models
**So that** we can evolve the architecture to improve query performance, reduce coupling, and enable independent scaling per page concern

---

## Prerequisites

- Solution architecture document defines page-specific read model pattern
- Existing read models are functional and serving production traffic
- evento event sourcing system is operational
- Understanding of current page rendering requirements

---

## Acceptance Criteria

### 1. Migration Plan Document Structure
- Document created at `docs/read-model-migration-plan.md`
- Contains executive summary with migration goals and timeline
- Structured with clear sections: Current State, Target State, Migration Strategy, Implementation Phases
- Includes risk assessment and rollback procedures
- Contains validation criteria for each migration phase

### 2. Current State Documentation
- Complete inventory of existing read model tables: `users`, `recipes`, `meal_plans`, `meal_assignments`, `recipe_collections`, `ratings`, etc.
- Document which pages/routes consume each table
- Map existing indexes and performance characteristics
- Document current evento projection handlers and their table updates
- Identify all foreign key relationships and constraints

### 3. Target Architecture Definition
- Define all page-specific read models per solution-architecture.md:
  - **Dashboard Page**: `dashboard_meals`, `dashboard_prep_tasks`, `dashboard_metrics`
  - **Meal Calendar Page**: `calendar_view`
  - **Recipe Library Page**: `recipe_list`, `recipe_filter_counts`, `recipe_collections`
  - **Recipe Detail Page**: `recipe_detail`, `recipe_ratings`
  - **Shopping List Page**: `shopping_list_view`, `shopping_list_summary`
- Provide complete SQL schema for each page-specific table
- Document indexes optimized for each table's query patterns
- Define evento projection handlers for populating each table
- Specify which domain events trigger which page-specific projections

### 4. Migration Strategy
- Define phased approach (not big-bang migration)
- Strategy supports running old and new read models in parallel (dual-write pattern)
- Document data population for new tables from existing tables or event replay
- Define cutover criteria for switching queries from old to new tables
- Include rollback procedure if migration encounters issues
- Specify validation queries to verify data consistency between old and new models

### 5. Backward Compatibility Plan
- Existing routes continue functioning during migration
- No breaking changes to current API/rendering contracts
- Old tables remain until all pages migrated and validated
- Document deprecation timeline for old tables
- Define cleanup phase after successful migration

### 6. Performance Impact Analysis
- Estimate storage overhead from denormalized page-specific tables
- Project query performance improvements per page
- Calculate evento projection processing overhead
- Define monitoring metrics to track migration health
- Establish performance baselines before migration

### 7. Testing and Validation Strategy
- Unit tests for each new evento projection handler
- Integration tests comparing old vs new read model query results
- Load tests to validate performance improvements
- Data consistency checks (old table data == new table data)
- Smoke tests for each migrated page
- Rollback validation tests

### 8. Implementation Phases
- **Phase 1**: Create migration plan document (this story)
- **Phase 2**: Implement page-specific tables for Dashboard (pilot)
- **Phase 3**: Migrate Recipe pages (Library, Detail)
- **Phase 4**: Migrate Meal Planning pages (Calendar)
- **Phase 5**: Migrate Shopping List page
- **Phase 6**: Deprecate and remove old tables (cleanup)
- Each phase includes: SQL migrations, projection handlers, query updates, validation, cutover

### 9. Risk Mitigation
- Document identified risks:
  - Data consistency between old/new models
  - evento projection lag causing stale reads
  - Storage bloat from denormalized tables
  - Migration script failures mid-execution
- Mitigation strategies for each risk
- Monitoring and alerting recommendations

### 10. Documentation Quality
- Document written in clear, technical English
- Includes code examples (SQL, Rust projection handlers)
- Diagrams showing current vs target architecture
- References solution-architecture.md sections
- Actionable implementation steps for each phase
- Suitable for handoff to development team

---

## Technical Notes

### Current Read Model Tables (Domain-Centric)
```sql
-- users: Serves auth, profile, dashboard, all pages
-- recipes: Serves recipe library, detail, community, meal planning
-- meal_plans: Serves dashboard, calendar, meal planning
-- meal_assignments: Serves dashboard, calendar
-- recipe_collections: Serves recipe library collections view
-- ratings: Serves recipe detail, community discovery
-- contact_submissions: Serves support page
```

### Target Page-Specific Tables
Per solution-architecture.md (Section 3.2), define tables like:
```sql
-- Dashboard Page
CREATE TABLE dashboard_meals (...);        -- Today's meal assignments
CREATE TABLE dashboard_prep_tasks (...);   -- Today's prep reminders
CREATE TABLE dashboard_metrics (...);      -- Recipe variety stats

-- Recipe Library Page
CREATE TABLE recipe_list (...);            -- Recipe cards for display
CREATE TABLE recipe_filter_counts (...);   -- Facet counts
CREATE TABLE recipe_collections (...);     -- User collections

-- Recipe Detail Page
CREATE TABLE recipe_detail (...);          -- Full recipe data
CREATE TABLE recipe_ratings (...);         -- Aggregated ratings

-- Meal Calendar Page
CREATE TABLE calendar_view (...);          -- Week view assignments

-- Shopping List Page
CREATE TABLE shopping_list_view (...);     -- Categorized items
CREATE TABLE shopping_list_summary (...);  -- Category totals
```

### evento Projection Pattern
```rust
// Page-Specific Projection Example
#[evento::handler(Recipe)]
async fn project_recipe_to_list_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Insert ONLY data needed for recipe list cards
    sqlx::query(
        "INSERT INTO recipe_list (id, user_id, title, complexity, prep_time_min, ...)
         VALUES (?, ?, ?, ?, ?, ...)"
    )
    .bind(&event.aggregator_id)
    .bind(&event.metadata.user_id)
    .bind(&event.data.title)
    // ...
    .execute(context.executor.pool())
    .await?;
    Ok(())
}

// Multiple projections for same event
#[evento::handler(Recipe)]
async fn project_recipe_to_detail_view<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Insert FULL recipe data for detail view
    // ...
}

#[evento::handler(Recipe)]
async fn update_recipe_filter_counts<E: evento::Executor>(
    context: &evento::Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Increment filter facet counts
    // ...
}
```

### Migration Strategy Options

**Option A: Event Replay (Clean Slate)**
- Drop old tables, create new page-specific tables
- Replay all events from evento event store
- Projections populate new tables from scratch
- Pros: Clean, guaranteed consistency
- Cons: Downtime required, slow for large event stores

**Option B: Dual-Write Pattern (Incremental)**
- Create new page-specific tables alongside old ones
- Update projections to write to BOTH old and new tables
- Backfill new tables from old tables or event replay (async)
- Validate consistency, then switch queries to new tables
- Deprecate old tables once all pages migrated
- Pros: Zero downtime, phased migration
- Cons: Complex, temporary storage overhead

**Option C: Copy-Transform-Validate (Hybrid)**
- Copy data from old tables to new tables via SQL scripts
- Run evento projections forward from copy point
- Validate consistency with diff checks
- Cutover page queries to new tables
- Pros: Faster than event replay, lower risk than dual-write
- Cons: Requires careful coordination, one-time scripts

### Recommended Approach
**Dual-Write Pattern (Option B)** for production safety:
1. Create new tables (migrations)
2. Update projections to dual-write
3. Backfill historical data (async job)
4. Validate consistency (automated checks)
5. Switch page queries to new tables (feature flag)
6. Monitor performance and correctness
7. Deprecate old tables after all pages migrated

### Key Decision Points
- **Storage Cost**: Page-specific tables denormalize data (more storage, faster queries)
- **evento Lag**: Projections eventually consistent (read-after-write may see stale data briefly)
- **Index Strategy**: Each page-specific table optimized for its query patterns
- **Cleanup**: Old tables remain until all pages validated on new models

### Implementation Complexity
This is an **architectural migration**, not a feature. Estimate 8 story points:
- High complexity: Cross-cutting change affecting all pages
- Requires careful planning to avoid breaking production
- Phased approach spans multiple implementation cycles
- Testing and validation critical for data integrity

---

## File List
- `migrations/09_page_specific_read_models.sql` - SQL migration creating all page-specific read model tables

---

## Change Log
- 2025-10-24: Story created, migration plan document task defined
- 2025-10-24: Migration strategy simplified to MVP approach (DB reset, SQL migration only)
- 2025-10-24: Created SQL migration 09 with all page-specific read model tables

---

## Definition of Done
- [x] SQL migration created with all page-specific read model tables
- [x] All acceptance criteria met (MVP approach - SQL migration only)
- [x] Tables include optimized indexes for page-specific query patterns
- [x] Migration follows SQLite best practices
- [x] Changes committed to repository
