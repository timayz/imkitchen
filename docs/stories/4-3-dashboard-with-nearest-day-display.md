# Story 4.3: Dashboard with Nearest Day Display

Status: drafted

## Story

As a user,
I want to see today's (or nearest upcoming day's) meals on my dashboard,
So that I know what to prepare without navigating the calendar.

## Acceptance Criteria

1. Dashboard displays nearest day's meals (appetizer, main, dessert) with recipe details
2. Advance preparation tasks shown if today's meals require prep
3. "Generate Meal Plan" button shown if no meal plans exist
4. "Regenerate" button shown if meal plans exist
5. Empty state guide displayed when no meal plans generated
6. Tests verify nearest day calculation and display logic

## Tasks / Subtasks

- [ ] Create dashboard route handler (AC: #1, #3, #4, #5)
  - [ ] Implement GET `/dashboard` route in `src/routes/dashboard.rs`
  - [ ] Extract user_id from JWT authentication
  - [ ] Query nearest day's meals using `queries::mealplans::get_nearest_day_meals()`
  - [ ] Determine dashboard state: no_plans, has_plans_with_meals, has_plans_empty_day
  - [ ] Render appropriate template with state-specific data
- [ ] Implement nearest day query function (AC: #1, #2)
  - [ ] Create `queries::mealplans::get_nearest_day_meals()` in `src/queries/mealplans.rs`
  - [ ] Calculate "nearest day": today if in current week, else next day in future weeks
  - [ ] Query meal_plan_recipe_snapshots for nearest day's 3 courses
  - [ ] Join with meal_plans to get week metadata
  - [ ] Return structured data: NearestDayMeals { date, appetizer, main, dessert, advance_prep_tasks }
  - [ ] Extract advance_prep_text from snapshots if present
- [ ] Create dashboard Askama template (AC: #1, #2, #3, #4, #5)
  - [ ] Create `templates/pages/dashboard.html` extending base template
  - [ ] Display nearest day header: "Today's Meals" or "Upcoming: {date}"
  - [ ] Show 3 meal cards (appetizer, main, dessert) reusing meal-card component
  - [ ] Display advance preparation section if any meals have advance_prep_text
  - [ ] Show "Generate Meal Plan" button with POST form if no plans exist
  - [ ] Show "Regenerate" button with confirmation modal if plans exist
  - [ ] Show empty state guide with onboarding steps if no plans
  - [ ] Style with Tailwind CSS for mobile-first responsive design
- [ ] Implement meal plan generation buttons (AC: #3, #4)
  - [ ] "Generate Meal Plan" button submits POST to `/mealplan/generate` route
  - [ ] "Regenerate" button triggers confirmation modal before POST to `/mealplan/regenerate`
  - [ ] Use Twinspark `ts-req` for async generation with pending state
  - [ ] Display loading spinner during generation (handled in Story 3.1 routes)
  - [ ] Redirect to calendar on successful generation
- [ ] Create empty state onboarding guide (AC: #5)
  - [ ] Create `templates/components/empty-state-guide.html` component
  - [ ] Display 3-step onboarding: "1. Add recipes, 2. Favorite recipes, 3. Generate meal plan"
  - [ ] Include links to recipe creation and community browse
  - [ ] Style with clear visual hierarchy and CTAs
  - [ ] Show only when user has no meal plans generated
- [ ] Handle advance preparation display (AC: #2)
  - [ ] Create `templates/components/advance-prep-card.html` component
  - [ ] Display advance_prep_text from each recipe snapshot
  - [ ] Group by meal (appetizer/main/dessert) with clear labels
  - [ ] Highlight time-sensitive prep tasks visually
  - [ ] Show only if at least one meal has advance_prep_text
- [ ] Write integration tests (AC: #6)
  - [ ] Create `tests/dashboard_test.rs`
  - [ ] Test nearest day calculation: today, tomorrow, next week scenarios
  - [ ] Test dashboard with meal plans: displays 3 courses correctly
  - [ ] Test dashboard with advance prep: displays prep tasks
  - [ ] Test dashboard without meal plans: shows empty state guide
  - [ ] Test "Generate" button displays when no plans exist
  - [ ] Test "Regenerate" button displays when plans exist
  - [ ] Test nearest day with empty slots (no recipe assigned)

## Dev Notes

### Architecture Patterns and Constraints

**Nearest Day Calculation Logic:**
- If today's date falls within an existing meal plan week → return today's meals
- Else find first future day with meal plan data → return that day's meals
- Edge case: Current week locked but no future weeks generated → show "Generate" button
- Edge case: All meal slots empty for nearest day → show "Browse community recipes" link per Story 4.8

**Dashboard State Machine:**
```
1. no_meal_plans → Empty state guide + "Generate Meal Plan" button
2. has_meal_plans_with_meals → Nearest day display + "Regenerate" button
3. has_meal_plans_empty_slots → Nearest day with empty warnings + "Regenerate" button
```

**Query Optimization:**
- Single query calculates nearest day and fetches meals
- Use SQL date functions to compare today's date with meal_plans.week_start_date
- Index on `meal_plans(user_id, week_start_date)` for efficient nearest day lookup
- Join meal_plan_recipe_snapshots for 3 courses in single query

**Advance Preparation Display:**
- Extract `advance_prep_text` from each snapshot
- Only display prep section if at least one meal has non-empty advance_prep_text
- Highlight prep tasks that need morning attention (e.g., "Marinate chicken overnight")
- Story 6.1 will add 8 AM reminder notifications based on this data

**Regeneration Confirmation:**
- "Regenerate" button triggers Twinspark modal (created in Story 3.4)
- Modal warns: "This will replace all future weeks. Continue?"
- Confirmation required before POST to `/mealplan/regenerate`
- Current week preserved per Story 3.3 locking logic

**Empty State UX:**
- Clear onboarding guide for new users with zero meal plans
- Steps: 1) Add/import recipes, 2) Favorite recipes, 3) Generate meal plan
- Links to `/recipes/new` and `/recipes/community` for immediate action
- Friendly, encouraging tone without pressure

### Project Structure Notes

**Files to Create/Modify:**
- `src/routes/dashboard.rs` - Dashboard route handler (NEW)
- `src/routes/mod.rs` - Register dashboard route (MODIFY)
- `src/queries/mealplans.rs` - Add `get_nearest_day_meals()` function (MODIFY)
- `templates/pages/dashboard.html` - Dashboard template (NEW)
- `templates/components/empty-state-guide.html` - Onboarding guide component (NEW)
- `templates/components/advance-prep-card.html` - Prep tasks display component (NEW)
- `tests/dashboard_test.rs` - Integration tests (NEW)

**Route Registration:**
```rust
// src/routes/mod.rs
pub mod dashboard;

// In router setup
.route("/dashboard", get(dashboard::get_dashboard))
```

**Query Function Structure:**
```rust
// src/queries/mealplans.rs
pub struct NearestDayMeals {
    pub date: String,  // ISO format YYYY-MM-DD
    pub day_name: String,  // "Monday", "Tuesday", etc.
    pub appetizer: Option<RecipeSnapshot>,
    pub main: Option<RecipeSnapshot>,
    pub dessert: Option<RecipeSnapshot>,
    pub has_advance_prep: bool,
}

pub async fn get_nearest_day_meals(
    pool: &SqlitePool,
    user_id: &str,
) -> anyhow::Result<Option<NearestDayMeals>> {
    // Calculate nearest day with SQL date logic
    // Join meal_plans + meal_plan_recipe_snapshots
    // Return structured data for template
}
```

**Freemium Integration (Story 4.5):**
- Story 4.5 will add freemium restrictions to dashboard
- Current story implements full functionality for premium/bypass users
- Story 4.5 will modify template to show upgrade prompt if nearest day outside accessible week

**Visual Mockup Alignment:**
- Implements `mockups/dashboard-premium.html` layout structure
- Nearest day header with date display
- 3 meal cards using consistent styling from calendar
- Advance prep section positioned prominently
- Empty state matches mockup onboarding guidance

### References

- [Source: docs/epics.md#Story 4.3 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR041 - Dashboard nearest day display]
- [Source: docs/PRD.md#FR044 - Empty state onboarding guide]
- [Source: docs/architecture.md#HTTP Routes - /dashboard route contract]
- [Source: docs/architecture.md#Data Architecture - meal_plans and snapshots tables]
- [Source: CLAUDE.md#Query Guidelines - Query function patterns]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Askama template structure]

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
