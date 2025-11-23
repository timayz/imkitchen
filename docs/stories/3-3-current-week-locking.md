# Story 3.3: Current Week Locking

Status: drafted

## Story

As a user,
I want my current week (today falls within Monday-Sunday) to be locked,
So that in-progress meals are preserved when I regenerate future weeks.

## Acceptance Criteria

1. Generation algorithm skips current week if it exists from previous generation
2. Current week determination: today's date falls within Monday-Sunday range
3. Visual lock icon/badge displayed on current week in calendar
4. Tooltip explains: "Current Week - Won't be regenerated"
5. Regeneration only affects future weeks (week start date > current Sunday)
6. Tests verify current week preservation across regenerations

## Tasks / Subtasks

- [ ] Implement current week detection logic (AC: #2)
  - [ ] Create helper function `is_current_week(week_start_date: NaiveDate, today: NaiveDate) -> bool`
  - [ ] Calculate Monday-Sunday range for given week_start_date
  - [ ] Check if today falls within that range
  - [ ] Return true if today is within range

- [ ] Update generation algorithm to skip current week (AC: #1, #5)
  - [ ] Before generating, load existing meal plans for user
  - [ ] Identify which existing week (if any) is the current week
  - [ ] Exclude current week from list of weeks to generate
  - [ ] Only generate weeks with start_date > current Sunday
  - [ ] Update `is_current_week` flag in meal_plans table

- [ ] Add is_current_week flag to meal plan data (AC: #1)
  - [ ] Update `WeekData` struct to include `is_current_week` boolean
  - [ ] Set flag during generation based on current week detection
  - [ ] Store flag in meal_plans projection table
  - [ ] Update query functions to return flag for calendar display

- [ ] Create calendar UI component for locked week indicator (AC: #3, #4)
  - [ ] Add lock icon SVG to static assets
  - [ ] Update calendar template to check `is_current_week` flag
  - [ ] Display lock badge on current week header
  - [ ] Add Tailwind styling for visual distinction
  - [ ] Include tooltip with explanation text

- [ ] Implement tooltip functionality (AC: #4)
  - [ ] Add tooltip HTML element with "Current Week - Won't be regenerated" text
  - [ ] Use CSS-only tooltip (no JavaScript required)
  - [ ] Position tooltip on hover over lock icon
  - [ ] Ensure mobile-friendly tap interaction

- [ ] Update regeneration command to preserve current week (AC: #5)
  - [ ] Create new `RegenerateFutureWeeks` command method
  - [ ] Load existing meal plans, identify current week
  - [ ] Calculate only future weeks to regenerate
  - [ ] Delete only future week projections (keep current week)
  - [ ] Emit AllFutureWeeksRegenerated event with preserved current week ID

- [ ] Write unit tests for current week detection (AC: #2, #6)
  - [ ] Test with today = Monday of week (should be current)
  - [ ] Test with today = Wednesday of week (should be current)
  - [ ] Test with today = Sunday of week (should be current)
  - [ ] Test with today = previous Sunday (should NOT be current)
  - [ ] Test with today = next Monday (should NOT be current)

- [ ] Write integration tests for week locking (AC: #1, #5, #6)
  - [ ] Generate initial meal plan with 4 weeks
  - [ ] Mock "today" to be in Week 2
  - [ ] Execute regenerate command
  - [ ] Verify Week 2 is_current_week=true and unchanged
  - [ ] Verify Weeks 3+ are regenerated with new recipes
  - [ ] Use evento::load to validate meal plan state

- [ ] Write E2E test for visual lock indicator (AC: #3, #4)
  - [ ] Use Playwright to load calendar page
  - [ ] Verify lock icon displayed on current week
  - [ ] Hover over lock icon
  - [ ] Verify tooltip text: "Current Week - Won't be regenerated"
  - [ ] Verify locked week styling differs from other weeks

## Dev Notes

### Architecture Patterns

- **State Preservation**: Regeneration command must query existing state before emitting events
- **UI Indicator**: Pure CSS tooltip to avoid JavaScript complexity
- **Week Boundary Logic**: Use chrono crate for date manipulation and week calculations

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/generator.rs` - Current week detection helper
- `crates/imkitchen-mealplan/src/command.rs` - Regeneration command
- `crates/imkitchen-mealplan/src/event.rs` - Add is_current_week to WeekData
- `src/queries/mealplans.rs` - Update query functions to return flag
- `templates/pages/mealplan/calendar.html` - Lock icon and tooltip
- `tests/e2e/user_flows.spec.ts` - E2E test for lock indicator

### Technical Constraints

**Current Week Calculation** [Source: epics.md Story 3.3 ACs]:
```rust
use chrono::{Datelike, Duration, NaiveDate, Weekday};

fn is_current_week(week_start_date: NaiveDate, today: NaiveDate) -> bool {
    // week_start_date is a Monday
    let week_end_date = week_start_date + Duration::days(6); // Sunday
    today >= week_start_date && today <= week_end_date
}

fn get_current_week_id(user_id: &str, today: NaiveDate, pool: &SqlitePool) -> Option<String> {
    // Query meal_plans for user where today falls in week range
    sqlx::query_scalar("SELECT id FROM meal_plans WHERE user_id = ? AND ? BETWEEN week_start_date AND date(week_start_date, '+6 days')")
        .bind(user_id)
        .bind(today.format("%Y-%m-%d").to_string())
        .fetch_optional(pool)
        .await
}
```

**Regeneration Logic** [Source: PRD.md FR020, FR023]:
- User clicks "Regenerate" button (requires confirmation modal - Story 3.4)
- Command loads existing meal plans
- Identifies current week by checking is_current_week flag or date range
- Calculates future weeks only (start_date > current Sunday)
- Deletes future week projections from queries.db
- Emits AllFutureWeeksRegenerated event with new weeks

**Lock Icon UI** [Source: Visual Design References]:
```html
<div class="week-header" data-is-current="{{week.is_current_week}}">
  {% if week.is_current_week %}
  <span class="lock-badge tooltip-container">
    <svg class="lock-icon" ...></svg>
    <span class="tooltip">Current Week - Won't be regenerated</span>
  </span>
  {% endif %}
  Week {{week.week_number}}
</div>
```

**Database Flag** [Source: architecture.md Data Architecture]:
- `meal_plans.is_current_week` column already defined in schema
- Set to true for current week, false for others
- Query handler updates flag during generation/regeneration

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Pure function tests for date calculations
  - Edge cases: today = Monday, Sunday, month boundaries
  - No database interaction needed
- **Integration Tests**: Full regeneration flow
  - Use sqlx::migrate! for setup
  - Mock time with fixed "today" date
  - Verify preservation via evento::load
  - NEVER use direct SQL in tests
- **E2E Tests**: Visual verification with Playwright
  - Navigate to calendar page
  - Verify lock icon presence and tooltip
  - Critical user experience test

### References

- [Source: epics.md#Epic 3 Story 3.3]
- [Source: PRD.md FR020 - Current week locking preservation]
- [Source: architecture.md Data Architecture - meal_plans.is_current_week]
- [Source: CLAUDE.md Testing Guidelines - E2E with Playwright]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
