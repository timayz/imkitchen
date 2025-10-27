# Story 9.1: Create Multi-Week Calendar Component

Status: Approved

## Story

As a frontend developer,
I want to build multi-week calendar view with tabs/carousel,
so that users can see and navigate between weeks.

## Acceptance Criteria

1. Askama template created at `templates/meal_plan/multi_week_calendar.html`
2. Template displays week tabs (Week 1, Week 2, etc.) with date ranges (e.g., "Week 1 (Oct 28 - Nov 3)")
3. Current week tab highlighted with distinct styling (border, background color) and lock icon 🔒
4. Clicking week tab triggers TwinSpark request: `ts-req="/plan/week/:week_id"` with `ts-target="#calendar-content"` and `ts-swap="innerHTML"`
5. Mobile view displays carousel with swipe navigation instead of tabs (progressive enhancement)
6. Each week displays 7-day grid with breakfast/lunch/dinner slots (Monday-Sunday order)
7. Meal slots show: recipe name, image thumbnail, prep time with icon
8. Styling uses Tailwind CSS 4.1+ utility classes with 8px spacing grid
9. Keyboard navigation between weeks using Tab key and Enter to select
10. Responsive design: Desktop (tabs), Mobile (carousel with left/right arrows)

## Tasks / Subtasks

- [ ] Create Askama template structure (AC: #1)
  - [ ] Create `templates/meal_plan/multi_week_calendar.html` with base structure
  - [ ] Define template parameters: `WeekReadModel` vector, `current_week_id`
  - [ ] Set up Askama template imports and context

- [ ] Implement week tab navigation (AC: #2, #3, #4)
  - [ ] Render week tabs with Week 1, Week 2, etc. labels and date ranges
  - [ ] Apply highlighting to current week tab (border-primary-500, bg-primary-50)
  - [ ] Add lock icon 🔒 to current week tab
  - [ ] Integrate TwinSpark attributes: `ts-req`, `ts-target`, `ts-swap`
  - [ ] Test TwinSpark partial update without page reload

- [ ] Implement mobile carousel view (AC: #5)
  - [ ] Create carousel HTML structure with left/right navigation arrows
  - [ ] Add responsive breakpoint: show tabs on desktop (@md:), carousel on mobile (<768px)
  - [ ] Implement touch/swipe gesture support for mobile
  - [ ] Test carousel navigation on mobile devices

- [ ] Build 7-day meal grid (AC: #6, #7)
  - [ ] Create 7-column grid layout (Monday-Sunday)
  - [ ] Add 3 rows per day: breakfast, lunch, dinner
  - [ ] Display recipe name, image thumbnail (200x200px), prep time icon
  - [ ] Use lazy loading for recipe images (loading="lazy")
  - [ ] Handle empty meal slots gracefully (placeholder or empty state)

- [ ] Apply Tailwind CSS styling (AC: #8)
  - [ ] Use Tailwind 4.1+ utility classes throughout
  - [ ] Implement 8px spacing grid (space-2, space-4, space-8)
  - [ ] Ensure responsive breakpoints: mobile-first approach
  - [ ] Apply consistent color scheme: primary, gray, white
  - [ ] Verify Tailwind purge configuration for production build

- [ ] Implement keyboard navigation (AC: #9)
  - [ ] Ensure week tabs are keyboard-focusable (tabindex or button elements)
  - [ ] Support Tab key for navigation between weeks
  - [ ] Support Enter/Space key to select week
  - [ ] Test logical tab order through component
  - [ ] Add visible focus indicators (2px border, 4px offset)

- [ ] Responsive design testing (AC: #10)
  - [ ] Test on mobile (375px): carousel with arrows
  - [ ] Test on tablet (768px): transition between carousel and tabs
  - [ ] Test on desktop (1920px): tabs layout
  - [ ] Verify touch targets ≥44x44px on mobile
  - [ ] Test landscape and portrait orientations

- [ ] Integration testing
  - [ ] Write Playwright test for week tab navigation
  - [ ] Verify TwinSpark partial update without full page reload
  - [ ] Test calendar renders with 5 weeks of meal data
  - [ ] Verify current week lock icon displayed correctly
  - [ ] Test keyboard navigation flow

## Dev Notes

### Architecture Patterns and Constraints

- **Server-Side Rendering:** Use Askama templates for compile-time checked, SEO-friendly HTML
- **Progressive Enhancement:** TwinSpark for partial updates, degrades gracefully without JavaScript
- **Event-Sourced Read Models:** Template consumes `WeekReadModel` from evento projections
- **Mobile-First Design:** Implement responsive breakpoints with Tailwind CSS utilities

### Source Tree Components

**Templates:**
- `templates/meal_plan/multi_week_calendar.html` - Main calendar component (create new)
- `templates/meal_plan/meal_slot.html` - Individual meal slot partial (may need to create)
- `templates/layout/base.html` - Base layout with TwinSpark script inclusion (existing)

**Backend Routes (Epic 8):**
- `GET /plan` - Returns full multi-week calendar page
- `GET /plan/week/:week_id` - Returns partial HTML fragment for week content

**Read Models:**
- `WeekReadModel` - Contains week data: id, start_date, end_date, status, is_locked, meal_assignments
- `MealAssignmentView` - Contains meal data: date, course_type, recipe, prep_required

**Styling:**
- Tailwind CSS 4.1+ configuration in `tailwind.config.js`
- Custom CSS for complex layouts in `static/css/custom.css` (minimal)

### Testing Standards

- **Integration Tests:** Playwright for end-to-end user flow testing
- **Accessibility:** WCAG AA compliance (keyboard navigation, ARIA labels, contrast ratios)
- **Performance:** First Contentful Paint <1.5s on mobile 3G
- **Browser Compatibility:** Chrome 90+, Firefox 88+, Safari 14+, Edge 90+

### TwinSpark Integration Pattern

```html
<button ts-req="/plan/week/{{week.id}}"
        ts-target="#calendar-content"
        ts-swap="innerHTML"
        ts-req-method="GET"
        class="tab-button">
  Week {{week_number}} ({{week.start_date}} - {{week.end_date}})
</button>

<div id="calendar-content">
  <!-- Week content rendered here, replaced by TwinSpark -->
</div>
```

### Project Structure Notes

**Template Location:** `templates/meal_plan/multi_week_calendar.html`
- Follows existing template structure pattern
- Integrates with Axum route handlers returning `Html<String>`

**Static Assets:** `static/`
- `static/css/tailwind.css` - Compiled Tailwind CSS
- `static/js/twinspark.js` - TwinSpark library (~10KB)

**Route Handler Pattern:**
```rust
pub async fn get_multi_week_calendar(
    State(state): State<AppState>,
    user: User,
) -> Result<Html<String>, AppError> {
    let weeks = query_all_weeks(&state.db, &user.id).await?;
    let template = MultiWeekCalendarTemplate { weeks, current_week_id };
    Ok(Html(template.render()?))
}
```

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.1]
- [Source: docs/tech-spec-epic-9.md#Detailed Design → Services and Modules → Template Modules]
- [Source: docs/tech-spec-epic-9.md#APIs and Interfaces → Backend Route Dependencies]
- [Source: docs/tech-spec-epic-9.md#Workflows and Sequencing → Workflow 1: Multi-Week Calendar Navigation]
- [Source: docs/epics.md#Epic 9 → Story 9.1]
- [Source: docs/architecture-update-meal-planning-enhancements.md (section 8)]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.1.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
