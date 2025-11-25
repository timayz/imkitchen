# Story 4.5: Dashboard Freemium Restrictions

Status: drafted

## Story

As a free tier user,
I want the dashboard to show nearest day only if within my accessible first week,
So that I understand the freemium limitations while seeing immediate value.

## Acceptance Criteria

1. Free tier dashboard shows nearest day only if it falls within first generated week
2. If nearest day is outside first week, show upgrade prompt: "Upgrade to see upcoming meals"
3. Premium tier dashboard always shows nearest day from any generated week
4. Upgrade prompt links to pricing/upgrade page
5. Tests verify dashboard display logic for free vs premium tiers

## Tasks / Subtasks

- [ ] Integrate Access Control in dashboard route (AC: #1, #2, #3)
  - [ ] Modify `src/routes/dashboard.rs` to use AccessControlService
  - [ ] Calculate which week contains nearest day
  - [ ] Check access: `access_control.can_view_week(user_id, week_number)`
  - [ ] If access denied → set dashboard state to `restricted`
  - [ ] If access allowed → proceed with normal nearest day display
- [ ] Update dashboard template for restricted state (AC: #2, #4)
  - [ ] Modify `templates/pages/dashboard.html` to handle restricted state
  - [ ] Display upgrade prompt section when state = restricted
  - [ ] Show message: "Upgrade to see upcoming meals beyond Week 1"
  - [ ] Display "Upgrade Now" CTA button linking to `/pricing`
  - [ ] Show feature teaser: "See all weeks, unlimited favorites, full calendar access"
  - [ ] Hide normal nearest day display when restricted
- [ ] Create upgrade prompt component (AC: #2, #4)
  - [ ] Create `templates/components/dashboard-upgrade-prompt.html`
  - [ ] Include lock icon and upgrade messaging
  - [ ] Display pricing preview: "From $9.99/month"
  - [ ] Link to pricing page with clear CTA
  - [ ] Style with Tailwind for visual prominence without being intrusive
  - [ ] Use color scheme consistent with locked weeks in calendar
- [ ] Modify nearest day query logic (AC: #1, #3)
  - [ ] Update `queries::mealplans::get_nearest_day_meals()` to return week_number
  - [ ] Dashboard route checks access before displaying
  - [ ] Premium users: nearest day always displayed regardless of week
  - [ ] Free users: nearest day displayed only if in week 1
- [ ] Add access check to dashboard handler (AC: #5)
  - [ ] Extract user_id from JWT
  - [ ] Query user profile for tier information
  - [ ] Check premium status via AccessControlService
  - [ ] Determine dashboard display mode: full_access or restricted
  - [ ] Pass mode to template for conditional rendering
- [ ] Write integration tests (AC: #5)
  - [ ] Extend `tests/dashboard_test.rs` with access control tests
  - [ ] Test free tier, nearest day in week 1: displays normally
  - [ ] Test free tier, nearest day in week 2+: shows upgrade prompt
  - [ ] Test premium tier, nearest day in any week: displays normally
  - [ ] Test upgrade prompt rendering: verify message and CTA link
  - [ ] Test global bypass: free user sees all weeks with bypass=true
  - [ ] Test per-user bypass: free user with premium_bypass sees all weeks

## Dev Notes

### Architecture Patterns and Constraints

**Access Control Integration:**
- Dashboard uses same AccessControlService as calendar (Story 4.4)
- Week-level access check: nearest day's week must be accessible
- Consistent freemium enforcement across dashboard and calendar
- No duplicate access logic - single service used everywhere

**Dashboard State Logic:**
```rust
// src/routes/dashboard.rs
pub async fn get_dashboard(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> impl IntoResponse {
    // Get nearest day meals
    let nearest_day = queries::mealplans::get_nearest_day_meals(&state.pool, &user.id).await?;

    if let Some(day) = nearest_day {
        // Check access to nearest day's week
        let can_view = state.access_control.can_view_week(&user.id, day.week_number).await?;

        if can_view {
            // Full access: display nearest day meals
            DashboardTemplate { state: "full", nearest_day: Some(day), ... }
        } else {
            // Restricted: show upgrade prompt
            DashboardTemplate { state: "restricted", nearest_day: None, ... }
        }
    } else {
        // No meal plans: show empty state (Story 4.3)
        DashboardTemplate { state: "empty", ... }
    }
}
```

**Freemium Conversion Touchpoints:**
- Dashboard upgrade prompt complements calendar locked weeks (Story 4.4)
- Reinforces value proposition: see upcoming meals beyond current week
- Links to pricing page for conversion (created in Story 6.3)
- Non-intrusive: prompt appears only when nearest day is inaccessible

**User Experience Flow:**
1. Free user generates meal plan (4-5 weeks)
2. Week 1 passes, nearest day now in week 2
3. Dashboard shows upgrade prompt instead of meals
4. User clicks "Upgrade Now" → pricing page → conversion
5. After upgrade, dashboard displays nearest day from any week

**Week Number Calculation:**
- Meal plan query returns week_number (1-5) for nearest day
- Week 1 = first generated week (always accessible to free tier)
- Weeks 2+ = restricted for free tier unless premium or bypass

**Testing Edge Cases:**
- Nearest day exactly on week boundary (Sunday → Monday transition)
- User with no meal plans (empty state, not restricted state)
- User with only current week locked (all future weeks regenerated)
- Premium user after subscription expires (should see restricted state)

### Project Structure Notes

**Files to Create/Modify:**
- `src/routes/dashboard.rs` - Integrate AccessControlService (MODIFY)
- `src/queries/mealplans.rs` - Add week_number to NearestDayMeals struct (MODIFY)
- `templates/pages/dashboard.html` - Add restricted state UI (MODIFY)
- `templates/components/dashboard-upgrade-prompt.html` - Upgrade prompt component (NEW)
- `tests/dashboard_test.rs` - Add access control tests (MODIFY)

**Query Modification:**
```rust
// src/queries/mealplans.rs
pub struct NearestDayMeals {
    pub date: String,
    pub day_name: String,
    pub week_number: usize,  // ADD THIS FIELD
    pub appetizer: Option<RecipeSnapshot>,
    pub main: Option<RecipeSnapshot>,
    pub dessert: Option<RecipeSnapshot>,
    pub has_advance_prep: bool,
}
```

**Template Conditional Rendering:**
```html
<!-- templates/pages/dashboard.html -->
{% if state == "restricted" %}
  {% include "components/dashboard-upgrade-prompt.html" %}
{% else if state == "full" %}
  <!-- Normal nearest day display from Story 4.3 -->
{% else if state == "empty" %}
  <!-- Empty state guide from Story 4.3 -->
{% endif %}
```

**Visual Mockup Alignment:**
- Implements `mockups/dashboard-free.html` upgrade prompt
- Restricted state UI matches calendar locked week styling
- Upgrade CTA consistent with modal in Story 4.4
- Free tier dashboard shows clear value demonstration

**Integration with Pricing Page:**
- Story 6.3 creates `/pricing` route with tier comparison
- Dashboard upgrade prompt links to that page
- Consistent messaging across all freemium touchpoints

### References

- [Source: docs/epics.md#Story 4.5 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR042 - Free tier dashboard restrictions]
- [Source: docs/PRD.md#FR043 - Premium tier dashboard full access]
- [Source: docs/architecture.md#ADR-004 - Centralized Access Control Service]
- [Source: docs/architecture.md#HTTP Routes - /dashboard route contract]
- [Source: CLAUDE.md#Query Guidelines - Query function structure]

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
