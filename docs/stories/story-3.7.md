# Story 3.7: Regenerate Full Meal Plan

Status: Approved

## Story

As a **user**,
I want to **completely regenerate my meal plan**,
so that **I can get fresh variety or restart after disruptions**.

## Acceptance Criteria

1. "Regenerate Meal Plan" button visible on calendar page
2. Confirmation dialog: "This will replace your entire meal plan. Continue?"
3. Clicking confirm triggers full meal plan regeneration
4. Algorithm runs with same logic as initial generation
5. Rotation state preserved (doesn't reset cycle)
6. New plan fills all slots with different recipe assignments
7. Calendar updates to show new plan
8. Shopping list regenerated for new plan
9. Old meal plan archived for audit trail
10. Generation respects same optimization factors (availability, complexity, prep timing)

## Tasks / Subtasks

### Task 1: Implement POST /plan/regenerate Route (AC: 1, 2, 3, 7)
- [ ] Create `RegenerateMealPlanForm` struct with optional `reason` field
  - [ ] Add to `src/routes/meal_plan.rs` with serde derives
  - [ ] Add validation (optional text field, max 500 characters)
- [ ] Implement `post_regenerate_meal_plan` route handler
  - [ ] Accept `Form(RegenerateMealPlanForm)` and Auth middleware
  - [ ] Query user's active meal plan from read model
  - [ ] Validate user has active meal plan to regenerate
  - [ ] Validate user has sufficient favorite recipes (>=7)
- [ ] Invoke domain command `meal_planning::regenerate_meal_plan(cmd)`
  - [ ] Pass `meal_plan_id`, `user_id`, `regeneration_reason`
  - [ ] Handle domain errors (insufficient recipes, no active plan, etc.)
- [ ] Return redirect to calendar view with success message
  - [ ] Flash message: "Meal plan regenerated successfully"
  - [ ] Redirect to GET /plan with auto-scroll to current week
- [ ] Write integration test:
  - [ ] Test: Regenerate meal plan creates new assignments
  - [ ] Test: Old meal plan marked inactive
  - [ ] Test: Rotation state preserved across regeneration
  - [ ] Test: Authorization check prevents cross-user regeneration

### Task 2: Implement Domain Command - RegenerateMealPlan (AC: 4, 5, 6, 9, 10)
- [ ] Create `RegenerateMealPlanCommand` struct in `crates/meal_planning/src/commands.rs`
  - [ ] Fields: meal_plan_id, user_id, regeneration_reason
- [ ] Implement `regenerate_meal_plan()` function in `crates/meal_planning/src/lib.rs`
  - [ ] Load existing MealPlan aggregate from evento event stream
  - [ ] Validate meal plan is active (status check)
  - [ ] Load user profile for algorithm constraints
  - [ ] Query all favorite recipes (>= 7 minimum)
  - [ ] Load current rotation state from aggregate
  - [ ] Invoke MealPlanningAlgorithm with same constraints as initial generation
  - [ ] Generate new assignments using algorithm (different from current)
  - [ ] Preserve rotation state (DO NOT reset cycle)
  - [ ] Emit `MealPlanRegenerated` event with new assignments
  - [ ] Mark old meal plan as inactive (is_active = false)
- [ ] Update MealPlan aggregate handler for `MealPlanRegenerated` event
  - [ ] Replace all assignments with new_assignments
  - [ ] Update rotation_state if any recipes used
  - [ ] Set updated_at timestamp
  - [ ] Keep is_active = true (same meal plan ID)
- [ ] Write unit tests:
  - [ ] Test: Regenerate creates different assignments
  - [ ] Test: Rotation state preserved (cycle_number unchanged)
  - [ ] Test: All constraints satisfied in new plan
  - [ ] Test: Algorithm determinism with different seed

### Task 3: Add Confirmation Modal for Regeneration (AC: 2)
- [ ] Create confirmation modal template
  - [ ] Title: "Regenerate Meal Plan?"
  - [ ] Message: "This will replace your entire meal plan. Continue?"
  - [ ] Optional reason field: "Reason (optional): [text input]"
  - [ ] Buttons: "Cancel" (dismiss modal), "Confirm" (submit form)
- [ ] Update "Regenerate Meal Plan" button in meal-calendar template
  - [ ] Change from direct POST to modal trigger
  - [ ] Add TwinSpark attributes to open confirmation modal
  - [ ] ts-req="/plan/regenerate/confirm" (GET modal HTML)
  - [ ] ts-target="#modal-container"
  - [ ] ts-swap="inner"
- [ ] Implement `GET /plan/regenerate/confirm` route
  - [ ] Render confirmation modal HTML
  - [ ] Include form with reason text field
  - [ ] Form action: POST /plan/regenerate
- [ ] Wire confirmation form submission
  - [ ] Standard form POST (no AJAX, full page reload)
  - [ ] Submit reason field if provided
  - [ ] Server-side validation and regeneration

### Task 4: Archive Old Meal Plan (AC: 9)
- [ ] Update `MealPlanRegenerated` event handler in aggregate
  - [ ] Set is_active = false on old meal plan (if creating new ID)
  - [ ] OR: Keep same meal plan ID and just replace assignments
  - [ ] Decision: Reuse same MealPlan aggregate ID (simpler)
- [ ] Implement read model projection for archived plans
  - [ ] `meal_plan_regenerated_handler()` subscription
  - [ ] Listen for `MealPlanRegenerated` events
  - [ ] Update `meal_assignments` table:
    - DELETE all assignments for meal_plan_id
    - INSERT new assignments from event
  - [ ] Update `meal_plans.updated_at` timestamp
  - [ ] DO NOT change is_active (stays true for same plan)
- [ ] Event sourcing preserves full history
  - [ ] Old assignments recoverable from event stream
  - [ ] Audit trail via MealPlanRegenerated events

### Task 5: Preserve Rotation State Across Regeneration (AC: 5)
- [ ] Modify `regenerate_meal_plan()` to pass current rotation state
  - [ ] Load rotation state from aggregate
  - [ ] Pass rotation_state to MealPlanningAlgorithm
  - [ ] Algorithm filters recipes by rotation (same as generation)
  - [ ] DO NOT call `rotation.reset_cycle()`
  - [ ] Keep cycle_number unchanged
- [ ] Update rotation state with new recipe usages
  - [ ] Mark newly assigned recipes as used
  - [ ] Emit `RecipeUsedInRotation` events for new assignments
  - [ ] Rotation cycle continues across regenerations
- [ ] Write unit tests:
  - [ ] Test: Rotation cycle_number unchanged after regeneration
  - [ ] Test: Previously used recipes not reassigned
  - [ ] Test: Rotation progress maintained

### Task 6: Trigger Shopping List Regeneration (AC: 8)
- [ ] Emit domain event `ShoppingListRegenerationRequested`
  - [ ] Include meal_plan_id, user_id
  - [ ] Triggered by `meal_plan_regenerated_handler()` projection
- [ ] Shopping domain subscription handler (Epic 4)
  - [ ] Listen for `ShoppingListRegenerationRequested`
  - [ ] Delete old shopping list for week
  - [ ] Generate new shopping list from updated assignments
  - [ ] User sees updated list on /shopping page
- [ ] Note: Shopping domain implementation in Epic 4
  - [ ] Emit event now, handler implemented later
  - [ ] Event schema documented for cross-domain contract

### Task 7: Update Meal Calendar Template (AC: 1, 7)
- [ ] Add "Regenerate Meal Plan" button to calendar page
  - [ ] Position: Top-right of calendar header
  - [ ] Icon: Refresh/circular arrow icon
  - [ ] Text: "Regenerate Meal Plan"
  - [ ] Style: Secondary button (less prominent than primary actions)
  - [ ] TwinSpark: Opens confirmation modal
- [ ] Calendar auto-updates after regeneration
  - [ ] Full page reload after POST /plan/regenerate
  - [ ] Server renders new meal assignments
  - [ ] Calendar displays updated recipe slots
- [ ] Add loading indicator during regeneration
  - [ ] Show spinner overlay while algorithm runs
  - [ ] Disable form submission during processing
  - [ ] Target: <5 seconds for 50 recipes

### Task 8: Write Comprehensive Test Suite (TDD)
- [ ] **Unit tests** (domain logic):
  - [ ] Test: regenerate_meal_plan() creates different assignments
  - [ ] Test: Rotation state preserved (cycle_number unchanged)
  - [ ] Test: All constraints satisfied (availability, complexity, rotation)
  - [ ] Test: Insufficient recipes validation (<7 favorites)
  - [ ] Test: No active meal plan error handling
  - [ ] Test: Algorithm determinism with seed variation
- [ ] **Integration tests** (full HTTP flow):
  - [ ] Test: GET /plan/regenerate/confirm returns modal HTML
  - [ ] Test: POST /plan/regenerate updates database
  - [ ] Test: Meal assignments replaced with new recipes
  - [ ] Test: Old assignments not present in new plan
  - [ ] Test: Authorization check prevents cross-user regeneration
  - [ ] Test: Regeneration with optional reason field
- [ ] **E2E tests** (Playwright):
  - [ ] Test: Full regeneration flow from calendar button to updated view
  - [ ] Test: Confirmation modal prevents accidental regeneration
  - [ ] Test: Cancel button dismisses modal without regeneration
- [ ] Test coverage: Maintain 80%+ code coverage

## Dev Notes

### Architecture Patterns
- **Event Sourcing**: MealPlanRegenerated event persisted to evento stream
- **CQRS**: Command updates aggregate, read model projection replaces assignments
- **Domain Events**: ShoppingListRegenerationRequested triggers cross-domain update
- **Server-Side Rendering**: Askama templates for modal and calendar view
- **Algorithm Reuse**: Same MealPlanningAlgorithm as Story 3.1 generation

### Key Components
- **Route**: `src/routes/meal_plan.rs::post_regenerate_meal_plan()` (NEW handler)
- **Route**: `src/routes/meal_plan.rs::get_regenerate_confirm()` (NEW modal route)
- **Domain Command**: `crates/meal_planning/src/lib.rs::regenerate_meal_plan()` (NEW)
- **Aggregate**: `crates/meal_planning/src/aggregate.rs::MealPlan` (UPDATE event handler)
- **Algorithm**: `crates/meal_planning/src/algorithm.rs::MealPlanningAlgorithm` (REUSE from 3.1)
- **Read Model Projection**: `crates/meal_planning/src/read_model.rs::meal_plan_regenerated_handler()` (NEW)
- **Templates**:
  - `templates/components/regenerate-confirmation-modal.html` (NEW)
  - `templates/pages/meal-calendar.html` (UPDATE with regenerate button)

### Data Flow
1. **User clicks "Regenerate Meal Plan"**:
   - GET /plan/regenerate/confirm
   - Route handler renders confirmation modal HTML
   - TwinSpark injects modal into DOM

2. **User confirms regeneration**:
   - POST /plan/regenerate with optional reason
   - Route handler validates authorization and active plan
   - Invoke domain command: meal_planning::regenerate_meal_plan()
   - Domain layer:
     - Load MealPlan aggregate from event stream
     - Load user profile and favorite recipes
     - Load current rotation state (preserved)
     - Run MealPlanningAlgorithm with same constraints
     - Generate new assignments (different from current)
     - Emit MealPlanRegenerated event
   - Evento subscription:
     - Delete old meal_assignments for meal_plan_id
     - Insert new meal_assignments from event
     - Update meal_plans.updated_at timestamp
     - Emit ShoppingListRegenerationRequested event
   - Route handler redirects to GET /plan with success flash

3. **Shopping list updates automatically**:
   - Shopping domain subscription listens for ShoppingListRegenerationRequested
   - Regenerates shopping list with new recipe ingredients
   - User sees updated list on /shopping page

### Project Structure Notes

**Alignment with Solution Architecture**:
- **evento Aggregate Pattern**: MealPlan aggregate handles MealPlanRegenerated event [Source: docs/solution-architecture.md#Event Sourcing]
- **CQRS Read Models**: meal_assignments table replaced via projection [Source: docs/solution-architecture.md#CQRS Implementation]
- **Algorithm Reuse**: MealPlanningAlgorithm from Story 3.1 [Source: docs/tech-spec-epic-3.md#MealPlanningAlgorithm]
- **Route Structure**: Follows /plan prefix for meal planning routes [Source: docs/solution-architecture.md#Page Routing]

**Lessons from Story 3.6**:
- **CSP Compliance**: Extract inline JavaScript to external files [Source: Story 3.6 Action Item #1]
- **Keyboard Navigation**: Support Escape to close modal, Enter to confirm [Source: Story 3.6 Action Item #2]
- **ARIA Landmarks**: Add role attributes and focus management [Source: Story 3.6 Action Item #3]
- **Error Handling**: Proper match statements, no .unwrap() [Source: Story 3.6 Completion Notes]
- **Test Coverage**: Maintain 80%+ with unit + integration tests [Source: Story 3.6 Test Results]

**New Components**:
- `src/routes/meal_plan.rs::post_regenerate_meal_plan()` - Regeneration route handler
- `src/routes/meal_plan.rs::get_regenerate_confirm()` - Confirmation modal route
- `crates/meal_planning/src/lib.rs::regenerate_meal_plan()` - Domain command function
- `crates/meal_planning/src/read_model.rs::meal_plan_regenerated_handler()` - Projection
- `templates/components/regenerate-confirmation-modal.html` - Confirmation UI
- `static/js/meal-regeneration.js` - CSP-compliant modal/keyboard interactions

### References

- [Source: docs/epics.md#Story 3.7] Regenerate Full Meal Plan requirements (lines 706-728)
- [Source: docs/tech-spec-epic-3.md#Story 3.7] Implementation checklist and acceptance criteria
- [Source: docs/tech-spec-epic-3.md#MealPlanRegenerated Event] Event definition and handling (lines 308-314, 399-407)
- [Source: docs/tech-spec-epic-3.md#MealPlanningAlgorithm] Algorithm reuse for regeneration (lines 69-167)
- [Source: docs/tech-spec-epic-3.md#RotationManager] Rotation state preservation (lines 232-274)
- [Source: docs/solution-architecture.md#Server-Side Rendering] Askama template patterns (lines 122-141)
- [Source: docs/solution-architecture.md#CQRS] Command/query segregation (lines 206-249)
- [Source: Story 3.6 Completion Notes] Lessons learned on CSP, accessibility, error handling

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.7.xml` (Generated: 2025-10-17)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
