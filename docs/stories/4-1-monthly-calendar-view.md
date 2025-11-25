# Story 4.1: Monthly Calendar View

Status: drafted

## Story

As a user,
I want to view my generated meal plans in a month-based calendar,
So that I can see all scheduled meals at a glance.

## Acceptance Criteria

1. Calendar displays one month at a time with week rows
2. Each day shows 3 course slots: appetizer, main, dessert
3. Recipe cards display recipe name, type icon, and preparation indicator
4. Empty slots show "Browse community recipes" link
5. Current week badge/lock icon displayed on locked week
6. Responsive layout adapts to mobile and desktop screens
7. Tests verify calendar rendering with generated meal plans

## Tasks / Subtasks

- [ ] Create calendar route handler (AC: #1)
  - [ ] Implement GET `/mealplan` route handler in `src/routes/mealplan/calendar.rs`
  - [ ] Extract user_id from JWT authentication
  - [ ] Query meal plans for current month using `queries::mealplans::get_current_month_plans()`
  - [ ] Calculate week rows (1-5) based on current month
- [ ] Implement meal plan query functions (AC: #1, #2)
  - [ ] Create `queries::mealplans::get_current_month_plans()` to fetch all weeks for current month
  - [ ] Create `queries::snapshots::get_snapshot_by_id()` to fetch recipe snapshot details
  - [ ] Join meal_plans with meal_plan_recipe_snapshots table
  - [ ] Return structured data: weeks → days → course slots (appetizer/main/dessert)
- [ ] Create Askama calendar template (AC: #1, #2, #3, #4, #5, #6)
  - [ ] Create `templates/pages/mealplan/calendar.html` extending base template
  - [ ] Implement month header with navigation placeholders
  - [ ] Render week rows (1-5) with 7-day columns
  - [ ] Display 3 course slots per day (appetizer, main, dessert) using meal-card component
  - [ ] Show recipe name, type icon (color-coded badges), and advance_prep_text indicator
  - [ ] Display "Browse community recipes" link for empty slots with Twinspark routing
  - [ ] Add current week badge/lock icon with tooltip "Current Week - Won't be regenerated"
  - [ ] Implement responsive CSS using Tailwind: desktop (grid layout), mobile (stacked days)
- [ ] Create reusable meal card component (AC: #3)
  - [ ] Create `templates/components/meal-card.html` for recipe display
  - [ ] Display recipe name, type badge (appetizer=blue, main=orange, dessert=pink)
  - [ ] Show preparation indicator if advance_prep_text exists
  - [ ] Handle accompaniment display for main courses with pairing
  - [ ] Apply consistent styling and touch-optimized tap targets
- [ ] Add calendar CSS styling (AC: #6)
  - [ ] Style week rows with Tailwind grid utilities
  - [ ] Color-code recipe type badges (appetizer/main/dessert)
  - [ ] Style current week lock icon/badge
  - [ ] Implement responsive breakpoints (mobile: stacked, desktop: grid)
  - [ ] Ensure touch-optimized UI for kitchen use
- [ ] Write integration tests (AC: #7)
  - [ ] Create `tests/calendar_test.rs`
  - [ ] Test calendar rendering with generated meal plans (3-5 weeks)
  - [ ] Verify 3 course slots per day displayed correctly
  - [ ] Verify empty slots show "Browse community recipes" link
  - [ ] Verify current week lock indicator displayed
  - [ ] Verify responsive layout on mobile viewport
  - [ ] Test with user having no meal plans (empty state)

## Dev Notes

### Architecture Patterns and Constraints

**Askama Server-Side Rendering:**
- Calendar route returns fully rendered HTML via Askama template
- No client-side data fetching - all meal plan data loaded server-side
- Template receives structured data: Vec<Week> containing Vec<Day> containing Vec<MealSlot>
- Reusable meal-card component for consistent recipe display across calendar

**Query Optimization:**
- Single query joins meal_plans with meal_plan_recipe_snapshots for entire month
- Index on `meal_plans(user_id, week_number)` for efficient month filtering
- Snapshots pre-joined to avoid N+1 queries per recipe card
- Query returns denormalized data structure matching template requirements

**Responsive Design:**
- Desktop: CSS Grid with 7-column week layout, all weeks visible
- Mobile: Stacked day cards, one week visible (carousel navigation added in Story 4.2)
- Tailwind breakpoints: `md:grid md:grid-cols-7` for desktop, default stacked for mobile
- Touch targets minimum 44x44px for kitchen use per NFR005

**Current Week Detection:**
- Query calculates current week: today's date falls within Monday-Sunday range
- Week marked with `is_current_week=true` in meal_plans table
- Calendar template displays lock icon/badge using conditional rendering
- Tooltip explains: "Current Week - Won't be regenerated" (Story 3.3 integration)

**Empty State Handling:**
- Empty meal slots (no snapshot_id) display "Browse community recipes" link
- Link routes to `/recipes/community?type={slot_type}` with filter parameter
- Users can discover and favorite recipes to fill gaps (Story 4.8 full implementation)
- Graceful degradation per FR021 - no minimum recipe count enforced

### Project Structure Notes

**Files to Create/Modify:**
- `src/routes/mealplan/calendar.rs` - Calendar route handler (NEW)
- `src/routes/mealplan/mod.rs` - Module declaration and route registration (MODIFY)
- `src/queries/mealplans.rs` - Meal plan query functions (NEW)
- `src/queries/snapshots.rs` - Recipe snapshot query functions (NEW)
- `src/queries/mod.rs` - Module declarations (MODIFY)
- `templates/pages/mealplan/calendar.html` - Calendar page template (NEW)
- `templates/components/meal-card.html` - Reusable meal card component (NEW)
- `tests/calendar_test.rs` - Integration tests (NEW)

**Database Schema Dependencies:**
- Depends on `meal_plans` table (created in Epic 3)
- Depends on `meal_plan_recipe_snapshots` table (created in Epic 3)
- No new migrations required for this story

**Route Registration:**
```rust
// src/routes/mealplan/mod.rs
pub mod calendar;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/mealplan", get(calendar::get_calendar))
}
```

**Visual Mockup Alignment:**
- Implements `mockups/calendar-premium.html` layout structure
- Uses color-coded recipe type badges (appetizer=blue, main=orange, dessert=pink)
- Current week lock indicator matches mockup visual design
- Empty slots match mockup "Browse community recipes" placeholder

### References

- [Source: docs/epics.md#Story 4.1 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR030 - Calendar display requirements]
- [Source: docs/PRD.md#FR031-FR034 - Calendar visibility and empty slot handling]
- [Source: docs/architecture.md#Data Architecture - meal_plans and meal_plan_recipe_snapshots tables]
- [Source: docs/architecture.md#HTTP Routes - /mealplan route contract]
- [Source: docs/architecture.md#Query Pattern - Query function structure]
- [Source: CLAUDE.md#Askama Guidelines - Template syntax and SSR patterns]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Tailwind 4.1+ styling]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

<!-- Debug logs will be added during implementation -->

### Completion Notes List

<!-- Completion notes will be added during implementation -->

### File List

<!-- Files created/modified will be listed during implementation -->
