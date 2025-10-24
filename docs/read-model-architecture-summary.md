# Read Model Architecture Summary

**Project:** imkitchen
**Date:** 2025-10-24
**Author:** Winston (Architect Agent)
**Status:** Architecture Update Complete

---

## Overview

This document summarizes the architectural changes made to implement **page-specific read models** in the imkitchen project. The new architecture optimizes query performance, reduces coupling, and aligns with Domain-Driven Design principles.

---

## Key Changes Implemented

### 1. Solution Architecture Document Updated

**File:** `/docs/solution-architecture.md`

**Changes:**
- ✅ Section 2.4 "Data Fetching Approach" rewritten to emphasize page-specific read models
- ✅ Added explanation of when to use read models vs `evento::load` (forms)
- ✅ Section 3.1 "Database Schema" updated with page-specific read model tables
- ✅ Section 3.2 "Event-to-ReadModel Projections" rewritten with page-specific projection examples
- ✅ Section 15.2 "Integration Tests" enhanced with `unsafe_oneshot` testing pattern

**Key Concepts Documented:**
- **Page-Specific Read Models**: Each page queries dedicated tables (dashboard_meals, recipe_list, etc.)
- **Form Data Consistency**: Use `evento::load` for authoritative form pre-population
- **Business Logic Location**: Keep validation/rules in aggregates, NOT handlers
- **Testing Pattern**: Use `unsafe_oneshot` in tests for deterministic projection testing

---

### 2. Migration Plan Created

**File:** `/docs/read-model-migration-plan.md`

**Contents:**
- **Current State Analysis**: Problems with domain-wide read models
- **Target State Architecture**: Page-specific tables and mapping
- **5-Phase Migration Strategy**:
  - Phase 1: Parallel Implementation (Weeks 1-2)
  - Phase 2: Update Route Handlers (Week 3)
  - Phase 3: Backfill Existing Data (Week 4)
  - Phase 4: Deprecate Old Tables (Week 5)
- **SQL Migration Files**: Complete schema definitions (migrations 101-106)
- **Projection Handler Code**: Rust implementation examples
- **Testing Strategy**: Unit tests with `unsafe_oneshot` pattern
- **Rollback Plan**: Risk mitigation and recovery procedures
- **Success Metrics**: Performance and coupling improvements

**Timeline:** 5 weeks total with minimal risk

---

### 3. Tech Spec Template Created

**File:** `/docs/tech-spec-template.md`

**Sections:**
1. **Overview**: Problem statement, success criteria, out of scope
2. **Architecture & Design**: Component diagrams, domain models, event flows
3. **Data Models**: Aggregates, page-specific read models, form data consistency
4. **API Endpoints**: Route definitions, handlers, endpoint table
5. **Business Logic**: Domain rules, command handlers, validation
6. **UI/UX Specifications**: Templates, TwinSpark interactions, accessibility
7. **Testing Strategy**: Unit tests, projection tests with `unsafe_oneshot`, integration tests, E2E tests
8. **Security & Performance**: Authentication, authorization, performance targets
9. **Implementation Plan**: Phase breakdown, dependencies, effort estimates
10. **Open Questions**: Technical, product, and design questions

**Key Features:**
- ✅ Read model specifications with SQL schemas and projection handlers
- ✅ Testing patterns with `unsafe_oneshot` for projections
- ✅ Form data consistency using `evento::load`
- ✅ Business logic in aggregates (not handlers)
- ✅ Tailwind CSS 4.1+ syntax references
- ✅ WCAG 2.1 AA accessibility requirements

---

## Architectural Principles

### 1. Page-Specific Read Models

**Pattern:**
```
Each page → Dedicated read model table → Optimized for that view
```

**Benefits:**
- ✅ No data over-fetching
- ✅ Clear bounded contexts
- ✅ Reduced coupling between pages
- ✅ Optimized query performance

**Example:**
```rust
// Dashboard queries dashboard_meals table
sqlx::query_as!(
    DashboardMeal,
    "SELECT meal_type, recipe_title, prep_required
     FROM dashboard_meals
     WHERE user_id = ? AND date = ?",
    user_id, today
)
.fetch_all(&pool)
.await?
```

---

### 2. Form Data Consistency

**Pattern:**
```
Forms requiring pre-population → Use evento::load (aggregate)
Display-only pages → Use page-specific read models
```

**Rationale:**
- Aggregates are authoritative source
- Prevents race conditions from stale read models
- Acceptable latency for edit forms

**Example:**
```rust
// Edit form handler - needs trusted data
async fn edit_recipe_form_handler(
    Path(recipe_id): Path<String>,
    State(executor): State<evento::Executor>,
) -> Result<impl IntoResponse> {
    // Load aggregate (NOT read model)
    let recipe = evento::load::<Recipe>(&recipe_id)
        .run(&executor)
        .await?;

    Ok(HtmlResponse(EditRecipeTemplate { recipe }))
}
```

---

### 3. Business Logic Location

**Pattern:**
```
Handlers → Thin orchestration (routing, validation structure)
Aggregates → Business logic (rules, validation, state management)
```

**Example:**
```rust
// CORRECT: Business logic in aggregate
impl Recipe {
    pub fn validate_create(&self, cmd: &CreateRecipeCommand) -> Result<(), DomainError> {
        if self.recipe_count >= 10 && self.tier == Tier::Free {
            return Err(DomainError::RecipeLimitReached);
        }
        Ok(())
    }
}

// Handler just orchestrates
async fn create_recipe_handler(
    Form(form): Form<CreateRecipeForm>,
    State(executor): State<evento::Executor>,
) -> Result<impl IntoResponse> {
    form.validate()?; // Structure validation only

    // Invoke domain command (business logic inside aggregate)
    let recipe_id = evento::create::<Recipe>()
        .data(&RecipeCreated { /* ... */ })
        .commit(&executor)
        .await?;

    Ok(Redirect::to(&format!("/recipes/{}", recipe_id)))
}
```

---

### 4. Testing Pattern: `unsafe_oneshot`

**Pattern:**
```
Production → evento::subscribe().run() (async)
Tests → evento::subscribe().unsafe_oneshot() (sync)
```

**Rationale:**
- `run()` processes events asynchronously (eventual consistency)
- `unsafe_oneshot()` blocks until all events processed (deterministic)
- Prevents race conditions in test assertions

**Example:**
```rust
#[tokio::test]
async fn test_recipe_created_updates_read_models() {
    let pool = setup_test_db().await;
    let executor = evento::Executor::new(pool.clone());

    // Create recipe (emits RecipeCreated event)
    let recipe_id = evento::create::<Recipe>()
        .data(&RecipeCreated { /* ... */ })
        .commit(&executor)
        .await?;

    // Process projections synchronously for deterministic testing
    evento::subscribe("test-projection")
        .aggregator::<Recipe>()
        .handler(project_recipe_to_list_view)
        .unsafe_oneshot(&executor) // Blocks until processed
        .await?;

    // Assert: Read model updated (no race conditions)
    let result = sqlx::query!("SELECT title FROM recipe_list WHERE id = ?", recipe_id)
        .fetch_one(&pool)
        .await?;

    assert_eq!(result.title, "Test Recipe");
}
```

---

## Page-Specific Read Model Mapping

**Important:** Pages can have **multiple read models**, each optimized for a specific concern.

| Page/Route | Read Model Tables | Data Included | Query Functions |
|------------|------------------|---------------|----------------|
| **Dashboard** (`/dashboard`) | `dashboard_meals`<br>`dashboard_prep_tasks`<br>`dashboard_metrics` | Today's meals<br>Today's prep tasks<br>Recipe variety stats | `get_dashboard_meals()`<br>`get_dashboard_prep_tasks()`<br>`get_dashboard_metrics()` |
| **Meal Calendar** (`/plan`) | `calendar_view` | Week view (Mon-Sun), all meals | `get_calendar_week()` |
| **Recipe Library** (`/recipes`) | `recipe_list`<br>`recipe_filter_counts` | Recipe cards<br>Filter facet counts | `get_recipe_list()`<br>`get_filter_counts()` |
| **Recipe Detail** (`/recipes/:id`) | `recipe_detail`<br>`recipe_ratings` | Full recipe<br>Aggregated ratings | `get_recipe_detail()`<br>`get_recipe_ratings()` |
| **Edit Recipe Form** (`/recipes/:id/edit`) | N/A (uses `evento::load`) | Aggregate state | `evento::load::<Recipe>()` |
| **Shopping List** (`/shopping`) | `shopping_list_view`<br>`shopping_list_summary` | Categorized items<br>Progress stats | `get_shopping_list()`<br>`get_shopping_summary()` |

### Example: Recipe Library Page (Multiple Read Models)

**Query 1 - Recipe Cards (content):**
```rust
sqlx::query_as!(
    RecipeCard,
    "SELECT id, title, recipe_image, complexity, prep_time_min, cook_time_min, is_favorite
     FROM recipe_list
     WHERE user_id = ?",
    user_id
)
```

**Query 2 - Filter Counts (facets for UI):**
```rust
sqlx::query_as!(
    FilterCount,
    "SELECT filter_type, filter_value, count
     FROM recipe_filter_counts
     WHERE user_id = ?",
    user_id
)
// Returns: [
//   { filter_type: "complexity", filter_value: "simple", count: 12 },
//   { filter_type: "complexity", filter_value: "moderate", count: 8 },
//   { filter_type: "favorite", filter_value: "true", count: 7 },
// ]
```

**Template Usage:**
```html
<!-- Filters sidebar with counts -->
<div class="filters">
  <h3>Complexity</h3>
  <label><input type="checkbox" value="simple"> Simple (12)</label>
  <label><input type="checkbox" value="moderate"> Moderate (8)</label>

  <h3>Favorites</h3>
  <label><input type="checkbox" value="true"> Favorites (7)</label>
</div>

<!-- Recipe cards -->
<div class="recipe-grid">
  {% for recipe in recipes %}
    <!-- Recipe card -->
  {% endfor %}
</div>
```

---

## Quick Reference: When to Use What

### Read Models (Page-Specific Tables)

**Use for:**
- ✅ Display-only pages (dashboards, lists, calendars)
- ✅ High-frequency reads (dashboard, recipe library)
- ✅ Complex queries (joins replaced by denormalized data)

**Example:** Recipe library page showing recipe cards

### Aggregates (`evento::load`)

**Use for:**
- ✅ Forms requiring pre-population (edit forms)
- ✅ Commands requiring validation (business logic)
- ✅ Trusted authoritative data source

**Example:** Edit recipe form pre-filled with current values

### When to Create Multiple Read Models per Page

**Create separate read model tables when:**
- ✅ Different concerns on same page (content vs metadata vs stats)
- ✅ Different update frequencies (filter counts change often, content rarely)
- ✅ Independent query patterns (complex queries vs simple lookups)
- ✅ Improves performance (avoid large joins or aggregations)

**Example - Recipe Library Page:**
- `recipe_list` - Content (20 recipes × 500 bytes = 10KB)
- `recipe_filter_counts` - Metadata (10 filter options × 50 bytes = 500 bytes)
- **Why separate?** Filter counts updated frequently (every recipe create/delete), but recipe content rarely changes after creation. Separate tables = faster filter queries.

**Don't create separate tables if:**
- ❌ Data is tightly coupled (always queried together)
- ❌ No performance benefit (both tables small and rarely change)
- ❌ Adds unnecessary complexity (maintenance overhead > benefit)

### When to Create Page-Specific Table

**Create new table if:**
- ✅ Page has unique data needs not served by existing tables
- ✅ Query performance requires optimization (avoid joins)
- ✅ Reduces coupling (page changes don't affect other pages)

**Don't create if:**
- ❌ Existing table already serves the page efficiently
- ❌ Minimal data differences (minor filtering, not structural)
- ❌ One-off use case (not worth maintenance overhead)

---

## Migration Status

### ✅ Completed
- [x] Solution Architecture updated with page-specific read model pattern
- [x] Migration plan created with 5-phase approach
- [x] Tech spec template created with read model sections

### 🚧 Pending
- [ ] Execute Phase 1: Create migration files, projection handlers (Weeks 1-2)
- [ ] Execute Phase 2: Update route handlers (Week 3)
- [ ] Execute Phase 3: Backfill existing data (Week 4)
- [ ] Execute Phase 4: Deprecate old tables (Week 5)

**Estimated Timeline:** 5 weeks from start to completion

---

## Additional Notes for Future Tasks

### Tailwind CSS 4.1+ Syntax

When implementing UI components, use Tailwind 4.1+ syntax:

```html
<!-- Modern Tailwind 4.1+ syntax -->
<div class="flex items-center gap-4 p-6 bg-primary-500 text-white rounded-xl">
  <button class="px-4 py-2 bg-white text-primary-500 hover:bg-gray-100 transition-colors">
    Action
  </button>
</div>
```

**Key Features:**
- Use `gap-*` instead of `space-x-*` / `space-y-*`
- Use `rounded-xl` / `rounded-2xl` for modern aesthetics
- Use `transition-colors` for smooth hover states
- Semantic color names: `bg-primary-500`, `text-error-500`

### Testing Best Practices

**Always:**
- ✅ Use `unsafe_oneshot` in projection tests
- ✅ Test business logic in aggregate unit tests
- ✅ Test route handlers in integration tests
- ✅ Test critical user flows in E2E tests (Playwright)

**Never:**
- ❌ Put business logic in handlers
- ❌ Use `run()` in tests expecting immediate results
- ❌ Test read models without projection handlers
- ❌ Skip accessibility testing (WCAG 2.1 AA required)

---

## Questions or Concerns?

**Architecture Questions:**
- Review `/docs/solution-architecture.md` Section 2.4 and 3.2
- Review `/docs/read-model-migration-plan.md` for detailed migration strategy

**Implementation Questions:**
- Use `/docs/tech-spec-template.md` as guide for feature specs
- See migration plan for code examples (projection handlers, query functions)

**Testing Questions:**
- See tech spec template Section 7 for testing patterns
- See migration plan for `unsafe_oneshot` examples

---

## Success Criteria

### Performance Improvements
- **Query Time:** <50ms (from 50-100ms with joins)
- **Projection Latency:** <100ms event-to-read-model

### Code Quality Improvements
- **Coupling:** Low (page changes don't affect other pages)
- **Test Coverage:** 80% (up from 65%)
- **Developer Velocity:** +30% (easier to reason about, less coordination)

### Architectural Alignment
- ✅ DDD principles (bounded contexts, aggregates)
- ✅ CQRS pattern (commands write, queries read from projections)
- ✅ Event sourcing (full audit trail, replay capability)
- ✅ Thin handlers (orchestration only, business logic in domain)

---

_Architecture update completed by Winston (Architect Agent) - 2025-10-24_
_All documentation synchronized and ready for implementation._
