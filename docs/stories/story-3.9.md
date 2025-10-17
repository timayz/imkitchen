# Story 3.9: Home Dashboard with Today's Meals

Status: Approved

## Story

As a **user**,
I want to **see today's meals on my dashboard**,
so that **I immediately know what to cook without navigating**.

## Acceptance Criteria

1. Home dashboard prominently displays "Today's Meals" section at top
2. Shows breakfast, lunch, and dinner assigned for today
3. Each meal displays: recipe title, image placeholder, prep time
4. Advance prep indicator if preparation required today for future meal
5. "View Full Calendar" link to navigate to week view
6. If no meal plan active, displays "Generate Meal Plan" call-to-action
7. Today's meals update automatically at midnight (new day)
8. Click recipe navigates to full recipe detail

## Tasks / Subtasks

### Task 1: Create Dashboard Route and Query Logic (AC: 1, 2, 3, 4, 8)
- [ ] Create `src/routes/dashboard.rs` module
  - [ ] Implement `GET /dashboard` route handler
  - [ ] Add authentication middleware requirement
  - [ ] Extract user ID from JWT claims
- [ ] Query today's meal assignments from read model
  - [ ] Add `get_todays_meals(user_id, date)` query to `crates/meal_planning/src/read_model.rs`
  - [ ] SELECT from `meal_assignments` WHERE user_id=? AND date=TODAY()
  - [ ] Join with recipes table for recipe details (title, prep_time, image_url)
- [ ] Handle case: no active meal plan exists
  - [ ] Query returns empty result → template shows CTA
- [ ] Map query results to `TodaysMealsData` view struct
  - [ ] Fields: breakfast, lunch, dinner (Option<MealSlotData>)
  - [ ] MealSlotData: recipe_id, title, image_url, prep_time, advance_prep_indicator
- [ ] Pass data to Askama template for rendering
- [ ] Write unit tests:
  - [ ] Test: query returns 3 meals (breakfast, lunch, dinner)
  - [ ] Test: query handles missing meals (optional slots)
  - [ ] Test: query filters by today's date correctly
  - [ ] Test: authentication required (401 if not logged in)

### Task 2: Update Dashboard Template (AC: 1, 2, 3, 4, 5, 8)
- [ ] Update `templates/pages/dashboard.html`
  - [ ] Add "Today's Meals" section at top of page
  - [ ] Display 3 meal cards: breakfast, lunch, dinner
  - [ ] Each card shows: recipe title, image placeholder, prep time badge
  - [ ] Conditional: if advance prep required today, show prep indicator icon
  - [ ] Add "View Full Calendar" link to `/plan` route
  - [ ] Wrap each meal card in clickable link to recipe detail (`/recipes/{id}`)
- [ ] Handle empty meals (no meal assigned for slot)
  - [ ] Show empty state: "No {meal_type} planned"
- [ ] Use Tailwind CSS for card styling
  - [ ] Responsive grid: 1 column mobile, 3 columns desktop
  - [ ] Card hover effect for clickability affordance
- [ ] Accessibility:
  - [ ] Semantic HTML (`<section>`, `<article>`)
  - [ ] ARIA labels for meal cards
  - [ ] Keyboard navigation (tab through cards)

### Task 3: Implement "No Meal Plan" State (AC: 6)
- [ ] Conditional template rendering
  - [ ] If query returns no meals → show empty state
  - [ ] Empty state: heading "No Active Meal Plan"
  - [ ] CTA button: "Generate Meal Plan" linking to `/plan/generate`
  - [ ] Helpful text: "Create your first meal plan to get started"
- [ ] Ensure button styling matches primary CTA pattern
- [ ] Write integration test:
  - [ ] Test: user with no meal plan sees CTA
  - [ ] Test: clicking CTA navigates to meal plan generation

### Task 4: Automatic Date Update Logic (AC: 7)
- [ ] Implement server-side date filtering
  - [ ] Query uses `CURRENT_DATE` SQL function (SQLite)
  - [ ] No client-side JavaScript needed (server-rendered on each page load)
- [ ] Verify behavior across timezones
  - [ ] Use server timezone (UTC or configured timezone)
  - [ ] Future enhancement: user timezone preference (out of scope)
- [ ] Write test:
  - [ ] Test: query returns different meals on different dates
  - [ ] Mock system date in test to verify filtering

### Task 5: Recipe Navigation Integration (AC: 8)
- [ ] Link meal cards to `/recipes/{recipe_id}` route
  - [ ] Use existing recipe detail route from Story 3.5
  - [ ] Ensure recipe ID passed correctly in template
- [ ] Test navigation flow:
  - [ ] Test: clicking meal card navigates to recipe detail
  - [ ] Test: recipe detail page loads with correct data

### Task 6: Integration with Meal Plan Generation (AC: 6)
- [ ] Verify `/plan/generate` route exists (from Story 3.1)
  - [ ] CTA button links to generation route
  - [ ] After meal plan generated, redirect back to `/dashboard`
- [ ] Test end-to-end flow:
  - [ ] Test: user with no plan clicks CTA → generates plan → returns to dashboard with meals displayed

### Task 7: Write Comprehensive Test Suite (TDD)
- [ ] **Unit tests** (dashboard route handler):
  - [ ] Test: authenticated user with active meal plan sees today's meals
  - [ ] Test: authenticated user without meal plan sees CTA
  - [ ] Test: unauthenticated user redirected to login
  - [ ] Test: query returns correct meal data structure
- [ ] **Integration tests** (database queries):
  - [ ] Test: `get_todays_meals()` query returns meals for current date
  - [ ] Test: query handles user with no meal plan (empty result)
  - [ ] Test: query joins with recipes table correctly
- [ ] **E2E tests** (Playwright - optional, can defer):
  - [ ] Test: dashboard page loads and displays today's meals
  - [ ] Test: clicking meal navigates to recipe detail
  - [ ] Test: clicking "View Full Calendar" navigates to calendar
  - [ ] Test: no meal plan state shows CTA button

## Dev Notes

### Architecture Patterns
- **Server-Side Rendering**: Full Askama template rendering with no client-side JavaScript required
- **CQRS Read Model**: Query from `meal_assignments` read model for fast dashboard display
- **Authentication**: JWT middleware protects dashboard route
- **Progressive Enhancement**: Server renders correct date on each request (no client-side date logic)
- **Responsive Design**: Mobile-first with Tailwind CSS responsive utilities

### Key Components
- **Route Handler**: `src/routes/dashboard.rs::dashboard_handler()` (NEW)
- **Read Model Query**: `crates/meal_planning/src/read_model.rs::get_todays_meals()` (NEW)
- **Template**: `templates/pages/dashboard.html` (UPDATE)
- **View Struct**: `TodaysMealsData` with breakfast/lunch/dinner slots (NEW)

### Data Flow
1. **Dashboard Load**:
   - User navigates to `/dashboard` (root after login)
   - Auth middleware validates JWT, extracts user_id
   - Route handler calls `get_todays_meals(user_id, CURRENT_DATE)`
   - Query returns meal assignments for today (breakfast, lunch, dinner)
   - Handler maps results to `TodaysMealsData` struct
   - Askama renders dashboard template with today's meals

2. **No Meal Plan State**:
   - Query returns empty result (no active meal plan)
   - Template conditionally renders "Generate Meal Plan" CTA
   - User clicks CTA → navigates to `/plan/generate`
   - After generation, redirects back to dashboard with meals displayed

3. **Recipe Navigation**:
   - User clicks meal card
   - Navigates to `/recipes/{recipe_id}` (existing route from Story 3.5)
   - Recipe detail page displays full recipe information

### Query Details

**SQL Query (SQLite):**
```sql
SELECT
  ma.date,
  ma.meal_type,
  ma.recipe_id,
  r.title,
  r.prep_time_min,
  r.cook_time_min,
  r.advance_prep_hours,
  r.image_url
FROM meal_assignments ma
INNER JOIN recipes r ON ma.recipe_id = r.id
WHERE ma.user_id = ?
  AND ma.date = CURRENT_DATE
ORDER BY
  CASE ma.meal_type
    WHEN 'breakfast' THEN 1
    WHEN 'lunch' THEN 2
    WHEN 'dinner' THEN 3
  END;
```

**View Struct:**
```rust
pub struct TodaysMealsData {
    pub breakfast: Option<MealSlotData>,
    pub lunch: Option<MealSlotData>,
    pub dinner: Option<MealSlotData>,
    pub has_meal_plan: bool,
}

pub struct MealSlotData {
    pub recipe_id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub total_time_min: u32, // prep_time + cook_time
    pub advance_prep_required: bool, // advance_prep_hours.is_some()
}
```

### Project Structure Notes

**Alignment with Solution Architecture:**
- **Dashboard Route**: New route at `/dashboard` as root after login [Source: docs/epics.md#Story 3.9, line 771]
- **Server-Side Rendering**: Askama templates with no client-side framework [Source: docs/solution-architecture.md#Server-Side Rendering]
- **CQRS Read Model**: Query optimized read model for fast dashboard display [Source: docs/solution-architecture.md#CQRS]
- **Authentication**: JWT middleware pattern [Source: docs/solution-architecture.md#Authentication]

**Lessons from Story 3.8:**
- **TDD First**: Write unit tests for query logic before implementation
- **Template Reuse**: Leverage existing meal card display patterns from `meal-calendar.html`
- **E2E Deferral**: Backend logic tests sufficient for MVP, E2E can follow (acceptable pattern)
- **Progressive Enhancement**: Server-side date filtering eliminates client JavaScript complexity

**New Components:**
- `src/routes/dashboard.rs` - Dashboard route handler (NEW)
- `crates/meal_planning/src/read_model.rs::get_todays_meals()` - Today's meals query (NEW)
- `TodaysMealsData` view struct (NEW in `src/routes/dashboard.rs`)

**Updated Components:**
- `templates/pages/dashboard.html` - Add "Today's Meals" section (UPDATE)
- `src/main.rs` or route registration - Register `/dashboard` route (UPDATE)

### Testing Strategy

**TDD Approach**:
1. Write test for `get_todays_meals()` query (returns 3 meals for today's date)
2. Implement query to pass test
3. Write test for dashboard route handler (authenticated user sees meals)
4. Implement route handler to pass test
5. Write test for "no meal plan" state (CTA displayed)
6. Implement conditional template rendering

**Query Tests:**
- Today's date filtering works correctly
- Empty result when no meal plan exists
- Join with recipes table returns complete data

**Route Tests:**
- Authentication required (401 without JWT)
- Correct data passed to template
- Template renders without errors

**Integration Tests:**
- End-to-end: user with meal plan sees today's meals
- End-to-end: user without meal plan sees CTA

### References

- [Source: docs/epics.md#Story 3.9] Dashboard requirements (lines 754-776)
- [Source: docs/tech-spec-epic-3.md#Dashboard] Dashboard technical details (if exists)
- [Source: docs/solution-architecture.md#Route Structure] Dashboard route pattern (lines 160-161)
- [Source: docs/solution-architecture.md#Server-Side Rendering] Askama template rendering (lines 122-141)
- [Source: docs/solution-architecture.md#Authentication] JWT middleware (lines 656-674)
- [Source: Story 3.5 Completion Notes] Recipe detail navigation pattern
- [Source: Story 3.8 Completion Notes] TDD lessons, progressive enhancement

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.9.xml` (Generated: 2025-10-17)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
