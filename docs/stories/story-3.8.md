# Story 3.8: Algorithm Transparency (Show Reasoning)

Status: Approved

## Story

As a **user**,
I want to **understand why meals were assigned to specific days**,
so that **I trust the automated system**.

## Acceptance Criteria

1. Hovering over (or tapping) info icon on meal slot shows reasoning tooltip
2. Reasoning displays: "Assigned to Saturday: more prep time available (Complex recipe, 75min total time)"
3. Or: "Assigned to Tuesday: Quick weeknight meal (Simple recipe, 30min total time)"
4. Or: "Prep tonight for tomorrow: Requires 4-hour marinade"
5. Reasoning adapts to actual assignment factors used by algorithm
6. Clear, human-readable language (no technical jargon)
7. Builds user trust in intelligent automation

## Tasks / Subtasks

### Task 1: Capture Assignment Reasoning in Algorithm (AC: 2, 3, 4, 5)
- [ ] Modify `MealPlanningAlgorithm::generate()` to capture reasoning
  - [ ] Add `assignment_reasoning` field to `MealAssignment` struct
  - [ ] Calculate reasoning factors during recipe scoring
  - [ ] Store reasoning string when recipe assigned to slot
- [ ] Implement `generate_reasoning_text()` function
  - [ ] Input: recipe, slot, user_profile, score_factors
  - [ ] Output: human-readable reasoning string
  - [ ] Template: "{day_context}: {primary_reason} ({details})"
- [ ] Reasoning templates for each constraint type:
  - [ ] **Weeknight time constraint**: "Quick weeknight meal (Simple recipe, {time}min total time)"
  - [ ] **Weekend complexity**: "More prep time available (Complex recipe, {time}min total time)"
  - [ ] **Advance prep**: "Prep tonight for tomorrow: Requires {hours}-hour {prep_type}"
  - [ ] **Freshness**: "Fresh ingredients optimal ({ingredient_type} best within 3 days of shopping)"
  - [ ] **Skill match**: "Matches your {skill_level} skill level"
  - [ ] **Default**: "Best fit for {day_name} based on your preferences"
- [ ] Write unit tests:
  - [ ] Test: Weeknight time constraint reasoning
  - [ ] Test: Weekend complexity reasoning
  - [ ] Test: Advance prep reasoning
  - [ ] Test: Multiple factors combined (prioritize primary reason)

### Task 2: Store Reasoning in Read Model (AC: 5)
- [ ] Add `assignment_reasoning` column to `meal_assignments` table
  - [ ] Migration: `ALTER TABLE meal_assignments ADD COLUMN assignment_reasoning TEXT`
  - [ ] Type: TEXT (stores human-readable explanation)
- [ ] Update `MealPlanGenerated` event to include reasoning
  - [ ] Add `assignment_reasoning` field to each assignment in event payload
  - [ ] Maintain backward compatibility (nullable field)
- [ ] Update `meal_plan_generated_handler()` projection
  - [ ] Extract assignment_reasoning from event
  - [ ] INSERT reasoning into meal_assignments.assignment_reasoning column
- [ ] Update `MealSlotReplaced` event to include reasoning
  - [ ] Add `assignment_reasoning` field for new assignment
- [ ] Write integration tests:
  - [ ] Test: Reasoning persisted to database
  - [ ] Test: Query returns reasoning with meal assignments
  - [ ] Test: Reasoning survives meal plan regeneration

### Task 3: Add Info Icon to Meal Slot UI (AC: 1)
- [ ] Update `templates/components/meal-slot.html`
  - [ ] Add info icon (ℹ️ or SVG) next to recipe title
  - [ ] Icon styled as subtle, non-intrusive (gray, small)
  - [ ] Icon positioned top-right of meal slot card
- [ ] Add ARIA attributes for accessibility
  - [ ] `aria-label="View assignment reasoning"`
  - [ ] `role="button"`
  - [ ] `tabindex="0"` for keyboard navigation
- [ ] Style icon with Tailwind CSS
  - [ ] Default: text-gray-400
  - [ ] Hover: text-gray-600
  - [ ] Mobile: larger tap target (min 44x44px)

### Task 4: Implement Tooltip Display Logic (AC: 1, 6)
- [ ] Create `templates/components/reasoning-tooltip.html` partial
  - [ ] Tooltip container with reasoning text
  - [ ] Styled with Tailwind: bg-gray-800, text-white, rounded, shadow
  - [ ] Max-width: 300px, padding: 12px
  - [ ] Arrow pointer to info icon
- [ ] Implement TwinSpark-based tooltip behavior
  - [ ] On hover/tap: show tooltip
  - [ ] On mouse leave/tap outside: hide tooltip
  - [ ] Position: absolute, relative to info icon
  - [ ] Z-index: 50 (above calendar content)
- [ ] Alternative: Create `static/js/reasoning-tooltip.js` for CSP compliance
  - [ ] Event listeners for hover/tap on info icons
  - [ ] Show/hide tooltip with fade-in/fade-out transitions
  - [ ] Keyboard support: focus icon → press Enter → show tooltip, Escape → hide
  - [ ] Mobile: tap to show, tap outside to dismiss
- [ ] Write CSS for tooltip positioning
  - [ ] Desktop: below icon with arrow pointing up
  - [ ] Mobile: centered modal-style overlay
- [ ] Accessibility considerations:
  - [ ] Tooltip has `role="tooltip"`
  - [ ] Info icon has `aria-describedby` pointing to tooltip ID
  - [ ] Keyboard navigation support (focus + Enter/Escape)

### Task 5: Update Meal Calendar Template (AC: 1, 7)
- [ ] Modify `templates/pages/meal-calendar.html`
  - [ ] Include info icon in each meal slot
  - [ ] Pass `assignment_reasoning` data to template
  - [ ] Render tooltip partial with reasoning text
- [ ] Update calendar query to include reasoning
  - [ ] Modify `get_meal_calendar()` read model query
  - [ ] SELECT assignment_reasoning from meal_assignments
  - [ ] Include reasoning in MealCalendarView struct
- [ ] Handle missing reasoning gracefully
  - [ ] If reasoning NULL or empty, hide info icon
  - [ ] Fallback text: "Meal assigned based on your preferences"

### Task 6: Mobile-Specific Tooltip Behavior (AC: 1)
- [ ] Detect mobile viewport (CSS media query or JS)
  - [ ] Mobile: <768px width
  - [ ] Desktop: >=768px width
- [ ] Mobile tooltip behavior:
  - [ ] Tap info icon → show tooltip as modal overlay
  - [ ] Tooltip centered on screen, not positioned near icon
  - [ ] Dark overlay background (bg-black/50)
  - [ ] Tap overlay → dismiss tooltip
  - [ ] Close button (X) in top-right of tooltip
- [ ] Desktop tooltip behavior:
  - [ ] Hover icon → show tooltip near icon
  - [ ] Positioned relative to icon (below with arrow)
  - [ ] Mouse leave → hide tooltip after 500ms delay

### Task 7: Write Comprehensive Test Suite (TDD)
- [ ] **Unit tests** (reasoning generation):
  - [ ] Test: generate_reasoning_text() for weeknight constraint
  - [ ] Test: generate_reasoning_text() for weekend complexity
  - [ ] Test: generate_reasoning_text() for advance prep
  - [ ] Test: generate_reasoning_text() for freshness constraint
  - [ ] Test: Prioritize primary reason when multiple factors
  - [ ] Test: Human-readable language (no jargon)
- [ ] **Integration tests** (persistence):
  - [ ] Test: Reasoning persisted to database on meal plan generation
  - [ ] Test: Query returns reasoning with meal assignments
  - [ ] Test: Reasoning included in meal calendar view
  - [ ] Test: Reasoning preserved during meal slot replacement
- [ ] **E2E tests** (Playwright):
  - [ ] Test: Info icon visible on meal slots
  - [ ] Test: Hover shows tooltip with reasoning text
  - [ ] Test: Tap shows tooltip on mobile
  - [ ] Test: Tooltip dismisses on click outside (mobile)
  - [ ] Test: Keyboard navigation (focus + Enter shows tooltip)
- [ ] Test coverage: Maintain 80%+ code coverage

## Dev Notes

### Architecture Patterns
- **Algorithm Enhancement**: Extend MealPlanningAlgorithm to capture decision rationale
- **Data Enrichment**: Store reasoning in read model for fast display
- **Progressive Enhancement**: Tooltips work without JavaScript (CSS :hover fallback)
- **Accessibility**: ARIA labels, keyboard navigation, screen reader support
- **Trust Building**: Transparency increases user confidence in automation

### Key Components
- **Algorithm**: `crates/meal_planning/src/algorithm.rs::generate_reasoning_text()` (NEW)
- **Aggregate**: `crates/meal_planning/src/aggregate.rs::MealPlan` (UPDATE - include reasoning in events)
- **Read Model**: `crates/meal_planning/src/read_model.rs` (UPDATE - query reasoning)
- **Migration**: `migrations/009_add_assignment_reasoning.sql` (NEW)
- **Template Component**: `templates/components/reasoning-tooltip.html` (NEW)
- **JavaScript**: `static/js/reasoning-tooltip.js` (NEW - CSP compliant)
- **Template Update**: `templates/pages/meal-calendar.html` (UPDATE - add info icons)

### Data Flow
1. **Meal Plan Generation**:
   - MealPlanningAlgorithm scores recipes for slots
   - For each assignment, generate reasoning text
   - Include reasoning in MealPlanGenerated event
   - Projection stores reasoning in meal_assignments table

2. **Display on Calendar**:
   - User views meal calendar (GET /plan)
   - Query meal_assignments with assignment_reasoning
   - Render meal slots with info icons
   - Pass reasoning to tooltip component

3. **Tooltip Interaction**:
   - User hovers/taps info icon
   - JavaScript/TwinSpark shows tooltip
   - Tooltip displays reasoning text
   - User reads explanation, builds trust
   - User dismisses tooltip (mouse leave/tap outside)

### Reasoning Examples

**Weeknight Time Constraint**:
- "Assigned to Tuesday: Quick weeknight meal (Simple recipe, 30min total time)"
- "Weeknight favorite: Ready in 25 minutes for busy evenings"

**Weekend Complexity**:
- "Assigned to Saturday: More prep time available (Complex recipe, 75min total time)"
- "Weekend project: This recipe takes time but worth the effort"

**Advance Prep**:
- "Prep tonight for tomorrow: Requires 4-hour marinade"
- "Start dough rising at 2pm for tomorrow's dinner"

**Freshness**:
- "Assigned to Monday: Fresh seafood best within 3 days of shopping"
- "Early-week salad: Uses produce at peak freshness"

**Skill Match**:
- "Matches your Intermediate skill level: 12 steps, moderate techniques"
- "Simple recipe perfect for weeknight cooking"

**Default Fallback**:
- "Best fit for Wednesday based on your preferences"
- "Meal assigned to balance variety throughout the week"

### Project Structure Notes

**Alignment with Solution Architecture**:
- **Algorithm Transparency**: Builds user trust in automation [Source: docs/PRD.md#User Trust]
- **evento Events**: Reasoning stored in event payload for audit [Source: docs/solution-architecture.md#Event Sourcing]
- **Read Model Extension**: Add reasoning column to existing table [Source: docs/solution-architecture.md#Data Models]
- **Progressive Enhancement**: Tooltips with TwinSpark and CSS fallback [Source: docs/solution-architecture.md#Progressive Enhancement]

**Lessons from Story 3.7**:
- **CSP Compliance**: Extract JavaScript to external file [Source: Story 3.7 Action Item #1]
- **Keyboard Navigation**: Support Escape/Enter for tooltip [Source: Story 3.7 Action Item #2]
- **ARIA Attributes**: Add role, aria-label, aria-describedby [Source: Story 3.7 Action Item #3]
- **Mobile UX**: Tap targets min 44x44px, modal overlay for mobile [Source: Story 3.7 Mobile Testing]

**New Components**:
- `crates/meal_planning/src/algorithm.rs::generate_reasoning_text()` - Reasoning generator
- `migrations/009_add_assignment_reasoning.sql` - Database schema update
- `templates/components/reasoning-tooltip.html` - Tooltip component
- `static/js/reasoning-tooltip.js` - Tooltip interaction logic (CSP compliant)

### Testing Strategy

**TDD Approach**:
1. Write test for reasoning generation (weeknight constraint)
2. Implement `generate_reasoning_text()` to pass test
3. Write test for reasoning persistence
4. Update event handler to store reasoning
5. Write E2E test for tooltip display
6. Implement tooltip UI and interaction

**Reasoning Quality Tests**:
- Human-readable language (no technical jargon)
- Concise (max 100 characters)
- Accurate (reflects actual algorithm decision)
- Helpful (explains "why" not just "what")

**Accessibility Tests**:
- Screen reader announces reasoning when focus on icon
- Keyboard navigation shows/hides tooltip
- Sufficient color contrast (4.5:1 minimum)
- Touch targets meet 44x44px minimum

### References

- [Source: docs/epics.md#Story 3.8] Algorithm Transparency requirements (lines 731-750)
- [Source: docs/tech-spec-epic-3.md#MealPlanningAlgorithm] Algorithm constraint types (lines 101-131)
- [Source: docs/tech-spec-epic-3.md#Algorithm Transparency] Reasoning generation (lines 450-520 if exists)
- [Source: docs/solution-architecture.md#Progressive Enhancement] TwinSpark patterns (lines 122-141)
- [Source: docs/solution-architecture.md#Accessibility] WCAG 2.1 compliance (lines 894-911)
- [Source: Story 3.7 Completion Notes] Lessons on CSP, accessibility, mobile UX

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.8.xml` (Generated: 2025-10-17)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
