# Story 9.3: Create Meal Planning Preferences Form

Status: Approved

## Story

As a frontend developer,
I want to build user preferences form,
so that users can customize meal planning algorithm.

## Acceptance Criteria

1. Template created at `templates/profile/meal_planning_preferences.html`
2. Form displays all preference fields populated with current user values from `UserPreferencesView`
3. Time constraints: numeric inputs for `max_prep_time_weeknight` and `max_prep_time_weekend` (minutes, 0-300 range)
4. Complexity toggle: checkbox for `avoid_consecutive_complex` with label "Avoid complex meals on consecutive days"
5. Cuisine variety: slider input (range 0.0-1.0, step 0.1) with labels "Repeat OK" to "Mix it up!"
6. Dietary restrictions: checkbox list (Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher)
7. Custom allergen: text input for `custom_dietary_restriction` with label "Custom dietary restriction (e.g., shellfish)"
8. Form validation: HTML5 (required, min, max) + server-side with inline error messages
9. Save button submits to `PUT /profile/meal-planning-preferences` with `action` and `method` attributes
10. Success: redirect to `/profile` with toast "Preferences saved successfully"

## Tasks / Subtasks

- [ ] Create preferences form template (AC: #1, #2)
  - [ ] Create `templates/profile/meal_planning_preferences.html`
  - [ ] Define template context: `UserPreferencesView` with current values
  - [ ] Set up form structure with semantic HTML (form, fieldset, legend)
  - [ ] Include CSRF token for form security

- [ ] Implement time constraint inputs (AC: #3)
  - [ ] Add numeric input for `max_prep_time_weeknight` (type="number", min="0", max="300")
  - [ ] Add numeric input for `max_prep_time_weekend` (type="number", min="0", max="300")
  - [ ] Label fields clearly: "Max prep time on weeknights (minutes)"
  - [ ] Pre-populate with current user values from `UserPreferencesView`
  - [ ] Add HTML5 validation attributes (required, min, max)

- [ ] Implement complexity toggle (AC: #4)
  - [ ] Add checkbox input: `name="avoid_consecutive_complex"`, `type="checkbox"`
  - [ ] Label: "Avoid complex meals on consecutive days"
  - [ ] Set `checked` attribute if user preference is true
  - [ ] Style checkbox with Tailwind: accessible focus states

- [ ] Implement cuisine variety slider (AC: #5)
  - [ ] Add range input: `type="range"`, `min="0"`, `max="1"`, `step="0.1"`
  - [ ] Display labels: "Repeat OK" (left), "Mix it up!" (right)
  - [ ] Show current value dynamically (JavaScript or server-rendered)
  - [ ] Pre-populate with current `cuisine_variety_weight` value
  - [ ] Style slider with Tailwind CSS utilities

- [ ] Implement dietary restrictions checkboxes (AC: #6)
  - [ ] Create checkbox list for: Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher
  - [ ] Each checkbox: `name="dietary_restrictions[]"`, `value="{restriction}"`
  - [ ] Pre-check boxes based on user's current `dietary_restrictions` array
  - [ ] Group checkboxes in fieldset with legend "Dietary Restrictions"
  - [ ] Ensure accessible labels with for/id attributes

- [ ] Add custom allergen input (AC: #7)
  - [ ] Add text input: `name="custom_dietary_restriction"`, `type="text"`
  - [ ] Label: "Custom dietary restriction (e.g., shellfish)"
  - [ ] Optional field (not required)
  - [ ] Pre-populate if user has custom restriction
  - [ ] Add placeholder text for guidance

- [ ] Implement form validation (AC: #8)
  - [ ] Add HTML5 validation: required, min, max, pattern attributes
  - [ ] Server-side validation: backend returns 422 with errors
  - [ ] Display inline error messages next to fields (text-red-500, text-sm)
  - [ ] Test validation: submit invalid values, verify error display
  - [ ] Ensure error messages are accessible (ARIA attributes)

- [ ] Configure form submission (AC: #9)
  - [ ] Set form action: `action="/profile/meal-planning-preferences"`
  - [ ] Set form method: `method="POST"` with hidden `_method="PUT"` field
  - [ ] Add "Save Preferences" button (type="submit", styled with primary colors)
  - [ ] Include CSRF token in hidden input field
  - [ ] Test form submission sends all fields correctly

- [ ] Handle success response (AC: #10)
  - [ ] Backend redirects to `/profile` with 303 See Other status
  - [ ] Display success toast: "Preferences saved successfully"
  - [ ] Implement toast/notification component (if not exists)
  - [ ] Test full flow: submit form → redirect → see toast

- [ ] Responsive design and styling
  - [ ] Full-width inputs on mobile (w-full)
  - [ ] Constrained layout on desktop (max-w-md or max-w-lg)
  - [ ] Consistent spacing with Tailwind (space-y-4, space-y-6)
  - [ ] Clear visual grouping of related fields (fieldset borders)
  - [ ] Test on mobile (375px), tablet (768px), desktop (1920px)

- [ ] Integration testing
  - [ ] Write Playwright test for form submission
  - [ ] Test: Load form → Verify fields pre-populated
  - [ ] Test: Update values → Submit → Verify preferences saved
  - [ ] Test: Submit invalid values → Verify error messages displayed
  - [ ] Test: Successful submission redirects to /profile with toast

## Dev Notes

### Architecture Patterns and Constraints

- **Server-Rendered Forms:** Use standard HTML form submission (no AJAX required)
- **Progressive Enhancement:** Form works with or without JavaScript
- **CSRF Protection:** Include CSRF token in all mutating forms
- **Validation Strategy:** HTML5 client-side + server-side validation for security

### Source Tree Components

**Templates:**
- `templates/profile/meal_planning_preferences.html` - New preferences form (create)
- `templates/profile/profile.html` - Profile page with navigation to preferences (may need update)
- `templates/components/toast.html` - Success toast notification (create if needed)

**Backend Routes (Epic 8):**
- `GET /profile/meal-planning-preferences` - Serve form with current values
- `PUT /profile/meal-planning-preferences` - Process form submission, update preferences

**Read Models:**
- `UserPreferencesView` - Contains all preference fields: max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight, dietary_restrictions

**Form DTO:**
```rust
pub struct UpdatePreferencesForm {
    pub max_prep_time_weeknight: u32,
    pub max_prep_time_weekend: u32,
    pub avoid_consecutive_complex: bool,
    pub cuisine_variety_weight: f32,
    pub dietary_restrictions: Vec<String>,
    pub custom_dietary_restriction: Option<String>,
}
```

### Testing Standards

- **Integration Tests:** Playwright verifies form submission and validation
- **Accessibility:** Proper labels, fieldsets, error announcements (ARIA)
- **Security:** CSRF token present, server-side validation enforced
- **Browser Compatibility:** Form works in Chrome 90+, Firefox 88+, Safari 14+

### Form Structure Pattern

```html
<form action="/profile/meal-planning-preferences" method="POST" class="max-w-md space-y-6">
  <input type="hidden" name="_method" value="PUT" />
  <input type="hidden" name="csrf_token" value="{{ csrf_token }}" />

  <fieldset>
    <legend class="text-lg font-semibold">Time Constraints</legend>
    <label for="max_prep_time_weeknight">Max prep time on weeknights (minutes)</label>
    <input type="number" id="max_prep_time_weeknight" name="max_prep_time_weeknight"
           min="0" max="300" value="{{ user_preferences.max_prep_time_weeknight }}" required />

    <label for="max_prep_time_weekend">Max prep time on weekends (minutes)</label>
    <input type="number" id="max_prep_time_weekend" name="max_prep_time_weekend"
           min="0" max="300" value="{{ user_preferences.max_prep_time_weekend }}" required />
  </fieldset>

  <fieldset>
    <legend>Meal Complexity</legend>
    <label>
      <input type="checkbox" name="avoid_consecutive_complex"
             {% if user_preferences.avoid_consecutive_complex %}checked{% endif %} />
      Avoid complex meals on consecutive days
    </label>
  </fieldset>

  <fieldset>
    <legend>Cuisine Variety</legend>
    <div class="flex justify-between text-sm text-gray-600">
      <span>Repeat OK</span>
      <span>Mix it up!</span>
    </div>
    <input type="range" name="cuisine_variety_weight"
           min="0" max="1" step="0.1"
           value="{{ user_preferences.cuisine_variety_weight }}" />
  </fieldset>

  <fieldset>
    <legend>Dietary Restrictions</legend>
    {% for restriction in ["Vegetarian", "Vegan", "GlutenFree", "DairyFree", "NutFree", "Halal", "Kosher"] %}
      <label>
        <input type="checkbox" name="dietary_restrictions[]" value="{{ restriction }}"
               {% if restriction in user_preferences.dietary_restrictions %}checked{% endif %} />
        {{ restriction }}
      </label>
    {% endfor %}
  </fieldset>

  <div>
    <label for="custom_dietary_restriction">Custom dietary restriction (e.g., shellfish)</label>
    <input type="text" id="custom_dietary_restriction" name="custom_dietary_restriction"
           value="{{ user_preferences.custom_dietary_restriction }}" />
  </div>

  <button type="submit" class="btn-primary">Save Preferences</button>
</form>
```

### Project Structure Notes

**New Files:**
- `templates/profile/meal_planning_preferences.html` - Preferences form template

**Modified Files:**
- May need to update profile navigation to link to preferences page

**Backend Integration:**
- Route handlers in `src/routes/profile.rs` (Epic 8)
- Form processing extracts data, validates, sends `UpdateMealPlanningPreferences` command

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.3 (AC-9.3.1 through AC-9.3.10)]
- [Source: docs/tech-spec-epic-9.md#Detailed Design → Services and Modules → Form Handling]
- [Source: docs/tech-spec-epic-9.md#Data Models and Contracts → User Preferences Read Model]
- [Source: docs/tech-spec-epic-9.md#Workflows → Workflow 3: Update Meal Planning Preferences]
- [Source: docs/epics.md#Epic 9 → Story 9.3]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.3.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
