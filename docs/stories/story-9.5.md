# Story 9.5: Add Week Regeneration UI with Confirmation

Status: Approved

## Story

As a frontend developer,
I want to create regeneration buttons with confirmation dialogs,
so that users can regenerate weeks safely.

## Acceptance Criteria

1. "Regenerate This Week" button added to each future week's calendar (not current/past weeks)
2. "Regenerate All Future Weeks" button added to main calendar navigation area
3. Clicking "Regenerate This Week" shows confirmation modal: "Replace meals for Week {X} ({date range})?"
4. Clicking "Regenerate All Future Weeks" shows modal: "Regenerate {N} future weeks? Your current week will be preserved."
5. Confirmation modal has Cancel and Confirm buttons (styled: secondary + primary)
6. Confirm triggers POST to `/plan/week/:week_id/regenerate` or `/plan/regenerate-all-future` with TwinSpark
7. Loading spinner shown during regeneration (overlay on calendar)
8. Success: Calendar updates with new meals via `ts-target="#calendar-content"` and `ts-swap="innerHTML"`
9. Error: Display error message in toast: "Failed to regenerate. Please try again."
10. Locked weeks display disabled text "Cannot Regenerate (week in progress)" instead of button

## Tasks / Subtasks

- [ ] Add "Regenerate This Week" button to future weeks (AC: #1, #10)
  - [ ] Add button to each future week's calendar header
  - [ ] Button label: "Regenerate This Week"
  - [ ] Conditional rendering: only show for weeks with status="future"
  - [ ] Locked weeks: Display disabled text "Cannot Regenerate (week in progress)"
  - [ ] Style button with secondary/warning colors (bg-yellow-500 or bg-orange-500)

- [ ] Add "Regenerate All Future Weeks" button (AC: #2)
  - [ ] Add button to main calendar navigation area (above week tabs)
  - [ ] Button label: "Regenerate All Future Weeks"
  - [ ] Style with prominent warning color (bg-red-500 or bg-orange-600)
  - [ ] Position clearly visible but not intrusive

- [ ] Create confirmation modal component (AC: #3, #4, #5)
  - [ ] Create `templates/components/regeneration_confirmation.html` modal template
  - [ ] Modal title: "Confirm Regeneration"
  - [ ] Dynamic message based on action: single week or all future weeks
  - [ ] Show week number and date range for single week
  - [ ] Show count of future weeks for "Regenerate All"
  - [ ] Cancel button: secondary styling (bg-gray-200, text-gray-700)
  - [ ] Confirm button: primary/danger styling (bg-red-600, text-white)

- [ ] Implement "Regenerate This Week" confirmation (AC: #3)
  - [ ] Button click opens modal with message: "Replace meals for Week {X} ({date range})?"
  - [ ] Pass week_id, week_number, date_range to modal
  - [ ] Modal rendered with dynamic data
  - [ ] Test modal display and data interpolation

- [ ] Implement "Regenerate All Future Weeks" confirmation (AC: #4)
  - [ ] Button click opens modal with message: "Regenerate {N} future weeks? Your current week will be preserved."
  - [ ] Calculate future weeks count (exclude current/locked week)
  - [ ] Display count in modal message
  - [ ] Test modal with various week counts (2, 3, 5 future weeks)

- [ ] Integrate TwinSpark for form submission (AC: #6)
  - [ ] Single week: POST to `/plan/week/:week_id/regenerate` with TwinSpark
  - [ ] All future weeks: POST to `/plan/regenerate-all-future` with TwinSpark
  - [ ] Set `ts-target="#calendar-content"` for content replacement
  - [ ] Set `ts-swap="innerHTML"` to replace calendar content
  - [ ] Include CSRF token in POST requests

- [ ] Implement loading spinner (AC: #7)
  - [ ] Create loading overlay component (semi-transparent backdrop with spinner)
  - [ ] Show spinner when POST request initiated
  - [ ] Position spinner over calendar area
  - [ ] Disable buttons during loading to prevent double submission
  - [ ] Hide spinner when response received (success or error)

- [ ] Handle success response (AC: #8)
  - [ ] Backend returns HTML fragment with updated calendar
  - [ ] TwinSpark replaces #calendar-content with new HTML
  - [ ] Verify new meals displayed in calendar
  - [ ] Hide modal and spinner
  - [ ] Optional: Show success toast "Meals regenerated successfully"

- [ ] Handle error response (AC: #9)
  - [ ] Backend returns error status (500, 422, etc.)
  - [ ] Display error toast: "Failed to regenerate. Please try again."
  - [ ] Keep modal open or close based on UX preference
  - [ ] Hide spinner
  - [ ] Log error to console for debugging

- [ ] Modal interaction and accessibility
  - [ ] Modal opens with fade-in animation
  - [ ] Modal closes on Cancel button click
  - [ ] Modal closes on Confirm button click (after request)
  - [ ] Modal closes on Escape key press
  - [ ] Modal closes on backdrop click (outside modal)
  - [ ] Focus trap: Tab cycles through Cancel and Confirm buttons
  - [ ] ARIA attributes: role="dialog", aria-labelledby, aria-describedby

- [ ] Responsive design
  - [ ] Modal centered on desktop with max-width (max-w-md)
  - [ ] Modal full-height on mobile for easy interaction
  - [ ] Buttons stacked vertically on mobile, horizontal on desktop
  - [ ] Touch-friendly button sizes on mobile (min-height 44px)
  - [ ] Test on mobile (375px), tablet (768px), desktop (1920px)

- [ ] Integration testing (AC: #6, #8, #9)
  - [ ] Write Playwright test for single week regeneration flow
  - [ ] Test: Click button → Modal opens → Click Confirm → Calendar updates
  - [ ] Write Playwright test for all future weeks regeneration
  - [ ] Test: Error handling → Verify toast displayed
  - [ ] Test: Locked week → Verify "Cannot Regenerate" text shown
  - [ ] Test: Cancel button closes modal without regenerating

## Dev Notes

### Architecture Patterns and Constraints

- **Progressive Enhancement:** Modal and regeneration work with or without heavy JavaScript
- **TwinSpark Integration:** POST requests with partial HTML updates (no full page reload)
- **Confirmation Pattern:** Always confirm destructive actions (regeneration replaces meals)
- **Loading States:** Show spinner during async operations for better UX

### Source Tree Components

**Templates:**
- `templates/components/regeneration_confirmation.html` - Modal component (create new)
- `templates/components/loading_spinner.html` - Loading overlay component (create new or reuse)
- `templates/meal_plan/multi_week_calendar.html` - Add regeneration buttons (modify)

**Backend Routes (Epic 8):**
- `POST /plan/week/:week_id/regenerate` - Regenerate single week
- `POST /plan/regenerate-all-future` - Regenerate all future weeks
- Both routes return HTML fragment (partial calendar update) or error status

**Read Models:**
- `WeekReadModel` - Contains `status` (current, future, past) and `is_locked` fields
- Modal needs week count, date ranges for confirmation message

### Testing Standards

- **Integration Tests:** Playwright verifies full regeneration flow with modal
- **Accessibility:** Modal keyboard navigation, ARIA attributes, focus management
- **Error Handling:** Test error cases (network failure, server error)
- **User Experience:** Confirm destructive actions, clear loading states

### Modal Component Pattern

```html
<!-- templates/components/regeneration_confirmation.html -->
<div id="regeneration-modal" class="modal hidden" role="dialog" aria-labelledby="modal-title" aria-describedby="modal-desc">
  <div class="modal-backdrop" onclick="closeModal()"></div>
  <div class="modal-content max-w-md">
    <h2 id="modal-title" class="text-xl font-semibold">Confirm Regeneration</h2>
    <p id="modal-desc" class="mt-2 text-gray-700">
      {% if regenerate_all %}
        Regenerate {{ future_weeks_count }} future weeks? Your current week will be preserved.
      {% else %}
        Replace meals for Week {{ week_number }} ({{ start_date }} - {{ end_date }})?
      {% endif %}
    </p>

    <div class="modal-actions mt-6 flex space-x-4">
      <button type="button" class="btn-secondary" onclick="closeModal()">Cancel</button>
      <button type="submit"
              class="btn-primary bg-red-600"
              ts-req="{% if regenerate_all %}/plan/regenerate-all-future{% else %}/plan/week/{{ week_id }}/regenerate{% endif %}"
              ts-req-method="POST"
              ts-target="#calendar-content"
              ts-swap="innerHTML"
              ts-req-before="showSpinner"
              ts-req-after="hideSpinner">
        Confirm
      </button>
    </div>
  </div>
</div>
```

### Button Integration

```html
<!-- In multi_week_calendar.html -->

<!-- Regenerate All Future Weeks Button -->
<div class="calendar-header">
  <button type="button"
          class="btn-warning bg-orange-600 text-white"
          onclick="openRegenerateAllModal({{ future_weeks_count }})">
    Regenerate All Future Weeks
  </button>
</div>

<!-- Regenerate This Week Button (per week) -->
{% for week in weeks %}
  <div class="week-header">
    <h3>Week {{ week.number }} ({{ week.start_date }} - {{ week.end_date }})</h3>

    {% if week.status == "future" and not week.is_locked %}
      <button type="button"
              class="btn-secondary bg-yellow-500"
              onclick="openRegenerateWeekModal('{{ week.id }}', {{ week.number }}, '{{ week.start_date }}', '{{ week.end_date }}')">
        Regenerate This Week
      </button>
    {% elif week.is_locked %}
      <span class="text-gray-500 text-sm">Cannot Regenerate (week in progress)</span>
    {% endif %}
  </div>
{% endfor %}
```

### JavaScript for Modal Control

```javascript
function openRegenerateWeekModal(weekId, weekNumber, startDate, endDate) {
  // Populate modal with week data
  document.getElementById('modal-desc').innerText =
    `Replace meals for Week ${weekNumber} (${startDate} - ${endDate})?`;
  document.getElementById('regeneration-modal').classList.remove('hidden');
}

function openRegenerateAllModal(futureWeeksCount) {
  // Populate modal with future weeks count
  document.getElementById('modal-desc').innerText =
    `Regenerate ${futureWeeksCount} future weeks? Your current week will be preserved.`;
  document.getElementById('regeneration-modal').classList.remove('hidden');
}

function closeModal() {
  document.getElementById('regeneration-modal').classList.add('hidden');
}

function showSpinner() {
  document.getElementById('loading-spinner').classList.remove('hidden');
}

function hideSpinner() {
  document.getElementById('loading-spinner').classList.add('hidden');
}

// Close modal on Escape key
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeModal();
});
```

### Project Structure Notes

**New Files:**
- `templates/components/regeneration_confirmation.html` - Modal component
- `templates/components/loading_spinner.html` - Loading spinner overlay (if not exists)

**Modified Files:**
- `templates/meal_plan/multi_week_calendar.html` - Add regeneration buttons and modal

**Backend Integration:**
- Routes already exist in Epic 8, return HTML fragments for TwinSpark
- Error responses return appropriate status codes

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.5 (AC-9.5.1 through AC-9.5.10)]
- [Source: docs/tech-spec-epic-9.md#Workflows → Workflow 2: Regenerate All Future Weeks]
- [Source: docs/tech-spec-epic-9.md#APIs and Interfaces → Backend Route Dependencies]
- [Source: docs/epics.md#Epic 9 → Story 9.5]
- [Source: docs/architecture-update-meal-planning-enhancements.md (regeneration UI patterns)]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.5.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
