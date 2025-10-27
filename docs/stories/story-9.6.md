# Story 9.6: Add Week Selector to Shopping List Page

Status: Approved

## Story

As a frontend developer,
I want to add week dropdown to shopping list,
so that users can view shopping lists for different weeks.

## Acceptance Criteria

1. Shopping list page updated at `templates/shopping/shopping_list.html` with week selector
2. Week selector: dropdown (select element) showing all weeks (Week 1, Week 2, ..., Week N)
3. Current week selected by default (selected attribute on option)
4. Changing selection triggers: `ts-req="/shopping?week_id={value}"` with `ts-target="#shopping-list-content"` and `ts-swap="innerHTML"` and `ts-trigger="change"`
5. Dropdown options show week dates: "Week 1 (Oct 28 - Nov 3)", "Week 2 (Nov 4 - Nov 10)"
6. Locked weeks marked with 🔒 icon in dropdown options (prepended to text)
7. Shopping list displays week start date at top: "Shopping List for Week of {Monday date}"
8. Mobile: Dropdown full-width (w-full), easy to tap (min-height 44px)
9. Playwright test verifies week selection updates shopping list content without page reload

## Tasks / Subtasks

- [ ] Add week selector dropdown to shopping list template (AC: #1, #2)
  - [ ] Open `templates/shopping/shopping_list.html` for modification
  - [ ] Add select element with id "week-selector"
  - [ ] Populate options with all weeks (Week 1, Week 2, etc.)
  - [ ] Each option value: week_id from backend data
  - [ ] Position dropdown prominently at top of page

- [ ] Set default selection to current week (AC: #3)
  - [ ] Identify current week from backend data (status="current" or is_locked=true)
  - [ ] Add `selected` attribute to current week option
  - [ ] Ensure correct week pre-selected when page loads
  - [ ] Test: Verify current week selected by default

- [ ] Integrate TwinSpark for week selection (AC: #4)
  - [ ] Add `ts-req="/shopping?week_id={value}"` attribute to select element
  - [ ] Set `ts-target="#shopping-list-content"` to replace list content
  - [ ] Set `ts-swap="innerHTML"` for content replacement
  - [ ] Set `ts-trigger="change"` to trigger on dropdown change
  - [ ] Test: Changing dropdown updates shopping list without page reload

- [ ] Format dropdown options with dates (AC: #5)
  - [ ] Option text format: "Week {N} ({start_date} - {end_date})"
  - [ ] Example: "Week 1 (Oct 28 - Nov 3)"
  - [ ] Date formatting: MMM DD format (Oct 28, Nov 4, etc.)
  - [ ] Ensure date ranges correct for each week
  - [ ] Test: Verify all dropdown options display correctly

- [ ] Add lock icon to current week option (AC: #6)
  - [ ] Prepend 🔒 icon to locked week option text
  - [ ] Example: "🔒 Week 1 (Oct 28 - Nov 3)"
  - [ ] Identify locked weeks from `is_locked` field
  - [ ] Ensure icon displays correctly in dropdown
  - [ ] Test: Verify lock icon shown for current week only

- [ ] Display week start date in shopping list header (AC: #7)
  - [ ] Add heading at top of shopping list: "Shopping List for Week of {Monday date}"
  - [ ] Format: "Shopping List for Week of October 28"
  - [ ] Date should be Monday (ISO 8601 week start)
  - [ ] Update heading when week changes via TwinSpark
  - [ ] Test: Verify heading updates correctly

- [ ] Implement mobile-responsive dropdown (AC: #8)
  - [ ] Apply `w-full` class for full-width on mobile
  - [ ] Set `min-height: 44px` for easy tapping (WCAG AA)
  - [ ] Increase font size on mobile (text-base or text-lg)
  - [ ] Test touch interaction on mobile devices (375px width)
  - [ ] Verify dropdown usable with fingers (no tiny hit areas)

- [ ] Backend integration
  - [ ] Verify `GET /shopping?week_id=:week_id` route exists (Epic 8)
  - [ ] Route returns HTML fragment with shopping list content
  - [ ] Ensure all weeks available via backend data
  - [ ] Test: Backend returns correct shopping list for each week_id

- [ ] Create shopping list content wrapper
  - [ ] Wrap shopping list in div with id "shopping-list-content"
  - [ ] TwinSpark replaces content inside this div
  - [ ] Preserve dropdown outside content wrapper (not replaced)
  - [ ] Test: Dropdown persists when content updates

- [ ] Responsive design and styling
  - [ ] Desktop: Dropdown aligned left or right, max-width (max-w-xs)
  - [ ] Mobile: Full-width dropdown, prominent placement
  - [ ] Clear visual hierarchy (dropdown above shopping list items)
  - [ ] Consistent spacing (mb-4 or mb-6 below dropdown)
  - [ ] Test on mobile (375px), tablet (768px), desktop (1920px)

- [ ] Integration testing (AC: #9)
  - [ ] Write Playwright test for week selection flow
  - [ ] Test: Load shopping list → Current week selected
  - [ ] Test: Select Week 2 → Shopping list updates without reload
  - [ ] Test: Verify new week's items displayed
  - [ ] Test: Verify heading updated to new week date
  - [ ] Test: Back/forward browser buttons work correctly (URL updated)

## Dev Notes

### Architecture Patterns and Constraints

- **Progressive Enhancement:** Shopping list works without JavaScript (dropdown submits form)
- **TwinSpark Partial Updates:** Only shopping list content updates, not full page
- **URL State Management:** Week ID in URL query parameter for bookmarking
- **Mobile-First Design:** Dropdown optimized for touch interaction

### Source Tree Components

**Templates:**
- `templates/shopping/shopping_list.html` - Shopping list page with week selector (modify)
- `templates/shopping/shopping_list_content.html` - Partial template for list content (may need to create)

**Backend Routes (Epic 8):**
- `GET /shopping` - Shopping list for current week (default)
- `GET /shopping?week_id=:week_id` - Shopping list for specific week
- Both routes return full page or HTML fragment based on TwinSpark header

**Read Models:**
- `ShoppingListView` - Contains week_id, start_date, items grouped by category
- `WeekReadModel` - Provides week list with dates for dropdown population

### Testing Standards

- **Integration Tests:** Playwright verifies week selection and content update
- **Accessibility:** Dropdown keyboard navigation, ARIA labels
- **Mobile UX:** Touch-friendly dropdown on mobile devices
- **URL Management:** Verify query parameter updates in browser URL

### Template Structure

```html
<!-- templates/shopping/shopping_list.html -->
<div class="shopping-list-page">
  <h1 class="text-2xl font-bold mb-4">Shopping List</h1>

  <!-- Week Selector Dropdown -->
  <div class="week-selector-container mb-6">
    <label for="week-selector" class="block text-sm font-medium mb-2">Select Week</label>
    <select id="week-selector"
            name="week_id"
            class="w-full md:max-w-xs px-4 py-2 border rounded"
            style="min-height: 44px;"
            ts-req="/shopping?week_id={value}"
            ts-target="#shopping-list-content"
            ts-swap="innerHTML"
            ts-trigger="change"
            ts-req-history="replace">
      {% for week in weeks %}
        <option value="{{ week.id }}"
                {% if week.is_locked %}selected{% endif %}>
          {% if week.is_locked %}🔒 {% endif %}Week {{ week.number }} ({{ week.start_date|date:"M d" }} - {{ week.end_date|date:"M d" }})
        </option>
      {% endfor %}
    </select>
  </div>

  <!-- Shopping List Content (replaced by TwinSpark) -->
  <div id="shopping-list-content">
    <h2 class="text-xl font-semibold mb-4">
      Shopping List for Week of {{ week_start_date|date:"F d" }}
    </h2>

    {% for category in shopping_list.categories %}
      <div class="category mb-6">
        <h3 class="text-lg font-medium mb-2">{{ category.name }}</h3>
        <ul class="space-y-2">
          {% for item in category.items %}
            <li class="flex items-center">
              <input type="checkbox" id="item-{{ item.id }}" class="mr-2" />
              <label for="item-{{ item.id }}">{{ item.quantity }} {{ item.name }}</label>
            </li>
          {% endfor %}
        </ul>
      </div>
    {% endfor %}
  </div>
</div>
```

### TwinSpark Integration Pattern

**Dropdown with TwinSpark:**
```html
<select ts-req="/shopping?week_id={value}"
        ts-target="#shopping-list-content"
        ts-swap="innerHTML"
        ts-trigger="change"
        ts-req-history="replace">
  <!-- Options populated from backend -->
</select>
```

**Backend Response (HTML Fragment):**
```html
<!-- Partial content returned for TwinSpark update -->
<h2 class="text-xl font-semibold mb-4">
  Shopping List for Week of November 4
</h2>

<div class="category mb-6">
  <h3 class="text-lg font-medium mb-2">Produce</h3>
  <ul class="space-y-2">
    <li><input type="checkbox" /> 2 lbs Chicken Breast</li>
    <li><input type="checkbox" /> 1 cup Basmati Rice</li>
  </ul>
</div>
```

### Project Structure Notes

**Modified Files:**
- `templates/shopping/shopping_list.html` - Add week selector dropdown

**Optional New Files:**
- `templates/shopping/shopping_list_content.html` - Partial template (if backend serves separate partial)

**Backend Integration:**
- `src/routes/shopping.rs` - Route handler returns HTML fragment for TwinSpark requests
- Detect TwinSpark via HTTP header, return full page or partial

### URL State Management

**TwinSpark `ts-req-history` Attribute:**
- `ts-req-history="replace"` - Updates URL without page reload
- Enables bookmarking specific week's shopping list
- Browser back/forward buttons work correctly
- Example URL: `/shopping?week_id=week_2_id`

### References

- [Source: docs/tech-spec-epic-9.md#Acceptance Criteria → Story 9.6 (AC-9.6.1 through AC-9.6.9)]
- [Source: docs/tech-spec-epic-9.md#Workflows → Workflow 5: Shopping List Week Selection]
- [Source: docs/tech-spec-epic-9.md#APIs and Interfaces → Backend Route Dependencies]
- [Source: docs/tech-spec-epic-9.md#TwinSpark Request/Response Flow]
- [Source: docs/epics.md#Epic 9 → Story 9.6]

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-9.6.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
