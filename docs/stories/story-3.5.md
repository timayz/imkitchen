# Story 3.5: View Recipe Details from Calendar

Status: Approved

## Story

As a **user viewing meal plan calendar**,
I want to **click a meal to see full recipe details**,
so that **I can review instructions before cooking**.

## Acceptance Criteria

1. Clicking recipe card on calendar opens recipe detail modal/page
2. Recipe detail displays: title, full ingredient list, step-by-step instructions with optional timers, prep/cook times, advance prep requirements
3. Dietary tags and complexity badge visible
4. "Replace This Meal" button available for quick substitution
5. Back/close navigation returns to calendar view
6. Recipe detail page optimized for kitchen use (large text, high contrast)
7. Instructions viewable in progressive disclosure (expand step-by-step)

## Tasks / Subtasks

### Task 1: Add Calendar Context to Recipe Detail Route (AC: 1, 5)
- [ ] Modify `GET /recipes/:id` route to accept optional query parameter `?from=calendar&meal_plan_id={id}&assignment_id={id}`
  - [ ] Parse query params in route handler
  - [ ] Pass context flags to template (is_from_calendar, meal_plan_id, assignment_id)
  - [ ] Conditionally set back navigation URL based on context
- [ ] Update meal calendar template to add context params to recipe links
  - [ ] Modify `templates/pages/meal-calendar.html` recipe title link
  - [ ] Add query string: `<a href="/recipes/{{ recipe_id }}?from=calendar&meal_plan_id={{ meal_plan.id }}&assignment_id={{ assignment.id }}">`
- [ ] Write integration test:
  - [ ] Test: Recipe detail with calendar context shows "Back to Calendar" link
  - [ ] Test: Recipe detail without context shows "Back to Dashboard" link

### Task 2: Implement "Replace This Meal" Button (AC: 4)
- [ ] Add conditional "Replace This Meal" button in recipe detail template
  - [ ] Only visible when `is_from_calendar == true`
  - [ ] Button positioned prominently near recipe header
  - [ ] TwinSpark AJAX behavior: `ts-req="/plan/meal/{{ assignment_id }}/replace"`
  - [ ] Button styling: prominent CTA (bg-yellow-500 hover:bg-yellow-600)
- [ ] Template conditional block:
  - [ ] `{% if is_from_calendar && assignment_id %}`
  - [ ] Render "Replace This Meal" button with assignment context
- [ ] Write E2E test:
  - [ ] Test: Button visible when viewing from calendar
  - [ ] Test: Button NOT visible when viewing from recipe library
  - [ ] Test: Clicking button triggers meal replacement (delegates to Story 3.6)

### Task 3: Kitchen Mode Styling Enhancement (AC: 6)
- [ ] Add kitchen mode toggle or query parameter `?kitchen_mode=true`
  - [ ] Option 1: URL parameter (simpler for MVP)
  - [ ] Option 2: Toggle button in template (future enhancement)
- [ ] Create kitchen mode CSS classes in template:
  - [ ] Large text: `text-2xl` for instructions, `text-xl` for ingredients
  - [ ] High contrast: `bg-white text-gray-900` with `border-4 border-gray-800`
  - [ ] Larger touch targets: `p-6` on buttons, `py-4` on list items
  - [ ] Simplified layout: hide edit/delete buttons, focus on content
- [ ] Conditional template rendering:
  - [ ] `{% if kitchen_mode %}` ... `{% else %}` ... `{% endif %}`
  - [ ] Apply kitchen mode classes to header, ingredients, instructions sections
- [ ] Add "Kitchen Mode" toggle button in normal view:
  - [ ] Link to same recipe with `?kitchen_mode=true` parameter
  - [ ] Icon: 🍳 "Kitchen View"
- [ ] Write E2E test:
  - [ ] Test: Kitchen mode renders with large text and high contrast
  - [ ] Test: Toggle button switches between normal and kitchen mode

### Task 4: Progressive Disclosure for Instructions (AC: 7)
- [ ] Implement collapsible instruction steps with TwinSpark
  - [ ] Each step has expand/collapse button (TwinSpark toggle)
  - [ ] Steps default to collapsed (show only step number and first line)
  - [ ] Clicking step expands full instruction text
  - [ ] Optional: "Expand All" / "Collapse All" buttons at top
- [ ] Update instruction rendering in `templates/pages/recipe-detail.html`:
  - [ ] Wrap each step in collapsible container
  - [ ] Add TwinSpark toggle attributes: `ts-toggle="#step-{{ loop.index }}"`
  - [ ] Hidden class toggle: `hidden` removed on click
- [ ] Styling for collapsed/expanded states:
  - [ ] Collapsed: truncate text with ellipsis, lighter background
  - [ ] Expanded: full text, highlighted background
  - [ ] Transition: smooth CSS transition on expand/collapse
- [ ] Write E2E test:
  - [ ] Test: Steps default to collapsed view
  - [ ] Test: Clicking step expands instruction text
  - [ ] Test: "Expand All" button expands all steps

### Task 5: Back Navigation Context Awareness (AC: 5)
- [ ] Update back button logic in recipe detail template
  - [ ] Read `from` query parameter
  - [ ] If `from=calendar`, back button href: `/plan`
  - [ ] If `from` not present, back button href: `/dashboard` or `/recipes`
  - [ ] Button text updates: "Back to Calendar" vs "Back to Dashboard"
- [ ] Template conditional:
  - [ ] `{% if is_from_calendar %}`
  - [ ] `<a href="/plan">← Back to Calendar</a>`
  - [ ] `{% else %}`
  - [ ] `<a href="/dashboard">← Back to Dashboard</a>`
- [ ] Write integration test:
  - [ ] Test: Back button href correct for calendar context
  - [ ] Test: Back button href correct for recipe library context

### Task 6: Ensure Dietary Tags and Complexity Badge Display (AC: 3)
- [ ] Verify existing template renders dietary tags correctly
  - [ ] Template already has dietary tags loop (line 93-99 in recipe-detail.html)
  - [ ] Complexity badge already rendered (line 74-85)
  - [ ] No code changes needed - VERIFICATION ONLY
- [ ] Add unit test for tag display:
  - [ ] Test: Recipe with dietary tags renders badges correctly
  - [ ] Test: Recipe with complexity level renders color-coded badge
  - [ ] Test: Recipe without tags renders gracefully (no badges)

### Task 7: Enhance Recipe Detail Display (AC: 2)
- [ ] Verify all required data displayed in template:
  - [ ] ✅ Title (line 14)
  - [ ] ✅ Full ingredient list with checkboxes (line 130-139)
  - [ ] ✅ Step-by-step instructions with timers (line 146-164)
  - [ ] ✅ Prep/cook times (line 42-60)
  - [ ] ✅ Advance prep requirements (line 102-106)
  - [ ] No code changes needed - VERIFICATION ONLY
- [ ] Add integration test:
  - [ ] Test: Recipe detail includes all required fields
  - [ ] Test: Optional timers display when present in instructions
  - [ ] Test: Advance prep warning visible when recipe requires prep

### Task 8: Write Comprehensive Test Suite (TDD)
- [ ] **Unit tests** (route handler logic):
  - [ ] Test: Query param parsing for calendar context
  - [ ] Test: Kitchen mode flag set correctly from query param
- [ ] **Integration tests** (full HTTP flow):
  - [ ] Test: GET /recipes/:id with calendar context renders correctly
  - [ ] Test: GET /recipes/:id?kitchen_mode=true applies kitchen styling
  - [ ] Test: Back navigation URL set correctly based on context
  - [ ] Test: "Replace This Meal" button visible only from calendar
- [ ] **E2E tests** (Playwright):
  - [ ] Test: User clicks recipe from calendar, sees full recipe detail
  - [ ] Test: Kitchen mode toggle works (large text, high contrast)
  - [ ] Test: Progressive disclosure: steps expand/collapse on click
  - [ ] Test: Back button returns to calendar from recipe detail
  - [ ] Test: "Replace This Meal" button triggers replacement flow
- [ ] Test coverage ≥80% for recipe detail route and template enhancements

## Dev Notes

### Architecture Patterns
- **Server-Side Rendering**: Askama template with context-aware rendering
- **Progressive Enhancement**: TwinSpark for collapsible steps and AJAX replacement
- **CQRS Read Model**: Recipe detail queried from `recipes` read model
- **Context Passing**: Query parameters for navigation context and view modes

### Key Components
- **Route**: `src/routes/recipes.rs::show_recipe_detail()` (update with context params)
- **Template**: `templates/pages/recipe-detail.html` (enhance with calendar context)
- **Query**: `crates/recipe/src/read_model.rs::query_recipe_by_id()` (existing)
- **Calendar Link**: `templates/pages/meal-calendar.html` (update recipe links)

### Data Flow
1. **User clicks recipe from calendar**:
   - Calendar template renders recipe link with context params
   - GET /recipes/:id?from=calendar&meal_plan_id={id}&assignment_id={id}
   - Route handler parses query params
   - Sets is_from_calendar=true, assignment_id, meal_plan_id
   - Template renders with "Replace This Meal" button and "Back to Calendar" link

2. **Kitchen Mode View**:
   - User clicks "Kitchen View" toggle
   - URL adds ?kitchen_mode=true parameter
   - Template applies large text and high contrast classes
   - Simplified layout for cooking focus

3. **Progressive Disclosure**:
   - Instructions default to collapsed (step number + preview)
   - TwinSpark toggle on click expands full text
   - CSS transition for smooth expand/collapse

### Testing Standards
- **TDD Required**: Write failing test first, then implement
- **Coverage Target**: ≥80% for recipe detail route enhancements
- **Test Types**:
  - Unit: Query param parsing, context flag logic
  - Integration: Full HTTP flow with calendar context
  - E2E: User interactions, kitchen mode, progressive disclosure

### Responsive Considerations
- **Mobile**: Large touch targets (44x44px minimum)
- **Kitchen Mode**: Extra large text for viewing from distance
- **Progressive Disclosure**: Essential for mobile (reduces scrolling)

### UI/UX Enhancements
- **Kitchen Mode**: Large text (2xl), high contrast, simplified layout
- **Progressive Disclosure**: Collapsed by default, expand on demand
- **Context-Aware Back Button**: Returns to calendar when appropriate
- **Replace This Meal**: Prominent CTA when viewing from meal plan

### Project Structure Notes
- **Template**: `templates/pages/recipe-detail.html` (ENHANCEMENT - not new file)
- **Route Handler**: `src/routes/recipes.rs::show_recipe_detail()` (UPDATE - add context params)
- **Calendar Template**: `templates/pages/meal-calendar.html` (UPDATE - add context to links)
- **No new domain crates**: Uses existing `recipe` and `meal_planning` crates

### References
- [Source: docs/epics.md#Story 3.5] View Recipe Details from Calendar requirements (lines 656-676)
- [Source: docs/tech-spec-epic-3.md#Story 3.5] Implementation checklist (search "Story 3.5")
- [Source: docs/solution-architecture.md#Server-Side Rendering] Askama template patterns (lines 122-141)
- [Source: docs/solution-architecture.md#TwinSpark] Progressive enhancement patterns (lines 536-558)
- [Source: templates/pages/recipe-detail.html] Existing recipe detail template (full file)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.5.xml` (Generated: 2025-10-17)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
